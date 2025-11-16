import {
  ApolloClient,
  ApolloProvider,
  defaultDataIdFromObject,
  HttpLink,
  InMemoryCache,
  split,
} from "@apollo/client";
import { loadDevMessages, loadErrorMessages } from "@apollo/client/dev";
import { onError } from "@apollo/client/link/error";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { getMainDefinition } from "@apollo/client/utilities";
import { createClient } from "graphql-ws";
import ReactDOM from "react-dom/client";
import { ErrorBoundary } from "react-error-boundary";

import App from "./App";
import "./index.css";
import { createFragmentRegistry } from "@apollo/client/cache";
import { CLOUD_ACCOUNT_FRAGMENT } from "@/gql/cloud.graphql";
import { BUILDER_FRAGMENT } from "./gql/builder.graphql";
import { PROJECT_FRAGMENT } from "./gql/project.graphql";
import { RUNNER_FRAGMENT } from "./gql/runner.graphql";
import { BASE_SERVICE_FRAGMENT, ServiceFragment } from "./gql/service.graphql";
import { SourceFragment } from "./gql/source.graphql";
import { ENV } from "./utils/env";

const httpLink = new HttpLink({
  uri: `${ENV.ORCHESTRATOR_URL}/graphql`,
  fetch: (uri, options) =>
    fetch(uri, {
      ...options,
      credentials: "include",
    }),
});

loadDevMessages();
loadErrorMessages();

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (networkError) {
    if (!("statusCode" in networkError)) {
      return;
    }

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    if (networkError.statusCode === 500) {
      throw new Error(networkError.message);
    }

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    if (networkError.statusCode === 401) {
      localStorage.setItem(
        "info",
        JSON.stringify({
          title: "Login Required",
          message: "Your session has expired",
        })
      );
      window.location.pathname = "/login";
      return;
    }

    if (networkError.statusCode === 403) {
      localStorage.setItem(
        "info",
        JSON.stringify({
          title: "Login Required",
          message:
            "You are not authorized to access this resource. Your credentials may have expired.",
        })
      );
      window.location.pathname = "/login";
      return;
    }
  }
  if (graphQLErrors) {
    if (graphQLErrors.length === 0) {
      return;
    }
    throw new Error(graphQLErrors[0]?.message);
  }
});

const wsLink = new GraphQLWsLink(
  createClient({
    url: ENV.WS_URL,
    connectionParams: {
      credentials: "include",
    },
  })
);

// The split function takes three parameters:
//
// * A function that's called for each operation to execute
// * The Link to use for an operation if the function returns a "truthy" value
// * The Link to use for an operation if the function returns a "falsy" value
const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === "OperationDefinition" &&
      definition.operation === "subscription"
    );
  },
  wsLink,
  httpLink
);

const graphqlClient = new ApolloClient({
  link: errorLink.concat(splitLink),
  cache: new InMemoryCache({
    fragments: createFragmentRegistry(
      PROJECT_FRAGMENT,
      RUNNER_FRAGMENT,
      BUILDER_FRAGMENT,
      SourceFragment,
      BASE_SERVICE_FRAGMENT,
      ServiceFragment,
      CLOUD_ACCOUNT_FRAGMENT
    ),
    dataIdFromObject(responseObject) {
      switch (responseObject.__typename) {
        case "Project": {
          return `Project:${responseObject.id}`;
        }
        default:
          return defaultDataIdFromObject(responseObject);
      }
    },
  }),
  dataMasking: false,
});

function ErrorFallback({
  error,
  resetErrorBoundary,
}: {
  error: Error;
  resetErrorBoundary: () => void;
}) {
  return (
    <div role="alert">
      <p>Something went wrong:</p>
      <pre>{error.message}</pre>
      <button onClick={resetErrorBoundary}>Try again</button>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <ErrorBoundary FallbackComponent={ErrorFallback}>
    <ApolloProvider client={graphqlClient}>
      <App />
    </ApolloProvider>
  </ErrorBoundary>
);
