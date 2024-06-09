import { Routes, Route } from "react-router-dom";

import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";

export enum RouteParmas {
  ProjectID = "projectId",
}

export function Router() {
  return (
    <Routes>
      <Route path="/login" element={<h1>Login</h1>} />
      <Route path="/" element={<ConsoleLayout />}>
        <Route path="/" element={<h1>General Project</h1>} />
        <Route path={`/:${RouteParmas.ProjectID}/`} element={<PageLayout />}>
          <Route path="services" element={<h1>Services</h1>} />
        </Route>
      </Route>
    </Routes>
  );
}
