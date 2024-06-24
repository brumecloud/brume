import { ApolloClient, InMemoryCache, ApolloProvider, HttpLink } from "@apollo/client";
import { onError } from "@apollo/client/link/error";
import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

const httpLink = new HttpLink({
  uri: "http://localhost:9877/graphql",
  credentials: "include",
});

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (graphQLErrors)
    graphQLErrors.map(({ message, locations, path }) =>
      console.log(`[GraphQL error]: Message: ${message}, Location: ${locations}, Path: ${path}`)
    );
  if (networkError) {
    localStorage.setItem(
      "error",
      JSON.stringify({
        title: "Network Error",
        message: networkError.message,
      })
    );
    window.location.pathname = "/login?error=true";
  }
});

const finalLink = errorLink.concat(httpLink);

const graphqlClient = new ApolloClient({
  link: finalLink,
  cache: new InMemoryCache(),
});

// biome-ignore lint/style/noNonNullAssertion: main root
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ApolloProvider client={graphqlClient}>
      <App />
    </ApolloProvider>
  </React.StrictMode>
);
