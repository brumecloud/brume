//! One test per call of the HTTP contract: the JSON keys of each answer are those of the panel's
//! types, `web/src/api/generated.ts`, plus the contract's additions. Then the edge cases.

mod fake_storage;
mod harness;

use std::collections::{BTreeMap, BTreeSet};

use brume_engine_core::{ClientId, Input, MAX_EVENTS_PAGE, Output};
use brume_proto::agent::{AgentToEngine, JobOutcome, MAX_FAILURE_MESSAGE_BYTES};
use brume_proto::api::{
    ApiCall, ApiRequest, ApiValue, DeploymentStatusDto, FailureKind, LogStream, NewChangeDto,
    NewProjectSourceDto, NewSharedVariableDto, ServiceStatusDto, SettingField, SettingValueDto,
    VariableKindDto, VariableLevelDto,
};
use serde_json::Value;

use crate::harness::{Harness, TUNNEL_TOKEN, last_desired};

/// The fields of each object type of the panel's generated types, `web/src/api/generated.ts`
/// (one `export type Name = { field: type, … };` per line, doc comments aside). Unions and
/// aliases are left out.
fn panel_interfaces() -> BTreeMap<String, BTreeSet<String>> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/api/generated.ts"
    );
    let source = std::fs::read_to_string(path).expect("web/src/api/generated.ts");
    let mut interfaces = BTreeMap::new();
    for declaration in without_comments(&source).split("export type ").skip(1) {
        let Some((name, body)) = declaration.split_once(" = ") else {
            continue;
        };
        let body = body.trim().trim_end_matches(';').trim();
        let Some(inner) = body
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
        else {
            continue;
        };
        if inner.contains('}') {
            // A union of objects.
            continue;
        }
        let fields = top_level_fields(inner)
            .into_iter()
            .filter_map(|field| field.split_once(':').map(|(key, _)| key.trim().to_owned()))
            .collect();
        interfaces.insert(name.trim().to_owned(), fields);
    }
    interfaces
}

/// `source` without its `/** … */` comments.
fn without_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/**") {
        out.push_str(&rest[..start]);
        let end = rest[start..].find("*/").expect("a closed comment");
        rest = &rest[start + end + 2..];
    }
    out.push_str(rest);
    out
}

/// The `field: type` entries of an object type's body, split on its top-level commas.
fn top_level_fields(inner: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0;
    for (index, c) in inner.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                fields.push(inner[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    fields.push(inner[start..].trim());
    fields
        .into_iter()
        .filter(|field| !field.is_empty())
        .collect()
}

/// The keys an object of the panel's `interface` must have, plus the contract's `added` ones.
fn expected(interface: &str, added: &[&str]) -> BTreeSet<String> {
    let mut keys = panel_interfaces()
        .remove(interface)
        .unwrap_or_else(|| panic!("generated.ts has no object type {interface}"));
    keys.extend(added.iter().map(|key| (*key).to_owned()));
    keys
}

fn set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("an object: {value}"))
        .keys()
        .cloned()
        .collect()
}

/// The JSON of the DTO inside an `ApiValue` (whose own JSON is `{"Variant": dto}`).
fn body(value: &ApiValue) -> Value {
    let json = serde_json::to_value(value).expect("serializes");
    match json {
        Value::Object(map) if map.len() == 1 => map.into_iter().next().expect("one").1,
        other => other,
    }
}

fn first(value: &Value) -> &Value {
    value
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or_else(|| panic!("a non-empty array: {value}"))
}

/// A project with an active `nginx` service, a plain and a secret variable, and a staging
/// environment: what the contract tests read.
struct World {
    harness: Harness,
    project: String,
    production: String,
    service: String,
    deployment: String,
}

fn world() -> World {
    let mut harness = Harness::new();
    let created = harness.create_project("nginx:1.29");
    let service = harness.services()[0].clone();
    let deployment = harness.deployments()[0].id.clone();
    harness.connect_agent();
    harness.activate(&deployment, &["/data"]);
    World {
        harness,
        project: created.project_id,
        production: service.environment_id,
        service: service.id,
        deployment,
    }
}

impl World {
    fn stage(&mut self, change: NewChangeDto) -> ApiValue {
        self.harness.ok(ApiCall::StageChange(change))
    }

    fn deploy(&mut self) -> String {
        self.harness.ok(ApiCall::DeployEnvironmentChanges {
            environment_id: self.production.clone(),
        });
        let deployment = self.newest(&self.service.clone());
        self.harness.activate(&deployment, &["/data"]);
        deployment
    }

    fn newest(&mut self, service: &str) -> String {
        self.harness
            .deployments()
            .into_iter()
            .filter(|deployment| deployment.service_id == service)
            .max_by(|a, b| (&a.created_at, &a.id).cmp(&(&b.created_at, &b.id)))
            .expect("a deployment")
            .id
    }

    fn set_shared(
        &mut self,
        level: VariableLevelDto,
        owner: &str,
        name: &str,
        kind: VariableKindDto,
        value: &str,
    ) -> ApiValue {
        self.harness
            .ok(ApiCall::SetSharedVariable(NewSharedVariableDto {
                level,
                owner_id: owner.to_owned(),
                name: name.to_owned(),
                kind,
                value: value.to_owned(),
            }))
    }
}

#[test]
fn session_calls() {
    let mut harness = Harness::unauthenticated();
    let token = harness.login_link();
    let session = harness.call(ApiCall::Login { token }).expect("logs in");
    let ApiValue::Session { token, admin } = &session else {
        panic!("unexpected {session:?}");
    };
    assert_eq!(admin, "admin");
    assert!(!token.is_empty());
    harness.session = Some(token.clone());

    let current = harness.ok(ApiCall::GetSession);
    assert_eq!(keys(&body(&current)), set(&["admin"]));
    assert_eq!(harness.ok(ApiCall::Logout), ApiValue::NoContent);
    assert_eq!(
        harness.failure(ApiCall::GetSession),
        FailureKind::Unauthorized
    );
}

#[test]
fn projects_calls() {
    let mut world = world();
    let projects = body(&world.harness.ok(ApiCall::ListProjects));
    assert_eq!(keys(first(&projects)), expected("Project", &[]));

    let created = body(&world.harness.ok(ApiCall::CreateProject {
        source: NewProjectSourceDto::Empty,
    }));
    assert_eq!(keys(&created), expected("CreatedProject", &[]));
    assert_eq!(created["environmentName"], "production");
    let names = body(&world.harness.ok(ApiCall::ListProjects));
    let names = names
        .as_array()
        .expect("projects")
        .iter()
        .map(|project| project["name"].as_str().expect("a name").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, ["nginx", "new-project"]);
    world.harness.create_project("nginx:1.30");
    let ApiValue::Projects(projects) = world.harness.ok(ApiCall::ListProjects) else {
        panic!("projects");
    };
    assert_eq!(projects[2].name, "nginx-2");

    for (source, kind) in [
        (
            NewProjectSourceDto::Github {
                repository: "org/app".to_owned(),
            },
            "github",
        ),
        (
            NewProjectSourceDto::Static {
                repository: "org/site".to_owned(),
            },
            "static",
        ),
        (NewProjectSourceDto::Postgres, "postgres"),
        (NewProjectSourceDto::Mongodb, "mongodb"),
        (NewProjectSourceDto::Redis, "redis"),
    ] {
        let failure = world
            .harness
            .call(ApiCall::CreateProject { source })
            .expect_err("refused");
        assert_eq!(failure.kind, FailureKind::Invalid);
        assert_eq!(
            failure.message,
            format!("{kind} sources are not available yet.")
        );
    }
}

#[test]
fn environments_calls() {
    let mut world = world();
    let environments = body(&world.harness.ok(ApiCall::ListEnvironments));
    assert_eq!(keys(first(&environments)), expected("Environment", &[]));

    let created = body(&world.harness.ok(ApiCall::CreateEnvironment {
        project_id: world.project.clone(),
        name: "staging".to_owned(),
    }));
    assert_eq!(keys(&created), expected("Environment", &[]));
    assert_eq!(created["name"], "staging");
    assert_eq!(created["projectId"], world.project.as_str());
    assert_eq!(
        world.harness.failure(ApiCall::CreateEnvironment {
            project_id: world.project.clone(),
            name: "Not A Label".to_owned(),
        }),
        FailureKind::Invalid
    );
    assert_eq!(
        world.harness.failure(ApiCall::CreateEnvironment {
            project_id: world.project.clone(),
            name: "staging".to_owned(),
        }),
        FailureKind::Conflict
    );
}

#[test]
fn services_calls() {
    let mut world = world();
    let services = body(&world.harness.ok(ApiCall::ListServices));
    let service = first(&services);
    assert_eq!(keys(service), expected("Service", &[]));
    assert_eq!(keys(&service["volume"]), expected("Volume", &[]));
    assert_eq!(service["kind"], "image");
    assert_eq!(service["status"], "online");

    let created = body(&world.harness.ok(ApiCall::CreateService {
        environment_id: world.production.clone(),
        image: "redis:8".to_owned(),
        name: None,
    }));
    assert_eq!(keys(&created), expected("Service", &[]));
    assert_eq!(created["status"], "deploying");
    assert_eq!(created["volume"], Value::Null);

    let settings = body(&world.harness.ok(ApiCall::GetServiceSettings {
        service_id: world.service.clone(),
    }));
    assert_eq!(keys(&settings), expected("ServiceSettings", &[]));
    assert_eq!(
        settings["source"],
        serde_json::json!({"kind": "image", "image": "nginx:1.29"})
    );
    assert_eq!(settings["cpuLimit"], 1.0);
    assert_eq!(settings["memoryLimitMb"], 512);
    assert_eq!(settings["egress"], false);

    let redis = created["id"].as_str().expect("an id").to_owned();
    assert_eq!(
        world.harness.ok(ApiCall::DeleteService {
            service_id: redis.clone()
        }),
        ApiValue::NoContent
    );
    assert_eq!(
        world
            .harness
            .failure(ApiCall::GetServiceSettings { service_id: redis }),
        FailureKind::NotFound
    );
}

#[test]
fn deployments_calls() {
    let mut world = world();
    let deployments = body(&world.harness.ok(ApiCall::ListDeployments));
    let deployment = first(&deployments);
    assert_eq!(keys(deployment), expected("Deployment", &[]));
    assert_eq!(deployment["status"], "active");
    assert_eq!(deployment["image"], "nginx:1.29");
    assert_eq!(deployment["commit"], Value::Null);
    assert_eq!(deployment["author"], "admin");

    let redeployed = body(&world.harness.ok(ApiCall::Redeploy {
        service_id: world.service.clone(),
    }));
    assert_eq!(keys(&redeployed), expected("Deployment", &[]));
    assert_eq!(redeployed["status"], "deploying");
    let second = redeployed["id"].as_str().expect("an id").to_owned();
    world.harness.activate(&second, &["/data"]);
    assert!(
        world
            .harness
            .deployment(&world.deployment.clone())
            .rollbackable
    );

    let rolled_back = body(&world.harness.ok(ApiCall::Rollback {
        deployment_id: world.deployment.clone(),
    }));
    assert_eq!(keys(&rolled_back), expected("Deployment", &[]));

    let revision = body(&world.harness.ok(ApiCall::GetRevision {
        deployment_id: world.deployment.clone(),
    }));
    assert_eq!(keys(&revision), expected("Revision", &[]));
    assert_eq!(revision["builder"], Value::Null);

    for stream in [LogStream::Build, LogStream::Deploy] {
        let lines = body(&world.harness.ok(ApiCall::DeploymentLogs {
            deployment_id: world.deployment.clone(),
            stream,
        }));
        assert_eq!(keys(first(&lines)), expected("LogLine", &[]));
    }

    // While the image resolves, the deployment is `building`.
    world.harness.create_project("redis:8");
    let building = body(&world.harness.ok(ApiCall::ListDeployments));
    assert!(
        building
            .as_array()
            .expect("deployments")
            .iter()
            .any(
                |deployment| deployment["status"] == "building" && deployment["image"] == "redis:8"
            )
    );
}

#[test]
fn variables_calls() {
    let mut world = world();
    let project = world.project.clone();
    let service = world.service.clone();
    world.set_shared(
        VariableLevelDto::Project,
        &project,
        "LOG_LEVEL",
        VariableKindDto::Plain,
        "info",
    );
    let redeploy = world.newest(&service);
    world.harness.activate(&redeploy, &["/data"]);
    world.set_shared(
        VariableLevelDto::Project,
        &project,
        "DB_PASSWORD",
        VariableKindDto::Secret,
        "pw",
    );
    world.stage(NewChangeDto::Variable {
        service_id: service.clone(),
        name: "GREETING".to_owned(),
        kind: VariableKindDto::Build,
        value: Some("hi".to_owned()),
    });
    world.stage(NewChangeDto::Variable {
        service_id: service.clone(),
        name: "DB_PASSWORD".to_owned(),
        kind: VariableKindDto::Secret,
        value: None,
    });
    let deployment = world.deploy();

    let variables = body(&world.harness.ok(ApiCall::ListVariables));
    let variable_keys = expected("Variable", &["reference"]);
    for variable in variables.as_array().expect("variables") {
        assert_eq!(keys(variable), variable_keys);
    }
    let reference = variables
        .as_array()
        .expect("variables")
        .iter()
        .find(|variable| variable["level"] == "service" && variable["name"] == "DB_PASSWORD")
        .expect("the reference");
    assert_eq!(reference["reference"], true);
    for variable in variables.as_array().expect("variables") {
        if variable != reference {
            assert_eq!(variable["reference"], false, "{variable}");
        }
    }
    assert_eq!(reference["value"], Value::Null);

    let revision = body(&world.harness.ok(ApiCall::GetRevision {
        deployment_id: deployment,
    }));
    let revision_variables = revision["variables"].as_array().expect("variables");
    assert_eq!(
        keys(first(&revision["variables"])),
        expected("RevisionVariable", &[])
    );
    let origins = revision_variables
        .iter()
        .map(|variable| {
            (
                variable["name"].as_str().expect("a name"),
                variable["origin"].as_str().expect("an origin"),
                variable["kind"].as_str().expect("a kind"),
                variable["value"].clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        origins,
        [
            ("DB_PASSWORD", "project", "secret", Value::Null),
            ("GREETING", "service", "build", Value::from("hi")),
            ("LOG_LEVEL", "project", "plain", Value::from("info")),
        ]
    );

    let impact = body(&world.harness.ok(ApiCall::VariableImpact {
        level: VariableLevelDto::Environment,
        owner_id: world.production.clone(),
        name: "LOG_LEVEL".to_owned(),
    }));
    assert_eq!(keys(&impact), set(&["services"]));
    assert_eq!(
        keys(first(&impact["services"])),
        set(&["serviceId", "environmentId"])
    );

    let set_plain = body(&world.set_shared(
        VariableLevelDto::Environment,
        &world.production.clone(),
        "LOG_LEVEL",
        VariableKindDto::Plain,
        "debug",
    ));
    assert_eq!(
        keys(&set_plain),
        set(&["variable", "deployments", "referencedBy"])
    );
    assert_eq!(keys(&set_plain["variable"]), variable_keys);
    assert_eq!(
        keys(first(&set_plain["deployments"])),
        expected("Deployment", &[])
    );
    let environment_variable = set_plain["variable"]["id"]
        .as_str()
        .expect("an id")
        .to_owned();
    let redeploy = set_plain["deployments"][0]["id"]
        .as_str()
        .expect("an id")
        .to_owned();
    world.harness.activate(&redeploy, &["/data"]);

    let secret = variables
        .as_array()
        .expect("variables")
        .iter()
        .find(|variable| variable["level"] == "project" && variable["name"] == "DB_PASSWORD")
        .expect("the shared secret")["id"]
        .as_str()
        .expect("an id")
        .to_owned();
    let revealed = body(&world.harness.ok(ApiCall::RevealVariable {
        variable_id: secret.clone(),
    }));
    assert_eq!(keys(&revealed), set(&["value"]));
    assert_eq!(revealed["value"], "pw");

    let refused = world
        .harness
        .call(ApiCall::RemoveSharedVariable {
            variable_id: secret,
        })
        .expect_err("still used");
    assert_eq!(refused.kind, FailureKind::Conflict);
    assert_eq!(refused.message, "DB_PASSWORD is still used by: nginx.");

    let removed = body(&world.harness.ok(ApiCall::RemoveSharedVariable {
        variable_id: environment_variable,
    }));
    assert_eq!(keys(&removed), set(&["deployments"]));
    assert_eq!(
        keys(first(&removed["deployments"])),
        expected("Deployment", &[])
    );
}

#[test]
fn changes_calls() {
    let mut world = world();
    let service = world.service.clone();
    let setting = body(&world.stage(NewChangeDto::Setting {
        service_id: service.clone(),
        field: SettingField::CpuLimit,
        value: SettingValueDto::Number(0.5),
    }));
    let target = ["id", "environmentId", "serviceId", "type"];
    let mut setting_keys = set(&target);
    setting_keys.extend(set(&["field", "value"]));
    assert_eq!(keys(&setting), setting_keys);
    assert_eq!(setting["type"], "setting");
    assert_eq!(setting["field"], "cpuLimit");

    world
        .harness
        .ok(ApiCall::SetSharedVariable(NewSharedVariableDto {
            level: VariableLevelDto::Project,
            owner_id: world.project.clone(),
            name: "DB_PASSWORD".to_owned(),
            kind: VariableKindDto::Secret,
            value: "pw".to_owned(),
        }));
    let reference = body(&world.stage(NewChangeDto::Variable {
        service_id: service.clone(),
        name: "DB_PASSWORD".to_owned(),
        kind: VariableKindDto::Secret,
        value: None,
    }));
    let mut variable_keys = set(&target);
    variable_keys.extend(set(&["name", "kind", "value", "reference"]));
    assert_eq!(keys(&reference), variable_keys);
    assert_eq!(reference["type"], "variable");
    assert_eq!(reference["value"], Value::Null);
    assert_eq!(reference["reference"], true);

    let removal = body(&world.stage(NewChangeDto::VariableRemoval {
        service_id: service.clone(),
        name: "DB_PASSWORD".to_owned(),
    }));
    let mut removal_keys = set(&target);
    removal_keys.insert("name".to_owned());
    assert_eq!(keys(&removal), removal_keys);
    assert_eq!(removal["type"], "variable-removal");
    let listed = body(&world.harness.ok(ApiCall::ListChanges));
    assert_eq!(
        listed.as_array().expect("changes").len(),
        1,
        "the removal undid the reference"
    );

    let id = setting["id"].as_str().expect("an id").to_owned();
    assert_eq!(
        world.harness.ok(ApiCall::DiscardChange {
            change_id: id.clone()
        }),
        ApiValue::NoContent
    );
    assert_eq!(
        world
            .harness
            .failure(ApiCall::DiscardChange { change_id: id }),
        FailureKind::NotFound
    );

    world.stage(NewChangeDto::Setting {
        service_id: service.clone(),
        field: SettingField::MemoryLimitMb,
        value: SettingValueDto::Number(256.0),
    });
    world.stage(NewChangeDto::Setting {
        service_id: service.clone(),
        field: SettingField::Egress,
        value: SettingValueDto::Bool(true),
    });
    let deployed = body(&world.harness.ok(ApiCall::DeployEnvironmentChanges {
        environment_id: world.production.clone(),
    }));
    assert_eq!(keys(&deployed), set(&["deployed"]));
    assert_eq!(deployed["deployed"], 2);
    let deployment = world.newest(&service);
    world.harness.activate(&deployment, &["/data"]);

    world.stage(NewChangeDto::Setting {
        service_id: service,
        field: SettingField::Port,
        value: SettingValueDto::Number(8080.0),
    });
    assert_eq!(
        world.harness.ok(ApiCall::DiscardEnvironmentChanges {
            environment_id: world.production.clone(),
        }),
        ApiValue::NoContent
    );
    assert!(matches!(
        world.harness.ok(ApiCall::ListChanges),
        ApiValue::Changes(changes) if changes.is_empty()
    ));
}

#[test]
fn settings_are_validated_when_staged() {
    let mut world = world();
    let service = world.service.clone();
    for (field, value) in [
        (
            SettingField::Domains,
            SettingValueDto::List(vec!["a.com".to_owned()]),
        ),
        (
            SettingField::Repository,
            SettingValueDto::Text("org/app".to_owned()),
        ),
        (SettingField::Port, SettingValueDto::Number(0.0)),
        (SettingField::Port, SettingValueDto::Number(80.5)),
        (SettingField::CpuLimit, SettingValueDto::Number(64.0)),
        (SettingField::MemoryLimitMb, SettingValueDto::Number(8.0)),
        (
            SettingField::Image,
            SettingValueDto::Text("Not An Image".to_owned()),
        ),
        (
            SettingField::Egress,
            SettingValueDto::Text("yes".to_owned()),
        ),
        (
            SettingField::StartCommand,
            SettingValueDto::Text("echo 'open".to_owned()),
        ),
    ] {
        let failure = world
            .harness
            .call(ApiCall::StageChange(NewChangeDto::Setting {
                service_id: service.clone(),
                field,
                value: value.clone(),
            }))
            .expect_err("refused");
        assert_eq!(failure.kind, FailureKind::Invalid, "{field:?} {value:?}");
    }
    assert!(matches!(
        world.harness.ok(ApiCall::ListChanges),
        ApiValue::Changes(changes) if changes.is_empty()
    ));
}

/// Stages the domain `value` on `service`.
fn stage_domain(
    harness: &mut Harness,
    service: &str,
    value: &str,
) -> Result<ApiValue, brume_proto::api::ApiFailure> {
    harness.call(ApiCall::StageChange(NewChangeDto::Setting {
        service_id: service.to_owned(),
        field: SettingField::Domain,
        value: SettingValueDto::Text(value.to_owned()),
    }))
}

fn staged_count(harness: &mut Harness) -> usize {
    match harness.ok(ApiCall::ListChanges) {
        ApiValue::Changes(changes) => changes.len(),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn ingress_call() {
    let mut world = world();
    assert_eq!(body(&world.harness.ok(ApiCall::GetIngress)), Value::Null);
    world.harness.configure_ingress("example.com", TUNNEL_TOKEN);
    let ingress = body(&world.harness.ok(ApiCall::GetIngress));
    assert_eq!(keys(&ingress), expected("Ingress", &[]));
    assert_eq!(ingress["zone"], "example.com");
    assert!(!ingress.to_string().contains(TUNNEL_TOKEN));
    let settings = body(&world.harness.ok(ApiCall::GetServiceSettings {
        service_id: world.service.clone(),
    }));
    assert_eq!(settings["domain"], Value::Null);
}

#[test]
fn staging_a_domain_requires_ingress() {
    // Given a server without a public entry,
    let mut world = world();
    let service = world.service.clone();

    // when a domain is staged, it is refused with how to set the entry up, and nothing is
    // staged.
    let failure = stage_domain(&mut world.harness, &service, "demo").expect_err("refused");
    assert_eq!(failure.kind, FailureKind::Conflict);
    assert!(
        failure.message.contains("brume install --domain"),
        "{}",
        failure.message
    );
    assert_eq!(staged_count(&mut world.harness), 0);
}

#[test]
fn invalid_subdomains_rejected() {
    // Given a server with a public entry,
    let mut world = world();
    world.harness.configure_ingress("example.com", TUNNEL_TOKEN);
    let service = world.service.clone();

    // when an invalid subdomain is staged, it is refused as invalid, and nothing is staged.
    let too_long = "a".repeat(64);
    for invalid in [
        "Demo",
        "-x",
        "a.b",
        too_long.as_str(),
        "demo.example.com",
        " demo",
    ] {
        let failure = stage_domain(&mut world.harness, &service, invalid).expect_err(invalid);
        assert_eq!(failure.kind, FailureKind::Invalid, "{invalid:?}");
        assert!(failure.message.contains("subdomain"), "{}", failure.message);
    }
    assert_eq!(staged_count(&mut world.harness), 0);
    // The longest label passes.
    assert!(stage_domain(&mut world.harness, &service, &"a".repeat(63)).is_ok());
}

#[test]
fn duplicate_subdomain_rejected() {
    // Given `demo` deployed on a service of one project, and a service of another project,
    let mut world = world();
    world.harness.configure_ingress("example.com", TUNNEL_TOKEN);
    let service = world.service.clone();
    world.stage(NewChangeDto::Setting {
        service_id: service.clone(),
        field: SettingField::Port,
        value: SettingValueDto::Number(80.0),
    });
    stage_domain(&mut world.harness, &service, "demo").expect("staged");
    world.deploy();
    world.harness.create_project("redis:8");
    let other = world
        .harness
        .services()
        .into_iter()
        .find(|candidate| candidate.id != service)
        .expect("the other service");

    // when the other one stages `demo`, it is refused, and nothing is staged;
    let failure = stage_domain(&mut world.harness, &other.id, "demo").expect_err("refused");
    assert_eq!(failure.kind, FailureKind::Conflict);
    assert!(failure.message.contains("demo"), "{}", failure.message);
    assert_eq!(staged_count(&mut world.harness), 0);

    // a subdomain staged by a service, not deployed yet, is taken too;
    stage_domain(&mut world.harness, &service, "shop").expect("staged");
    let failure = stage_domain(&mut world.harness, &other.id, "shop").expect_err("refused");
    assert_eq!(failure.kind, FailureKind::Conflict);
    assert_eq!(staged_count(&mut world.harness), 1);

    // and a free one passes.
    assert!(stage_domain(&mut world.harness, &other.id, "blog").is_ok());
}

#[test]
fn environment_log_stream_call() {
    let mut world = world();
    let out = world.harness.input(Input::Api {
        client: ClientId(3),
        request: ApiRequest {
            id: 77,
            session: world.harness.session.clone(),
            body: ApiCall::OpenLogStream {
                environment_id: world.production.clone(),
                since_ms: None,
            },
        },
    });
    assert!(matches!(
        out.as_slice(),
        [Output::OpenLogStream { client: ClientId(3), request: 77, query }, ..]
            if query.environment_id == world.production && query.follow
    ));
}

#[test]
fn events_call() {
    let mut world = world();
    let page = body(&world.harness.ok(ApiCall::ListEvents {
        project_id: world.project.clone(),
        before: None,
        limit: 50,
    }));
    assert_eq!(keys(&page), set(&["events", "nextBefore"]));
    assert_eq!(
        keys(first(&page["events"])),
        set(&[
            "seq",
            "at",
            "type",
            "actor",
            "environmentId",
            "serviceId",
            "deploymentId",
            "summary"
        ])
    );
    assert_eq!(page["nextBefore"], Value::Null);
    for limit in [0, MAX_EVENTS_PAGE + 1] {
        assert_eq!(
            world.harness.failure(ApiCall::ListEvents {
                project_id: world.project.clone(),
                before: None,
                limit,
            }),
            FailureKind::Invalid
        );
    }
}

#[test]
fn image_resolution_failure_is_shown_and_previous_keeps_running() {
    let mut world = world();
    let service = world.service.clone();
    world.stage(NewChangeDto::Setting {
        service_id: service.clone(),
        field: SettingField::Image,
        value: SettingValueDto::Text("nginx:9.99".to_owned()),
    });
    world.harness.ok(ApiCall::DeployEnvironmentChanges {
        environment_id: world.production.clone(),
    });
    let failed = world.newest(&service);
    assert_eq!(
        world.harness.deployment(&failed).status,
        DeploymentStatusDto::Building
    );

    // `é` straddles byte 1024: the cut falls back to the character boundary before it.
    let docker = format!(
        "manifest unknown: {}é{}",
        "x".repeat(1005),
        "y".repeat(2000)
    );
    let out = world.harness.agent(AgentToEngine::JobResult {
        job_id: failed.clone(),
        outcome: JobOutcome::Failed {
            message: docker.clone(),
        },
    });
    assert_eq!(
        world.harness.deployment(&failed).status,
        DeploymentStatusDto::Failed
    );

    let ApiValue::LogLines(lines) = world.harness.ok(ApiCall::DeploymentLogs {
        deployment_id: failed,
        stream: LogStream::Deploy,
    }) else {
        panic!("lines");
    };
    let line = lines.last().expect("the failure");
    assert_eq!(line.level, brume_proto::api::LogLevel::Error);
    let message = line
        .message
        .strip_prefix("Image resolution failed: ")
        .expect("the failure line");
    assert_eq!(message.len(), MAX_FAILURE_MESSAGE_BYTES - 1);
    assert!(message.ends_with('x'));
    assert!(docker.starts_with(message));

    // The active container keeps running, with its alias.
    let state = last_desired(&out).unwrap_or_else(|| world.harness.desired_now());
    assert_eq!(state.containers.len(), 1);
    assert_eq!(state.containers[0].deployment_id, world.deployment);
    assert_eq!(state.containers[0].networks[0].aliases, vec!["nginx"]);
    assert_eq!(
        world.harness.service(&service).status,
        ServiceStatusDto::Online
    );
}

#[test]
fn delete_service_during_deployment() {
    let mut harness = Harness::new();
    let created = harness.create_project("nginx:1.29");
    let service = harness.services()[0].id.clone();
    let deployment = harness.deployments()[0].id.clone();
    harness.connect_agent();
    assert_eq!(
        harness.ok(ApiCall::DeleteService {
            service_id: service.clone()
        }),
        ApiValue::NoContent
    );
    assert!(harness.services().is_empty());
    assert!(harness.deployments().is_empty());

    // A late result changes nothing, and nothing runs.
    let out = harness.resolve(&deployment, &[]);
    assert!(last_desired(&out).is_none_or(|state| state.containers.is_empty()));
    assert!(harness.desired_now().containers.is_empty());

    let ApiValue::Events(page) = harness.ok(ApiCall::ListEvents {
        project_id: created.project_id,
        before: None,
        limit: 10,
    }) else {
        panic!("events");
    };
    let kinds = page
        .events
        .iter()
        .map(|event| event.kind.as_str())
        .collect::<Vec<_>>();
    assert_eq!(&kinds[..2], ["ServiceDeleted", "DeploymentFailed"]);
    assert_eq!(
        page.events[1].summary,
        "The deployment of nginx failed: the service was deleted."
    );
}

#[test]
fn create_service_derives_unique_slug() {
    let mut harness = Harness::new();
    harness.create_project("nginx:1.29");
    let environment = harness.services()[0].environment_id.clone();
    let mut create = |image: &str, name: Option<&str>| match harness.ok(ApiCall::CreateService {
        environment_id: environment.clone(),
        image: image.to_owned(),
        name: name.map(ToOwned::to_owned),
    }) {
        ApiValue::Service(service) => (service.name, service.slug),
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(
        create("nginx:1.29", None),
        ("nginx".to_owned(), "nginx-2".to_owned())
    );
    assert_eq!(
        create("nginx:1.29", None),
        ("nginx".to_owned(), "nginx-3".to_owned())
    );
    assert_eq!(
        create("ghcr.io/org/my_app:v1", None),
        ("my_app".to_owned(), "my-app".to_owned())
    );
    assert_eq!(
        create("redis:8", Some("My API (v2)")),
        ("My API (v2)".to_owned(), "my-api-v2".to_owned())
    );
    assert_eq!(
        create("redis:8", Some("💥")),
        ("💥".to_owned(), "service".to_owned())
    );

    let long = "a".repeat(64);
    let (_, first) = create("redis:8", Some(&long));
    let (_, second) = create("redis:8", Some(&long));
    assert_eq!(first, "a".repeat(63));
    assert_eq!(second, format!("{}-2", "a".repeat(61)));
}

#[test]
fn impact_lists_services_across_environments() {
    let mut world = world();
    let ApiValue::Environment(staging) = world.harness.ok(ApiCall::CreateEnvironment {
        project_id: world.project.clone(),
        name: "staging".to_owned(),
    }) else {
        panic!("environment");
    };
    let ApiValue::Service(staging_service) = world.harness.ok(ApiCall::CreateService {
        environment_id: staging.id.clone(),
        image: "nginx:1.29".to_owned(),
        name: None,
    }) else {
        panic!("service");
    };
    let impact = |world: &mut World, level: VariableLevelDto, owner: &str| match world.harness.ok(
        ApiCall::VariableImpact {
            level,
            owner_id: owner.to_owned(),
            name: "API_URL".to_owned(),
        },
    ) {
        ApiValue::VariableImpact(impact) => impact
            .services
            .into_iter()
            .map(|service| (service.service_id, service.environment_id))
            .collect::<BTreeSet<_>>(),
        other => panic!("unexpected {other:?}"),
    };
    let project = world.project.clone();
    let both = BTreeSet::from([
        (world.service.clone(), world.production.clone()),
        (staging_service.id.clone(), staging.id.clone()),
    ]);
    assert_eq!(
        impact(&mut world, VariableLevelDto::Project, &project),
        both
    );
    assert_eq!(
        impact(&mut world, VariableLevelDto::Environment, &staging.id),
        BTreeSet::from([(staging_service.id.clone(), staging.id.clone())])
    );

    // Staging overrides it: a project value no longer reaches staging.
    let deployment = world.newest(&staging_service.id);
    world.harness.activate(&deployment, &[]);
    world.set_shared(
        VariableLevelDto::Environment,
        &staging.id,
        "API_URL",
        VariableKindDto::Plain,
        "https://staging",
    );
    assert_eq!(
        impact(&mut world, VariableLevelDto::Project, &project),
        BTreeSet::from([(world.service.clone(), world.production.clone())])
    );
    assert_eq!(
        world.harness.failure(ApiCall::VariableImpact {
            level: VariableLevelDto::Service,
            owner_id: world.service.clone(),
            name: "API_URL".to_owned(),
        }),
        FailureKind::Invalid
    );
}

#[test]
fn events_page_paginates_and_hides_values() {
    let mut world = world();
    let project = world.project.clone();
    let service = world.service.clone();
    world.set_shared(
        VariableLevelDto::Project,
        &project,
        "API_URL",
        VariableKindDto::Plain,
        "plain-value-🙂",
    );
    let redeploy = world.newest(&service);
    world.harness.activate(&redeploy, &["/data"]);
    world.set_shared(
        VariableLevelDto::Project,
        &project,
        "STRIPE_KEY",
        VariableKindDto::Secret,
        "s3cr3t-🚀",
    );
    world.stage(NewChangeDto::Variable {
        service_id: service.clone(),
        name: "TOKEN".to_owned(),
        kind: VariableKindDto::Secret,
        value: Some("s3cr3t-own".to_owned()),
    });
    world.deploy();
    let ApiValue::Variables(variables) = world.harness.ok(ApiCall::ListVariables) else {
        panic!("variables");
    };
    let token = variables
        .iter()
        .find(|v| v.name == "TOKEN")
        .expect("listed")
        .id
        .clone();
    world
        .harness
        .ok(ApiCall::RevealVariable { variable_id: token });
    // Another project's events stay out.
    world.harness.create_project("redis:8");

    let mut events = Vec::new();
    let mut before = None;
    let mut pages = 0;
    loop {
        pages += 1;
        let ApiValue::Events(page) = world.harness.ok(ApiCall::ListEvents {
            project_id: project.clone(),
            before,
            limit: 4,
        }) else {
            panic!("events");
        };
        assert!(page.events.len() <= 4);
        let full = page.events.len() == 4;
        events.extend(page.events);
        match page.next_before {
            Some(next) => {
                assert!(full, "only the last page is short");
                before = Some(next);
            }
            None => break,
        }
    }
    assert!(pages > 2, "{pages} pages");
    let ApiValue::Events(all) = world.harness.ok(ApiCall::ListEvents {
        project_id: project.clone(),
        before: None,
        limit: MAX_EVENTS_PAGE,
    }) else {
        panic!("events");
    };
    assert_eq!(all.events.len(), events.len());
    assert_eq!(all.next_before, None);
    let seqs = events.iter().map(|event| event.seq).collect::<Vec<_>>();
    let mut sorted = seqs.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    sorted.dedup();
    assert_eq!(seqs, sorted, "newest first, without duplicates");
    assert_eq!(
        events.last().map(|event| event.kind.as_str()),
        Some("ProjectCreated")
    );
    assert!(events.iter().all(|event| !event.summary.contains("redis")));
    let kinds = events
        .iter()
        .map(|event| event.kind.as_str())
        .collect::<BTreeSet<_>>();
    for kind in [
        "ProjectCreated",
        "EnvironmentCreated",
        "ServiceCreated",
        "DeploymentRequested",
        "RevisionCreated",
        "DeploymentStarted",
        "DeploymentHealthy",
        "DeploymentSucceeded",
        "SharedVariableSet",
        "ServiceSpecChanged",
        "SecretVersionAdded",
        "SecretRevealed",
    ] {
        assert!(kinds.contains(kind), "{kind} is listed");
    }
    assert!(
        !kinds.contains("LoginSucceeded"),
        "audit of sessions is not a project's"
    );
    let text = serde_json::to_string(&events).expect("serializes");
    for value in ["plain-value", "s3cr3t"] {
        assert!(!text.contains(value), "{value} leaked: {text}");
    }
    let revealed = events
        .iter()
        .find(|event| event.kind == "SecretRevealed")
        .expect("revealed");
    assert_eq!(revealed.summary, "Revealed secret TOKEN of nginx.");
    assert_eq!(revealed.actor, "admin");
    assert_eq!(revealed.service_id.as_deref(), Some(service.as_str()));
}
