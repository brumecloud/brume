import { InstallCommand } from "~/components/install-command";
import { MistCanvas } from "~/components/mist-canvas";
import { TextLink } from "~/components/text-link";
import { site } from "~/content";

export function Hero() {
  return (
    <section
      id="top"
      aria-labelledby="hero-title"
      className="relative isolate flex min-h-svh flex-col overflow-hidden"
    >
      <MistCanvas shape="hero" seed={1} stars />
      <div className="mx-auto w-full max-w-page px-6 pt-[clamp(8rem,22vh,13rem)] pb-44">
        <p className="text-pixel text-brand">~/brume · alpha 0.1 · open source</p>
        <h1
          id="hero-title"
          className="mt-5 text-[clamp(2.6rem,7vw,5rem)] leading-[1.02] font-semibold tracking-[-0.045em]"
        >
          Your apps, online.
          <br />
          Your server, invisible.
          <span
            aria-hidden="true"
            className="ml-[0.12em] inline-block h-[0.78em] w-[0.4em] translate-y-[0.06em] animate-blink bg-brand motion-reduce:animate-none"
          />
        </h1>
        <p className="mt-7 max-w-[36rem] text-lg leading-relaxed text-pretty text-ink-muted">
          An open source platform to deploy containers, static sites and databases on your own
          server. One command to install,{" "}
          <code className="font-mono text-[0.9em] text-ink">git push</code> to deploy, zero inbound
          ports.
        </p>
        <InstallCommand command={site.installCommand} className="mt-9 max-w-[52rem]" />
        <p className="mt-6 flex flex-wrap gap-7 text-sm text-ink-muted">
          <TextLink href="#how">How it works ↓</TextLink>
          <TextLink href={site.repo}>Star on GitHub ↗</TextLink>
        </p>
      </div>
      <p
        aria-hidden="true"
        className="text-pixel absolute inset-x-0 bottom-6 text-center text-ink-muted"
      >
        scroll into the mist ↓
      </p>
    </section>
  );
}
