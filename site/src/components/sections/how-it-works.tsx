import type { ReactNode } from "react";

import { PlugZapIcon } from "~/components/animate-ui/icons/plug-zap";
import { SendIcon } from "~/components/animate-ui/icons/send";
import { TerminalIcon } from "~/components/animate-ui/icons/terminal";
import { HoverAnimate } from "~/components/hover-animate";
import { type AnimatedIcon, IconTile } from "~/components/icon-tile";
import { Section } from "~/components/section";
import { Arrow, Dim, Link, Ok, Prompt, Terminal } from "~/components/terminal";
import type { Tone } from "~/components/tone";
import { site } from "~/content";

interface Step {
  readonly icon: AnimatedIcon;
  readonly tone: Tone;
  readonly title: string;
  readonly text: string;
  readonly terminal: ReactNode;
}

// The outputs are illustrative until the alpha ships.
const steps: readonly Step[] = [
  {
    icon: TerminalIcon,
    tone: "amber",
    title: "Install",
    text: "One command on a fresh Debian 12 or Ubuntu 24.04 server. Brume checks its own signature, closes the firewall, joins your tailnet, creates a Cloudflare tunnel and starts saving its state to your bucket.",
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
  {
    icon: PlugZapIcon,
    tone: "violet",
    title: "Connect GitHub",
    text: "Install the Brume GitHub App on a repository and pick the branch to deploy. Each build gets a token that expires within the hour, never a personal token.",
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
  {
    icon: SendIcon,
    tone: "emerald",
    title: "git push",
    text: "Every push to that branch builds with Railpack and deploys by image digest. A rollback redeploys the exact same image, with no rebuild.",
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
];

export function HowItWorks() {
  return (
    <Section
      id="how"
      kicker="How it works"
      tone="amber"
      title="From a fresh VPS to production in three steps."
    >
      <ol className="mt-14 grid gap-16">
        {steps.map((step, index) => (
          <li
            key={step.title}
            className="reveal grid items-center gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)] lg:gap-12"
          >
            <HoverAnimate>
              <div className="flex items-center gap-3">
                <IconTile icon={step.icon} tone={step.tone} />
                <p className="text-label text-ink-dim">Step {index + 1}</p>
              </div>
              <h3 className="mt-4 text-2xl font-semibold tracking-[-0.025em]">{step.title}</h3>
              <p className="mt-3 leading-relaxed text-ink-muted">{step.text}</p>
            </HoverAnimate>
            {step.terminal}
          </li>
        ))}
      </ol>
    </Section>
  );
}
