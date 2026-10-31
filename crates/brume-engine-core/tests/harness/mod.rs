//! Drives an `Engine` over a `FakeStorage`, with a manual clock and seeded randomness. A
//! directory module, so that Cargo does not build it as a test of its own.

#![allow(dead_code)]

use brume_domain::{Ctx, Timestamp};
use brume_engine_core::{ClientId, Engine, Input, Output, desired_state};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::agent::{
    AgentToEngine, ContainerHealth, DesiredState, EngineToAgent, Job, JobOutcome,
    ObservedContainer, ObservedState,
};
use brume_proto::api::{
    ApiCall, ApiFailure, ApiReply, ApiRequest, ApiValue, CreatedProjectDto, DeploymentDto,
    EnvironmentDto, FailureKind, NewProjectSourceDto, ServiceDto,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::fake_storage::FakeStorage;

pub const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct TestCtx {
    pub now: Timestamp,
    rng: ChaCha8Rng,
}

impl TestCtx {
    pub fn new(seed: u64) -> Self {
        Self {
            now: Timestamp::from_millis(1_793_450_096_789),
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn advance(&mut self, millis: u64) {
        self.now = self.now.plus_millis(millis);
    }
}

impl Ctx for TestCtx {
    fn now(&self) -> Timestamp {
        self.now
    }

    fn fill_random(&mut self, buf: &mut [u8]) {
        rand::Rng::fill_bytes(&mut self.rng, buf);
    }
}

pub struct Harness {
    pub ctx: TestCtx,
    pub storage: FakeStorage,
    pub engine: Engine<FakeStorage>,
    pub session: Option<String>,
    next_request: u64,
}

impl Harness {
    /// A fresh engine, with a logged-in session.
    pub fn new() -> Self {
        let mut harness = Self::unauthenticated();
        let token = harness.login_link();
        match harness.call(ApiCall::Login { token }) {
            Ok(ApiValue::Session { token, .. }) => harness.session = Some(token),
            other => panic!("login failed: {other:?}"),
        }
        harness
    }

    /// A fresh engine, without a session.
    pub fn unauthenticated() -> Self {
        let mut ctx = TestCtx::new(7);
        let storage = FakeStorage::default();
        let engine = Engine::open(storage.clone(), &mut ctx).expect("opens");
        Self {
            ctx,
            storage,
            engine,
            session: None,
            next_request: 0,
        }
    }

    /// Opens a new engine on the same storage, as after a restart.
    pub fn reopen(&mut self) {
        self.engine = Engine::open(self.storage.clone(), &mut self.ctx).expect("reopens");
    }

    pub fn input(&mut self, input: Input) -> Vec<Output> {
        let mut out = Vec::new();
        self.engine.handle(&mut self.ctx, input, &mut out);
        out
    }

    pub fn login_link(&mut self) -> String {
        let out = self.input(Input::Admin {
            client: ClientId(99),
            request: AdminRequest::CreateLoginLink,
        });
        match out.as_slice() {
            [
                Output::AdminReply {
                    reply: AdminReply::LoginLink { token, .. },
                    ..
                },
                ..,
            ] => token.clone(),
            other => panic!("no login link: {other:?}"),
        }
    }

    /// Sends a call with the harness's session, and returns its reply.
    pub fn call(&mut self, body: ApiCall) -> Result<ApiValue, ApiFailure> {
        let session = self.session.clone();
        self.call_with(session, body)
    }

    pub fn call_with(
        &mut self,
        session: Option<String>,
        body: ApiCall,
    ) -> Result<ApiValue, ApiFailure> {
        self.next_request += 1;
        let id = self.next_request;
        let out = self.input(Input::Api {
            client: ClientId(1),
            request: ApiRequest { id, session, body },
        });
        out.into_iter()
            .find_map(|output| match output {
                Output::Reply {
                    reply:
                        ApiReply {
                            id: reply_id,
                            result,
                        },
                    ..
                } if reply_id == id => Some(result),
                _ => None,
            })
            .expect("a reply")
    }

    pub fn ok(&mut self, body: ApiCall) -> ApiValue {
        match self.call(body) {
            Ok(value) => value,
            Err(failure) => panic!("call failed: {failure:?}"),
        }
    }

    pub fn failure(&mut self, body: ApiCall) -> FailureKind {
        match self.call(body) {
            Ok(_) => panic!("the call succeeded"),
            Err(failure) => failure.kind,
        }
    }

    pub fn create_project(&mut self, image: &str) -> CreatedProjectDto {
        match self.ok(ApiCall::CreateProject {
            source: NewProjectSourceDto::Image {
                image: image.to_owned(),
            },
        }) {
            ApiValue::CreatedProject(created) => created,
            other => panic!("unexpected {other:?}"),
        }
    }

    pub fn environments(&mut self) -> Vec<EnvironmentDto> {
        match self.ok(ApiCall::ListEnvironments) {
            ApiValue::Environments(environments) => environments,
            other => panic!("unexpected {other:?}"),
        }
    }

    pub fn services(&mut self) -> Vec<ServiceDto> {
        match self.ok(ApiCall::ListServices) {
            ApiValue::Services(services) => services,
            other => panic!("unexpected {other:?}"),
        }
    }

    pub fn deployments(&mut self) -> Vec<DeploymentDto> {
        match self.ok(ApiCall::ListDeployments) {
            ApiValue::Deployments(deployments) => deployments,
            other => panic!("unexpected {other:?}"),
        }
    }

    pub fn deployment(&mut self, id: &str) -> DeploymentDto {
        self.deployments()
            .into_iter()
            .find(|deployment| deployment.id == id)
            .expect("the deployment exists")
    }

    pub fn service(&mut self, id: &str) -> ServiceDto {
        self.services()
            .into_iter()
            .find(|service| service.id == id)
            .expect("the service exists")
    }

    pub fn connect_agent(&mut self) -> Vec<Output> {
        let machine = self.engine.machine().expect("registered").clone();
        self.input(Input::AgentConnected { machine })
    }

    pub fn agent(&mut self, message: AgentToEngine) -> Vec<Output> {
        self.input(Input::FromAgent(message))
    }

    pub fn resolve(&mut self, deployment: &str, volumes: &[&str]) -> Vec<Output> {
        self.agent(AgentToEngine::JobResult {
            job_id: deployment.to_owned(),
            outcome: JobOutcome::ImageResolved {
                digest: format!("nginx@sha256:{DIGEST}"),
                volumes: volumes.iter().map(|path| (*path).to_owned()).collect(),
                exposed_ports: vec![80],
            },
        })
    }

    /// Reports the containers of `desired` with these healths, by deployment id; others are
    /// healthy. Each keeps the aliases the desired state gives it.
    pub fn observe(
        &mut self,
        desired: &DesiredState,
        health: &[(&str, ContainerHealth)],
    ) -> Vec<Output> {
        let containers = desired
            .containers
            .iter()
            .map(|container| ObservedContainer {
                name: container.name.clone(),
                deployment_id: container.deployment_id.clone(),
                health: health
                    .iter()
                    .find(|(deployment, _)| *deployment == container.deployment_id)
                    .map_or(ContainerHealth::Healthy, |(_, health)| health.clone()),
                aliases: container.networks[0].aliases.clone(),
                restarts: 0,
            })
            .collect();
        self.agent(AgentToEngine::Observed(ObservedState { containers }))
    }

    /// Takes a deployment that the agent must resolve to active.
    pub fn activate(&mut self, deployment: &str, volumes: &[&str]) -> DesiredState {
        if self.deployment(deployment).status == brume_proto::api::DeploymentStatusDto::Building {
            self.resolve(deployment, volumes);
        }
        loop {
            let state = self.desired_now();
            if self.deployment(deployment).status == brume_proto::api::DeploymentStatusDto::Active {
                return state;
            }
            self.observe(&state, &[]);
        }
    }

    /// The desired state now.
    pub fn desired_now(&mut self) -> DesiredState {
        desired_state(
            self.engine.world(),
            self.engine.storage(),
            self.engine.position(),
        )
        .expect("computes")
    }
}

pub fn last_desired(out: &[Output]) -> Option<DesiredState> {
    out.iter().rev().find_map(|output| match output {
        Output::ToAgent(EngineToAgent::DesiredState(state)) => Some(state.clone()),
        _ => None,
    })
}

pub fn jobs(out: &[Output]) -> Vec<Job> {
    out.iter()
        .filter_map(|output| match output {
            Output::ToAgent(EngineToAgent::Job(job)) => Some(job.clone()),
            _ => None,
        })
        .collect()
}

pub fn wake_at(out: &[Output]) -> Option<Timestamp> {
    out.iter().rev().find_map(|output| match output {
        Output::WakeAt(at) => Some(*at),
        _ => None,
    })
}
