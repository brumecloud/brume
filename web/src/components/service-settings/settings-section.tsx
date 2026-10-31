import type { ReactNode } from "react";

interface SettingsSectionProps {
  readonly id: string;
  readonly icon: ReactNode;
  readonly title: string;
  readonly children: ReactNode;
}

// Reference: infra/screens/04e-service-settings-*.png. One step of the timeline on the left.
export function SettingsSection({ id, icon, title, children }: SettingsSectionProps) {
  return (
    <section id={id} className="relative flex scroll-mt-6 gap-5">
      <span className="z-10 flex size-9 shrink-0 items-center justify-center rounded-full border bg-card text-muted-foreground [&_svg]:size-4">
        {icon}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-6 pt-1.5 pb-12">
        <h3 className="font-medium text-lg text-muted-foreground">{title}</h3>
        {children}
      </div>
    </section>
  );
}
