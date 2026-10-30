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
import { DomainsSetting } from "./domains-setting";
import { BranchSetting, RepositorySetting } from "./github-source-settings";
import { LimitSetting } from "./limit-setting";
import { SettingsSection } from "./settings-section";
import { SwitchSetting } from "./switch-setting";
import { TextSetting } from "./text-setting";
import { useSettingCommit } from "./use-setting-commit";

const SECTIONS: Record<SettingsSectionId, { readonly title: string; readonly icon: ReactNode }> = {
  source: { title: "Source", icon: <CodeXmlIcon /> },
  network: { title: "Réseau", icon: <NetworkIcon /> },
  deploy: { title: "Déploiement", icon: <RocketIcon /> },
  resources: { title: "Ressources", icon: <CpuIcon /> },
  volume: { title: "Volume", icon: <HardDriveIcon /> },
  danger: { title: "Zone dangereuse", icon: <TriangleAlertIcon /> },
};

const TEMPLATE_NAMES = { postgres: "PostgreSQL", mongodb: "MongoDB", redis: "Redis" } as const;

interface ServiceSettingsViewProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
}

// Reference: infra/screens/04e-service-settings-*.png.
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
  const { commit, error } = useSettingCommit(service.id, settings, service.domains, changes);
  const staged = withStagedSettings(settings, service.domains, changes);
  const fields = stagedFields(changes);
  const { source } = staged.settings;
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
                  label="Déploiement automatique"
                  description="Déployer à chaque push sur la branche."
                  text="Déployer automatiquement"
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
                description="Tirée et épinglée par son digest à chaque déploiement."
                value={source.image}
                staged={fields.has("image")}
                onCommit={(value) => commit("image", value)}
              />
            )}
            {source.kind === "template" && (
              <ReadOnlySetting
                label="Template"
                description="Les versions mineures suivent les releases de Brume."
              >
                <ServiceIcon kind={source.template} />
                {TEMPLATE_NAMES[source.template]} {source.version}
              </ReadOnlySetting>
            )}
          </SettingsSection>
        )}
        {sections.includes("network") && (
          <SettingsSection id="settings-network" {...SECTIONS.network}>
            {!isDatabase(service.kind) && (
              <DomainsSetting
                domains={staged.domains}
                staged={fields.has("domains")}
                onCommit={(domains) => commit("domains", domains)}
              />
            )}
            <ReadOnlySetting
              label="Nom interne"
              description="Les services de l'environnement le joignent sous ce nom."
            >
              <span className="font-mono">{service.slug}</span>
            </ReadOnlySetting>
            {isDatabase(service.kind) ? (
              <ReadOnlySetting label="Accès Internet sortant">
                Coupé : une base ne sort jamais sur Internet.
              </ReadOnlySetting>
            ) : (
              <SwitchSetting
                label="Accès Internet sortant"
                description="Coupez-le pour un service qui n'appelle rien à l'extérieur."
                text="Autoriser les connexions sortantes"
                checked={staged.settings.egress}
                staged={fields.has("egress")}
                onCommit={(checked) => commit("egress", checked)}
              />
            )}
          </SettingsSection>
        )}
        {sections.includes("deploy") && (
          <SettingsSection id="settings-deploy" {...SECTIONS.deploy}>
            <TextSetting
              key={`startCommand:${staged.settings.startCommand ?? ""}`}
              field="startCommand"
              label="Commande de démarrage"
              description="Vide : la commande de l'image."
              value={staged.settings.startCommand ?? ""}
              placeholder="pnpm start"
              staged={fields.has("startCommand")}
              onCommit={(value) => commit("startCommand", value)}
            />
            <TextSetting
              key={`port:${staged.settings.port ?? ""}`}
              field="port"
              label="Port"
              description="Le port sur lequel le service écoute."
              value={String(staged.settings.port ?? "")}
              placeholder="3000"
              staged={fields.has("port")}
              onCommit={(value) => commit("port", value)}
            />
            <TextSetting
              key={`healthcheckPath:${staged.settings.healthcheckPath ?? ""}`}
              field="healthcheckPath"
              label="Healthcheck"
              description="Le trafic bascule quand ce chemin répond 200. Vide : celui de l'image."
              value={staged.settings.healthcheckPath ?? ""}
              placeholder="/health"
              staged={fields.has("healthcheckPath")}
              onCommit={(value) => commit("healthcheckPath", value)}
            />
          </SettingsSection>
        )}
        {sections.includes("resources") && (
          <SettingsSection id="settings-resources" {...SECTIONS.resources}>
            <LimitSetting
              key={`cpu:${staged.settings.cpuLimit}`}
              label="CPU"
              steps={CPU_LIMITS}
              value={staged.settings.cpuLimit}
              format={formatCpu}
              staged={fields.has("cpuLimit")}
              onCommit={(value) => commit("cpuLimit", value)}
            />
            <LimitSetting
              key={`memory:${staged.settings.memoryLimitMb}`}
              label="Mémoire"
              steps={MEMORY_LIMITS_MB}
              value={staged.settings.memoryLimitMb}
              format={formatMemory}
              staged={fields.has("memoryLimitMb")}
              onCommit={(value) => commit("memoryLimitMb", value)}
            />
          </SettingsSection>
        )}
        {sections.includes("volume") && service.volume && (
          <SettingsSection id="settings-volume" {...SECTIONS.volume}>
            <ReadOnlySetting label="Nom">{service.volume.name}</ReadOnlySetting>
            <ReadOnlySetting label="Monté sur">
              <span className="font-mono">{service.volume.mountPath}</span>
            </ReadOnlySetting>
          </SettingsSection>
        )}
        {sections.includes("danger") && (
          <SettingsSection id="settings-danger" {...SECTIONS.danger}>
            <div className="flex items-center justify-between gap-4 rounded-lg border border-destructive/32 bg-background px-4 py-3">
              <span className="text-muted-foreground text-sm">
                Supprimer le service, ses déploiements et son volume.
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
