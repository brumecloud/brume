//! `decide` and `evolve`, in "given, when, then" form.

use std::collections::BTreeMap;

use brume_domain::{
    AdminId, BlobId, Command, Ctx, DeployReason, DeploymentId, DeploymentPhase, EnvironmentId,
    Event, EventEnvelope, ImageDigestRef, ImageRef, Level, MachineId, NewService, PlainKind, Port,
    ProjectId, ProjectName, RejectionKind, ResolvedSecret, RevisionId, RevisionPlan, RevisionValue,
    ServiceChanges, ServiceId, ServiceName, ServiceSpec, ServiceVariable, ServiceVariableValue,
    SharedValue, Slug, SpecChange, Timestamp, VarName, VariableId, VariableOwner, World, decide,
    evolve, resolve_revision_variables,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct TestCtx {
    now: Timestamp,
    rng: ChaCha8Rng,
}

impl Ctx for TestCtx {
    fn now(&self) -> Timestamp {
        self.now
    }

    fn fill_random(&mut self, buf: &mut [u8]) {
        rand::Rng::fill_bytes(&mut self.rng, buf);
    }
}

fn ctx() -> TestCtx {
    TestCtx {
        now: Timestamp::from_millis(1_793_450_096_789),
        rng: ChaCha8Rng::seed_from_u64(7),
    }
}

fn image(raw: &str) -> ImageRef {
    ImageRef::parse(raw).expect("valid image")
}

fn digest(name: &str) -> ImageDigestRef {
    ImageDigestRef::parse(&format!("{name}@sha256:{DIGEST}")).expect("valid digest")
}

fn var(raw: &str) -> VarName {
    VarName::parse(raw).expect("valid name")
}

fn slug(raw: &str) -> Slug {
    Slug::parse(raw).expect("valid slug")
}

fn envelope(seq: u64, event: Event) -> EventEnvelope {
    EventEnvelope {
        seq,
        generation: 1,
        stream: event.stream(),
        schema_version: 1,
        at: Timestamp::from_millis(1_793_450_096_789 + seq),
        actor: "engine".to_owned(),
        correlation_id: None,
        causation_id: None,
        prev_hash: "0".repeat(64),
        event,
    }
}

/// The world after these events.
fn given(events: Vec<Event>) -> World {
    let mut world = World::new();
    apply(&mut world, events);
    world
}

fn apply(world: &mut World, events: Vec<Event>) {
    for event in events {
        evolve(world, &envelope(1, event));
    }
}

/// A project with one `nginx` service in `production`, built through `decide`.
struct Scenario {
    ctx: TestCtx,
    world: World,
    admin: AdminId,
    project: ProjectId,
    production: EnvironmentId,
    service: ServiceId,
    first: DeploymentId,
}

impl Scenario {
    fn new() -> Self {
        let mut ctx = ctx();
        let admin = AdminId::new(&mut ctx);
        let project = ProjectId::new(&mut ctx);
        let production = EnvironmentId::new(&mut ctx);
        let service = ServiceId::new(&mut ctx);
        let first = DeploymentId::new(&mut ctx);
        let mut scenario = Self {
            ctx,
            world: World::new(),
            admin: admin.clone(),
            project: project.clone(),
            production: production.clone(),
            service: service.clone(),
            first: first.clone(),
        };
        scenario.run(Command::CreateProject {
            project,
            name: ProjectName::parse("Shop").expect("valid name"),
            production: production.clone(),
            first_service: Some(NewService {
                service,
                environment: production,
                name: ServiceName::parse("nginx").expect("valid name"),
                slug: slug("nginx"),
                image: image("nginx:1.29"),
                deployment: first,
                author: admin,
            }),
        });
        scenario
    }

    /// The scenario with its first deployment active.
    fn active() -> Self {
        let mut scenario = Self::new();
        let first = scenario.first.clone();
        scenario.activate(&first);
        scenario
    }

    fn try_run(&self, command: &Command) -> Result<Vec<Event>, brume_domain::Rejection> {
        decide(&self.world, command)
    }

    /// Decides and applies `command`, which must be accepted.
    fn run(&mut self, command: Command) -> Vec<Event> {
        let events = decide(&self.world, &command).expect("accepted");
        apply(&mut self.world, events.clone());
        events
    }

    /// Takes a deployment from wherever it is to active.
    fn activate(&mut self, deployment: &DeploymentId) {
        let phase = self
            .world
            .deployment(deployment)
            .expect("exists")
            .phase
            .clone();
        if phase == DeploymentPhase::Resolving {
            let plan = self
                .world
                .deployment(deployment)
                .expect("exists")
                .plan
                .clone();
            let (digest, volumes) = match plan {
                RevisionPlan::Reuse { digest, volumes } => (digest, volumes),
                RevisionPlan::Resolve { .. } => (digest("nginx"), Vec::new()),
                RevisionPlan::Existing { .. } => panic!("an existing revision starts at once"),
            };
            let revision = RevisionId::new(&mut self.ctx);
            self.run(Command::ImageResolved {
                deployment: deployment.clone(),
                revision,
                digest,
                volumes,
            });
        }
        self.run(Command::DeploymentHealthy {
            deployment: deployment.clone(),
        });
        self.run(Command::DeploymentPromoted {
            deployment: deployment.clone(),
        });
        assert_eq!(
            self.world.deployment(deployment).expect("exists").phase,
            DeploymentPhase::Active
        );
    }

    fn deploy_changes(&mut self, changes: Vec<SpecChange>) -> (DeploymentId, Command) {
        let deployment = DeploymentId::new(&mut self.ctx);
        let command = Command::DeployChanges {
            environment: self.production.clone(),
            services: vec![ServiceChanges {
                service: self.service.clone(),
                deployment: deployment.clone(),
                changes,
            }],
            author: self.admin.clone(),
        };
        (deployment, command)
    }

    fn set_project_secret(&mut self, name: &str) -> VariableId {
        let variable = VariableId::new(&mut self.ctx);
        self.run(Command::SetSharedVariable {
            variable: variable.clone(),
            owner: VariableOwner::Project(self.project.clone()),
            name: var(name),
            value: SharedValue::Secret { version: 1 },
            deployments: Vec::new(),
            author: self.admin.clone(),
        });
        variable
    }

    fn active_revision(&self) -> RevisionId {
        self.world
            .active_deployment(&self.service)
            .and_then(|deployment| deployment.revision.clone())
            .expect("an active revision")
    }
}

#[test]
fn create_project_with_image_requests_first_deployment() {
    let mut ctx = ctx();
    let admin = AdminId::new(&mut ctx);
    let project = ProjectId::new(&mut ctx);
    let production = EnvironmentId::new(&mut ctx);
    let service = ServiceId::new(&mut ctx);
    let deployment = DeploymentId::new(&mut ctx);
    let name = ProjectName::parse("Shop").expect("valid name");

    let events = decide(
        &World::new(),
        &Command::CreateProject {
            project: project.clone(),
            name: name.clone(),
            production: production.clone(),
            first_service: Some(NewService {
                service: service.clone(),
                environment: production.clone(),
                name: ServiceName::parse("nginx").expect("valid name"),
                slug: slug("nginx"),
                image: image("nginx:1.29"),
                deployment: deployment.clone(),
                author: admin.clone(),
            }),
        },
    )
    .expect("accepted");

    assert_eq!(
        events,
        vec![
            Event::ProjectCreated {
                project: project.clone(),
                name,
            },
            Event::EnvironmentCreated {
                environment: production.clone(),
                project,
                name: slug("production"),
            },
            Event::ServiceCreated {
                service: service.clone(),
                environment: production,
                name: ServiceName::parse("nginx").expect("valid name"),
                slug: slug("nginx"),
                spec: ServiceSpec::new(image("nginx:1.29")),
            },
            Event::DeploymentRequested {
                deployment,
                service,
                revision: RevisionPlan::Resolve {
                    image: image("nginx:1.29"),
                },
                reason: DeployReason::Created,
                author: admin,
            },
        ]
    );
}

#[test]
fn environment_name_is_unique_in_project() {
    let mut ctx = ctx();
    let project = ProjectId::new(&mut ctx);
    let world = given(vec![
        Event::ProjectCreated {
            project: project.clone(),
            name: ProjectName::parse("Shop").expect("valid name"),
        },
        Event::EnvironmentCreated {
            environment: EnvironmentId::new(&mut ctx),
            project: project.clone(),
            name: slug("production"),
        },
    ]);

    let duplicate = decide(
        &world,
        &Command::CreateEnvironment {
            environment: EnvironmentId::new(&mut ctx),
            project: project.clone(),
            name: slug("production"),
        },
    );
    assert_eq!(
        duplicate.map_err(|rejection| rejection.kind),
        Err(RejectionKind::Conflict)
    );

    let staging = EnvironmentId::new(&mut ctx);
    let events = decide(
        &world,
        &Command::CreateEnvironment {
            environment: staging.clone(),
            project: project.clone(),
            name: slug("staging"),
        },
    );
    assert_eq!(
        events,
        Ok(vec![Event::EnvironmentCreated {
            environment: staging,
            project,
            name: slug("staging"),
        }])
    );
}

#[test]
fn name_is_either_variable_or_secret() {
    let mut scenario = Scenario::active();
    scenario.set_project_secret("API_KEY");

    let blob = BlobId::new(&mut scenario.ctx);
    let variable = VariableId::new(&mut scenario.ctx);
    let (_, command) = scenario.deploy_changes(vec![SpecChange::PlainVariable {
        name: var("API_KEY"),
        variable,
        kind: PlainKind::Plain,
        blob,
    }]);
    let rejection = scenario.try_run(&command).expect_err("rejected");
    assert_eq!(rejection.kind, RejectionKind::Conflict);
    assert!(rejection.message.contains("API_KEY"), "{rejection}");

    // The same name as a shared plain variable, at the environment level, is refused too.
    let variable = VariableId::new(&mut scenario.ctx);
    let blob = BlobId::new(&mut scenario.ctx);
    let refused = scenario.try_run(&Command::SetSharedVariable {
        variable,
        owner: VariableOwner::Environment(scenario.production.clone()),
        name: var("API_KEY"),
        value: SharedValue::Plain {
            kind: PlainKind::Plain,
            blob,
        },
        deployments: Vec::new(),
        author: scenario.admin.clone(),
    });
    assert_eq!(
        refused.map_err(|rejection| rejection.kind),
        Err(RejectionKind::Conflict)
    );
}

#[test]
fn secret_change_redeploys_same_revision() {
    let mut scenario = Scenario::active();
    let token = VariableId::new(&mut scenario.ctx);
    let (with_token, command) = scenario.deploy_changes(vec![SpecChange::OwnSecret {
        name: var("TOKEN"),
        variable: token.clone(),
        version: 1,
    }]);
    // A new secret name changes the revision.
    let events = scenario.run(command);
    assert!(matches!(
        &events[2],
        Event::DeploymentRequested {
            revision: RevisionPlan::Reuse { .. },
            ..
        }
    ));
    scenario.activate(&with_token);
    let active = scenario.active_revision();

    let (deployment, command) = scenario.deploy_changes(vec![SpecChange::OwnSecret {
        name: var("TOKEN"),
        variable: token.clone(),
        version: 2,
    }]);
    let events = scenario.run(command);

    let mut spec = scenario
        .world
        .service(&scenario.service)
        .expect("exists")
        .spec
        .clone();
    spec.variables.insert(
        var("TOKEN"),
        ServiceVariable {
            id: token.clone(),
            value: ServiceVariableValue::OwnSecret { version: 2 },
        },
    );
    assert_eq!(
        events,
        vec![
            Event::ServiceSpecChanged {
                service: scenario.service.clone(),
                spec,
            },
            Event::SecretVersionAdded {
                variable: token.clone(),
                version: 2,
            },
            Event::DeploymentRequested {
                deployment: deployment.clone(),
                service: scenario.service.clone(),
                revision: RevisionPlan::Existing {
                    revision: active.clone(),
                },
                reason: DeployReason::Changes,
                author: scenario.admin.clone(),
            },
            Event::DeploymentStarted {
                deployment,
                revision: active,
                secrets: vec![ResolvedSecret {
                    name: var("TOKEN"),
                    variable: token,
                    version: 2,
                }],
            },
        ]
    );
}

#[test]
fn image_change_resolves_new_image() {
    let mut scenario = Scenario::active();
    let (_, command) = scenario.deploy_changes(vec![SpecChange::Image(image("nginx:1.30"))]);
    let events = scenario.try_run(&command).expect("accepted");
    assert!(matches!(
        &events[..],
        [
            Event::ServiceSpecChanged { .. },
            Event::DeploymentRequested {
                revision: RevisionPlan::Resolve { image },
                reason: DeployReason::Changes,
                ..
            },
        ] if image.as_str() == "nginx:1.30"
    ));

    // Any other setting reuses the digest of the active revision.
    let (_, command) = scenario.deploy_changes(vec![SpecChange::Port(Some(
        Port::new(8080).expect("valid port"),
    ))]);
    let events = scenario.try_run(&command).expect("accepted");
    assert!(matches!(
        &events[..],
        [
            Event::ServiceSpecChanged { .. },
            Event::DeploymentRequested {
                revision: RevisionPlan::Reuse { digest: reused, volumes },
                ..
            },
        ] if reused == &digest("nginx") && volumes.is_empty()
    ));
}

#[test]
fn rollback_restores_spec_and_rejects_missing_secret() {
    let mut scenario = Scenario::active();
    scenario.set_project_secret("DB_PASSWORD");
    let reference = VariableId::new(&mut scenario.ctx);
    let (with_secret, command) = scenario.deploy_changes(vec![
        SpecChange::SharedSecretReference {
            name: var("DB_PASSWORD"),
            variable: reference.clone(),
        },
        SpecChange::Port(Some(Port::new(8080).expect("valid port"))),
    ]);
    scenario.run(command);
    scenario.activate(&with_secret);
    let restored_spec = scenario
        .world
        .service(&scenario.service)
        .expect("exists")
        .spec
        .clone();
    let target_revision = scenario.active_revision();

    let (without_secret, command) = scenario.deploy_changes(vec![
        SpecChange::RemoveVariable {
            name: var("DB_PASSWORD"),
        },
        SpecChange::Port(None),
    ]);
    scenario.run(command);
    scenario.activate(&without_secret);
    let shared = scenario
        .world
        .shared_variable_named(
            &VariableOwner::Project(scenario.project.clone()),
            &var("DB_PASSWORD"),
        )
        .expect("exists")
        .id
        .clone();
    scenario.run(Command::RemoveSharedVariable {
        variable: shared,
        deployments: Vec::new(),
        author: scenario.admin.clone(),
    });

    let rollback = DeploymentId::new(&mut scenario.ctx);
    let command = Command::Rollback {
        target: with_secret.clone(),
        deployment: rollback.clone(),
        author: scenario.admin.clone(),
    };
    let rejection = scenario.try_run(&command).expect_err("rejected");
    assert_eq!(rejection.kind, RejectionKind::Conflict);
    assert!(rejection.message.contains("DB_PASSWORD"), "{rejection}");

    let secret = scenario.set_project_secret("DB_PASSWORD");
    let events = scenario.run(command);
    assert_eq!(
        events,
        vec![
            Event::ServiceSpecChanged {
                service: scenario.service.clone(),
                spec: restored_spec,
            },
            Event::DeploymentRequested {
                deployment: rollback.clone(),
                service: scenario.service.clone(),
                revision: RevisionPlan::Existing {
                    revision: target_revision.clone(),
                },
                reason: DeployReason::Rollback {
                    target: with_secret,
                },
                author: scenario.admin.clone(),
            },
            Event::DeploymentStarted {
                deployment: rollback,
                revision: target_revision,
                secrets: vec![ResolvedSecret {
                    name: var("DB_PASSWORD"),
                    variable: secret,
                    version: 1,
                }],
            },
        ]
    );
}

#[test]
fn shared_secret_in_use_cannot_be_removed() {
    let mut scenario = Scenario::active();
    let secret = scenario.set_project_secret("DB_PASSWORD");
    let reference = VariableId::new(&mut scenario.ctx);
    let (deployment, command) = scenario.deploy_changes(vec![SpecChange::SharedSecretReference {
        name: var("DB_PASSWORD"),
        variable: reference,
    }]);
    scenario.run(command);
    scenario.activate(&deployment);

    let rejection = scenario
        .try_run(&Command::RemoveSharedVariable {
            variable: secret,
            deployments: Vec::new(),
            author: scenario.admin.clone(),
        })
        .expect_err("rejected");
    assert_eq!(rejection.kind, RejectionKind::Conflict);
    assert_eq!(rejection.message, "DB_PASSWORD is still used by: nginx.");
}

#[test]
fn one_deployment_in_progress_per_service() {
    let mut scenario = Scenario::new();
    let (_, command) = scenario.deploy_changes(vec![SpecChange::Port(Some(
        Port::new(8080).expect("valid port"),
    ))]);
    let rejection = scenario.try_run(&command).expect_err("rejected");
    assert_eq!(rejection.kind, RejectionKind::Conflict);
    assert_eq!(
        rejection.message,
        "A deployment is already in progress for nginx."
    );

    let first = scenario.first.clone();
    scenario.activate(&first);
    let redeploy = Command::Redeploy {
        service: scenario.service.clone(),
        deployment: DeploymentId::new(&mut scenario.ctx),
        author: scenario.admin.clone(),
    };
    scenario.run(redeploy);
    let again = Command::Redeploy {
        service: scenario.service.clone(),
        deployment: DeploymentId::new(&mut scenario.ctx),
        author: scenario.admin.clone(),
    };
    assert_eq!(
        scenario.try_run(&again).map_err(|rejection| rejection.kind),
        Err(RejectionKind::Conflict)
    );
}

#[test]
fn resolve_revision_variables_precedence() {
    let mut scenario = Scenario::active();
    let set = |scenario: &mut Scenario, owner: VariableOwner, name: &str| {
        let variable = VariableId::new(&mut scenario.ctx);
        let blob = BlobId::new(&mut scenario.ctx);
        let impacted = brume_domain::impacted_services(&scenario.world, &owner, &var(name));
        let deployments = impacted
            .into_iter()
            .map(|service| (service, DeploymentId::new(&mut scenario.ctx)))
            .collect::<Vec<_>>();
        scenario.run(Command::SetSharedVariable {
            variable,
            owner,
            name: var(name),
            value: SharedValue::Plain {
                kind: PlainKind::Plain,
                blob: blob.clone(),
            },
            deployments: deployments.clone(),
            author: scenario.admin.clone(),
        });
        for (_, deployment) in deployments {
            scenario.activate(&deployment);
        }
        blob
    };
    let project = VariableOwner::Project(scenario.project.clone());
    let environment = VariableOwner::Environment(scenario.production.clone());
    let a_project = set(&mut scenario, project.clone(), "A");
    let _b_project = set(&mut scenario, project.clone(), "B");
    let b_environment = set(&mut scenario, environment.clone(), "B");
    let _c_environment = set(&mut scenario, environment, "C");
    scenario.set_project_secret("S");

    let c_service = BlobId::new(&mut scenario.ctx);
    let variable = VariableId::new(&mut scenario.ctx);
    let (deployment, command) = scenario.deploy_changes(vec![SpecChange::PlainVariable {
        name: var("C"),
        variable,
        kind: PlainKind::Plain,
        blob: c_service.clone(),
    }]);
    scenario.run(command);
    scenario.activate(&deployment);

    let resolved = resolve_revision_variables(&scenario.world, &scenario.service);
    let summary = resolved
        .iter()
        .map(|variable| match &variable.value {
            RevisionValue::Plain { blob, .. } => (
                variable.name.to_string(),
                variable.origin,
                Some(blob.clone()),
            ),
            RevisionValue::Secret { .. } => (variable.name.to_string(), variable.origin, None),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        vec![
            ("A".to_owned(), Level::Project, Some(a_project)),
            ("B".to_owned(), Level::Environment, Some(b_environment)),
            ("C".to_owned(), Level::Service, Some(c_service)),
        ]
    );

    // A referenced project secret is kept, by name only.
    let variable = VariableId::new(&mut scenario.ctx);
    let (deployment, command) = scenario.deploy_changes(vec![SpecChange::SharedSecretReference {
        name: var("S"),
        variable,
    }]);
    scenario.run(command);
    scenario.activate(&deployment);
    let resolved = resolve_revision_variables(&scenario.world, &scenario.service);
    let secret = resolved
        .iter()
        .find(|variable| variable.name.as_str() == "S")
        .expect("referenced secret");
    assert_eq!(secret.origin, Level::Project);
    assert!(matches!(
        secret.value,
        RevisionValue::Secret {
            service_variable: None,
            ..
        }
    ));
}

#[test]
fn deployment_lifecycle() {
    let mut scenario = Scenario::new();
    let first = scenario.first.clone();
    let revision = RevisionId::new(&mut scenario.ctx);
    let events = scenario.run(Command::ImageResolved {
        deployment: first.clone(),
        revision: revision.clone(),
        digest: digest("nginx"),
        volumes: Vec::new(),
    });
    assert!(matches!(
        &events[..],
        [
            Event::RevisionCreated { number: 1, spec, .. },
            Event::DeploymentStarted { secrets, .. },
        ] if spec.digest == digest("nginx") && spec.image.as_str() == "nginx:1.29"
            && secrets.is_empty()
    ));
    assert_eq!(
        scenario.world.deployment(&first).map(|d| d.phase.clone()),
        Some(DeploymentPhase::Starting)
    );

    let events = scenario.run(Command::DeploymentHealthy {
        deployment: first.clone(),
    });
    assert_eq!(
        events,
        vec![Event::DeploymentHealthy {
            deployment: first.clone()
        }]
    );
    let events = scenario.run(Command::DeploymentPromoted {
        deployment: first.clone(),
    });
    assert_eq!(
        events,
        vec![Event::DeploymentSucceeded {
            deployment: first.clone()
        }]
    );
    assert_eq!(
        scenario
            .world
            .active_deployment(&scenario.service)
            .map(|d| d.id.clone()),
        Some(first.clone())
    );

    let second = DeploymentId::new(&mut scenario.ctx);
    scenario.run(Command::Redeploy {
        service: scenario.service.clone(),
        deployment: second.clone(),
        author: scenario.admin.clone(),
    });
    assert_eq!(
        scenario
            .world
            .in_progress_deployment(&scenario.service)
            .map(|d| d.id.clone()),
        Some(second.clone())
    );
    scenario.activate(&second);
    assert_eq!(
        scenario.world.deployment(&first).map(|d| d.phase.clone()),
        Some(DeploymentPhase::Superseded)
    );
    assert_eq!(
        scenario.world.deployment(&second).map(|d| d.phase.clone()),
        Some(DeploymentPhase::Active)
    );
    assert_eq!(scenario.world.deployments_in_progress().count(), 0);
}

#[test]
fn delete_service_fails_its_deployment_in_progress() {
    let mut scenario = Scenario::new();
    let events = scenario.run(Command::DeleteService {
        service: scenario.service.clone(),
    });
    assert!(matches!(
        &events[..],
        [
            Event::DeploymentFailed {
                reason: brume_domain::FailureReason::Replaced,
                ..
            },
            Event::ServiceDeleted { .. },
        ]
    ));
    assert_eq!(scenario.world.services_in(&scenario.production).count(), 0);
}

#[test]
fn envelope_is_flat_json_and_round_trips() {
    let mut ctx = ctx();
    let machine = MachineId::new(&mut ctx);
    let envelope = envelope(
        3,
        Event::MachineRegistered {
            machine: machine.clone(),
        },
    );
    let json = serde_json::to_value(&envelope).expect("serializes");
    assert_eq!(json["type"], "MachineRegistered");
    assert_eq!(json["machine"], machine.as_str());
    assert_eq!(json["seq"], 3);
    assert_eq!(json["stream"], format!("machine:{machine}"));
    let bytes = serde_json::to_vec(&envelope).expect("serializes");
    assert_eq!(
        serde_json::from_slice::<EventEnvelope>(&bytes).ok(),
        Some(envelope)
    );

    // A deep event round-trips too.
    let mut spec = ServiceSpec::new(image("nginx:1.29"));
    spec.variables = BTreeMap::from([(
        var("TOKEN"),
        ServiceVariable {
            id: VariableId::new(&mut ctx),
            value: ServiceVariableValue::OwnSecret { version: 3 },
        },
    )]);
    let deep = self::envelope(
        4,
        Event::ServiceSpecChanged {
            service: ServiceId::new(&mut ctx),
            spec,
        },
    );
    let bytes = serde_json::to_vec(&deep).expect("serializes");
    assert_eq!(
        serde_json::from_slice::<EventEnvelope>(&bytes).ok(),
        Some(deep)
    );
}
