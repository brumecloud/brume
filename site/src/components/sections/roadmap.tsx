import { Section } from "~/components/section";
import { type MilestoneStatus, copy, milestones } from "~/content";

// Gray at rest: each milestone takes its color while lit.

const statusStyles: Record<
  MilestoneStatus,
  { readonly label: string; readonly node: string; readonly marker: string }
> = {
  current: {
    label: "text-ink-muted lit:text-amber-400",
    node: "bg-ink-muted lit:bg-amber-400",
    marker: "bg-ink-dim lit:bg-amber-400",
  },
  next: {
    label: "text-ink-dim lit:text-violet-400",
    node: "bg-night ring-2 ring-edge-strong lit:ring-violet-400",
    marker: "bg-ink-dim lit:bg-violet-400",
  },
  later: {
    label: "text-ink-dim lit:text-sky-400",
    node: "bg-night ring-2 ring-edge-strong lit:ring-sky-400",
    marker: "bg-ink-dim lit:bg-sky-400",
  },
  ongoing: {
    label: "text-ink-dim lit:text-emerald-400",
    node: "bg-night ring-2 ring-edge-strong lit:ring-emerald-400",
    marker: "bg-ink-dim lit:bg-emerald-400",
  },
};

export function Roadmap() {
  return (
    <Section id="roadmap" title={copy.roadmap.title}>
      <ol className="timeline mt-14 grid gap-12">
        {milestones.map((milestone) => {
          const style = statusStyles[milestone.status];
          return (
            <li
              key={milestone.title}
              data-light
              className="group reveal relative grid gap-3 pl-10 md:grid-cols-[11rem_minmax(0,1fr)] md:gap-8"
            >
              <span
                aria-hidden="true"
                className={`absolute top-1 left-0 size-4 transition duration-300 ${style.node}`}
              >
                {milestone.status === "current" ? (
                  <span className="absolute inset-0 animate-ping bg-ink/40 lit:bg-amber-400/60 motion-reduce:animate-none" />
                ) : null}
              </span>
              <p className={`text-label pt-0.5 transition-colors duration-300 ${style.label}`}>
                {milestone.when}
              </p>
              <div>
                <h3 className="text-xl font-semibold tracking-[-0.02em]">{milestone.title}</h3>
                <ul
                  className={`mt-4 gap-x-8 text-ink-muted ${milestone.items.length > 4 ? "sm:columns-2" : ""}`}
                >
                  {milestone.items.map((item) => (
                    <li key={item} className="mb-2.5 flex break-inside-avoid gap-3 leading-snug">
                      <span
                        aria-hidden="true"
                        className={`mt-[0.45em] size-1.5 shrink-0 transition-colors duration-300 ${style.marker}`}
                      />
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </li>
          );
        })}
      </ol>
    </Section>
  );
}
