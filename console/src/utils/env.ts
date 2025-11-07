type EnvVariables = {
  ORCHESTRATOR_URL: string;
  WS_URL: string;
};

const ENVS = ["local", "prod"] as const;

const BrumeEnv: { [key in (typeof ENVS)[number]]: EnvVariables } = {
  local: {
    ORCHESTRATOR_URL: "http://localhost:9877",
    WS_URL: "ws://localhost:9877/graphql",
  },
  prod: {
    ORCHESTRATOR_URL: "https://api.brume.dev",
    WS_URL: "wss://api.brume.dev/graphql",
  },
};

const getEnv = (): EnvVariables => {
  const path = window.location.pathname;
  const env = path.split("/")[0];

  if (env?.startsWith("http://localhost")) {
    return BrumeEnv.local;
  }

  if (env?.startsWith("https://api.brume.dev")) {
    return BrumeEnv.prod;
  }

  return BrumeEnv.local;
};

export const ENV = getEnv();
