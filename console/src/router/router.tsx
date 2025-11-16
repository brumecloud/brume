import { createRoutesFromElements, Route } from "react-router-dom";
import { ErrorPage } from "@/pages/error";
import { Login } from "@/pages/login";
import { Logout } from "@/pages/logout";
import MonitoringPage from "@/pages/monitoring";
import { Project } from "@/pages/project";
import { ProjectVariable } from "@/pages/project/project-variable";
import { BuilderPage } from "@/pages/service/builder";
import { RunnerPage } from "@/pages/service/runner";
import { SettingPage } from "@/pages/service/settings";
import { SourcePage } from "@/pages/service/source";
import { VariablesPage } from "@/pages/service/variables";
import { ServicePage } from "@/pages/services";
import { AccountPage } from "@/pages/settings/account";
import { AwsPage } from "@/pages/settings/cloud/aws";
import { CloudAccountPage } from "@/pages/settings/cloud/id/overview";
import { CloudSettingPage } from "@/pages/settings/cloud/id/setting";
import { CloudStackPage } from "@/pages/settings/cloud/id/stack";
import { CloudsPage } from "@/pages/settings/clouds";
import { DomainPage } from "@/pages/settings/domain";
import { AddDomain } from "@/pages/settings/domain/add";
import Stacks from "@/pages/stack";
import { DeployStack } from "@/pages/stack/deploy";
import { Marketplace } from "@/pages/stack/marketplace";
import { StackView } from "@/pages/stack/stack-view";
import { ProjectLayout } from "@/router/layout/project.layout";
import { SettingsOutlet } from "@/router/layout/settings";
import { RouteParams } from "@/router/router.param";
import { CloudLayout } from "./layout/cloud.layout";
import { ConsoleLayout } from "./layout/console.layout";
import { PageLayout } from "./layout/page.layout";
import { ProjectSettingsLayout } from "./layout/project-settings.layout";
import { RunnerLayout } from "./layout/runner.layout";

export const Router = createRoutesFromElements(
  <Route errorElement={<ErrorPage />} path="/">
    <Route element={<Login />} path="/login" />
    <Route element={<Logout />} path="/logout" />
    <Route element={<ConsoleLayout />} path="/">
      <Route element={<PageLayout />} path="/">
        <Route path="/overview">
          <Route element={<Stacks />} index />
          <Route element={<DeployStack />} path="deploy" />
          <Route element={<StackView />} path={`:${RouteParams.StackID}`} />
        </Route>
        <Route element={<Marketplace />} path="/marketplace" />

        <Route element={<MonitoringPage />} path="/monitoring" />
        {/* <Route path="/network" element={<h1>Network</h1>} />
        <Route path="/monitoring" element={<h1>Monitoring</h1>} /> */}
        <Route element={<SettingsOutlet />} path="/settings">
          <Route element={<AccountPage />} path="account" />
          <Route element={<CloudsPage />} path="clouds" />
          <Route element={<DomainPage />} path="domains" />
        </Route>

        <Route element={<AddDomain />} path="/settings/domains/add" />
        <Route path="/settings/cloud">
          <Route element={<AwsPage />} path="aws" />
          <Route
            element={<CloudLayout />}
            path={`:${RouteParams.CloudAccountID}`}
          >
            <Route element={<CloudAccountPage />} path="overview" />
            <Route element={<CloudStackPage />} path="stacks" />
            <Route element={<CloudSettingPage />} path="settings" />
          </Route>
        </Route>

        <Route element={<ProjectLayout />} path={`/:${RouteParams.ProjectID}/`}>
          <Route element={<ProjectVariable />} path="variables" />
          <Route element={<h1>Project Metrics</h1>} path="metrics" />
          <Route element={<ServicePage />} path="services" />

          <Route path={`services/:${RouteParams.ServiceID}`}>
            <Route element={<BuilderPage />} path="builder" />
            <Route element={<SourcePage />} path="source" />
            <Route element={<RunnerLayout />} path="runner">
              <Route element={<RunnerPage />} index />
            </Route>
            <Route element={<h1>Service Network</h1>} path="network" />
            <Route element={<ProjectSettingsLayout />} path="settings">
              <Route element={<SettingPage />} index />
              <Route element={<VariablesPage />} path="variables" />
            </Route>
          </Route>

          <Route element={<Project />} index />
        </Route>

        <Route element={<h1>Home</h1>} index />
      </Route>
    </Route>
  </Route>
);
