import type { ReactNode } from "react";

import { PlugZapIcon } from "~/components/animate-ui/icons/plug-zap";
import { SendIcon } from "~/components/animate-ui/icons/send";
import { TerminalIcon } from "~/components/animate-ui/icons/terminal";
import { Command } from "~/components/command";
import { HoverAnimate } from "~/components/hover-animate";
import { type AnimatedIcon, IconTile } from "~/components/icon-tile";
import { Section } from "~/components/section";
import { Arrow, Dim, Link, Ok, Prompt, Terminal } from "~/components/terminal";
import type { Tone } from "~/components/tone";
import { type StepId, copy, site, steps } from "~/content";

interface StepVisual {
  readonly icon: AnimatedIcon;
  readonly tone: Tone;
  readonly terminal: ReactNode;
}

// The outputs are illustrative until the alpha ships.
const visuals: Record<StepId, StepVisual> = {
  install: {
    icon: TerminalIcon,
    tone: "amber",
    terminal: (
      <Terminal
        title="root@vps"
        wrap
        lines={[
          <>
            <Prompt />
            {site.installCommand}
          </>,
          <>
            <Ok />
            binary signature verified (minisign)
          </>,
          <>
            <Ok />
            firewall closed, no inbound ports
          </>,
          <>
            <Ok />
            joined your tailnet as brume
          </>,
          <>
            <Ok />
            cloudflare tunnel ready for *.example.com
          </>,
          <>
            <Ok />
            state saved to s3://brume-state, encrypted
          </>,
          <>
            <Arrow />
            panel: <Link>https://brume.your-tailnet.ts.net</Link>
          </>,
        ]}
      />
    ),
  },
  github: {
    icon: PlugZapIcon,
    tone: "violet",
    terminal: (
      <Terminal
        title="panel · new service"
        lines={[
          <>
            <Dim>{"repo    "}</Dim>
            {" acme/api"}
          </>,
          <>
            <Dim>{"branch  "}</Dim>
            {" main"}
          </>,
          <>
            <Dim>{"build   "}</Dim>
            {" railpack, auto-detected"}
          </>,
          <>
            <Dim>{"domain  "}</Dim>
            {" api.example.com"}
          </>,
          <>
            <Ok />
            service created
          </>,
        ]}
      />
    ),
  },
  push: {
    icon: SendIcon,
    tone: "emerald",
    terminal: (
      <Terminal
        title="~/acme/api"
        lines={[
          <>
            <Prompt />
            git push origin main
          </>,
          <>
            <Arrow />
            build #42 · railpack · node 22
          </>,
          <>
            <Ok />
            image sha256:9f2c…e41a
          </>,
          <>
            <Ok />
            live at <Link>https://api.example.com</Link>
          </>,
        ]}
      />
    ),
  },
};

export function HowItWorks() {
  return (
    <Section id="how" title={copy.howItWorks.title}>
      <ol className="mt-14 grid gap-16">
        {steps.map((step, index) => (
          <li
            key={step.title}
            data-light
            className="reveal grid items-center gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)] lg:gap-12"
          >
            <HoverAnimate className="group">
              <div className="flex items-center gap-3">
                <IconTile icon={visuals[step.id].icon} tone={visuals[step.id].tone} />
                <p className="text-label text-ink-dim">Step {index + 1}</p>
              </div>
              <h3 className="mt-4 text-2xl font-semibold tracking-[-0.025em]">
                {step.command ? <Command className="text-xl">{step.title}</Command> : step.title}
              </h3>
              <p className="mt-3 leading-relaxed text-ink-muted">{step.text}</p>
            </HoverAnimate>
            {visuals[step.id].terminal}
          </li>
        ))}
      </ol>
    </Section>
  );
}
