import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useSuspenseQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { serviceById, serviceSettingsQuery } from "~/api/services";
import type { ProjectId, ServiceId, ServiceSource } from "~/api/types";
import { domainUrl, formatLimits } from "~/helpers/services";

import { DeleteServiceDialog } from "./delete-service-dialog";

const TEMPLATE_NAMES = { postgres: "PostgreSQL", mongodb: "MongoDB", redis: "Redis" } as const;

interface ServiceSettingsViewProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
}

// Reference: infra/screens/04e-service-settings-*.png. Read-only for now: editing a field
// creates a revision, which comes with the deployment details.
export function ServiceSettingsView({
  projectId,
  environmentName,
  serviceId,
}: ServiceSettingsViewProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: settings } = useSuspenseQuery(serviceSettingsQuery(serviceId));
  if (!service) {
    return null;
  }

  return (
    <div className="flex flex-col gap-8 p-6">
      <Section title="Source">
        <SourceRows source={settings.source} />
      </Section>
      <Section title="Réseau">
        <Row label="Nom interne">
          <span className="font-mono">{service.slug}</span>
        </Row>
        <Row label="Domaines publics">
          {service.domains.length === 0
            ? "Aucun"
            : service.domains.map((domain) => (
                <a
                  key={domain}
                  href={domainUrl(domain)}
                  target="_blank"
                  rel="noreferrer"
                  className="block hover:underline"
                >
                  {domain}
                </a>
              ))}
        </Row>
        <Row label="Accès Internet sortant">{settings.egress ? "Autorisé" : "Coupé"}</Row>
      </Section>
      <Section title="Déploiement">
        <Row label="Commande">
          <span className="font-mono">{settings.startCommand ?? "Celle de l'image"}</span>
        </Row>
        <Row label="Port">{settings.port ?? "—"}</Row>
        <Row label="Healthcheck">
          <span className="font-mono">{settings.healthcheckPath ?? "Celui de l'image"}</span>
        </Row>
        <Row label="Limites">{formatLimits(settings.cpuLimit, settings.memoryLimitMb)}</Row>
      </Section>
      {service.volume && (
        <Section title="Volume">
          <Row label="Nom">{service.volume.name}</Row>
          <Row label="Monté sur">
            <span className="font-mono">{service.volume.mountPath}</span>
          </Row>
        </Section>
      )}
      <Section title="Zone dangereuse">
        <div className="flex items-center justify-between gap-4 px-4 py-3">
          <span className="text-muted-foreground text-sm">
            Supprimer le service, ses déploiements et son volume.
          </span>
          <DeleteServiceDialog
            projectId={projectId}
            environmentName={environmentName}
            service={service}
          />
        </div>
      </Section>
    </div>
  );
}

function SourceRows({ source }: { readonly source: ServiceSource }) {
  switch (source.kind) {
    case "github":
      return (
        <>
          <Row label="Dépôt">{source.repository}</Row>
          <Row label="Branche">{source.branch}</Row>
          <Row label="Déploiement automatique">
            {source.autoDeploy ? "À chaque push sur la branche" : "Désactivé"}
          </Row>
        </>
      );
    case "image":
      return (
        <Row label="Image">
          <span className="font-mono">{source.image}</span>
        </Row>
      );
    case "template":
      return (
        <Row label="Template">
          {TEMPLATE_NAMES[source.template]} {source.version}
        </Row>
      );
  }
}

function Section({ title, children }: { readonly title: string; readonly children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-medium">{title}</h3>
      <div className="flex flex-col divide-y rounded-xl border bg-background">{children}</div>
    </section>
  );
}

function Row({ label, children }: { readonly label: string; readonly children: ReactNode }) {
  return (
    <div className="grid grid-cols-1 gap-1 px-4 py-3 text-sm sm:grid-cols-3 sm:gap-4">
      <span className="text-muted-foreground">{label}</span>
      <span className="min-w-0 break-words sm:col-span-2">{children}</span>
    </div>
  );
}
