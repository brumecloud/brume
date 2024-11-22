import { ErrorPage } from "@/pages/error";
import { Login } from "@/pages/login";
import { Project } from "@/pages/project";
import { Projects } from "@/pages/projects";
import { BuilderPage } from "@/pages/service/builder";
import { DeploymentsPage } from "@/pages/service/deployments";
import { LogsPage } from "@/pages/service/logs";
import { RunnerPage } from "@/pages/service/runner";
import { SettingPage } from "@/pages/service/settings";
import { VariablesPage } from "@/pages/service/variables";
import { ServicePage } from "@/pages/services";
import { Route, createRoutesFromElements } from "react-router-dom";

import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";
import { RunnerLayout } from "./layout/runner.layout";

export enum RouteParams {
  ProjectID = "projectId",
  ServiceID = "serviceId",
}

export const Router = createRoutesFromElements(
  <Route path="/" errorElement={<ErrorPage />}>
    <Route path="/login" element={<Login />} />
    <Route path="/" element={<ConsoleLayout />}>
      <Route path="/" element={<PageLayout />}>
        <Route path="/overview" element={<Projects />} />
        <Route path="/compute" element={<h1>Compute</h1>} />
        <Route path="/network" element={<h1>Network</h1>} />
        <Route path="/monitoring" element={<h1>Monitoring</h1>} />
        <Route path="/settings" element={<h1>Settings</h1>} />

        <Route path={`/:${RouteParams.ProjectID}/`}>
          <Route
            path="variables"
            element={<h1>Project Variables</h1>}
          />
          <Route path="logs" element={<LogsPage />} />
          <Route path="metrics" element={<h1>Project Metrics</h1>} />
          <Route path="services" element={<ServicePage />} />

          <Route path={`services/:${RouteParams.ServiceID}`}>
            <Route path="builder" element={<BuilderPage />} />
            <Route path="runner" element={<RunnerLayout />}>
              <Route path="variables" element={<VariablesPage />} />
              <Route
                path="deployments"
                element={<DeploymentsPage />}
              />
              <Route index element={<RunnerPage />} />
            </Route>
            <Route
              path="network"
              element={<h1>Service Network</h1>}
            />
            <Route path="settings" element={<SettingPage />} />
          </Route>

          <Route index element={<Project />} />
        </Route>

        <Route index element={<h1>Home</h1>} />
      </Route>
    </Route>
  </Route>
);
