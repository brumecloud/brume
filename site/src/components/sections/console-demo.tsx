import { ConsoleStudio } from "~/components/console-studio";
import { SectionHeading } from "~/components/section";
import { Arrow, Dim, Link, Ok, Terminal, Wait } from "~/components/terminal";
import { agentPoints, copy } from "~/content";

export function ConsoleDemo() {
  return (
    <section id="console" aria-labelledby="console-title" className="scroll-mt-8 py-24">
      <div className="mx-auto w-full max-w-page px-6">
        <SectionHeading
          id="console-title"
          title={copy.console.title}
          lede={`${copy.console.lede} ${copy.console.invitation}`}
        />
      </div>
      {/* Wider than the text: the shot takes the width of the screen. */}
      <ConsoleStudio />

      <div
        id="agents"
        className="mx-auto mt-24 grid w-full max-w-page scroll-mt-8 items-center gap-10 px-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] lg:gap-14"
      >
        <div data-light className="group reveal">
          <h3 className="text-[clamp(1.5rem,2.8vw,2rem)] leading-tight font-semibold tracking-[-0.03em] text-balance">
            {copy.console.agentTitle}
          </h3>
          <p className="mt-4 max-w-[48ch] text-[17px] leading-relaxed text-pretty text-ink-muted">
            {copy.console.agentLede}
          </p>
          <ul className="mt-6 grid gap-3">
            {agentPoints.map((point) => (
              <li key={point} className="flex gap-3 leading-snug">
                <span
                  aria-hidden="true"
                  className="w-3 shrink-0 text-ink-dim transition-colors duration-300 lit:text-emerald-400"
                >
                  ✓
                </span>
                {point}
              </li>
            ))}
          </ul>
        </div>
        {/* An AI agent driving Brume through MCP. Illustrative until the alpha ships. */}
        <div data-light className="reveal">
          <Terminal
            title="claude code · ~/acme/api"
            wrap
            lines={[
              <>
                <Dim>{"> "}</Dim>
                ship this folder, with a Postgres database
              </>,
              <>
                <Ok />
                postgres ready · DATABASE_URL set as a secret
              </>,
              <>
                <Ok />
                build #12 · railpack · node 22
              </>,
              <>
                <Arrow />
                live at <Link>https://api.example.com</Link>
              </>,
              <>
                <Dim>{"> "}</Dim>
                now delete the old staging environment
              </>,
              <>
                <Wait />
                approval required · waiting for you in the panel
              </>,
            ]}
          />
        </div>
      </div>
    </section>
  );
}
