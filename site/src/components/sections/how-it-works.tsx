import type { ReactNode } from "react";

import { Section } from "~/components/section";
import { Arrow, Dim, Link, Ok, Prompt, Terminal } from "~/components/terminal";
import { site } from "~/content";

interface Step {
  readonly title: string;
  readonly text: string;
  readonly terminal: ReactNode;
}

// The outputs are illustrative until the alpha ships.
const steps: readonly Step[] = [
  {
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
    <Section id="how" kicker="How it works" title="From a fresh VPS to production in three steps.">
      <ol className="mt-14 grid gap-16">
        {steps.map((step, index) => (
          <li
            key={step.title}
            className="grid items-center gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)] lg:gap-12"
          >
            <div>
              <p className="text-pixel text-brand">{String(index + 1).padStart(2, "0")}</p>
              <h3 className="mt-2.5 text-2xl font-semibold tracking-[-0.025em]">{step.title}</h3>
              <p className="mt-3 leading-relaxed text-ink-muted">{step.text}</p>
            </div>
            {step.terminal}
          </li>
        ))}
      </ol>
    </Section>
  );
}
