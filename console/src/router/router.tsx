import { Login } from "@/pages/login";
import { Projects } from "@/pages/projects";
import { Routes, Route } from "react-router-dom";

import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";

export enum RouteParams {
  ProjectID = "projectId",
  ServiceID = "serviceId",
}

export function Router() {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route path="/" element={<ConsoleLayout />}>
        <Route path="/" element={<PageLayout />}>
          <Route path="/overview" element={<Projects />} />
          <Route path="/compute" element={<h1>Compute</h1>} />
          <Route path="/network" element={<h1>Network</h1>} />
          <Route path="/monitoring" element={<h1>Monitoring</h1>} />
          <Route path="/settings" element={<h1>Settings</h1>} />

          <Route path={`/:${RouteParams.ProjectID}/`}>
            <Route path="variables" element={<h1>Project Variables</h1>} />
            <Route path="logs" element={<h1>Project Logs</h1>} />
            <Route path="metrics" element={<h1>Project Metrics</h1>} />
            <Route path="services" element={<h1>Project Services</h1>} />

            <Route path={`services/:${RouteParams.ServiceID}`}>
              <Route path="builder" element={<h1>Service Builder</h1>} />
              <Route path="runner" element={<h1>Service Runner</h1>} />
              <Route path="network" element={<h1>Service Network</h1>} />
              <Route path="settings" element={<h1>Service Settings</h1>} />
            </Route>

            <Route index element={<h1>Project Home</h1>} />
          </Route>

          <Route index element={<h1>Home</h1>} />
        </Route>
      </Route>
    </Routes>
  );
}
