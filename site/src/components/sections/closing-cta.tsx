import { InstallCommand } from "~/components/install-command";
import { MistCanvas } from "~/components/mist-canvas";
import { TextLink } from "~/components/text-link";
import { requirements, site } from "~/content";

export function ClosingCta() {
  return (
    <section
      id="install"
      aria-labelledby="install-title"
      className="relative isolate overflow-hidden pt-28 pb-60"
    >
      <MistCanvas shape="floor" palette="dawn" seed={3} stars />
      <div className="mx-auto flex w-full max-w-page flex-col items-center px-6 text-center">
        <h2
          id="install-title"
          className="text-[clamp(1.875rem,4.4vw,3.25rem)] leading-[1.1] font-semibold tracking-[-0.035em] text-balance"
        >
          Make your server invisible.
        </h2>
        <p className="mt-4 text-[17px] text-ink-muted">One command on a fresh VPS.</p>
        <InstallCommand
          command={site.installCommand}
          className="mt-9 w-full max-w-[52rem] text-left"
        />
        <p className="mt-4 max-w-[46rem] text-sm text-pretty text-ink-dim">{requirements}</p>
        <p className="mt-6 flex flex-wrap justify-center gap-7 text-sm text-ink-muted">
          <TextLink href="#how">How it works ↑</TextLink>
          <TextLink href={site.repo}>Star on GitHub ↗</TextLink>
        </p>
      </div>
    </section>
  );
}
