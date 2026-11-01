import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useSuspenseQuery } from "@tanstack/react-query";
import {
  CodeXmlIcon,
  CpuIcon,
  HardDriveIcon,
  NetworkIcon,
  RocketIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { type ReactNode, useState } from "react";

import { serviceChanges } from "~/api/changes";
import { serviceById, serviceSettingsQuery } from "~/api/services";
import type { ProjectId, Service, ServiceId, ServiceSettings, StagedChange } from "~/api/types";
import { ServiceIcon } from "~/components/services/service-icon";
import { CPU_LIMITS, MEMORY_LIMITS_MB, stagedFields, withStagedSettings } from "~/helpers/changes";
import { isNearBottom } from "~/helpers/logs";
import {
  type SettingsSectionId,
  currentSection,
  settingsSections,
} from "~/helpers/service-settings";
import { formatCpu, formatMemory, isDatabase } from "~/helpers/services";

import { DeleteServiceDialog } from "./delete-service-dialog";
import { DomainSetting } from "./domain-setting";
import { BranchSetting, RepositorySetting } from "./github-source-settings";
import { LimitSetting } from "./limit-setting";
import { SettingsSection } from "./settings-section";
import { SwitchSetting } from "./switch-setting";
import { TextSetting } from "./text-setting";
import { useSettingCommit } from "./use-setting-commit";

const SECTIONS: Record<SettingsSectionId, { readonly title: string; readonly icon: ReactNode }> = {
  source: { title: "Source", icon: <CodeXmlIcon /> },
  network: { title: "Networking", icon: <NetworkIcon /> },
  deploy: { title: "Deploy", icon: <RocketIcon /> },
  resources: { title: "Resources", icon: <CpuIcon /> },
  volume: { title: "Volume", icon: <HardDriveIcon /> },
  danger: { title: "Danger zone", icon: <TriangleAlertIcon /> },
};

const TEMPLATE_NAMES = { postgres: "PostgreSQL", mongodb: "MongoDB", redis: "Redis" } as const;

interface ServiceSettingsViewProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
}

export function ServiceSettingsView({
  projectId,
  environmentName,
  serviceId,
}: ServiceSettingsViewProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: settings } = useSuspenseQuery(serviceSettingsQuery(serviceId));
  const { data: changes } = useLiveSuspenseQuery({ query: serviceChanges(serviceId) });
  if (!service) {
    return null;
  }

  return (
    <SettingsForm
      projectId={projectId}
      environmentName={environmentName}
      service={service}
      settings={settings}
      changes={changes}
    />
  );
}

interface SettingsFormProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly service: Service;
  readonly settings: ServiceSettings;
  readonly changes: readonly StagedChange[];
}

// Every edit is a draft: outlined in blue, applied by the deploy bar of the canvas.
function SettingsForm({
  projectId,
  environmentName,
  service,
  settings,
  changes,
}: SettingsFormProps) {
  const { commit, error } = useSettingCommit(service.id, settings, changes);
  const staged = withStagedSettings(settings, changes);
  const fields = stagedFields(changes);
  const { source } = staged;
  const sections = settingsSections(service);
  const [current, setCurrent] = useState(sections[0]);
  // A click on the nav pins its section, even when the bottom of the page stops it short of the
  // top. The highlight follows the scroll again once the admin scrolls by hand.
  const [pinned, setPinned] = useState(false);

  // The form scrolls on its own, so the nav can follow the section being read.
  return (
    <div
      className="flex h-full gap-8 overflow-auto p-6 motion-safe:scroll-smooth"
      onScroll={(event) => {
        if (!pinned) {
          setCurrent(readCurrentSection(event.currentTarget, sections));
        }
      }}
      onWheel={() => setPinned(false)}
      onTouchMove={() => setPinned(false)}
      onKeyDown={() => setPinned(false)}
    >
      <div className="relative min-w-0 flex-1">
        <span aria-hidden="true" className="absolute top-0 bottom-0 left-4.5 w-px bg-border" />
        {error && (
          <p role="alert" className="mb-4 text-destructive-foreground text-sm">
            {error.message}
          </p>
        )}
        {sections.includes("source") && (
          <SettingsSection id="settings-source" {...SECTIONS.source}>
            {source.kind === "github" && (
              <>
                <RepositorySetting
                  value={source.repository}
                  staged={fields.has("repository")}
                  onCommit={(value) => commit("repository", value)}
                />
                <BranchSetting
                  repository={source.repository}
                  environmentName={environmentName}
                  value={source.branch}
                  staged={fields.has("branch")}
                  onCommit={(value) => commit("branch", value)}
                />
                <SwitchSetting
                  label="Auto deploy"
                  description="Deploy on every push to the branch."
                  text="Deploy automatically"
                  checked={source.autoDeploy}
                  staged={fields.has("autoDeploy")}
                  onCommit={(checked) => commit("autoDeploy", checked)}
                />
              </>
            )}
            {source.kind === "image" && (
              <TextSetting
                key={`image:${source.image}`}
                field="image"
                label="Image"
                description="Pulled and pinned by its digest on every deployment."
                value={source.image}
                staged={fields.has("image")}
                onCommit={(value) => commit("image", value)}
              />
            )}
            {source.kind === "template" && (
              <ReadOnlySetting label="Template" description="Minor versions follow Brume releases.">
                <ServiceIcon kind={source.template} />
                {TEMPLATE_NAMES[source.template]} {source.version}
              </ReadOnlySetting>
            )}
          </SettingsSection>
        )}
        {sections.includes("network") && (
          <SettingsSection id="settings-network" {...SECTIONS.network}>
            {!isDatabase(service.kind) && (
              <DomainSetting
                key={`domain:${staged.domain ?? ""}`}
                value={staged.domain ?? ""}
                domains={service.domains}
                staged={fields.has("domain")}
                onCommit={(value) => commit("domain", value)}
              />
            )}
            <ReadOnlySetting
              label="Internal hostname"
              description="Other services in the environment reach it at this name."
            >
              <span className="font-mono">{service.slug}</span>
            </ReadOnlySetting>
            {isDatabase(service.kind) ? (
              <ReadOnlySetting label="Outbound internet access">
                Off: a database never reaches the internet.
              </ReadOnlySetting>
            ) : (
              <SwitchSetting
                label="Outbound internet access"
                description="Turn it off for a service that makes no outside calls."
                text="Allow outbound connections"
                checked={staged.egress}
                staged={fields.has("egress")}
                onCommit={(checked) => commit("egress", checked)}
              />
            )}
          </SettingsSection>
        )}
        {sections.includes("deploy") && (
          <SettingsSection id="settings-deploy" {...SECTIONS.deploy}>
            <TextSetting
              key={`startCommand:${staged.startCommand ?? ""}`}
              field="startCommand"
              label="Start command"
              description="Leave empty to use the image's command."
              value={staged.startCommand ?? ""}
              placeholder="pnpm start"
              staged={fields.has("startCommand")}
              onCommit={(value) => commit("startCommand", value)}
            />
            <TextSetting
              key={`port:${staged.port ?? ""}`}
              field="port"
              label="Port"
              description="The port the service listens on."
              value={String(staged.port ?? "")}
              placeholder="3000"
              staged={fields.has("port")}
              onCommit={(value) => commit("port", value)}
            />
            <TextSetting
              key={`healthcheckPath:${staged.healthcheckPath ?? ""}`}
              field="healthcheckPath"
              label="Healthcheck"
              description="Traffic switches over once this path returns 200. Leave empty to use the image's."
              value={staged.healthcheckPath ?? ""}
              placeholder="/health"
              staged={fields.has("healthcheckPath")}
              onCommit={(value) => commit("healthcheckPath", value)}
            />
          </SettingsSection>
        )}
        {sections.includes("resources") && (
          <SettingsSection id="settings-resources" {...SECTIONS.resources}>
            <LimitSetting
              key={`cpu:${staged.cpuLimit}`}
              label="CPU"
              steps={CPU_LIMITS}
              value={staged.cpuLimit}
              format={formatCpu}
              staged={fields.has("cpuLimit")}
              onCommit={(value) => commit("cpuLimit", value)}
            />
            <LimitSetting
              key={`memory:${staged.memoryLimitMb}`}
              label="Memory"
              steps={MEMORY_LIMITS_MB}
              value={staged.memoryLimitMb}
              format={formatMemory}
              staged={fields.has("memoryLimitMb")}
              onCommit={(value) => commit("memoryLimitMb", value)}
            />
          </SettingsSection>
        )}
        {sections.includes("volume") && service.volume && (
          <SettingsSection id="settings-volume" {...SECTIONS.volume}>
            <ReadOnlySetting label="Name">{service.volume.name}</ReadOnlySetting>
            <ReadOnlySetting label="Mounted at">
              <span className="font-mono">{service.volume.mountPath}</span>
            </ReadOnlySetting>
          </SettingsSection>
        )}
        {sections.includes("danger") && (
          <SettingsSection id="settings-danger" {...SECTIONS.danger}>
            <div className="flex items-center justify-between gap-4 rounded-lg border border-destructive/32 bg-background px-4 py-3">
              <span className="text-muted-foreground text-sm">
                Delete the service and its deployments.
                {service.volume && " Its volume and its data stay."}
              </span>
              <DeleteServiceDialog
                projectId={projectId}
                environmentName={environmentName}
                service={service}
              />
            </div>
          </SettingsSection>
        )}
      </div>
      <nav className="sticky top-0 hidden w-32 shrink-0 flex-col gap-2 self-start pt-2 text-muted-foreground text-sm lg:flex">
        {sections.map((section) => (
          <a
            key={section}
            href={`#settings-${section}`}
            aria-current={section === current ? "location" : undefined}
            onClick={() => {
              setCurrent(section);
              setPinned(true);
            }}
            className="transition-colors hover:text-foreground aria-[current=location]:text-foreground"
          >
            {SECTIONS[section].title}
          </a>
        ))}
      </nav>
    </div>
  );
}

// Reads where each section sits in the scroll area, then lets the helper pick one.
function readCurrentSection(area: HTMLElement, sections: readonly SettingsSectionId[]) {
  const areaTop = area.getBoundingClientRect().top;
  const positions = sections.map((id) => ({
    id,
    top: (area.querySelector(`#settings-${id}`)?.getBoundingClientRect().top ?? 0) - areaTop,
  }));
  return currentSection(
    positions,
    isNearBottom(area.scrollTop, area.clientHeight, area.scrollHeight),
  );
}

interface ReadOnlySettingProps {
  readonly label: string;
  readonly description?: string;
  readonly children: ReactNode;
}

function ReadOnlySetting({ label, description, children }: ReadOnlySettingProps) {
  return (
    <div className="flex flex-col gap-2">
      <div>
        <p className="font-medium text-sm">{label}</p>
        {description && <p className="text-muted-foreground text-sm">{description}</p>}
      </div>
      <div className="flex items-center gap-2 rounded-lg border bg-background px-4 py-2.5 text-sm">
        {children}
      </div>
    </div>
  );
}
