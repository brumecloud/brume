//! The calls of `brume api`: the session check, then one handler per call.

use brume_domain::{Ctx, EnvironmentId, ValidationError};
use brume_proto::api::{
    ApiCall, ApiFailure, ApiReply, ApiRequest, ApiValue, FailureKind, SessionDto,
};
use brume_proto::telemetry::LogQuery;

use crate::engine::{ClientId, Engine, Output, failure};
use crate::storage::{Storage, StoredSession};

/// How many past lines a log stream starts with.
pub const LOG_BACKLOG_LINES: u32 = 500;

impl<S: Storage> Engine<S> {
    /// Answers a call. Every call but `Login` needs a valid session.
    pub(crate) fn api(
        &mut self,
        ctx: &mut impl Ctx,
        client: ClientId,
        request: ApiRequest,
        out: &mut Vec<Output>,
    ) {
        let ApiRequest { id, session, body } = request;
        let result = match body {
            ApiCall::Login { token } => self.login(ctx, &token),
            ApiCall::OpenLogStream {
                environment_id,
                since_ms,
            } => {
                let query = self
                    .authenticate(ctx, session.as_deref())
                    .and_then(|_| self.log_query(&environment_id, since_ms));
                match query {
                    Ok(query) => {
                        out.push(Output::OpenLogStream {
                            client,
                            request: id,
                            query,
                        });
                        return;
                    }
                    Err(failure) => Err(failure),
                }
            }
            call => match self.authenticate(ctx, session.as_deref()) {
                Ok(session) => self.call(ctx, &session, call),
                Err(failure) => Err(failure),
            },
        };
        out.push(Output::Reply {
            client,
            reply: ApiReply { id, result },
        });
    }

    fn call(
        &mut self,
        ctx: &mut impl Ctx,
        session: &StoredSession,
        call: ApiCall,
    ) -> Result<ApiValue, ApiFailure> {
        let admin = &session.admin;
        match call {
            ApiCall::Login { .. } | ApiCall::OpenLogStream { .. } => Err(failure(
                FailureKind::Invalid,
                "This call is not available here.".to_owned(),
            )),
            ApiCall::GetSession => Ok(ApiValue::CurrentSession(SessionDto {
                admin: self.admin_name(admin),
            })),
            ApiCall::Logout => self.logout(ctx, session),
            ApiCall::ListProjects => Ok(ApiValue::Projects(self.list_projects())),
            ApiCall::CreateProject { source } => self.create_project(ctx, admin, source),
            ApiCall::ListEnvironments => Ok(ApiValue::Environments(self.list_environments())),
            ApiCall::CreateEnvironment { project_id, name } => {
                self.create_environment(ctx, admin, &project_id, &name)
            }
            ApiCall::ListServices => Ok(ApiValue::Services(self.list_services())),
            ApiCall::CreateService {
                environment_id,
                image,
                name,
            } => self.create_service(ctx, admin, &environment_id, &image, name.as_deref()),
            ApiCall::GetServiceSettings { service_id } => self.service_settings(&service_id),
            ApiCall::DeleteService { service_id } => self.delete_service(ctx, admin, &service_id),
            ApiCall::ListDeployments => Ok(ApiValue::Deployments(self.list_deployments())),
            ApiCall::Redeploy { service_id } => self.redeploy(ctx, admin, &service_id),
            ApiCall::Rollback { deployment_id } => self.rollback(ctx, admin, &deployment_id),
            ApiCall::GetRevision { deployment_id } => self.revision(&deployment_id),
            ApiCall::DeploymentLogs {
                deployment_id,
                stream,
            } => self.deployment_logs(&deployment_id, stream),
            ApiCall::ListVariables => self.list_variables(),
            ApiCall::VariableImpact {
                level,
                owner_id,
                name,
            } => self.variable_impact(level, &owner_id, &name),
            ApiCall::SetSharedVariable(variable) => self.set_shared_variable(ctx, admin, variable),
            ApiCall::RemoveSharedVariable { variable_id } => {
                self.remove_shared_variable(ctx, admin, &variable_id)
            }
            ApiCall::RevealVariable { variable_id } => {
                self.reveal_variable(ctx, admin, &variable_id)
            }
            ApiCall::ListChanges => self.list_changes(),
            ApiCall::StageChange(change) => self.stage_change(ctx, change),
            ApiCall::DiscardChange { change_id } => self.discard_change(&change_id),
            ApiCall::DeployEnvironmentChanges { environment_id } => {
                self.deploy_environment_changes(ctx, admin, &environment_id)
            }
            ApiCall::DiscardEnvironmentChanges { environment_id } => {
                self.discard_environment_changes(&environment_id)
            }
            ApiCall::ListEvents {
                project_id,
                before,
                limit,
            } => self.list_events(&project_id, before, limit),
        }
    }

    fn log_query(
        &self,
        environment_id: &str,
        since_ms: Option<u64>,
    ) -> Result<LogQuery, ApiFailure> {
        let environment = parse_id(environment_id, EnvironmentId::parse, "environment")?;
        if self.world.environment(&environment).is_none() {
            return Err(not_found("environment"));
        }
        Ok(LogQuery {
            environment_id: environment.to_string(),
            since_ms,
            limit: LOG_BACKLOG_LINES,
            follow: true,
        })
    }
}

/// `"This <what> does not exist."`
pub(crate) fn not_found(what: &str) -> ApiFailure {
    failure(
        FailureKind::NotFound,
        format!("This {what} does not exist."),
    )
}

/// A refused value, with the newtype's message.
pub(crate) fn invalid(error: ValidationError) -> ApiFailure {
    failure(FailureKind::Invalid, error.to_string())
}

/// An id from a path: one that does not parse names nothing, so it is not found.
pub(crate) fn parse_id<T>(
    raw: &str,
    parse: fn(&str) -> Result<T, ValidationError>,
    what: &str,
) -> Result<T, ApiFailure> {
    parse(raw).map_err(|_| not_found(what))
}
