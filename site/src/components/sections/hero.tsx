import { BrandIcon } from "~/components/brand-icon";
import { type BrandIconName, brandIcons } from "~/components/brand-icons";
import { InstallCommand } from "~/components/install-command";
import { MistCanvas } from "~/components/mist-canvas";
import { TextLink } from "~/components/text-link";
import { site } from "~/content";

const worksWith: readonly BrandIconName[] = ["github", "cloudflare", "tailscale", "docker"];

export function Hero() {
  return (
    <section
      id="top"
      aria-labelledby="hero-title"
      className="relative isolate flex min-h-svh flex-col overflow-hidden"
    >
      <MistCanvas shape="hero" seed={1} stars />
      <div className="mx-auto w-full max-w-page px-6 pt-[clamp(8rem,22vh,13rem)] pb-44">
        <h1
          id="hero-title"
          className="text-[clamp(3.5rem,11vw,8rem)] leading-[0.95] font-semibold tracking-[-0.05em]"
        >
          just push it
          <span
            aria-hidden="true"
            className="ml-[0.12em] inline-block h-[0.78em] w-[0.4em] translate-y-[0.06em] animate-blink bg-brand motion-reduce:animate-none"
          />
        </h1>
        <p className="mt-8 max-w-[38rem] text-lg leading-relaxed text-pretty text-ink-muted">
          Brume deploys your containers, static sites and databases on your own server. Open source,
          one command to install, and zero inbound ports: your apps are online, your server stays
          invisible.
        </p>
        <InstallCommand command={site.installCommand} className="mt-9 max-w-[52rem]" />
        <p className="mt-6 flex flex-wrap gap-7 text-sm text-ink-muted">
          <TextLink href="#how">How it works ↓</TextLink>
          <TextLink href={site.repo}>Star on GitHub ↗</TextLink>
        </p>
        <div className="mt-12 flex flex-wrap items-center gap-x-6 gap-y-3 text-sm text-ink-muted">
          <span className="text-label text-ink-dim">Works with</span>
          <ul className="flex flex-wrap items-center gap-x-6 gap-y-3">
            {worksWith.map((item) => (
              <li key={item} className="flex items-center gap-2">
                <BrandIcon name={item} className="size-4" />
                {brandIcons[item].title}
              </li>
            ))}
          </ul>
        </div>
      </div>
      <p
        aria-hidden="true"
        className="text-label absolute inset-x-0 bottom-6 text-center text-ink-muted"
      >
        scroll into the mist ↓
      </p>
    </section>
  );
}
