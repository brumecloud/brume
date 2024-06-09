import { Routes, Route } from "react-router-dom";

import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";

export enum RouteParams {
  ProjectID = "projectId",
}

export function Router() {
  return (
    <Routes>
      <Route path="/login" element={<h1>Login</h1>} />
      <Route path="/" element={<ConsoleLayout />}>
        <Route path="/" element={<PageLayout />}>
          <Route index element={<h1>Home</h1>} />
          <Route path={`/:${RouteParams.ProjectID}/`}>
            <Route path="services" element={<h1>Services</h1>} />
          </Route>
        </Route>
      </Route>
    </Routes>
  );
}
