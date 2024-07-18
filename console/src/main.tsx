import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider,
  HttpLink,
  defaultDataIdFromObject,
} from "@apollo/client";
import { loadErrorMessages, loadDevMessages } from "@apollo/client/dev";
import { onError } from "@apollo/client/link/error";
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
      console.log(`[GraphQL error]: Message: ${message}, Location: ${locations}, Path: ${path}`)
    );
  if (networkError) {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (networkError.statusCode === 401) {
      localStorage.setItem(
        "info",
        JSON.stringify({ title: "Login Required", message: "Your session has expired" })
      );
      window.location.pathname = "/login";
    } else {
      localStorage.setItem(
        "error",
        JSON.stringify({ title: "Got an error from server", message: networkError.message })
      );
      window.location.pathname = "/login";
    }
  }
});

const finalLink = errorLink.concat(httpLink);

const graphqlClient = new ApolloClient({
  link: finalLink,
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
  <React.StrictMode>
    <ApolloProvider client={graphqlClient}>
      <App />
    </ApolloProvider>
  </React.StrictMode>
);
