//! The reads of the HTTP contract, served from the `World`, and the commands that are not
//! staged changes.

use brume_domain::{
    AdminId, AuditEvent, BlobId, Command, Ctx, Deployment, DeploymentId, DeploymentPhase,
    EnvironmentId, ImageRef, NewService, PlainKind, PlainValue, ProjectId, ProjectName,
    RevisionPlan, RevisionValue, SecretValue, Service, ServiceId, ServiceName, ServiceVariable,
    ServiceVariableValue, SharedValue, SharedVariable, Slug, VarName, VariableId, VariableOwner,
    VariableRef, impacted_services, resolve_secrets, secret_redeploy_targets,
};
use brume_proto::api::{
    ApiFailure, ApiValue, CreatedProjectDto, DeploymentDto, DeploymentStatusDto, EnvironmentDto,
    FailureKind, ImpactedServiceDto, NewProjectSourceDto, NewSharedVariableDto, ProjectDto,
    RevealedValueDto, RevisionDto, RevisionVariableDto, ServiceDto, ServiceKindDto,
    ServiceSettingsDto, ServiceSourceDto, SharedVariableRemovedDto, SharedVariableSetDto,
    VariableDto, VariableImpactDto, VariableKindDto, VariableLevelDto, VolumeDto,
};
use secrecy::ExposeSecret;

use crate::api::{invalid, not_found, parse_id};
use crate::desired::{current_volume, public_host};
use crate::engine::{Actor, Engine, failure, storage_failure};
use crate::storage::{Storage, Transaction};

/// The name of a project or service when nothing else names it.
const DEFAULT_NAME: &str = "new-project";

/// The base of a derived slug when the name has no letter or digit.
const DEFAULT_SLUG: &str = "service";

/// The longest name derived from an image, in characters.
const MAX_DERIVED_NAME_CHARS: usize = 48;

/// The name an image suggests: its last path component, without the tag.
/// `ghcr.io/org/my_app:v1` gives `my_app`.
pub fn image_base_name(image: &ImageRef) -> String {
    let path = image.as_str();
    let last = path.rsplit('/').next().unwrap_or(path);
    let name = last.split(':').next().unwrap_or(last);
    name.chars().take(MAX_DERIVED_NAME_CHARS).collect()
}

/// A slug from a name, unique according to `taken`: `my_app` gives `my-app`, then `my-app-2`.
pub fn derive_slug(name: &str, taken: impl Fn(&Slug) -> bool) -> Slug {
    let mut base = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            base.push(c);
        } else if !base.is_empty() && !base.ends_with('-') {
            base.push('-');
        }
    }
    let base = base.trim_end_matches('-');
    let base = if base.is_empty() { DEFAULT_SLUG } else { base };
    for suffix in 1.. {
        let tail = if suffix == 1 {
            String::new()
        } else {
            format!("-{suffix}")
        };
        let head = base[..base.len().min(63 - tail.len())].trim_end_matches('-');
        let slug = Slug::parse(&format!("{head}{tail}")).expect("a derived slug is valid");
        if !taken(&slug) {
            return slug;
        }
    }
    unreachable!("some suffix is free")
}

fn kind_dto(kind: PlainKind) -> VariableKindDto {
    match kind {
        PlainKind::Plain => VariableKindDto::Plain,
        PlainKind::Build => VariableKindDto::Build,
    }
}

fn level_dto(level: brume_domain::Level) -> VariableLevelDto {
    match level {
        brume_domain::Level::Project => VariableLevelDto::Project,
        brume_domain::Level::Environment => VariableLevelDto::Environment,
        brume_domain::Level::Service => VariableLevelDto::Service,
    }
}

impl<S: Storage> Engine<S> {
    /// A blob, as text.
    pub(crate) fn blob_text(&self, blob: &BlobId) -> Result<String, ApiFailure> {
        match self.storage.blob(blob).map_err(storage_failure)? {
            Some(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            None => Err(failure(
                FailureKind::Internal,
                "A stored value is missing.".to_owned(),
            )),
        }
    }

    pub(crate) fn list_projects(&self) -> Vec<ProjectDto> {
        self.world
            .projects()
            .map(|project| ProjectDto {
                id: project.id.to_string(),
                name: project.name.to_string(),
                description: project.description.clone(),
                updated_at: project.updated_at.to_rfc3339(),
            })
            .collect()
    }

    pub(crate) fn create_project(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        source: NewProjectSourceDto,
    ) -> Result<ApiValue, ApiFailure> {
        let image = match source {
            NewProjectSourceDto::Empty => None,
            NewProjectSourceDto::Image { image } => Some(ImageRef::parse(&image).map_err(invalid)?),
            other => {
                let kind = match other {
                    NewProjectSourceDto::Github { .. } => "github",
                    NewProjectSourceDto::Static { .. } => "static",
                    NewProjectSourceDto::Postgres => "postgres",
                    NewProjectSourceDto::Mongodb => "mongodb",
                    _ => "redis",
                };
                return Err(failure(
                    FailureKind::Invalid,
                    format!("{kind} sources are not available yet."),
                ));
            }
        };
        let base = image
            .as_ref()
            .map(image_base_name)
            .unwrap_or_else(|| DEFAULT_NAME.to_owned());
        let name = self.unique_project_name(&base);
        let project = ProjectId::new(ctx);
        let production = EnvironmentId::new(ctx);
        let first_service = image.map(|image| {
            let name = ServiceName::parse(&base)
                .unwrap_or_else(|_| ServiceName::parse(DEFAULT_SLUG).expect("valid name"));
            NewService {
                service: ServiceId::new(ctx),
                environment: production.clone(),
                slug: derive_slug(name.as_str(), |_| false),
                name,
                image,
                deployment: DeploymentId::new(ctx),
                author: admin.clone(),
            }
        });
        let command = Command::CreateProject {
            project: project.clone(),
            name,
            production,
            first_service,
        };
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(ApiValue::CreatedProject(CreatedProjectDto {
            project_id: project.to_string(),
            environment_name: "production".to_owned(),
        }))
    }

    /// `base`, or `base-2`, `base-3`… if a project has it.
    fn unique_project_name(&self, base: &str) -> ProjectName {
        let base = ProjectName::parse(base)
            .unwrap_or_else(|_| ProjectName::parse(DEFAULT_NAME).expect("valid name"));
        let taken = |name: &str| self.world.projects().any(|p| p.name.as_str() == name);
        let mut name = base.to_string();
        let mut suffix = 2;
        while taken(&name) {
            name = format!("{base}-{suffix}");
            suffix += 1;
        }
        ProjectName::parse(&name).unwrap_or(base)
    }

    pub(crate) fn list_environments(&self) -> Vec<EnvironmentDto> {
        self.world
            .environments()
            .map(|environment| EnvironmentDto {
                id: environment.id.to_string(),
                project_id: environment.project.to_string(),
                name: environment.name.to_string(),
            })
            .collect()
    }

    pub(crate) fn create_environment(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        project_id: &str,
        name: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let project = parse_id(project_id, ProjectId::parse, "project")?;
        let name = Slug::parse(name).map_err(invalid)?;
        let environment = EnvironmentId::new(ctx);
        let command = Command::CreateEnvironment {
            environment: environment.clone(),
            project: project.clone(),
            name: name.clone(),
        };
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(ApiValue::Environment(EnvironmentDto {
            id: environment.to_string(),
            project_id: project.to_string(),
            name: name.to_string(),
        }))
    }

    pub(crate) fn service_dto(&self, service: &Service) -> ServiceDto {
        ServiceDto {
            id: service.id.to_string(),
            environment_id: service.environment.to_string(),
            name: service.name.to_string(),
            slug: service.slug.to_string(),
            kind: ServiceKindDto::Image,
            status: self.service_status(&service.id),
            // The service's public host, once its active deployment serves it.
            domains: self
                .world
                .active_deployment(&service.id)
                .and_then(|active| public_host(&self.world, service, active))
                .map(|(host, _)| host)
                .into_iter()
                .collect(),
            volume: current_volume(&self.world, service).map(|(name, path)| VolumeDto {
                name,
                mount_path: path.to_string(),
            }),
            // Edges come from template connection secrets, which this milestone does not have.
            dependencies: Vec::new(),
        }
    }

    pub(crate) fn list_services(&self) -> Vec<ServiceDto> {
        self.world
            .services()
            .filter(|service| !service.deleted)
            .map(|service| self.service_dto(service))
            .collect()
    }

    pub(crate) fn create_service(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        environment_id: &str,
        image: &str,
        name: Option<&str>,
    ) -> Result<ApiValue, ApiFailure> {
        let environment = parse_id(environment_id, EnvironmentId::parse, "environment")?;
        if self.world.environment(&environment).is_none() {
            return Err(not_found("environment"));
        }
        let image = ImageRef::parse(image).map_err(invalid)?;
        let name = match name {
            Some(name) => ServiceName::parse(name).map_err(invalid)?,
            None => ServiceName::parse(&image_base_name(&image))
                .unwrap_or_else(|_| ServiceName::parse(DEFAULT_SLUG).expect("valid name")),
        };
        let slug = derive_slug(name.as_str(), |slug| {
            self.world.service_by_slug(&environment, slug).is_some()
        });
        let service = ServiceId::new(ctx);
        let command = Command::CreateService(NewService {
            service: service.clone(),
            environment,
            name,
            slug,
            image,
            deployment: DeploymentId::new(ctx),
            author: admin.clone(),
        });
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        let created = self.world.service(&service).expect("just created");
        Ok(ApiValue::Service(self.service_dto(created)))
    }

    /// A live service from a path id.
    pub(crate) fn live_service(&self, service_id: &str) -> Result<&Service, ApiFailure> {
        let service = parse_id(service_id, ServiceId::parse, "service")?;
        self.world
            .live_service(&service)
            .ok_or_else(|| not_found("service"))
    }

    pub(crate) fn service_settings(&self, service_id: &str) -> Result<ApiValue, ApiFailure> {
        let service = self.live_service(service_id)?;
        let spec = &service.spec;
        Ok(ApiValue::ServiceSettings(ServiceSettingsDto {
            source: ServiceSourceDto::Image {
                image: spec.image.to_string(),
            },
            start_command: spec.start_command.as_ref().map(ToString::to_string),
            port: spec.port.map(|port| port.get()),
            healthcheck_path: spec.healthcheck_path.as_ref().map(ToString::to_string),
            cpu_limit: f64::from(spec.cpu.get()) / 1000.0,
            memory_limit_mb: spec.memory.get(),
            egress: service.egress,
            domain: service.domain.as_ref().map(ToString::to_string),
        }))
    }

    pub(crate) fn delete_service(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        service_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let service = self.live_service(service_id)?.id.clone();
        // Its staged changes go with it, in the same transaction.
        let staged_deletes = self
            .storage
            .staged_changes()
            .map_err(storage_failure)?
            .into_iter()
            .filter(|change| change.service == service)
            .map(|change| change.id)
            .collect();
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &Command::DeleteService { service },
            Transaction {
                staged_deletes,
                ..Transaction::default()
            },
        )?;
        Ok(ApiValue::NoContent)
    }

    pub(crate) fn deployment_dto(&self, deployment: &Deployment) -> DeploymentDto {
        let revision = deployment
            .revision
            .as_ref()
            .and_then(|revision| self.world.revision(revision));
        let image = match (&deployment.plan, revision) {
            (_, Some(revision)) => Some(revision.spec.image.to_string()),
            (RevisionPlan::Resolve { image }, None) => Some(image.to_string()),
            (RevisionPlan::Reuse { digest, .. }, None) => Some(digest.to_string()),
            (RevisionPlan::Existing { .. }, None) => None,
        };
        let status = match deployment.phase {
            DeploymentPhase::Resolving => DeploymentStatusDto::Building,
            DeploymentPhase::Starting | DeploymentPhase::Promoting => {
                DeploymentStatusDto::Deploying
            }
            DeploymentPhase::Active => DeploymentStatusDto::Active,
            DeploymentPhase::Superseded => DeploymentStatusDto::Superseded,
            DeploymentPhase::Failed(_) => DeploymentStatusDto::Failed,
        };
        let live = self.world.live_service(&deployment.service).is_some();
        DeploymentDto {
            id: deployment.id.to_string(),
            service_id: deployment.service.to_string(),
            status,
            created_at: deployment.created_at.to_rfc3339(),
            author: self.admin_name(&deployment.author),
            commit: None,
            image,
            rollbackable: live
                && revision.is_some()
                && deployment.phase == DeploymentPhase::Superseded,
        }
    }

    pub(crate) fn list_deployments(&self) -> Vec<DeploymentDto> {
        self.world
            .deployments()
            .filter(|deployment| self.world.live_service(&deployment.service).is_some())
            .map(|deployment| self.deployment_dto(deployment))
            .collect()
    }

    /// The new deployment, once its command is committed.
    fn new_deployment(&self, deployment: &DeploymentId) -> ApiValue {
        let deployment = self.world.deployment(deployment).expect("just requested");
        ApiValue::Deployment(self.deployment_dto(deployment))
    }

    pub(crate) fn redeploy(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        service_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let service = self.live_service(service_id)?.id.clone();
        let deployment = DeploymentId::new(ctx);
        let command = Command::Redeploy {
            service,
            deployment: deployment.clone(),
            author: admin.clone(),
        };
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(self.new_deployment(&deployment))
    }

    pub(crate) fn rollback(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        deployment_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let target = parse_id(deployment_id, DeploymentId::parse, "deployment")?;
        let deployment = DeploymentId::new(ctx);
        let command = Command::Rollback {
            target,
            deployment: deployment.clone(),
            author: admin.clone(),
        };
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(self.new_deployment(&deployment))
    }

    pub(crate) fn revision(&self, deployment_id: &str) -> Result<ApiValue, ApiFailure> {
        let deployment = parse_id(deployment_id, DeploymentId::parse, "deployment")?;
        let deployment = self
            .world
            .deployment(&deployment)
            .ok_or_else(|| not_found("deployment"))?;
        let Some(revision) = deployment
            .revision
            .as_ref()
            .and_then(|revision| self.world.revision(revision))
        else {
            return Err(failure(
                FailureKind::NotFound,
                "This deployment has no revision yet.".to_owned(),
            ));
        };
        let mut variables = Vec::with_capacity(revision.spec.variables.len());
        for variable in &revision.spec.variables {
            let (kind, value) = match &variable.value {
                RevisionValue::Plain { kind, blob, .. } => {
                    (kind_dto(*kind), Some(self.blob_text(blob)?))
                }
                RevisionValue::Secret { .. } => (VariableKindDto::Secret, None),
            };
            variables.push(RevisionVariableDto {
                name: variable.name.to_string(),
                kind,
                value,
                origin: level_dto(variable.origin),
            });
        }
        Ok(ApiValue::Revision(RevisionDto {
            digest: revision.spec.digest.to_string(),
            builder: None,
            start_command: revision
                .spec
                .start_command
                .as_ref()
                .map(ToString::to_string),
            variables,
        }))
    }

    fn shared_variable_dto(&self, shared: &SharedVariable) -> Result<VariableDto, ApiFailure> {
        let (level, owner_id) = match &shared.owner {
            VariableOwner::Project(project) => (VariableLevelDto::Project, project.to_string()),
            VariableOwner::Environment(environment) => {
                (VariableLevelDto::Environment, environment.to_string())
            }
        };
        let (kind, value) = match &shared.value {
            SharedValue::Plain { kind, blob } => (kind_dto(*kind), Some(self.blob_text(blob)?)),
            SharedValue::Secret { .. } => (VariableKindDto::Secret, None),
        };
        Ok(VariableDto {
            id: shared.id.to_string(),
            level,
            owner_id,
            name: shared.name.to_string(),
            kind,
            value,
            reference: false,
        })
    }

    fn service_variable_dto(
        &self,
        service: &Service,
        name: &VarName,
        variable: &ServiceVariable,
    ) -> Result<VariableDto, ApiFailure> {
        let (kind, value, reference) = match &variable.value {
            ServiceVariableValue::Plain { kind, blob } => {
                (kind_dto(*kind), Some(self.blob_text(blob)?), false)
            }
            ServiceVariableValue::OwnSecret { .. } => (VariableKindDto::Secret, None, false),
            ServiceVariableValue::SharedSecret => (VariableKindDto::Secret, None, true),
        };
        Ok(VariableDto {
            id: variable.id.to_string(),
            level: VariableLevelDto::Service,
            owner_id: service.id.to_string(),
            name: name.to_string(),
            kind,
            value,
            reference,
        })
    }

    pub(crate) fn list_variables(&self) -> Result<ApiValue, ApiFailure> {
        let mut variables = Vec::new();
        for shared in self.world.shared_variables() {
            variables.push(self.shared_variable_dto(shared)?);
        }
        for service in self.world.services().filter(|service| !service.deleted) {
            for (name, variable) in &service.spec.variables {
                variables.push(self.service_variable_dto(service, name, variable)?);
            }
        }
        Ok(ApiValue::Variables(variables))
    }

    /// The owner of a shared variable, from the panel's level and id.
    fn owner(&self, level: VariableLevelDto, owner_id: &str) -> Result<VariableOwner, ApiFailure> {
        match level {
            VariableLevelDto::Project => {
                let project = parse_id(owner_id, ProjectId::parse, "project")?;
                match self.world.project(&project) {
                    Some(_) => Ok(VariableOwner::Project(project)),
                    None => Err(not_found("project")),
                }
            }
            VariableLevelDto::Environment => {
                let environment = parse_id(owner_id, EnvironmentId::parse, "environment")?;
                match self.world.environment(&environment) {
                    Some(_) => Ok(VariableOwner::Environment(environment)),
                    None => Err(not_found("environment")),
                }
            }
            VariableLevelDto::Service => Err(failure(
                FailureKind::Invalid,
                "A service variable is a staged change of the service.".to_owned(),
            )),
        }
    }

    fn impacted_dtos(&self, services: &[ServiceId]) -> Vec<ImpactedServiceDto> {
        services
            .iter()
            .filter_map(|service| self.world.service(service))
            .map(|service| ImpactedServiceDto {
                service_id: service.id.to_string(),
                environment_id: service.environment.to_string(),
            })
            .collect()
    }

    pub(crate) fn variable_impact(
        &self,
        level: VariableLevelDto,
        owner_id: &str,
        name: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let owner = self.owner(level, owner_id)?;
        let name = VarName::parse(name).map_err(invalid)?;
        let services = impacted_services(&self.world, &owner, &name);
        Ok(ApiValue::VariableImpact(VariableImpactDto {
            services: self.impacted_dtos(&services),
        }))
    }

    pub(crate) fn set_shared_variable(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        new: NewSharedVariableDto,
    ) -> Result<ApiValue, ApiFailure> {
        let owner = self.owner(new.level, &new.owner_id)?;
        let name = VarName::parse(&new.name).map_err(invalid)?;
        let variable = match self.world.shared_variable_named(&owner, &name) {
            Some(existing) => existing.id.clone(),
            None => VariableId::new(ctx),
        };
        let mut tx = Transaction::default();
        let (value, deployments) = match new.kind {
            VariableKindDto::Plain | VariableKindDto::Build => {
                let value = PlainValue::parse(&new.value).map_err(invalid)?;
                let blob = BlobId::new(ctx);
                tx.blobs
                    .push((blob.clone(), value.as_str().as_bytes().to_vec()));
                let kind = match new.kind {
                    VariableKindDto::Build => PlainKind::Build,
                    _ => PlainKind::Plain,
                };
                let deployments = impacted_services(&self.world, &owner, &name)
                    .into_iter()
                    .map(|service| (service, DeploymentId::new(ctx)))
                    .collect::<Vec<_>>();
                (SharedValue::Plain { kind, blob }, deployments)
            }
            VariableKindDto::Secret => {
                let value = SecretValue::parse(&new.value).map_err(invalid)?;
                let version = self.history.next_secret_version(&variable);
                tx.secrets
                    .push((variable.clone(), version, value.into_secret_string()));
                (SharedValue::Secret { version }, Vec::new())
            }
        };
        let is_secret = value.is_secret();
        let command = Command::SetSharedVariable {
            variable: variable.clone(),
            owner: owner.clone(),
            name: name.clone(),
            value,
            deployments: deployments.clone(),
            author: admin.clone(),
        };
        self.commit(ctx, &Actor::Admin(admin.clone()), &command, tx)?;
        let shared = self
            .world
            .shared_variable(&variable)
            .expect("just set")
            .clone();
        // The services that a redeploy brings to this value: by name, as a deployment
        // resolves it, so a new override counts though no deployment resolved it yet.
        let referenced_by = if is_secret {
            secret_redeploy_targets(&self.world, &owner, &name)
                .into_iter()
                .map(|service| service.to_string())
                .collect()
        } else {
            Vec::new()
        };
        Ok(ApiValue::SharedVariableSet(SharedVariableSetDto {
            variable: self.shared_variable_dto(&shared)?,
            deployments: deployments
                .iter()
                .filter_map(|(_, deployment)| self.world.deployment(deployment))
                .map(|deployment| self.deployment_dto(deployment))
                .collect(),
            referenced_by,
        }))
    }

    pub(crate) fn remove_shared_variable(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        variable_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let variable = parse_id(variable_id, VariableId::parse, "variable")?;
        let shared = match self.world.variable(&variable) {
            Some(VariableRef::Shared(shared)) => shared.clone(),
            Some(VariableRef::Service { .. }) => {
                return Err(failure(
                    FailureKind::Invalid,
                    "A service variable is removed by a staged change of the service.".to_owned(),
                ));
            }
            None => return Err(not_found("variable")),
        };
        let deployments = match shared.value {
            SharedValue::Plain { .. } => {
                impacted_services(&self.world, &shared.owner, &shared.name)
                    .into_iter()
                    .map(|service| (service, DeploymentId::new(ctx)))
                    .collect()
            }
            SharedValue::Secret { .. } => Vec::new(),
        };
        let command = Command::RemoveSharedVariable {
            variable,
            deployments: deployments.clone(),
            author: admin.clone(),
        };
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(ApiValue::SharedVariableRemoved(SharedVariableRemovedDto {
            deployments: deployments
                .iter()
                .filter_map(|(_, deployment)| self.world.deployment(deployment))
                .map(|deployment| self.deployment_dto(deployment))
                .collect(),
        }))
    }

    /// An audited read of a secret's current value.
    pub(crate) fn reveal_variable(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        variable_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let variable = parse_id(variable_id, VariableId::parse, "variable")?;
        let source = match self.world.variable(&variable) {
            None => return Err(not_found("variable")),
            Some(VariableRef::Shared(SharedVariable {
                value: SharedValue::Secret { version },
                ..
            })) => Some((variable.clone(), *version)),
            Some(VariableRef::Service {
                variable:
                    ServiceVariable {
                        value: ServiceVariableValue::OwnSecret { version },
                        ..
                    },
                ..
            }) => Some((variable.clone(), *version)),
            Some(VariableRef::Service {
                service,
                name,
                variable:
                    ServiceVariable {
                        value: ServiceVariableValue::SharedSecret,
                        ..
                    },
            }) => resolve_secrets(&self.world, &service.id, std::slice::from_ref(name))
                .ok()
                .and_then(|resolved| resolved.into_iter().next())
                .map(|secret| (secret.variable, secret.version)),
            Some(_) => {
                return Err(failure(
                    FailureKind::Invalid,
                    "Only a secret can be revealed.".to_owned(),
                ));
            }
        };
        let Some((secret, version)) = source else {
            return Err(failure(
                FailureKind::NotFound,
                "This secret reference resolves to no shared secret.".to_owned(),
            ));
        };
        let Some(value) = self
            .storage
            .secret(&secret, version)
            .map_err(storage_failure)?
        else {
            return Err(failure(
                FailureKind::Internal,
                "A stored value is missing.".to_owned(),
            ));
        };
        let command = Command::RecordAudit(AuditEvent::SecretRevealed {
            variable,
            admin: admin.clone(),
        });
        self.commit(
            ctx,
            &Actor::Admin(admin.clone()),
            &command,
            Transaction::default(),
        )?;
        Ok(ApiValue::RevealedValue(RevealedValueDto {
            value: value.expose_secret().to_owned(),
        }))
    }
}
