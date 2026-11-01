//! Hand-written `Debug` impls that mask session tokens, login tokens and the values of
//! variables, so that a `tracing::debug!(?request)` cannot leak them. The wire format does not
//! change.

use std::fmt;

use crate::admin::AdminReply;
use crate::api::{
    ApiCall, ApiRequest, ApiValue, NewChangeDto, NewSharedVariableDto, RevealedValueDto,
};

/// Prints in place of a masked value.
struct Redacted;

impl fmt::Debug for Redacted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// `Name { field: value, … }`, with the bindings of a struct variant as the fields.
macro_rules! debug_struct {
    ($f:expr, $name:literal $(, $field:ident)*) => {
        $f.debug_struct($name)$(.field(stringify!($field), $field))*.finish()
    };
}

/// `Name(value)`, for a newtype variant.
macro_rules! debug_tuple {
    ($f:expr, $name:literal, $value:expr) => {
        $f.debug_tuple($name).field($value).finish()
    };
}

impl fmt::Debug for ApiRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiRequest")
            .field("id", &self.id)
            .field("session", &self.session.as_ref().map(|_| Redacted))
            .field("body", &self.body)
            .finish()
    }
}

impl fmt::Debug for ApiCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login { token: _ } => f.debug_struct("Login").field("token", &Redacted).finish(),
            Self::GetSession => f.write_str("GetSession"),
            Self::Logout => f.write_str("Logout"),
            Self::ListProjects => f.write_str("ListProjects"),
            Self::CreateProject { source } => debug_struct!(f, "CreateProject", source),
            Self::ListEnvironments => f.write_str("ListEnvironments"),
            Self::CreateEnvironment { project_id, name } => {
                debug_struct!(f, "CreateEnvironment", project_id, name)
            }
            Self::ListServices => f.write_str("ListServices"),
            Self::CreateService {
                environment_id,
                image,
                name,
            } => {
                debug_struct!(f, "CreateService", environment_id, image, name)
            }
            Self::GetServiceSettings { service_id } => {
                debug_struct!(f, "GetServiceSettings", service_id)
            }
            Self::DeleteService { service_id } => debug_struct!(f, "DeleteService", service_id),
            Self::ListDeployments => f.write_str("ListDeployments"),
            Self::Redeploy { service_id } => debug_struct!(f, "Redeploy", service_id),
            Self::Rollback { deployment_id } => debug_struct!(f, "Rollback", deployment_id),
            Self::GetRevision { deployment_id } => debug_struct!(f, "GetRevision", deployment_id),
            Self::DeploymentLogs {
                deployment_id,
                stream,
            } => {
                debug_struct!(f, "DeploymentLogs", deployment_id, stream)
            }
            Self::ListVariables => f.write_str("ListVariables"),
            Self::VariableImpact {
                level,
                owner_id,
                name,
            } => {
                debug_struct!(f, "VariableImpact", level, owner_id, name)
            }
            Self::SetSharedVariable(variable) => debug_tuple!(f, "SetSharedVariable", variable),
            Self::RemoveSharedVariable { variable_id } => {
                debug_struct!(f, "RemoveSharedVariable", variable_id)
            }
            Self::RevealVariable { variable_id } => {
                debug_struct!(f, "RevealVariable", variable_id)
            }
            Self::ListChanges => f.write_str("ListChanges"),
            Self::StageChange(change) => debug_tuple!(f, "StageChange", change),
            Self::DiscardChange { change_id } => debug_struct!(f, "DiscardChange", change_id),
            Self::DeployEnvironmentChanges { environment_id } => {
                debug_struct!(f, "DeployEnvironmentChanges", environment_id)
            }
            Self::DiscardEnvironmentChanges { environment_id } => {
                debug_struct!(f, "DiscardEnvironmentChanges", environment_id)
            }
            Self::OpenLogStream {
                environment_id,
                since_ms,
            } => {
                debug_struct!(f, "OpenLogStream", environment_id, since_ms)
            }
            Self::ListEvents {
                project_id,
                before,
                limit,
            } => {
                debug_struct!(f, "ListEvents", project_id, before, limit)
            }
            Self::GetIngress => f.write_str("GetIngress"),
        }
    }
}

impl fmt::Debug for ApiValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoContent => f.write_str("NoContent"),
            Self::Session { token: _, admin } => f
                .debug_struct("Session")
                .field("token", &Redacted)
                .field("admin", admin)
                .finish(),
            Self::CurrentSession(session) => debug_tuple!(f, "CurrentSession", session),
            Self::Projects(projects) => debug_tuple!(f, "Projects", projects),
            Self::CreatedProject(created) => debug_tuple!(f, "CreatedProject", created),
            Self::Environments(environments) => debug_tuple!(f, "Environments", environments),
            Self::Environment(environment) => debug_tuple!(f, "Environment", environment),
            Self::Services(services) => debug_tuple!(f, "Services", services),
            Self::Service(service) => debug_tuple!(f, "Service", service),
            Self::ServiceSettings(settings) => debug_tuple!(f, "ServiceSettings", settings),
            Self::Deployments(deployments) => debug_tuple!(f, "Deployments", deployments),
            Self::Deployment(deployment) => debug_tuple!(f, "Deployment", deployment),
            Self::Revision(revision) => debug_tuple!(f, "Revision", revision),
            Self::LogLines(lines) => debug_tuple!(f, "LogLines", lines),
            Self::Variables(variables) => debug_tuple!(f, "Variables", variables),
            Self::VariableImpact(impact) => debug_tuple!(f, "VariableImpact", impact),
            Self::SharedVariableSet(set) => debug_tuple!(f, "SharedVariableSet", set),
            Self::SharedVariableRemoved(removed) => {
                debug_tuple!(f, "SharedVariableRemoved", removed)
            }
            Self::RevealedValue(revealed) => debug_tuple!(f, "RevealedValue", revealed),
            Self::Changes(changes) => debug_tuple!(f, "Changes", changes),
            Self::Change(change) => debug_tuple!(f, "Change", change),
            Self::ChangesDeployed(deployed) => debug_tuple!(f, "ChangesDeployed", deployed),
            Self::Events(page) => debug_tuple!(f, "Events", page),
            Self::Ingress(ingress) => debug_tuple!(f, "Ingress", ingress),
            Self::LogStreamOpened => f.write_str("LogStreamOpened"),
        }
    }
}

impl fmt::Debug for NewChangeDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setting {
                service_id,
                field,
                value,
            } => {
                debug_struct!(f, "Setting", service_id, field, value)
            }
            // Masked whatever the kind: `None`, a shared secret reference, stays visible.
            Self::Variable {
                service_id,
                name,
                kind,
                value,
            } => f
                .debug_struct("Variable")
                .field("service_id", service_id)
                .field("name", name)
                .field("kind", kind)
                .field("value", &value.as_ref().map(|_| Redacted))
                .finish(),
            Self::VariableRemoval { service_id, name } => {
                debug_struct!(f, "VariableRemoval", service_id, name)
            }
        }
    }
}

impl fmt::Debug for NewSharedVariableDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewSharedVariableDto")
            .field("level", &self.level)
            .field("owner_id", &self.owner_id)
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("value", &Redacted)
            .finish()
    }
}

impl fmt::Debug for RevealedValueDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevealedValueDto")
            .field("value", &Redacted)
            .finish()
    }
}

impl fmt::Debug for AdminReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoginLink {
                token: _,
                expires_at,
            } => f
                .debug_struct("LoginLink")
                .field("token", &Redacted)
                .field("expires_at", expires_at)
                .finish(),
            Self::IngressConfigured => f.write_str("IngressConfigured"),
            Self::Ingress(ingress) => f.debug_tuple("Ingress").field(ingress).finish(),
            Self::Refused { reason } => f.debug_struct("Refused").field("reason", reason).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::AdminReply;
    use crate::api::{
        ApiCall, ApiReply, ApiRequest, ApiValue, NewChangeDto, NewSharedVariableDto,
        RevealedValueDto, VariableKindDto, VariableLevelDto,
    };

    const SECRET: &str = "s3cr3t-🚀";

    fn assert_masked(debug: &str) {
        assert!(!debug.contains(SECRET), "{debug}");
        assert!(debug.contains("<redacted>"), "{debug}");
    }

    #[test]
    fn api_request_debug_masks_the_session() {
        let request = ApiRequest {
            id: 7,
            session: Some(SECRET.to_owned()),
            body: ApiCall::ListProjects,
        };
        let debug = format!("{request:?}");
        assert_masked(&debug);
        assert!(
            debug.contains("id: 7") && debug.contains("ListProjects"),
            "{debug}"
        );

        let anonymous = ApiRequest {
            id: 8,
            session: None,
            body: ApiCall::GetSession,
        };
        assert!(format!("{anonymous:?}").contains("session: None"));
    }

    #[test]
    fn api_call_debug_masks_the_login_token() {
        let debug = format!(
            "{:?}",
            ApiCall::Login {
                token: SECRET.to_owned()
            }
        );
        assert_masked(&debug);
        assert!(debug.starts_with("Login"), "{debug}");

        let other = format!(
            "{:?}",
            ApiCall::DeleteService {
                service_id: "svc".to_owned()
            }
        );
        assert_eq!(other, r#"DeleteService { service_id: "svc" }"#);
    }

    #[test]
    fn api_value_debug_masks_the_session_token() {
        let value = ApiValue::Session {
            token: SECRET.to_owned(),
            admin: "adm".to_owned(),
        };
        let debug = format!("{value:?}");
        assert_masked(&debug);
        assert!(debug.contains(r#"admin: "adm""#), "{debug}");
    }

    #[test]
    fn new_change_debug_masks_the_value() {
        let change = NewChangeDto::Variable {
            service_id: "svc".to_owned(),
            name: "TOKEN".to_owned(),
            kind: VariableKindDto::Secret,
            value: Some(SECRET.to_owned()),
        };
        assert_masked(&format!("{change:?}"));
        assert_masked(&format!("{:?}", ApiCall::StageChange(change)));
    }

    #[test]
    fn new_shared_variable_debug_masks_the_value() {
        let variable = NewSharedVariableDto {
            level: VariableLevelDto::Project,
            owner_id: "prj".to_owned(),
            name: "API_KEY".to_owned(),
            kind: VariableKindDto::Secret,
            value: SECRET.to_owned(),
        };
        assert_masked(&format!("{variable:?}"));
        assert_masked(&format!("{:?}", ApiCall::SetSharedVariable(variable)));
    }

    #[test]
    fn revealed_value_debug_masks_the_value() {
        let revealed = RevealedValueDto {
            value: SECRET.to_owned(),
        };
        assert_masked(&format!("{revealed:?}"));
        let reply = ApiReply {
            id: 1,
            result: Ok(ApiValue::RevealedValue(revealed)),
        };
        assert_masked(&format!("{reply:?}"));
    }

    #[test]
    fn admin_reply_debug_masks_the_token() {
        let reply = AdminReply::LoginLink {
            token: SECRET.to_owned(),
            expires_at: "2026-10-31T12:49:56.789Z".to_owned(),
        };
        let debug = format!("{reply:?}");
        assert_masked(&debug);
        assert!(debug.contains("2026-10-31T12:49:56.789Z"), "{debug}");
    }
}
