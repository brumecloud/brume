//! The calls from `brume api` to the engine on `api.sock`, and the DTOs of the HTTP API.
//!
//! Each DTO has the JSON shape of its type in `web/src/api/types.ts`, plus the additions of the
//! HTTP contract: its name with a `Dto` suffix, camelCase fields, and the same tag key and values
//! for unions. Identifiers and instants travel as strings, instants in RFC 3339. With the `ts`
//! feature, the DTOs derive `ts_rs::TS` under their `types.ts` names.

use serde::{Deserialize, Serialize};

/// A call from `brume api` to the engine. On a multiplexed connection, `id` pairs it with its
/// `ApiReply`. Its `Debug` masks the session token.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiRequest {
    /// Increases with each request on the connection.
    pub id: u64,
    /// The session token of the `brume_session` cookie, if the request has one.
    pub session: Option<String>,
    /// What the panel asks for.
    pub body: ApiCall,
}

/// The engine's answer to the `ApiRequest` with the same `id`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiReply {
    /// The `id` of the request.
    pub id: u64,
    /// The value, or why the call failed.
    pub result: Result<ApiValue, ApiFailure>,
}

/// A failed call. `brume api` maps the kind to an HTTP status and sends `{"message": …}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiFailure {
    /// The class of failure.
    pub kind: FailureKind,
    /// An English sentence for the panel. It never contains a secret value.
    pub message: String,
}

/// The class of a failed call, one per HTTP status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FailureKind {
    /// 401: no session, or an expired one.
    Unauthorized,
    /// 404: the target does not exist.
    NotFound,
    /// 400: the request breaks a validation rule.
    Invalid,
    /// 409: the request conflicts with the current state.
    Conflict,
    /// 500: the engine failed.
    Internal,
}

/// One call per route of the HTTP contract. Path and query parameters become fields, and a JSON
/// body that has a DTO is embedded as is. Each variant names the `ApiValue` it gets. Its `Debug`
/// masks the login token and the values of variables.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiCall {
    /// `POST /api/session`: consumes a login link and opens a session. Answer: `Session`.
    Login {
        /// The login link's token.
        token: String,
    },
    /// `GET /api/session`. Answer: `CurrentSession`.
    GetSession,
    /// `DELETE /api/session`: closes the request's session. Answer: `NoContent`.
    Logout,
    /// `GET /api/projects`. Answer: `Projects`.
    ListProjects,
    /// `POST /api/projects`. Answer: `CreatedProject`.
    CreateProject {
        /// What the project starts with.
        source: NewProjectSourceDto,
    },
    /// `GET /api/environments`. Answer: `Environments`.
    ListEnvironments,
    /// `POST /api/projects/:projectId/environments`. Answer: `Environment`.
    CreateEnvironment {
        /// The project that gets the environment.
        project_id: String,
        /// The environment's name, a slug.
        name: String,
    },
    /// `GET /api/services`. Answer: `Services`.
    ListServices,
    /// `POST /api/environments/:environmentId/services`. Answer: `Service`.
    CreateService {
        /// The environment that gets the service.
        environment_id: String,
        /// The image reference to deploy.
        image: String,
        /// The service's name; without one, the engine derives it from the image.
        name: Option<String>,
    },
    /// `GET /api/services/:id/settings`. Answer: `ServiceSettings`.
    GetServiceSettings {
        /// The service.
        service_id: String,
    },
    /// `DELETE /api/services/:id`. Answer: `NoContent`.
    DeleteService {
        /// The service.
        service_id: String,
    },
    /// `GET /api/deployments`. Answer: `Deployments`.
    ListDeployments,
    /// `POST /api/services/:id/redeploy`: deploys the active revision again. Answer:
    /// `Deployment`.
    Redeploy {
        /// The service.
        service_id: String,
    },
    /// `POST /api/deployments/:id/rollback`: deploys that deployment's revision again. Answer:
    /// `Deployment`.
    Rollback {
        /// The deployment to go back to.
        deployment_id: String,
    },
    /// `GET /api/deployments/:id/revision`. Answer: `Revision`.
    GetRevision {
        /// The deployment.
        deployment_id: String,
    },
    /// `GET /api/deployments/:id/logs?stream=`. Answer: `LogLines`.
    DeploymentLogs {
        /// The deployment.
        deployment_id: String,
        /// Build or deploy.
        stream: LogStream,
    },
    /// `GET /api/variables`. Answer: `Variables`.
    ListVariables,
    /// `GET /api/variables/impact?level=&ownerId=&name=`: the services that a shared variable
    /// of that name would redeploy. Answer: `VariableImpact`.
    VariableImpact {
        /// The level of the shared variable.
        level: VariableLevelDto,
        /// The project or environment that holds it.
        owner_id: String,
        /// The variable's name.
        name: String,
    },
    /// `POST /api/variables`. Answer: `SharedVariableSet`.
    SetSharedVariable(NewSharedVariableDto),
    /// `DELETE /api/variables/:id`. Answer: `SharedVariableRemoved`, or a `Conflict` failure.
    RemoveSharedVariable {
        /// The shared variable.
        variable_id: String,
    },
    /// `POST /api/variables/:id/reveal`: an audited read of a secret. Answer: `RevealedValue`.
    RevealVariable {
        /// The secret.
        variable_id: String,
    },
    /// `GET /api/changes`. Answer: `Changes`.
    ListChanges,
    /// `POST /api/changes`. Answer: `Change`.
    StageChange(NewChangeDto),
    /// `DELETE /api/changes/:id`. Answer: `NoContent`.
    DiscardChange {
        /// The staged change.
        change_id: String,
    },
    /// `POST /api/environments/:id/changes/deploy`. Answer: `ChangesDeployed`.
    DeployEnvironmentChanges {
        /// The environment.
        environment_id: String,
    },
    /// `DELETE /api/environments/:id/changes`. Answer: `NoContent`.
    DiscardEnvironmentChanges {
        /// The environment.
        environment_id: String,
    },
    /// `GET /api/environments/:id/logs`, the SSE stream, on a connection of its own. The engine
    /// always answers first with one `ApiReply`. After `LogStreamOpened`, it relays
    /// `telemetry::LogStreamFrame`s from telemetry on the connection until `End` or until either
    /// side closes. After a failure, nothing follows.
    OpenLogStream {
        /// The environment.
        environment_id: String,
        /// Only lines from this instant on, in milliseconds since the Unix epoch.
        since_ms: Option<u64>,
    },
    /// `GET /api/events?projectId=&before=&limit=`: a page of the project's activity, newest
    /// first. Answer: `Events`.
    ListEvents {
        /// The project.
        project_id: String,
        /// Only events with a sequence number below this one.
        before: Option<u64>,
        /// The page size, at most 200.
        limit: u32,
    },
}

/// A successful answer: one variant per response shape of the HTTP contract. Its `Debug` masks
/// the session token and revealed values.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiValue {
    /// No body: `204 No Content`.
    NoContent,
    /// `Login`: `brume api` sets the token as the session cookie and never sends it in a body.
    Session {
        /// The new session's token.
        token: String,
        /// The admin who logged in.
        admin: String,
    },
    /// `GetSession`.
    CurrentSession(SessionDto),
    /// `ListProjects`.
    Projects(Vec<ProjectDto>),
    /// `CreateProject`.
    CreatedProject(CreatedProjectDto),
    /// `ListEnvironments`.
    Environments(Vec<EnvironmentDto>),
    /// `CreateEnvironment`.
    Environment(EnvironmentDto),
    /// `ListServices`.
    Services(Vec<ServiceDto>),
    /// `CreateService`.
    Service(ServiceDto),
    /// `GetServiceSettings`.
    ServiceSettings(ServiceSettingsDto),
    /// `ListDeployments`.
    Deployments(Vec<DeploymentDto>),
    /// `Redeploy` and `Rollback`.
    Deployment(DeploymentDto),
    /// `GetRevision`.
    Revision(RevisionDto),
    /// `DeploymentLogs`.
    LogLines(Vec<LogLineDto>),
    /// `ListVariables`.
    Variables(Vec<VariableDto>),
    /// `VariableImpact`.
    VariableImpact(VariableImpactDto),
    /// `SetSharedVariable`.
    SharedVariableSet(SharedVariableSetDto),
    /// `RemoveSharedVariable`.
    SharedVariableRemoved(SharedVariableRemovedDto),
    /// `RevealVariable`.
    RevealedValue(RevealedValueDto),
    /// `ListChanges`.
    Changes(Vec<StagedChangeDto>),
    /// `StageChange`.
    Change(StagedChangeDto),
    /// `DeployEnvironmentChanges`.
    ChangesDeployed(ChangesDeployedDto),
    /// `ListEvents`.
    Events(EventsPageDto),
    /// `OpenLogStream`: the session is valid, and `telemetry::LogStreamFrame`s follow on this
    /// connection.
    LogStreamOpened,
}

/// `GET /api/session`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Session"))]
pub struct SessionDto {
    /// The logged-in admin.
    pub admin: String,
}

/// `Project`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Project"))]
pub struct ProjectDto {
    #[cfg_attr(feature = "ts", ts(type = "ProjectId"))]
    pub id: String,
    pub name: String,
    pub description: String,
    /// RFC 3339.
    pub updated_at: String,
}

/// `Environment`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Environment"))]
pub struct EnvironmentDto {
    #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
    pub id: String,
    #[cfg_attr(feature = "ts", ts(type = "ProjectId"))]
    pub project_id: String,
    /// A slug, also the URL segment of the environment.
    pub name: String,
}

/// `ServiceKind`: fixed when the service is created.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ServiceKind"))]
pub enum ServiceKindDto {
    Github,
    Image,
    Static,
    Postgres,
    Mongodb,
    Redis,
}

/// `DatabaseKind`: the service kinds that come from a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "DatabaseKind"))]
pub enum DatabaseKindDto {
    Postgres,
    Mongodb,
    Redis,
}

/// `ServiceStatus`, computed from the latest deployment and the observed containers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ServiceStatus"))]
pub enum ServiceStatusDto {
    Online,
    Deploying,
    Crashed,
    Stopped,
}

/// `Volume`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Volume"))]
pub struct VolumeDto {
    pub name: String,
    pub mount_path: String,
}

/// `Service`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Service"))]
pub struct ServiceDto {
    #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
    pub id: String,
    #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
    pub environment_id: String,
    pub name: String,
    /// The service's immutable host name on the environment network.
    pub slug: String,
    pub kind: ServiceKindDto,
    pub status: ServiceStatusDto,
    pub domains: Vec<String>,
    pub volume: Option<VolumeDto>,
    /// The services this one connects to, derived from the secrets it references.
    #[cfg_attr(feature = "ts", ts(type = "Array<ServiceId>"))]
    pub dependencies: Vec<String>,
}

/// `NewProjectSource`: what `POST /api/projects` starts the project with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "NewProjectSource"))]
pub enum NewProjectSourceDto {
    Empty,
    Github { repository: String },
    Static { repository: String },
    Postgres,
    Mongodb,
    Redis,
    Image { image: String },
}

/// `CreatedProject`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "CreatedProject"))]
pub struct CreatedProjectDto {
    #[cfg_attr(feature = "ts", ts(type = "ProjectId"))]
    pub project_id: String,
    pub environment_name: String,
}

/// `DeploymentStatus`. `building` while the image resolves; `superseded` once a later
/// deployment replaces this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "DeploymentStatus"))]
pub enum DeploymentStatusDto {
    Building,
    Deploying,
    Active,
    Failed,
    Superseded,
}

/// `Commit`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Commit"))]
pub struct CommitDto {
    pub sha: String,
    pub message: String,
    pub branch: String,
}

/// `Deployment`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Deployment"))]
pub struct DeploymentDto {
    #[cfg_attr(feature = "ts", ts(type = "DeploymentId"))]
    pub id: String,
    #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
    pub service_id: String,
    pub status: DeploymentStatusDto,
    /// RFC 3339.
    pub created_at: String,
    pub author: String,
    pub commit: Option<CommitDto>,
    pub image: Option<String>,
    pub rollbackable: bool,
}

/// `VariableKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "VariableKind"))]
pub enum VariableKindDto {
    Plain,
    Build,
    Secret,
}

/// `VariableLevel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "VariableLevel"))]
pub enum VariableLevelDto {
    Project,
    Environment,
    Service,
}

/// `Variable`, with the contract's `reference`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Variable"))]
pub struct VariableDto {
    #[cfg_attr(feature = "ts", ts(type = "VariableId"))]
    pub id: String,
    pub level: VariableLevelDto,
    /// The project, environment or service that holds the variable.
    pub owner_id: String,
    pub name: String,
    pub kind: VariableKindDto,
    /// Always null for a secret: only `RevealVariable` reads one.
    pub value: Option<String>,
    /// True for a service secret that points to a shared secret.
    pub reference: bool,
}

/// `ServiceSource`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ServiceSource"))]
pub enum ServiceSourceDto {
    Github {
        repository: String,
        branch: String,
        auto_deploy: bool,
    },
    Image {
        image: String,
    },
    Template {
        template: DatabaseKindDto,
        version: String,
    },
}

/// `ServiceSettings`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ServiceSettings"))]
pub struct ServiceSettingsDto {
    pub source: ServiceSourceDto,
    pub start_command: Option<String>,
    pub port: Option<u16>,
    pub healthcheck_path: Option<String>,
    /// In CPUs: `0.25` is a quarter of a CPU.
    pub cpu_limit: f64,
    pub memory_limit_mb: u32,
    /// Outbound internet access.
    pub egress: bool,
}

/// `LogLevel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// `LogLine`. The message is hostile: apps copy outside requests into their logs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "LogLine"))]
pub struct LogLineDto {
    pub id: String,
    /// RFC 3339.
    pub timestamp: String,
    #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
    pub service_id: String,
    pub level: LogLevel,
    pub message: String,
}

/// `DeploymentLogStream`: the logs of a deployment's build, or of its rollout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "DeploymentLogStream"))]
pub enum LogStream {
    Build,
    Deploy,
}

/// `RevisionVariable`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "RevisionVariable"))]
pub struct RevisionVariableDto {
    pub name: String,
    pub kind: VariableKindDto,
    /// Null for a secret: a revision only holds its name.
    pub value: Option<String>,
    pub origin: VariableLevelDto,
}

/// `Revision`: the immutable spec that a deployment put in service.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "Revision"))]
pub struct RevisionDto {
    pub digest: String,
    pub builder: Option<String>,
    pub start_command: Option<String>,
    pub variables: Vec<RevisionVariableDto>,
}

/// `SettingField`: the service settings that a change can stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SettingField {
    Repository,
    Branch,
    AutoDeploy,
    Image,
    StartCommand,
    Port,
    HealthcheckPath,
    CpuLimit,
    MemoryLimitMb,
    Egress,
    Domains,
}

/// `SettingValue`, untagged: the JSON type tells the variants apart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "SettingValue"))]
pub enum SettingValueDto {
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<String>),
}

/// `StagedChange`: edited in the panel, applied with the other changes of its environment by
/// one deploy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "StagedChange"))]
pub enum StagedChangeDto {
    Setting {
        #[cfg_attr(feature = "ts", ts(type = "ChangeId"))]
        id: String,
        #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
        environment_id: String,
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        field: SettingField,
        value: SettingValueDto,
    },
    Variable {
        #[cfg_attr(feature = "ts", ts(type = "ChangeId"))]
        id: String,
        #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
        environment_id: String,
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        name: String,
        kind: VariableKindDto,
        /// Null for a secret: the engine keeps the value until the deploy.
        value: Option<String>,
    },
    VariableRemoval {
        #[cfg_attr(feature = "ts", ts(type = "ChangeId"))]
        id: String,
        #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
        environment_id: String,
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        name: String,
    },
}

/// `NewChange`: what the panel stages. The engine finds the environment and assigns the id. Its
/// `Debug` masks the value of a variable.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "NewChange"))]
pub enum NewChangeDto {
    Setting {
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        field: SettingField,
        value: SettingValueDto,
    },
    Variable {
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        name: String,
        kind: VariableKindDto,
        /// For a secret, null references the shared secret of the same name.
        value: Option<String>,
    },
    VariableRemoval {
        #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
        service_id: String,
        name: String,
    },
}

/// The body of `POST /api/variables`: a shared variable or secret, at the project or
/// environment level. Its `Debug` masks the value.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "NewSharedVariable"))]
pub struct NewSharedVariableDto {
    pub level: VariableLevelDto,
    pub owner_id: String,
    pub name: String,
    pub kind: VariableKindDto,
    pub value: String,
}

/// The answer of `POST /api/variables`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "SharedVariableSet"))]
pub struct SharedVariableSetDto {
    pub variable: VariableDto,
    /// The deployments that a plain variable started.
    pub deployments: Vec<DeploymentDto>,
    /// The services whose active deployment uses this secret, to redeploy by hand.
    #[cfg_attr(feature = "ts", ts(type = "Array<ServiceId>"))]
    pub referenced_by: Vec<String>,
}

/// The answer of `DELETE /api/variables/:id`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(rename = "SharedVariableRemoved")
)]
pub struct SharedVariableRemovedDto {
    pub deployments: Vec<DeploymentDto>,
}

/// A service that a shared variable reaches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ImpactedService"))]
pub struct ImpactedServiceDto {
    #[cfg_attr(feature = "ts", ts(type = "ServiceId"))]
    pub service_id: String,
    #[cfg_attr(feature = "ts", ts(type = "EnvironmentId"))]
    pub environment_id: String,
}

/// The answer of `GET /api/variables/impact`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "VariableImpact"))]
pub struct VariableImpactDto {
    pub services: Vec<ImpactedServiceDto>,
}

/// The answer of `POST /api/variables/:id/reveal`, the only one that carries a secret value.
/// Its `Debug` masks the value.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "RevealedValue"))]
pub struct RevealedValueDto {
    pub value: String,
}

/// The answer of `POST /api/environments/:id/changes/deploy`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ChangesDeployed"))]
pub struct ChangesDeployedDto {
    /// The number of changes deployed.
    pub deployed: u32,
}

/// `ActivityEvent`: one event of the log, without any value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "ActivityEvent"))]
pub struct ActivityEventDto {
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub seq: u64,
    /// RFC 3339.
    pub at: String,
    /// The event's type, such as `DeploymentRequested`.
    #[serde(rename = "type")]
    pub kind: String,
    pub actor: String,
    #[cfg_attr(feature = "ts", ts(type = "EnvironmentId | null"))]
    pub environment_id: Option<String>,
    #[cfg_attr(feature = "ts", ts(type = "ServiceId | null"))]
    pub service_id: Option<String>,
    #[cfg_attr(feature = "ts", ts(type = "DeploymentId | null"))]
    pub deployment_id: Option<String>,
    /// An English sentence with the names and slugs of the entities.
    pub summary: String,
}

/// The answer of `GET /api/events`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "EventsPage"))]
pub struct EventsPageDto {
    pub events: Vec<ActivityEventDto>,
    /// The `before` of the next page, or null on the last page.
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    pub next_before: Option<u64>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ActivityEventDto, ApiFailure, ApiReply, ApiValue, ChangesDeployedDto, CommitDto,
        CreatedProjectDto, DatabaseKindDto, DeploymentDto, DeploymentStatusDto, EnvironmentDto,
        EventsPageDto, FailureKind, ImpactedServiceDto, LogLevel, LogLineDto, NewChangeDto,
        NewProjectSourceDto, NewSharedVariableDto, ProjectDto, RevealedValueDto, RevisionDto,
        RevisionVariableDto, ServiceDto, ServiceKindDto, ServiceSettingsDto, ServiceSourceDto,
        ServiceStatusDto, SessionDto, SettingField, SettingValueDto, SharedVariableSetDto,
        StagedChangeDto, VariableDto, VariableImpactDto, VariableKindDto, VariableLevelDto,
        VolumeDto,
    };

    fn to_json<T: serde::Serialize>(value: &T) -> serde_json::Value {
        serde_json::to_value(value).expect("serializes")
    }

    fn variable() -> VariableDto {
        VariableDto {
            id: "var".to_owned(),
            level: VariableLevelDto::Project,
            owner_id: "prj".to_owned(),
            name: "API_KEY".to_owned(),
            kind: VariableKindDto::Secret,
            value: None,
            reference: false,
        }
    }

    fn deployment() -> DeploymentDto {
        DeploymentDto {
            id: "dep".to_owned(),
            service_id: "svc".to_owned(),
            status: DeploymentStatusDto::Building,
            created_at: "2026-10-31T12:34:56.789Z".to_owned(),
            author: "admin".to_owned(),
            commit: None,
            image: Some("nginx:1.29".to_owned()),
            rollbackable: true,
        }
    }

    #[test]
    fn project_and_environment_have_the_shape_of_types_ts() {
        let project = ProjectDto {
            id: "prj".to_owned(),
            name: "Shop".to_owned(),
            description: String::new(),
            updated_at: "2026-10-31T12:34:56.789Z".to_owned(),
        };
        assert_eq!(
            to_json(&project),
            json!({
                "id": "prj", "name": "Shop", "description": "",
                "updatedAt": "2026-10-31T12:34:56.789Z",
            })
        );
        let environment = EnvironmentDto {
            id: "env".to_owned(),
            project_id: "prj".to_owned(),
            name: "production".to_owned(),
        };
        assert_eq!(
            to_json(&environment),
            json!({"id": "env", "projectId": "prj", "name": "production"})
        );
        let created = CreatedProjectDto {
            project_id: "prj".to_owned(),
            environment_name: "production".to_owned(),
        };
        assert_eq!(
            to_json(&created),
            json!({"projectId": "prj", "environmentName": "production"})
        );
    }

    #[test]
    fn small_answers_have_their_shapes() {
        assert_eq!(
            to_json(&SessionDto {
                admin: "adm".to_owned()
            }),
            json!({"admin": "adm"})
        );
        assert_eq!(
            to_json(&RevealedValueDto {
                value: "s3cr3t".to_owned()
            }),
            json!({"value": "s3cr3t"})
        );
        assert_eq!(
            to_json(&ChangesDeployedDto { deployed: 3 }),
            json!({"deployed": 3})
        );
    }

    #[test]
    fn service_has_the_shape_of_types_ts() {
        let service = ServiceDto {
            id: "svc".to_owned(),
            environment_id: "env".to_owned(),
            name: "API".to_owned(),
            slug: "api".to_owned(),
            kind: ServiceKindDto::Image,
            status: ServiceStatusDto::Online,
            domains: vec![],
            volume: Some(VolumeDto {
                name: "data".to_owned(),
                mount_path: "/data".to_owned(),
            }),
            dependencies: vec!["db".to_owned()],
        };
        assert_eq!(
            to_json(&service),
            json!({
                "id": "svc", "environmentId": "env", "name": "API", "slug": "api", "kind": "image",
                "status": "online", "domains": [], "volume": {"name": "data", "mountPath": "/data"},
                "dependencies": ["db"],
            })
        );
    }

    #[test]
    fn deployment_and_revision_have_the_shape_of_types_ts() {
        let mut pushed = deployment();
        pushed.commit = Some(CommitDto {
            sha: "abc".to_owned(),
            message: "Fix".to_owned(),
            branch: "main".to_owned(),
        });
        assert_eq!(
            to_json(&pushed),
            json!({
                "id": "dep", "serviceId": "svc", "status": "building",
                "createdAt": "2026-10-31T12:34:56.789Z", "author": "admin",
                "commit": {"sha": "abc", "message": "Fix", "branch": "main"},
                "image": "nginx:1.29", "rollbackable": true,
            })
        );
        let revision = RevisionDto {
            digest: "sha256:abc".to_owned(),
            builder: None,
            start_command: None,
            variables: vec![RevisionVariableDto {
                name: "PORT".to_owned(),
                kind: VariableKindDto::Build,
                value: Some("80".to_owned()),
                origin: VariableLevelDto::Environment,
            }],
        };
        assert_eq!(
            to_json(&revision),
            json!({
                "digest": "sha256:abc", "builder": null, "startCommand": null,
                "variables": [{"name": "PORT", "kind": "build", "value": "80", "origin": "environment"}],
            })
        );
    }

    #[test]
    fn service_settings_have_the_shape_of_types_ts() {
        let settings = ServiceSettingsDto {
            source: ServiceSourceDto::Image {
                image: "nginx:1.29".to_owned(),
            },
            start_command: None,
            port: Some(80),
            healthcheck_path: Some("/".to_owned()),
            cpu_limit: 0.25,
            memory_limit_mb: 512,
            egress: false,
        };
        assert_eq!(
            to_json(&settings),
            json!({
                "source": {"kind": "image", "image": "nginx:1.29"}, "startCommand": null,
                "port": 80, "healthcheckPath": "/", "cpuLimit": 0.25, "memoryLimitMb": 512,
                "egress": false,
            })
        );
        let template = ServiceSourceDto::Template {
            template: DatabaseKindDto::Postgres,
            version: "18".to_owned(),
        };
        assert_eq!(
            to_json(&template),
            json!({"kind": "template", "template": "postgres", "version": "18"})
        );
        let github = ServiceSourceDto::Github {
            repository: "org/app".to_owned(),
            branch: "main".to_owned(),
            auto_deploy: true,
        };
        assert_eq!(
            to_json(&github),
            json!({"kind": "github", "repository": "org/app", "branch": "main", "autoDeploy": true})
        );
    }

    #[test]
    fn staged_changes_have_the_shape_of_types_ts() {
        let setting = StagedChangeDto::Setting {
            id: "chg".to_owned(),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            field: SettingField::MemoryLimitMb,
            value: SettingValueDto::Number(1024.0),
        };
        assert_eq!(
            to_json(&setting),
            json!({
                "type": "setting", "id": "chg", "environmentId": "env", "serviceId": "svc",
                "field": "memoryLimitMb", "value": 1024.0,
            })
        );
        let secret = StagedChangeDto::Variable {
            id: "chg".to_owned(),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            name: "TOKEN".to_owned(),
            kind: VariableKindDto::Secret,
            value: None,
        };
        assert_eq!(
            to_json(&secret),
            json!({
                "type": "variable", "id": "chg", "environmentId": "env", "serviceId": "svc",
                "name": "TOKEN", "kind": "secret", "value": null,
            })
        );
        let removal = StagedChangeDto::VariableRemoval {
            id: "chg".to_owned(),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            name: "TOKEN".to_owned(),
        };
        assert_eq!(
            to_json(&removal),
            json!({
                "type": "variable-removal", "id": "chg", "environmentId": "env",
                "serviceId": "svc", "name": "TOKEN",
            })
        );
    }

    #[test]
    fn new_changes_parse_what_the_panel_sends() {
        let parse = |json: serde_json::Value| serde_json::from_value::<NewChangeDto>(json).ok();
        assert_eq!(
            parse(
                json!({"serviceId": "svc", "type": "setting", "field": "autoDeploy", "value": true})
            ),
            Some(NewChangeDto::Setting {
                service_id: "svc".to_owned(),
                field: SettingField::AutoDeploy,
                value: SettingValueDto::Bool(true),
            })
        );
        assert_eq!(
            parse(
                json!({"serviceId": "svc", "type": "setting", "field": "domains", "value": ["a.example"]})
            ),
            Some(NewChangeDto::Setting {
                service_id: "svc".to_owned(),
                field: SettingField::Domains,
                value: SettingValueDto::List(vec!["a.example".to_owned()]),
            })
        );
        assert_eq!(
            parse(
                json!({"serviceId": "svc", "type": "setting", "field": "image", "value": "redis:8"})
            ),
            Some(NewChangeDto::Setting {
                service_id: "svc".to_owned(),
                field: SettingField::Image,
                value: SettingValueDto::Text("redis:8".to_owned()),
            })
        );
        assert_eq!(
            parse(json!({"serviceId": "svc", "type": "setting", "field": "port", "value": 8080})),
            Some(NewChangeDto::Setting {
                service_id: "svc".to_owned(),
                field: SettingField::Port,
                value: SettingValueDto::Number(8080.0),
            })
        );
        assert_eq!(
            parse(
                json!({"serviceId": "svc", "type": "variable", "name": "TOKEN", "kind": "secret", "value": null})
            ),
            Some(NewChangeDto::Variable {
                service_id: "svc".to_owned(),
                name: "TOKEN".to_owned(),
                kind: VariableKindDto::Secret,
                value: None,
            })
        );
        assert_eq!(
            parse(json!({"serviceId": "svc", "type": "variable-removal", "name": "TOKEN"})),
            Some(NewChangeDto::VariableRemoval {
                service_id: "svc".to_owned(),
                name: "TOKEN".to_owned()
            })
        );
        assert_eq!(
            parse(json!({"serviceId": "svc", "type": "setting", "field": "nope", "value": 1})),
            None
        );
        assert_eq!(
            parse(json!({"serviceId": "svc", "type": "setting", "field": "port", "value": null})),
            None
        );
    }

    #[test]
    fn project_sources_parse_every_kind_of_types_ts() {
        let parse =
            |json: serde_json::Value| serde_json::from_value::<NewProjectSourceDto>(json).ok();
        assert_eq!(
            parse(json!({"kind": "empty"})),
            Some(NewProjectSourceDto::Empty)
        );
        assert_eq!(
            parse(json!({"kind": "image", "image": "nginx:1.29"})),
            Some(NewProjectSourceDto::Image {
                image: "nginx:1.29".to_owned()
            })
        );
        assert_eq!(
            parse(json!({"kind": "github", "repository": "org/app"})),
            Some(NewProjectSourceDto::Github {
                repository: "org/app".to_owned()
            })
        );
        assert_eq!(
            parse(json!({"kind": "static", "repository": "org/site"})),
            Some(NewProjectSourceDto::Static {
                repository: "org/site".to_owned()
            })
        );
        assert_eq!(
            parse(json!({"kind": "postgres"})),
            Some(NewProjectSourceDto::Postgres)
        );
        assert_eq!(
            parse(json!({"kind": "mongodb"})),
            Some(NewProjectSourceDto::Mongodb)
        );
        assert_eq!(
            parse(json!({"kind": "redis"})),
            Some(NewProjectSourceDto::Redis)
        );
        assert_eq!(parse(json!({"kind": "ftp"})), None);
    }

    #[test]
    fn contract_additions_have_their_shapes() {
        let mut referenced = variable();
        referenced.level = VariableLevelDto::Service;
        referenced.reference = true;
        assert_eq!(
            to_json(&referenced),
            json!({
                "id": "var", "level": "service", "ownerId": "prj", "name": "API_KEY",
                "kind": "secret", "value": null, "reference": true,
            })
        );
        let set = SharedVariableSetDto {
            variable: variable(),
            deployments: vec![deployment()],
            referenced_by: vec!["svc".to_owned()],
        };
        let set_json = to_json(&set);
        assert_eq!(set_json["referencedBy"], json!(["svc"]));
        assert_eq!(set_json["deployments"][0]["serviceId"], json!("svc"));

        let impact = VariableImpactDto {
            services: vec![ImpactedServiceDto {
                service_id: "svc".to_owned(),
                environment_id: "env".to_owned(),
            }],
        };
        assert_eq!(
            to_json(&impact),
            json!({"services": [{"serviceId": "svc", "environmentId": "env"}]})
        );

        let new_variable = serde_json::from_value::<NewSharedVariableDto>(json!({
            "level": "environment", "ownerId": "env", "name": "LOG_LEVEL", "kind": "plain", "value": "debug",
        }));
        assert_eq!(
            new_variable.ok(),
            Some(NewSharedVariableDto {
                level: VariableLevelDto::Environment,
                owner_id: "env".to_owned(),
                name: "LOG_LEVEL".to_owned(),
                kind: VariableKindDto::Plain,
                value: "debug".to_owned(),
            })
        );

        let page = EventsPageDto {
            events: vec![ActivityEventDto {
                seq: 7,
                at: "2026-10-31T12:34:56.789Z".to_owned(),
                kind: "DeploymentRequested".to_owned(),
                actor: "admin:adm".to_owned(),
                environment_id: Some("env".to_owned()),
                service_id: Some("svc".to_owned()),
                deployment_id: None,
                summary: "Deployment requested for api".to_owned(),
            }],
            next_before: None,
        };
        assert_eq!(
            to_json(&page),
            json!({
                "events": [{
                    "seq": 7, "at": "2026-10-31T12:34:56.789Z", "type": "DeploymentRequested",
                    "actor": "admin:adm", "environmentId": "env", "serviceId": "svc",
                    "deploymentId": null, "summary": "Deployment requested for api",
                }],
                "nextBefore": null,
            })
        );
    }

    #[test]
    fn log_line_has_the_shape_of_types_ts() {
        let line = LogLineDto {
            id: "42".to_owned(),
            timestamp: "2026-10-31T12:34:56.789Z".to_owned(),
            service_id: "svc".to_owned(),
            level: LogLevel::Warn,
            message: "slow".to_owned(),
        };
        assert_eq!(
            to_json(&line),
            json!({
                "id": "42", "timestamp": "2026-10-31T12:34:56.789Z", "serviceId": "svc",
                "level": "warn", "message": "slow",
            })
        );
    }

    #[test]
    fn reply_round_trips_success_and_failure() {
        for reply in [
            ApiReply {
                id: 1,
                result: Ok(ApiValue::Variables(vec![variable()])),
            },
            ApiReply {
                id: 2,
                result: Ok(ApiValue::NoContent),
            },
            ApiReply {
                id: 3,
                result: Err(ApiFailure {
                    kind: FailureKind::Conflict,
                    message: "API_KEY is still used by: api.".to_owned(),
                }),
            },
        ] {
            let json = serde_json::to_vec(&reply).expect("serializes");
            assert_eq!(serde_json::from_slice::<ApiReply>(&json).ok(), Some(reply));
        }
    }
}

#[cfg(all(test, feature = "ts"))]
mod ts_tests {
    use ts_rs::{Config, TS};

    use super::{
        ActivityEventDto, CreatedProjectDto, DatabaseKindDto, DeploymentDto, DeploymentStatusDto,
        EnvironmentDto, EventsPageDto, LogLevel, LogLineDto, LogStream, NewChangeDto,
        NewProjectSourceDto, ProjectDto, RevisionDto, RevisionVariableDto, ServiceDto,
        ServiceKindDto, ServiceSettingsDto, ServiceSourceDto, ServiceStatusDto, SettingField,
        SettingValueDto, StagedChangeDto, VariableDto, VariableKindDto, VariableLevelDto,
        VolumeDto,
    };

    #[test]
    fn declarations_use_the_names_and_fields_of_types_ts() {
        let cfg = Config::default();
        for (decl, name) in [
            (ProjectDto::decl(&cfg), "Project"),
            (EnvironmentDto::decl(&cfg), "Environment"),
            (ServiceKindDto::decl(&cfg), "ServiceKind"),
            (DatabaseKindDto::decl(&cfg), "DatabaseKind"),
            (ServiceStatusDto::decl(&cfg), "ServiceStatus"),
            (VolumeDto::decl(&cfg), "Volume"),
            (ServiceDto::decl(&cfg), "Service"),
            (NewProjectSourceDto::decl(&cfg), "NewProjectSource"),
            (CreatedProjectDto::decl(&cfg), "CreatedProject"),
            (DeploymentStatusDto::decl(&cfg), "DeploymentStatus"),
            (DeploymentDto::decl(&cfg), "Deployment"),
            (VariableKindDto::decl(&cfg), "VariableKind"),
            (VariableLevelDto::decl(&cfg), "VariableLevel"),
            (VariableDto::decl(&cfg), "Variable"),
            (ServiceSourceDto::decl(&cfg), "ServiceSource"),
            (ServiceSettingsDto::decl(&cfg), "ServiceSettings"),
            (LogLevel::decl(&cfg), "LogLevel"),
            (LogLineDto::decl(&cfg), "LogLine"),
            (LogStream::decl(&cfg), "DeploymentLogStream"),
            (RevisionVariableDto::decl(&cfg), "RevisionVariable"),
            (RevisionDto::decl(&cfg), "Revision"),
            (SettingField::decl(&cfg), "SettingField"),
            (SettingValueDto::decl(&cfg), "SettingValue"),
            (StagedChangeDto::decl(&cfg), "StagedChange"),
            (NewChangeDto::decl(&cfg), "NewChange"),
        ] {
            assert!(decl.starts_with(&format!("type {name} = ")), "{decl}");
        }

        let service = ServiceDto::decl(&cfg);
        assert!(
            service.contains("environmentId: EnvironmentId"),
            "{service}"
        );
        assert!(
            service.contains("dependencies: Array<ServiceId>"),
            "{service}"
        );
        let change = StagedChangeDto::decl(&cfg);
        assert!(
            change.contains(r#"{ "type": "variable-removal", id: ChangeId"#),
            "{change}"
        );
        let event = ActivityEventDto::decl(&cfg);
        assert!(
            event.contains("seq: number") && event.contains("type: string"),
            "{event}"
        );
        let page = EventsPageDto::decl(&cfg);
        assert!(page.contains("nextBefore: number | null"), "{page}");
    }
}
