import { ErrorPage } from "@/pages/error";
import { Login } from "@/pages/login";
import MonitoringPage from "@/pages/monitoring";
import { Project } from "@/pages/project";
import { ProjectVariable } from "@/pages/project/project-variable";
import { BuilderPage } from "@/pages/service/builder";
import { DeploymentsPage } from "@/pages/service/deployments";
import { LogsPage } from "@/pages/service/logs";
import { RunnerPage } from "@/pages/service/runner";
import { SettingPage } from "@/pages/service/settings";
import { VariablesPage } from "@/pages/service/variables";
import { ServicePage } from "@/pages/services";
import { AccountPage } from "@/pages/settings/account";
import { AwsPage } from "@/pages/settings/cloud/aws";
import { CloudsPage } from "@/pages/settings/clouds";
import { DomainPage } from "@/pages/settings/domain";
import { AddDomain } from "@/pages/settings/domain/add";
import { GitPage } from "@/pages/settings/git";
import Stacks from "@/pages/stack";
import { DeployStack } from "@/pages/stack/deploy";
import { Marketplace } from "@/pages/stack/marketplace";
import { StackView } from "@/pages/stack/stack-view";
import { ProjectLayout } from "@/router/layout/project.layout";
import { SettingsOutlet } from "@/router/layout/settings";
import { RouteParams } from "@/router/router.param";
import { createRoutesFromElements, Route } from "react-router-dom";

import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";
import { RunnerLayout } from "./layout/runner.layout";

export const Router = createRoutesFromElements(
  <Route path="/" errorElement={<ErrorPage />}>
    <Route path="/login" element={<Login />} />
    <Route path="/" element={<ConsoleLayout />}>
      <Route path="/" element={<PageLayout />}>
        <Route path="/overview">
          <Route index element={<Stacks />} />
          <Route path="marketplace" element={<Marketplace />} />
          <Route path="deploy" element={<DeployStack />} />
          <Route
            path={`:${RouteParams.StackID}`}
            element={<StackView />}
          />
        </Route>

        <Route path="/monitoring" element={<MonitoringPage />} />
        {/* <Route path="/network" element={<h1>Network</h1>} />
        <Route path="/monitoring" element={<h1>Monitoring</h1>} /> */}
        <Route path="/settings" element={<SettingsOutlet />}>
          <Route path="account" element={<AccountPage />} />
          <Route path="clouds" element={<CloudsPage />} />
          <Route path="domains" element={<DomainPage />} />
          <Route path="git" element={<GitPage />} />
        </Route>

        <Route path="/settings/domains/add" element={<AddDomain />} />
        <Route path="/settings/cloud">
          <Route path="aws" element={<AwsPage />} />
        </Route>

        <Route
          path={`/:${RouteParams.ProjectID}/`}
          element={<ProjectLayout />}>
          <Route path="variables" element={<ProjectVariable />} />
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
