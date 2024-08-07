import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider,
  HttpLink,
  defaultDataIdFromObject,
  split,
} from "@apollo/client";
import {
  loadErrorMessages,
  loadDevMessages,
} from "@apollo/client/dev";
import { onError } from "@apollo/client/link/error";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { getMainDefinition } from "@apollo/client/utilities";
import { createClient } from "graphql-ws";
import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

const httpLink = new HttpLink({
  uri: "http://localhost:9877/graphql",
  credentials: "include",
});

loadDevMessages();
loadErrorMessages();

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (graphQLErrors)
    graphQLErrors.map(({ message, locations, path }) =>
      console.log(
        `[GraphQL error]: Message: ${message}, Location: ${locations}, Path: ${path}`
      )
    );
  if (networkError) {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (networkError.statusCode === 401) {
      localStorage.setItem(
        "info",
        JSON.stringify({
          title: "Login Required",
          message: "Your session has expired",
        })
      );
      window.location.pathname = "/login";
    } else {
      localStorage.setItem(
        "error",
        JSON.stringify({
          title: "Got an error from server",
          message: networkError.message,
        })
      );
      window.location.pathname = "/login";
    }
  }
});

const wsLink = new GraphQLWsLink(
  createClient({
    url: "ws://localhost:9877/graphql",
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
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <ApolloProvider client={graphqlClient}>
    <App />
  </ApolloProvider>
);
