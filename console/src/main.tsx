import { ApolloClient, InMemoryCache, ApolloProvider } from "@apollo/client";
import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

const graphqlClient = new ApolloClient({
  uri: "http://localhost:9877/graphql",
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
