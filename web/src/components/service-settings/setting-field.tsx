import type { ReactNode } from "react";

interface SettingFieldProps {
  readonly label: string;
  readonly description?: string;
  // A value waits for the next deploy: the control gets a blue outline.
  readonly staged: boolean;
  readonly error?: string | undefined;
  readonly children: ReactNode;
}

export function SettingField({ label, description, staged, error, children }: SettingFieldProps) {
  return (
    <div className="flex flex-col gap-2">
      <div>
        <p className="font-medium text-sm">{label}</p>
        {description && <p className="text-muted-foreground text-sm">{description}</p>}
      </div>
      <div
        data-staged={staged}
        className="rounded-lg data-[staged=true]:outline-2 data-[staged=true]:outline-info data-[staged=true]:outline-offset-2"
      >
        {children}
      </div>
      {error && <p className="text-destructive-foreground text-sm">{error}</p>}
    </div>
  );
}
