//! The engine's state machine: sessions, the deployment flow with a fake agent, recovery,
//! the hash chain, staged changes and secrets.

mod fake_storage;
mod harness;

use brume_domain::Timestamp;
use brume_engine_core::{DEPLOYMENT_TIMEOUT_MILLIS, Engine, Input, OpenError, short_id};
use brume_proto::agent::{ContainerHealth, EnvValue, Healthcheck, Job};
use brume_proto::api::{
    ApiCall, ApiValue, DeploymentStatusDto, FailureKind, LogStream, NewChangeDto,
    NewSharedVariableDto, ServiceStatusDto, SettingField, SettingValueDto, StagedChangeDto,
    VariableKindDto, VariableLevelDto,
};
use secrecy::ExposeSecret;

use crate::harness::{DIGEST, Harness, jobs, last_desired, wake_at};

const SECRET: &str = "s3cr3t-🚀";

fn stage(harness: &mut Harness, change: NewChangeDto) {
    harness.ok(ApiCall::StageChange(change));
}

fn changes(harness: &mut Harness) -> Vec<StagedChangeDto> {
    match harness.ok(ApiCall::ListChanges) {
        ApiValue::Changes(changes) => changes,
        other => panic!("unexpected {other:?}"),
    }
}

fn deploy_changes(harness: &mut Harness, environment_id: &str) {
    harness.ok(ApiCall::DeployEnvironmentChanges {
        environment_id: environment_id.to_owned(),
    });
}

fn setting(service_id: &str, field: SettingField, value: SettingValueDto) -> NewChangeDto {
    NewChangeDto::Setting {
        service_id: service_id.to_owned(),
        field,
        value,
    }
}

fn variable(
    service_id: &str,
    name: &str,
    kind: VariableKindDto,
    value: Option<&str>,
) -> NewChangeDto {
    NewChangeDto::Variable {
        service_id: service_id.to_owned(),
        name: name.to_owned(),
        kind,
        value: value.map(ToOwned::to_owned),
    }
}

/// A project with an active `nginx` service: its service id, environment id and deployment.
fn active_project(harness: &mut Harness, volumes: &[&str]) -> (String, String, String) {
    harness.create_project("nginx:1.29");
    let service = harness.services()[0].clone();
    let deployment = harness.deployments()[0].id.clone();
    harness.connect_agent();
    harness.activate(&deployment, volumes);
    (service.id, service.environment_id, deployment)
}

fn newest_deployment(harness: &mut Harness, service_id: &str) -> String {
    harness
        .deployments()
        .into_iter()
        .filter(|deployment| deployment.service_id == service_id)
        .max_by(|a, b| (&a.created_at, &a.id).cmp(&(&b.created_at, &b.id)))
        .expect("a deployment")
        .id
}

#[test]
fn login_link_is_single_use() {
    let mut harness = Harness::unauthenticated();
    let token = harness.login_link();
    let session = match harness.call(ApiCall::Login {
        token: token.clone(),
    }) {
        Ok(ApiValue::Session { token, admin }) => {
            assert_eq!(admin, "admin");
            token
        }
        other => panic!("unexpected {other:?}"),
    };
    assert_ne!(session, token);
    assert_eq!(
        harness
            .call(ApiCall::Login { token })
            .map_err(|failure| failure.kind),
        Err(FailureKind::Unauthorized)
    );

    // A link expires after 15 minutes.
    let late = harness.login_link();
    harness.ctx.advance(15 * 60 * 1000);
    assert_eq!(
        harness
            .call(ApiCall::Login { token: late })
            .map_err(|failure| failure.kind),
        Err(FailureKind::Unauthorized)
    );

    // Each attempt is in the log; no token is.
    let events = harness
        .storage
        .events()
        .into_iter()
        .map(|event| String::from_utf8(event.bytes).expect("utf-8"))
        .collect::<Vec<_>>();
    let count = |kind: &str| {
        events
            .iter()
            .filter(|event| event.contains(&format!(r#""type":"{kind}""#)))
            .count()
    };
    assert_eq!(count("LoginSucceeded"), 1);
    assert_eq!(count("LoginFailed"), 2);
    assert!(events.iter().all(|event| !event.contains(&session)));
}

#[test]
fn request_without_session_is_unauthorized() {
    let mut harness = Harness::new();
    let session = harness.session.clone();
    assert!(harness.call(ApiCall::ListProjects).is_ok());
    assert!(matches!(
        harness.call(ApiCall::GetSession),
        Ok(ApiValue::CurrentSession(session)) if session.admin == "admin"
    ));

    let link = harness.login_link();
    for token in [None, Some("forged".to_owned()), Some(link)] {
        for call in [
            ApiCall::ListProjects,
            ApiCall::GetSession,
            ApiCall::ListChanges,
        ] {
            assert_eq!(
                harness
                    .call_with(token.clone(), call)
                    .map_err(|failure| failure.kind),
                Err(FailureKind::Unauthorized)
            );
        }
    }

    // A refused log stream is a reply, not a stream.
    let out = harness.input(Input::Api {
        client: brume_engine_core::ClientId(5),
        request: brume_proto::api::ApiRequest {
            id: 42,
            session: None,
            body: ApiCall::OpenLogStream {
                environment_id: "x".to_owned(),
                since_ms: None,
            },
        },
    });
    assert!(matches!(
        out.as_slice(),
        [brume_engine_core::Output::Reply { reply, .. }, ..]
            if reply.id == 42 && reply.result.as_ref().is_err_and(|f| f.kind == FailureKind::Unauthorized)
    ));

    // Logging out, or 12 hours, ends the session.
    harness.ctx.advance(12 * 60 * 60 * 1000);
    assert_eq!(
        harness
            .call_with(session, ApiCall::ListProjects)
            .map_err(|f| f.kind),
        Err(FailureKind::Unauthorized)
    );
    let mut fresh = Harness::new();
    assert_eq!(fresh.call(ApiCall::Logout), Ok(ApiValue::NoContent));
    assert_eq!(
        fresh.call(ApiCall::ListProjects).map_err(|f| f.kind),
        Err(FailureKind::Unauthorized)
    );
}

#[test]
fn deploy_flow_reaches_active() {
    let mut harness = Harness::new();
    let created = harness.create_project("nginx:1.29");
    assert_eq!(created.environment_name, "production");
    let service = harness.services()[0].clone();
    assert_eq!(service.slug, "nginx");
    assert_eq!(service.status, ServiceStatusDto::Deploying);
    let deployment = harness.deployments()[0].clone();
    assert_eq!(deployment.status, DeploymentStatusDto::Building);
    assert_eq!(deployment.image.as_deref(), Some("nginx:1.29"));

    let out = harness.connect_agent();
    assert_eq!(
        jobs(&out),
        vec![Job::ResolveImage {
            job_id: deployment.id.clone(),
            image: "nginx:1.29".to_owned(),
        }]
    );
    let state = last_desired(&out).expect("a desired state");
    assert!(state.containers.is_empty());
    assert_eq!(state.networks.len(), 1);
    assert_eq!(
        state.networks[0].name,
        format!("brume-env-{}", short_id(&service.environment_id))
    );
    assert!(state.networks[0].internal);

    // Resolved: the container starts without the alias.
    let out = harness.resolve(&deployment.id, &[]);
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers.len(), 1);
    let container = &state.containers[0];
    assert_eq!(
        container.name,
        format!("brume-nginx-{}", short_id(&deployment.id))
    );
    assert_eq!(container.image, format!("nginx@sha256:{DIGEST}"));
    assert!(container.networks[0].aliases.is_empty());
    assert_eq!(
        container.healthcheck,
        Healthcheck::Running { grace_secs: 5 }
    );
    assert_eq!(container.stop_timeout_secs, 10);
    assert_eq!((container.cpu_millis, container.memory_mb), (1000, 512));
    assert_eq!(
        harness.deployment(&deployment.id).status,
        DeploymentStatusDto::Deploying
    );

    // Healthy: it takes the alias.
    let out = harness.observe(&state, &[]);
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers[0].networks[0].aliases, vec!["nginx"]);

    // The agent reports the alias: the deployment is active.
    harness.observe(&state, &[]);
    assert_eq!(
        harness.deployment(&deployment.id).status,
        DeploymentStatusDto::Active
    );
    assert_eq!(
        harness.service(&service.id).status,
        ServiceStatusDto::Online
    );

    let logs = match harness.ok(ApiCall::DeploymentLogs {
        deployment_id: deployment.id.clone(),
        stream: LogStream::Deploy,
    }) {
        ApiValue::LogLines(lines) => lines,
        other => panic!("unexpected {other:?}"),
    };
    let messages = logs
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        [
            "Starting container",
            "Healthcheck passed",
            "Deployment active"
        ]
    );
    let build = match harness.ok(ApiCall::DeploymentLogs {
        deployment_id: deployment.id.clone(),
        stream: LogStream::Build,
    }) {
        ApiValue::LogLines(lines) => lines,
        other => panic!("unexpected {other:?}"),
    };
    let messages = build
        .iter()
        .map(|line| line.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        [
            "Resolving image nginx:1.29".to_owned(),
            format!("Resolved nginx@sha256:{DIGEST}"),
        ]
    );
}

#[test]
fn failed_healthcheck_keeps_previous_container() {
    let mut harness = Harness::new();
    let (service, environment, first) = active_project(&mut harness, &[]);
    stage(
        &mut harness,
        setting(&service, SettingField::Port, SettingValueDto::Number(80.0)),
    );
    stage(
        &mut harness,
        setting(
            &service,
            SettingField::HealthcheckPath,
            SettingValueDto::Text("/".to_owned()),
        ),
    );
    deploy_changes(&mut harness, &environment);
    let second = newest_deployment(&mut harness, &service);
    assert_eq!(
        harness.deployment(&second).status,
        DeploymentStatusDto::Deploying
    );

    let state = harness.desired_now();
    assert_eq!(state.containers.len(), 2);
    let new = state
        .containers
        .iter()
        .find(|container| container.deployment_id == second)
        .expect("the new container");
    assert_eq!(
        new.healthcheck,
        Healthcheck::Http {
            port: 80,
            path: "/".to_owned(),
            timeout_secs: 60
        }
    );
    assert!(new.networks[0].aliases.is_empty());

    // Unhealthy within the grace period: still deploying.
    let unhealthy = ContainerHealth::Unhealthy {
        reason: "connection refused".to_owned(),
    };
    harness.observe(&state, &[(second.as_str(), unhealthy.clone())]);
    assert_eq!(
        harness.deployment(&second).status,
        DeploymentStatusDto::Deploying
    );

    harness.ctx.advance(61_000);
    let out = harness.observe(&state, &[(second.as_str(), unhealthy)]);
    assert_eq!(
        harness.deployment(&second).status,
        DeploymentStatusDto::Failed
    );
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers.len(), 1);
    assert_eq!(state.containers[0].deployment_id, first);
    assert_eq!(state.containers[0].networks[0].aliases, vec!["nginx"]);
    assert_eq!(
        harness.deployment(&first).status,
        DeploymentStatusDto::Active
    );
    assert_eq!(harness.service(&service).status, ServiceStatusDto::Online);
}

#[test]
fn volume_service_stops_old_container_then_restores_it_on_failure() {
    let mut harness = Harness::new();
    let (service, _, first) = active_project(&mut harness, &["/data"]);
    let state = harness.desired_now();
    let volume = format!("brume-vol-{}-0", short_id(&service));
    assert_eq!(state.volumes.len(), 1);
    assert_eq!(state.volumes[0].name, volume);
    assert_eq!(state.containers[0].mounts[0].volume, volume);
    assert_eq!(state.containers[0].mounts[0].path, "/data");
    assert_eq!(state.containers[0].stop_timeout_secs, 60);
    assert!(matches!(
        &harness.service(&service).volume,
        Some(found) if found.name == volume && found.mount_path == "/data"
    ));

    harness.ok(ApiCall::Redeploy {
        service_id: service.clone(),
    });
    let second = newest_deployment(&mut harness, &service);
    let state = harness.desired_now();
    assert_eq!(state.containers.len(), 1, "the old container stops first");
    assert_eq!(state.containers[0].deployment_id, second);
    assert_eq!(state.containers[0].mounts[0].volume, volume);

    let out = harness.observe(
        &state,
        &[(second.as_str(), ContainerHealth::Exited { code: 1 })],
    );
    assert_eq!(
        harness.deployment(&second).status,
        DeploymentStatusDto::Failed
    );
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers.len(), 1, "the old container comes back");
    assert_eq!(state.containers[0].deployment_id, first);
    assert_eq!(state.containers[0].networks[0].aliases, vec!["nginx"]);
}

#[test]
fn reopen_resumes_resolving_deployment() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let deployment = harness.deployments()[0].clone();
    let created_at = Timestamp::parse_rfc3339(&deployment.created_at).expect("an instant");

    harness.reopen();
    let out = harness.connect_agent();
    assert_eq!(
        jobs(&out),
        vec![Job::ResolveImage {
            job_id: deployment.id.clone(),
            image: "nginx:1.29".to_owned(),
        }]
    );
    assert_eq!(
        wake_at(&out),
        Some(created_at.plus_millis(DEPLOYMENT_TIMEOUT_MILLIS))
    );
    // The agent's Hello asks again.
    let out = harness.agent(brume_proto::agent::AgentToEngine::Hello {
        version: "0.1.0".to_owned(),
        applied_position: None,
    });
    assert_eq!(jobs(&out).len(), 1);
    assert!(last_desired(&out).is_some());
}

#[test]
fn deployment_times_out_after_five_minutes() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let deployment = harness.deployments()[0].clone();
    let service = harness.services()[0].id.clone();
    harness.connect_agent();
    let deadline = Timestamp::parse_rfc3339(&deployment.created_at)
        .expect("an instant")
        .plus_millis(DEPLOYMENT_TIMEOUT_MILLIS);
    let deployment = deployment.id;

    // Each input comes one millisecond later: this tick is one before the deadline.
    harness.ctx.now = Timestamp::from_millis(deadline.as_millis() - 2);
    harness.input(Input::Tick);
    assert_eq!(
        harness.deployment(&deployment).status,
        DeploymentStatusDto::Building
    );
    harness.input(Input::Tick);
    assert_eq!(
        harness.deployment(&deployment).status,
        DeploymentStatusDto::Failed
    );
    assert_eq!(harness.service(&service).status, ServiceStatusDto::Crashed);

    // A late result changes nothing.
    harness.resolve(&deployment, &[]);
    assert_eq!(
        harness.deployment(&deployment).status,
        DeploymentStatusDto::Failed
    );
}

#[test]
fn tampered_event_breaks_chain() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let count = harness.storage.events().len();
    assert!(count >= 5);
    assert!(Engine::open(harness.storage.clone(), &mut harness.ctx).is_ok());

    harness.storage.tamper(3, 20);
    match Engine::open(harness.storage.clone(), &mut harness.ctx) {
        Err(OpenError::BrokenChain { seq }) => assert_eq!(seq, 3),
        Err(other) => panic!("unexpected error {other}"),
        Ok(_) => panic!("a tampered log opened"),
    }
}

#[test]
fn secret_values_never_in_events_or_listings() {
    let mut harness = Harness::new();
    let (service, environment, _) = active_project(&mut harness, &[]);
    stage(
        &mut harness,
        variable(&service, "API_TOKEN", VariableKindDto::Secret, Some(SECRET)),
    );
    let staged = serde_json::to_string(&harness.ok(ApiCall::ListChanges)).expect("serializes");
    assert!(staged.contains("API_TOKEN"));
    assert!(!staged.contains("s3cr3t"), "{staged}");

    deploy_changes(&mut harness, &environment);
    let deployment = newest_deployment(&mut harness, &service);
    let state = harness.desired_now();
    let container = state
        .containers
        .iter()
        .find(|container| container.deployment_id == deployment)
        .expect("the new container");
    let token = container
        .env
        .iter()
        .find(|env| env.name == "API_TOKEN")
        .expect("the secret is injected");
    match &token.value {
        EnvValue::Secret(value) => assert_eq!(value.expose_secret(), SECRET),
        EnvValue::Plain(_) => panic!("a secret is injected as a secret"),
    }

    // A shared secret, too.
    let project = harness.ok(ApiCall::ListProjects);
    let ApiValue::Projects(projects) = project else {
        panic!("unexpected {project:?}");
    };
    harness.ok(ApiCall::SetSharedVariable(NewSharedVariableDto {
        level: VariableLevelDto::Project,
        owner_id: projects[0].id.clone(),
        name: "STRIPE_KEY".to_owned(),
        kind: VariableKindDto::Secret,
        value: format!("shared-{SECRET}"),
    }));

    let listed = harness.ok(ApiCall::ListVariables);
    let ApiValue::Variables(variables) = &listed else {
        panic!("unexpected {listed:?}");
    };
    let listed = serde_json::to_string(&listed).expect("serializes");
    assert!(!listed.contains("s3cr3t"), "{listed}");
    let own = variables
        .iter()
        .find(|variable| variable.name == "API_TOKEN")
        .expect("listed");
    assert_eq!(own.value, None);
    let shared = variables
        .iter()
        .find(|variable| variable.name == "STRIPE_KEY")
        .expect("listed")
        .id
        .clone();

    assert!(matches!(
        harness.ok(ApiCall::RevealVariable { variable_id: own.id.clone() }),
        ApiValue::RevealedValue(revealed) if revealed.value == SECRET
    ));
    assert!(matches!(
        harness.ok(ApiCall::RevealVariable { variable_id: shared }),
        ApiValue::RevealedValue(revealed) if revealed.value == format!("shared-{SECRET}")
    ));

    let needle = "s3cr3t".as_bytes();
    let contains = |haystack: &[u8]| haystack.windows(needle.len()).any(|w| w == needle);
    let events = harness.storage.events();
    assert!(events.iter().all(|event| !contains(&event.bytes)));
    assert!(harness.storage.blobs().iter().all(|blob| !contains(blob)));
    let log = events
        .iter()
        .map(|event| String::from_utf8_lossy(&event.bytes).into_owned())
        .collect::<String>();
    assert!(log.contains(r#""type":"SecretRevealed""#));
    assert!(log.contains(r#""type":"SecretVersionAdded""#));
    let changes = serde_json::to_string(&harness.ok(ApiCall::ListChanges)).expect("serializes");
    assert!(!changes.contains("s3cr3t"));
}

#[test]
fn staging_same_target_replaces_and_reverting_removes() {
    let mut harness = Harness::new();
    let (service, environment, _) = active_project(&mut harness, &[]);

    stage(
        &mut harness,
        setting(
            &service,
            SettingField::Port,
            SettingValueDto::Number(8080.0),
        ),
    );
    let first = changes(&mut harness);
    assert_eq!(first.len(), 1);
    stage(
        &mut harness,
        setting(
            &service,
            SettingField::Port,
            SettingValueDto::Number(9090.0),
        ),
    );
    let second = changes(&mut harness);
    assert_eq!(second.len(), 1);
    assert!(matches!(
        (&first[0], &second[0]),
        (
            StagedChangeDto::Setting { id: a, .. },
            StagedChangeDto::Setting { id: b, value: SettingValueDto::Number(port), .. },
        ) if a == b && *port == 9090.0
    ));
    // Back to the applied value (no port): the change goes.
    stage(
        &mut harness,
        setting(
            &service,
            SettingField::Port,
            SettingValueDto::Text(String::new()),
        ),
    );
    assert!(changes(&mut harness).is_empty());

    stage(
        &mut harness,
        variable(&service, "GREETING", VariableKindDto::Plain, Some("hi")),
    );
    stage(
        &mut harness,
        variable(&service, "GREETING", VariableKindDto::Plain, Some("hello")),
    );
    assert!(matches!(
        changes(&mut harness).as_slice(),
        [StagedChangeDto::Variable { value: Some(value), .. }] if value == "hello"
    ));
    deploy_changes(&mut harness, &environment);
    assert!(changes(&mut harness).is_empty());

    stage(
        &mut harness,
        variable(&service, "GREETING", VariableKindDto::Plain, Some("hello")),
    );
    assert!(changes(&mut harness).is_empty(), "the applied value");
    stage(
        &mut harness,
        NewChangeDto::VariableRemoval {
            service_id: service.clone(),
            name: "GREETING".to_owned(),
        },
    );
    assert!(matches!(
        changes(&mut harness).as_slice(),
        [StagedChangeDto::VariableRemoval { .. }]
    ));
    stage(
        &mut harness,
        variable(&service, "GREETING", VariableKindDto::Plain, Some("hello")),
    );
    assert!(changes(&mut harness).is_empty(), "the removal is undone");

    // Discarding goes through the same table.
    stage(
        &mut harness,
        setting(&service, SettingField::Egress, SettingValueDto::Bool(true)),
    );
    harness.ok(ApiCall::DiscardEnvironmentChanges {
        environment_id: environment,
    });
    assert!(changes(&mut harness).is_empty());
}

#[test]
fn shared_plain_variable_redeploys_impacted_services() {
    let mut harness = Harness::new();
    let (production_service, _, _) = active_project(&mut harness, &[]);
    let ApiValue::Projects(projects) = harness.ok(ApiCall::ListProjects) else {
        panic!("projects");
    };
    let project = projects[0].id.clone();
    let ApiValue::Environment(staging) = harness.ok(ApiCall::CreateEnvironment {
        project_id: project.clone(),
        name: "staging".to_owned(),
    }) else {
        panic!("environment");
    };
    let ApiValue::Service(staging_service) = harness.ok(ApiCall::CreateService {
        environment_id: staging.id.clone(),
        image: "redis:8".to_owned(),
        name: None,
    }) else {
        panic!("service");
    };
    let deployment = newest_deployment(&mut harness, &staging_service.id);
    harness.activate(&deployment, &[]);

    let ApiValue::VariableImpact(impact) = harness.ok(ApiCall::VariableImpact {
        level: VariableLevelDto::Project,
        owner_id: project.clone(),
        name: "LOG_LEVEL".to_owned(),
    }) else {
        panic!("impact");
    };
    assert_eq!(impact.services.len(), 2);

    let ApiValue::SharedVariableSet(set) =
        harness.ok(ApiCall::SetSharedVariable(NewSharedVariableDto {
            level: VariableLevelDto::Project,
            owner_id: project,
            name: "LOG_LEVEL".to_owned(),
            kind: VariableKindDto::Plain,
            value: "debug".to_owned(),
        }))
    else {
        panic!("set");
    };
    assert_eq!(set.variable.value.as_deref(), Some("debug"));
    let mut redeployed = set
        .deployments
        .iter()
        .map(|deployment| deployment.service_id.clone())
        .collect::<Vec<_>>();
    redeployed.sort();
    let mut expected = vec![production_service, staging_service.id];
    expected.sort();
    assert_eq!(redeployed, expected);

    // Each new revision carries the value, with its origin.
    for deployment in &set.deployments {
        let state = harness.desired_now();
        let container = state
            .containers
            .iter()
            .find(|container| container.deployment_id == deployment.id)
            .expect("the new container starts");
        assert!(
            container
                .env
                .iter()
                .any(|env| env.name == "LOG_LEVEL"
                    && env.value == EnvValue::Plain("debug".to_owned()))
        );
    }
}

#[test]
fn docker_names_are_unique_for_ids_drawn_together() {
    let mut harness = Harness::new();
    let (nginx, production, _) = active_project(&mut harness, &[]);
    let ApiValue::Projects(projects) = harness.ok(ApiCall::ListProjects) else {
        panic!("projects");
    };
    let ApiValue::Environment(staging) = harness.ok(ApiCall::CreateEnvironment {
        project_id: projects[0].id.clone(),
        name: "staging".to_owned(),
    }) else {
        panic!("environment");
    };
    // Everything below is drawn within a few milliseconds: the ids share their clock prefix.
    for (environment, image) in [(&staging.id, "redis:8"), (&production, "postgres:17")] {
        let ApiValue::Service(service) = harness.ok(ApiCall::CreateService {
            environment_id: environment.clone(),
            image: image.to_owned(),
            name: None,
        }) else {
            panic!("service");
        };
        let deployment = newest_deployment(&mut harness, &service.id);
        harness.activate(&deployment, &["/data"]);
    }
    harness.ok(ApiCall::Redeploy { service_id: nginx });

    let state = harness.desired_now();
    let unique = |names: Vec<&String>| {
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len() == names.len()
    };
    assert_eq!(
        state.containers.len(),
        4,
        "two nginx, a redis and a postgres"
    );
    assert!(unique(state.containers.iter().map(|c| &c.name).collect()));
    assert_eq!(state.networks.len(), 2);
    assert!(unique(state.networks.iter().map(|n| &n.name).collect()));
    assert_eq!(state.volumes.len(), 2);
    assert!(unique(state.volumes.iter().map(|v| &v.name).collect()));
}

#[test]
fn failed_commit_changes_nothing() {
    let mut harness = Harness::new();
    let position = harness.engine.position();
    harness.storage.fail_next_commit();
    assert_eq!(
        harness
            .call(ApiCall::CreateProject {
                source: brume_proto::api::NewProjectSourceDto::Image {
                    image: "nginx:1.29".to_owned(),
                },
            })
            .map_err(|failure| failure.kind),
        Err(FailureKind::Internal)
    );
    assert_eq!(harness.engine.position(), position);
    assert!(matches!(harness.ok(ApiCall::ListProjects), ApiValue::Projects(p) if p.is_empty()));

    let (service, environment, _) = active_project(&mut harness, &[]);
    stage(
        &mut harness,
        variable(&service, "API_TOKEN", VariableKindDto::Secret, Some(SECRET)),
    );
    let deployments = harness.deployments().len();
    let position = harness.engine.position();
    harness.storage.fail_next_commit();
    assert_eq!(
        harness
            .call(ApiCall::DeployEnvironmentChanges {
                environment_id: environment.clone(),
            })
            .map_err(|failure| failure.kind),
        Err(FailureKind::Internal)
    );
    assert_eq!(harness.engine.position(), position);
    assert_eq!(changes(&mut harness).len(), 1, "the change stays staged");
    assert_eq!(harness.deployments().len(), deployments);

    // The retry writes version 1 of the secret: the failed attempt wrote nothing.
    deploy_changes(&mut harness, &environment);
    assert!(changes(&mut harness).is_empty());
    assert_eq!(harness.deployments().len(), deployments + 1);
    harness.reopen();
}

#[test]
fn accepted_log_stream_is_handed_to_the_shell() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let environment = harness.environments()[0].id.clone();
    let out = harness.input(Input::Api {
        client: brume_engine_core::ClientId(5),
        request: brume_proto::api::ApiRequest {
            id: 9,
            session: harness.session.clone(),
            body: ApiCall::OpenLogStream {
                environment_id: environment.clone(),
                since_ms: Some(1_000),
            },
        },
    });
    let stream = out
        .iter()
        .find_map(|output| match output {
            brume_engine_core::Output::OpenLogStream {
                client,
                request,
                query,
            } => Some((*client, *request, query.clone())),
            _ => None,
        })
        .expect("a stream to open");
    assert_eq!(stream.0, brume_engine_core::ClientId(5));
    assert_eq!(stream.1, 9);
    assert_eq!(stream.2.environment_id, environment);
    assert_eq!(stream.2.since_ms, Some(1_000));
    assert!(stream.2.follow);
    assert!(
        !out.iter()
            .any(|output| matches!(output, brume_engine_core::Output::Reply { .. })),
        "the shell writes the opening reply"
    );

    assert_eq!(
        harness
            .call(ApiCall::OpenLogStream {
                environment_id: "0190a3f2-7c1e-7b3a-9f00-1234567890ab".to_owned(),
                since_ms: None,
            })
            .map_err(|failure| failure.kind),
        Err(FailureKind::NotFound)
    );
}

#[test]
fn reopen_resumes_starting_deployment_and_times_out() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let deployment = harness.deployments()[0].clone();
    harness.connect_agent();
    harness.resolve(&deployment.id, &[]);
    assert_eq!(
        harness.deployment(&deployment.id).status,
        DeploymentStatusDto::Deploying
    );

    harness.reopen();
    let created_at = Timestamp::parse_rfc3339(&deployment.created_at).expect("an instant");
    let out = harness.input(Input::Tick);
    assert_eq!(
        wake_at(&out),
        Some(created_at.plus_millis(DEPLOYMENT_TIMEOUT_MILLIS))
    );
    harness.ctx.advance(DEPLOYMENT_TIMEOUT_MILLIS);
    harness.input(Input::Tick);
    assert_eq!(
        harness.deployment(&deployment.id).status,
        DeploymentStatusDto::Failed
    );
}

#[test]
fn failure_while_promoting_keeps_previous_container() {
    let mut harness = Harness::new();
    let (service, _, first) = active_project(&mut harness, &[]);
    harness.ok(ApiCall::Redeploy {
        service_id: service.clone(),
    });
    let second = newest_deployment(&mut harness, &service);
    let state = harness.desired_now();
    // Healthy, but the agent has not moved the alias yet.
    let out = harness.agent(brume_proto::agent::AgentToEngine::Observed(
        brume_proto::agent::ObservedState {
            containers: state
                .containers
                .iter()
                .map(|container| brume_proto::agent::ObservedContainer {
                    name: container.name.clone(),
                    deployment_id: container.deployment_id.clone(),
                    health: ContainerHealth::Healthy,
                    aliases: Vec::new(),
                    restarts: 0,
                })
                .collect(),
        },
    ));
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers.len(), 2);
    assert!(
        state
            .containers
            .iter()
            .all(|container| container.networks[0].aliases == ["nginx"])
    );

    let out = harness.observe(
        &state,
        &[(second.as_str(), ContainerHealth::Exited { code: 137 })],
    );
    assert_eq!(
        harness.deployment(&second).status,
        DeploymentStatusDto::Failed
    );
    let state = last_desired(&out).expect("a desired state");
    assert_eq!(state.containers.len(), 1);
    assert_eq!(state.containers[0].deployment_id, first);
}

#[test]
fn reveal_follows_a_reference_and_set_reports_its_users() {
    let mut harness = Harness::new();
    let (service, environment, _) = active_project(&mut harness, &[]);
    let ApiValue::Projects(projects) = harness.ok(ApiCall::ListProjects) else {
        panic!("projects");
    };
    let set_secret = |harness: &mut Harness, value: &str| match harness.ok(
        ApiCall::SetSharedVariable(NewSharedVariableDto {
            level: VariableLevelDto::Project,
            owner_id: projects[0].id.clone(),
            name: "DB_PASSWORD".to_owned(),
            kind: VariableKindDto::Secret,
            value: value.to_owned(),
        }),
    ) {
        ApiValue::SharedVariableSet(set) => set,
        other => panic!("unexpected {other:?}"),
    };
    let first = set_secret(&mut harness, "first-value");
    assert!(first.referenced_by.is_empty());
    assert!(first.deployments.is_empty(), "a secret redeploys nothing");

    stage(
        &mut harness,
        variable(&service, "DB_PASSWORD", VariableKindDto::Secret, None),
    );
    deploy_changes(&mut harness, &environment);
    let deployment = newest_deployment(&mut harness, &service);
    harness.activate(&deployment, &[]);

    let second = set_secret(&mut harness, "second-value");
    assert_eq!(second.variable.id, first.variable.id, "the same variable");
    assert_eq!(second.referenced_by, vec![service.clone()]);

    let ApiValue::Variables(variables) = harness.ok(ApiCall::ListVariables) else {
        panic!("variables");
    };
    let reference = variables
        .iter()
        .find(|variable| variable.owner_id == service && variable.name == "DB_PASSWORD")
        .expect("the reference");
    assert!(reference.reference);
    assert!(matches!(
        harness.ok(ApiCall::RevealVariable { variable_id: reference.id.clone() }),
        ApiValue::RevealedValue(revealed) if revealed.value == "second-value"
    ));
}

#[test]
fn read_only_engine_refuses_every_write() {
    let mut harness = Harness::new();
    let (service, environment, _) = active_project(&mut harness, &[]);
    let position = harness.engine.position();
    harness.engine.set_read_only();
    assert!(harness.engine.is_read_only());

    for call in [
        ApiCall::Redeploy {
            service_id: service.clone(),
        },
        ApiCall::StageChange(setting(
            &service,
            SettingField::Egress,
            SettingValueDto::Bool(true),
        )),
        ApiCall::DeployEnvironmentChanges {
            environment_id: environment.clone(),
        },
        ApiCall::Logout,
    ] {
        let failure = harness.call(call).expect_err("refused");
        assert_eq!(failure.kind, FailureKind::Conflict);
        assert_eq!(failure.message, "Another server writes to this bucket.");
    }
    // Reads still work, and agent reports change nothing.
    assert_eq!(harness.services().len(), 1);
    harness.ctx.advance(DEPLOYMENT_TIMEOUT_MILLIS);
    harness.input(Input::Tick);
    assert_eq!(harness.engine.position(), position);
}

#[test]
fn every_tick_rearms_the_next_deadline() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let deployment = harness.deployments()[0].clone();
    let deadline = Timestamp::parse_rfc3339(&deployment.created_at)
        .expect("an instant")
        .plus_millis(DEPLOYMENT_TIMEOUT_MILLIS);
    // The shell drops its timer when it fires: a tick that leaves the deadline must ask again.
    for _ in 0..2 {
        let out = harness.input(Input::Tick);
        assert_eq!(wake_at(&out), Some(deadline));
    }
    // Other inputs do not repeat it.
    let out = harness.input(Input::AgentDisconnected);
    assert_eq!(wake_at(&out), None);
}
