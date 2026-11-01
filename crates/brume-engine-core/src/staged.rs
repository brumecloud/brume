//! Staged changes: drafts outside the log, deployed together per environment.

use std::collections::BTreeMap;

use brume_domain::{
    AdminId, BlobId, ChangeId, Command, CpuMillis, Ctx, DeploymentId, EnvironmentId, HttpPath,
    ImageRef, MemoryMb, PlainKind, PlainValue, Port, SecretValue, Service, ServiceChanges,
    ServiceId, ServiceVariableValue, SpecChange, StartCommand, VarName, VariableId, VariableOwner,
};
use brume_proto::api::{
    ApiFailure, ApiValue, ChangesDeployedDto, FailureKind, NewChangeDto, SettingField,
    SettingValueDto, StagedChangeDto, VariableKindDto,
};

use crate::api::{invalid, not_found, parse_id};
use crate::engine::{Actor, Engine, failure, storage_failure};
use crate::storage::{StagedBody, StagedValue, Storage, StoredChange, Transaction};

/// What a staged change targets: a newer change to the same target replaces it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Setting(SettingField),
    Variable(VarName),
}

fn target(body: &StagedBody) -> Target {
    match body {
        StagedBody::Setting { field, .. } => Target::Setting(*field),
        StagedBody::Variable { name, .. } | StagedBody::VariableRemoval { name } => {
            Target::Variable(name.clone())
        }
    }
}

fn field_name(field: SettingField) -> String {
    serde_json::to_value(field)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "setting".to_owned())
}

/// The spec change of a setting, validated.
pub(crate) fn setting_change(
    field: SettingField,
    value: &SettingValueDto,
) -> Result<SpecChange, ApiFailure> {
    let wrong_type = || {
        failure(
            FailureKind::Invalid,
            format!("Invalid {}: wrong type.", field_name(field)),
        )
    };
    let whole = |number: f64| -> Option<u32> {
        (number.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&number))
            .then_some(number as u32)
    };
    match (field, value) {
        (SettingField::Image, SettingValueDto::Text(image)) => {
            Ok(SpecChange::Image(ImageRef::parse(image).map_err(invalid)?))
        }
        (SettingField::StartCommand, SettingValueDto::Text(command)) if command.is_empty() => {
            Ok(SpecChange::StartCommand(None))
        }
        (SettingField::StartCommand, SettingValueDto::Text(command)) => Ok(
            SpecChange::StartCommand(Some(StartCommand::parse(command).map_err(invalid)?)),
        ),
        (SettingField::Port, SettingValueDto::Text(port)) if port.is_empty() => {
            Ok(SpecChange::Port(None))
        }
        (SettingField::Port, SettingValueDto::Number(port)) => {
            let port = whole(*port)
                .and_then(|port| u16::try_from(port).ok())
                .ok_or_else(wrong_type)?;
            Ok(SpecChange::Port(Some(Port::new(port).map_err(invalid)?)))
        }
        (SettingField::HealthcheckPath, SettingValueDto::Text(path)) if path.is_empty() => {
            Ok(SpecChange::HealthcheckPath(None))
        }
        (SettingField::HealthcheckPath, SettingValueDto::Text(path)) => Ok(
            SpecChange::HealthcheckPath(Some(HttpPath::parse(path).map_err(invalid)?)),
        ),
        (SettingField::CpuLimit, SettingValueDto::Number(cpus)) => {
            let millis = whole((cpus * 1000.0).round()).ok_or_else(wrong_type)?;
            Ok(SpecChange::Cpu(CpuMillis::new(millis).map_err(invalid)?))
        }
        (SettingField::MemoryLimitMb, SettingValueDto::Number(memory)) => {
            let memory = whole(*memory).ok_or_else(wrong_type)?;
            Ok(SpecChange::Memory(MemoryMb::new(memory).map_err(invalid)?))
        }
        (SettingField::Egress, SettingValueDto::Bool(egress)) => Ok(SpecChange::Egress(*egress)),
        (
            SettingField::Repository
            | SettingField::Branch
            | SettingField::AutoDeploy
            | SettingField::Domains,
            _,
        ) => Err(failure(
            FailureKind::Invalid,
            format!(
                "The {} setting is not available for an image service yet.",
                field_name(field)
            ),
        )),
        _ => Err(wrong_type()),
    }
}

/// Whether a setting change would leave the service as it is.
fn setting_is_applied(service: &Service, change: &SpecChange) -> bool {
    let spec = &service.spec;
    match change {
        SpecChange::Image(image) => &spec.image == image,
        SpecChange::StartCommand(command) => &spec.start_command == command,
        SpecChange::Port(port) => &spec.port == port,
        SpecChange::HealthcheckPath(path) => &spec.healthcheck_path == path,
        SpecChange::Cpu(cpu) => &spec.cpu == cpu,
        SpecChange::Memory(memory) => &spec.memory == memory,
        SpecChange::Egress(egress) => &service.egress == egress,
        _ => false,
    }
}

fn change_dto(change: &StoredChange) -> StagedChangeDto {
    let id = change.id.to_string();
    let environment_id = change.environment.to_string();
    let service_id = change.service.to_string();
    match &change.body {
        StagedBody::Setting { field, value } => StagedChangeDto::Setting {
            id,
            environment_id,
            service_id,
            field: *field,
            value: serde_json::from_value(value.clone())
                .unwrap_or_else(|_| SettingValueDto::Text(String::new())),
        },
        StagedBody::Variable { name, kind, value } => StagedChangeDto::Variable {
            id,
            environment_id,
            service_id,
            name: name.to_string(),
            kind: *kind,
            // A staged secret never leaves the engine.
            value: match value {
                Some(StagedValue::Plain(value)) => Some(value.clone()),
                Some(StagedValue::Secret(_)) | None => None,
            },
            reference: *kind == VariableKindDto::Secret && value.is_none(),
        },
        StagedBody::VariableRemoval { name } => StagedChangeDto::VariableRemoval {
            id,
            environment_id,
            service_id,
            name: name.to_string(),
        },
    }
}

impl<S: Storage> Engine<S> {
    pub(crate) fn list_changes(&self) -> Result<ApiValue, ApiFailure> {
        let changes = self.storage.staged_changes().map_err(storage_failure)?;
        Ok(ApiValue::Changes(changes.iter().map(change_dto).collect()))
    }

    /// Stages a change, replacing the one with the same target. A change back to the applied
    /// value removes the staged one instead, except a secret value, which is never compared.
    pub(crate) fn stage_change(
        &mut self,
        ctx: &mut impl Ctx,
        change: NewChangeDto,
    ) -> Result<ApiValue, ApiFailure> {
        let (service_id, body, applied) = match change {
            NewChangeDto::Setting {
                service_id,
                field,
                value,
            } => {
                let service = self.live_service(&service_id)?;
                let change = setting_change(field, &value)?;
                let applied = setting_is_applied(service, &change);
                let value = serde_json::to_value(&value).map_err(|_| {
                    failure(FailureKind::Invalid, "Invalid setting value.".to_owned())
                })?;
                (service_id, StagedBody::Setting { field, value }, applied)
            }
            NewChangeDto::Variable {
                service_id,
                name,
                kind,
                value,
            } => {
                let name = VarName::parse(&name).map_err(invalid)?;
                let (value, applied) = self.staged_variable(&service_id, &name, kind, value)?;
                (
                    service_id,
                    StagedBody::Variable { name, kind, value },
                    applied,
                )
            }
            NewChangeDto::VariableRemoval { service_id, name } => {
                let name = VarName::parse(&name).map_err(invalid)?;
                let service = self.live_service(&service_id)?;
                let defined = service.spec.variables.contains_key(&name);
                if !defined
                    && self
                        .staged(&service.id.clone(), &Target::Variable(name.clone()))?
                        .is_none()
                {
                    return Err(failure(
                        FailureKind::NotFound,
                        format!("{name} is not a variable of {}.", service.slug),
                    ));
                }
                // Removing a variable that only a staged change adds discards that change.
                (service_id, StagedBody::VariableRemoval { name }, !defined)
            }
        };
        let service = self.live_service(&service_id)?;
        let (service, environment) = (service.id.clone(), service.environment.clone());
        let existing = self.staged(&service, &target(&body))?;
        let id = existing
            .as_ref()
            .map(|change| change.id.clone())
            .unwrap_or_else(|| ChangeId::new(ctx));
        let change = StoredChange {
            id: id.clone(),
            environment,
            service,
            body,
        };
        let dto = change_dto(&change);
        let tx = if applied {
            Transaction {
                staged_deletes: existing.map(|change| change.id).into_iter().collect(),
                ..Transaction::default()
            }
        } else {
            Transaction {
                staged_upserts: vec![change],
                ..Transaction::default()
            }
        };
        self.write(tx)?;
        Ok(ApiValue::Change(dto))
    }

    /// The staged change of a service with that target.
    fn staged(
        &self,
        service: &ServiceId,
        target_: &Target,
    ) -> Result<Option<StoredChange>, ApiFailure> {
        Ok(self
            .storage
            .staged_changes()
            .map_err(storage_failure)?
            .into_iter()
            .find(|change| &change.service == service && &target(&change.body) == target_))
    }

    /// Validates a staged variable: its value, the name rule, and whether it equals the
    /// applied one. A secret value never does.
    fn staged_variable(
        &self,
        service_id: &str,
        name: &VarName,
        kind: VariableKindDto,
        value: Option<String>,
    ) -> Result<(Option<StagedValue>, bool), ApiFailure> {
        let service = self.live_service(service_id)?;
        self.check_name_class(service, name, kind == VariableKindDto::Secret)?;
        let current = service.spec.variables.get(name);
        match (kind, value) {
            (VariableKindDto::Plain | VariableKindDto::Build, Some(value)) => {
                let value = PlainValue::parse(&value).map_err(invalid)?;
                let plain_kind = match kind {
                    VariableKindDto::Build => PlainKind::Build,
                    _ => PlainKind::Plain,
                };
                let applied = match current.map(|variable| &variable.value) {
                    Some(ServiceVariableValue::Plain { kind, blob }) if *kind == plain_kind => {
                        self.blob_text(blob)? == value.as_str()
                    }
                    _ => false,
                };
                Ok((Some(StagedValue::Plain(value.to_string())), applied))
            }
            (VariableKindDto::Plain | VariableKindDto::Build, None) => Err(failure(
                FailureKind::Invalid,
                "A variable needs a value.".to_owned(),
            )),
            (VariableKindDto::Secret, Some(value)) => {
                // Always staged, never compared with the applied value: a change that vanished
                // when it matched would tell whoever stages guesses the secret.
                let value = SecretValue::parse(&value).map_err(invalid)?;
                Ok((Some(StagedValue::Secret(value.into_secret_string())), false))
            }
            (VariableKindDto::Secret, None) => {
                let project = self
                    .world
                    .project_of_environment(&service.environment)
                    .cloned()
                    .ok_or_else(|| not_found("project"))?;
                let shared = [
                    VariableOwner::Environment(service.environment.clone()),
                    VariableOwner::Project(project),
                ]
                .iter()
                .any(|owner| {
                    self.world
                        .shared_variable_named(owner, name)
                        .is_some_and(|shared| shared.value.is_secret())
                });
                if !shared {
                    return Err(failure(
                        FailureKind::Conflict,
                        format!(
                            "{name} is not a shared secret of this environment or its project."
                        ),
                    ));
                }
                let applied = current
                    .is_some_and(|variable| variable.value == ServiceVariableValue::SharedSecret);
                Ok((None, applied))
            }
        }
    }

    /// Refuses a name that is the other kind (variable or secret) anywhere in the project,
    /// except at this service's own entry, which the change replaces.
    fn check_name_class(
        &self,
        service: &Service,
        name: &VarName,
        secret: bool,
    ) -> Result<(), ApiFailure> {
        let Some(project) = self.world.project_of_environment(&service.environment) else {
            return Err(not_found("project"));
        };
        let shared_clash = self.world.shared_variables().any(|shared| {
            let in_project = match &shared.owner {
                VariableOwner::Project(owner) => owner == project,
                VariableOwner::Environment(environment) => {
                    self.world.project_of_environment(environment) == Some(project)
                }
            };
            in_project && &shared.name == name && shared.value.is_secret() != secret
        });
        let service_clash = self
            .world
            .services_of_project(project)
            .filter(|other| other.id != service.id)
            .any(|other| {
                other
                    .spec
                    .variables
                    .get(name)
                    .is_some_and(|variable| variable.value.is_secret() != secret)
            });
        if shared_clash || service_clash {
            let other = if secret { "a variable" } else { "a secret" };
            return Err(failure(
                FailureKind::Conflict,
                format!(
                    "{name} is already {other} in this project. A name is either a variable or a secret."
                ),
            ));
        }
        Ok(())
    }

    pub(crate) fn discard_change(&mut self, change_id: &str) -> Result<ApiValue, ApiFailure> {
        let change = parse_id(change_id, ChangeId::parse, "change")?;
        let exists = self
            .storage
            .staged_changes()
            .map_err(storage_failure)?
            .iter()
            .any(|stored| stored.id == change);
        if !exists {
            return Err(not_found("change"));
        }
        self.write(Transaction {
            staged_deletes: vec![change],
            ..Transaction::default()
        })?;
        Ok(ApiValue::NoContent)
    }

    pub(crate) fn discard_environment_changes(
        &mut self,
        environment_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let environment = self.environment(environment_id)?;
        let staged_deletes = self
            .storage
            .staged_changes()
            .map_err(storage_failure)?
            .into_iter()
            .filter(|change| change.environment == environment)
            .map(|change| change.id)
            .collect();
        self.write(Transaction {
            staged_deletes,
            ..Transaction::default()
        })?;
        Ok(ApiValue::NoContent)
    }

    fn environment(&self, environment_id: &str) -> Result<EnvironmentId, ApiFailure> {
        let environment = parse_id(environment_id, EnvironmentId::parse, "environment")?;
        match self.world.environment(&environment) {
            Some(_) => Ok(environment),
            None => Err(not_found("environment")),
        }
    }

    /// Turns the staged changes of an environment into one `DeployChanges`: a blob per plain
    /// value, a secret version per secret value, a deployment per service. The changes go in
    /// the same transaction.
    pub(crate) fn deploy_environment_changes(
        &mut self,
        ctx: &mut impl Ctx,
        admin: &AdminId,
        environment_id: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let environment = self.environment(environment_id)?;
        let mut by_service: BTreeMap<ServiceId, Vec<StoredChange>> = BTreeMap::new();
        for change in self.storage.staged_changes().map_err(storage_failure)? {
            if change.environment == environment {
                by_service
                    .entry(change.service.clone())
                    .or_default()
                    .push(change);
            }
        }
        let mut tx = Transaction::default();
        let mut services = Vec::new();
        let mut deployed = 0;
        for (service_id, changes) in by_service {
            tx.staged_deletes
                .extend(changes.iter().map(|change| change.id.clone()));
            // The changes of a deleted service just go.
            let Some(service) = self.world.live_service(&service_id) else {
                continue;
            };
            let mut spec_changes = Vec::with_capacity(changes.len());
            for change in changes {
                deployed += 1;
                let existing = |name: &VarName| {
                    service
                        .spec
                        .variables
                        .get(name)
                        .map(|variable| variable.id.clone())
                };
                let spec_change = match change.body {
                    StagedBody::Setting { field, value } => {
                        let value = serde_json::from_value(value).map_err(|_| {
                            failure(
                                FailureKind::Internal,
                                "A staged setting is corrupt.".to_owned(),
                            )
                        })?;
                        setting_change(field, &value)?
                    }
                    StagedBody::Variable {
                        name,
                        kind,
                        value: Some(StagedValue::Plain(value)),
                    } => {
                        let blob = BlobId::new(ctx);
                        tx.blobs.push((blob.clone(), value.into_bytes()));
                        SpecChange::PlainVariable {
                            variable: existing(&name).unwrap_or_else(|| VariableId::new(ctx)),
                            name,
                            kind: match kind {
                                VariableKindDto::Build => PlainKind::Build,
                                _ => PlainKind::Plain,
                            },
                            blob,
                        }
                    }
                    StagedBody::Variable {
                        name,
                        value: Some(StagedValue::Secret(value)),
                        ..
                    } => {
                        let variable = existing(&name).unwrap_or_else(|| VariableId::new(ctx));
                        let version = self.history.next_secret_version(&variable);
                        tx.secrets.push((variable.clone(), version, value));
                        SpecChange::OwnSecret {
                            name,
                            variable,
                            version,
                        }
                    }
                    StagedBody::Variable {
                        name, value: None, ..
                    } => SpecChange::SharedSecretReference {
                        variable: existing(&name).unwrap_or_else(|| VariableId::new(ctx)),
                        name,
                    },
                    StagedBody::VariableRemoval { name } => SpecChange::RemoveVariable { name },
                };
                spec_changes.push(spec_change);
            }
            services.push(ServiceChanges {
                service: service_id,
                deployment: DeploymentId::new(ctx),
                changes: spec_changes,
            });
        }
        if services.is_empty() {
            self.write(tx)?;
        } else {
            let command = Command::DeployChanges {
                environment,
                services,
                author: admin.clone(),
            };
            self.commit(ctx, &Actor::Admin(admin.clone()), &command, tx)?;
        }
        Ok(ApiValue::ChangesDeployed(ChangesDeployedDto { deployed }))
    }
}
