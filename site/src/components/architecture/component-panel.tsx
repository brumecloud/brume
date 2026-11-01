import { componentById } from "~/architecture/components";
import { type ArchComponent, type ComponentId, neighbors } from "~/architecture/model";
import { docsPage } from "~/content";

import { ComponentIcon } from "./component-icon";

/** The sections that explain a component. The panel and the list under the diagram share them. */
export function ComponentDetails({
  component,
  onSelect,
}: {
  readonly component: ArchComponent;
  /** In the panel, the neighbors are buttons. In the list, links to their section. */
  readonly onSelect?: (id: ComponentId) => void;
}) {
  const heading = "text-label text-ink-dim";
  return (
    <div className="grid gap-6">
      <section>
        <h4 className={heading}>Why it&apos;s here</h4>
        <p className="mt-2 leading-relaxed text-pretty">{component.why}</p>
      </section>
      <section>
        <h4 className={heading}>How it works</h4>
        <ul className="mt-2 grid gap-2 text-[15px] leading-relaxed text-ink-muted">
          {component.how.map((point) => (
            <li key={point} className="flex gap-2.5">
              <span aria-hidden="true" className="mt-[0.6em] size-1 shrink-0 bg-ink-dim" />
              {point}
            </li>
          ))}
        </ul>
      </section>
      <section>
        <h4 className={heading}>Security rules</h4>
        <ul className="mt-2 grid gap-2.5 text-[15px] leading-relaxed">
          {component.rules.map((rule) => (
            <li key={rule.rule} className="flex gap-2.5">
              <span aria-hidden="true" className="shrink-0 text-emerald-400">
                ✓
              </span>
              <span>
                <strong className="font-semibold">{rule.rule}</strong>{" "}
                <span className="text-ink-muted">{rule.why}</span>
              </span>
            </li>
          ))}
        </ul>
      </section>
      {component.ifCompromised ? (
        <section className="pixel-border bg-terminal p-4">
          <h4 className={`${heading} text-rose-400/90`}>If it&apos;s compromised</h4>
          <p className="mt-2 text-[15px] leading-relaxed text-pretty">{component.ifCompromised}</p>
        </section>
      ) : null}
      <section>
        <h4 className={heading}>Talks to</h4>
        <ul className="mt-2.5 flex flex-wrap gap-2 text-sm">
          {neighbors(component.id).map((id) => {
            const label = componentById(id).label;
            const chip =
              "pixel-border inline-block bg-surface px-2.5 py-1 transition-colors hover:text-brand";
            return (
              <li key={id}>
                {onSelect ? (
                  <button
                    type="button"
                    onClick={() => onSelect(id)}
                    className={`${chip} cursor-pointer`}
                  >
                    {label}
                  </button>
                ) : (
                  <a href={`#component-${id}`} className={chip}>
                    {label}
                  </a>
                )}
              </li>
            );
          })}
        </ul>
      </section>
      <p className="text-sm text-ink-muted">
        <a
          href={docsPage(component.docs)}
          className="border-b border-dashed border-edge-strong pb-0.5 transition-colors hover:text-brand"
        >
          Read the docs ↗
        </a>
      </p>
    </div>
  );
}

interface ComponentPanelProps {
  readonly id: ComponentId;
  readonly onSelect: (id: ComponentId) => void;
  readonly onClose: () => void;
}

/**
 * The selected component, beside the diagram, in a few words: its role, its rules, what an
 * attacker gets. The full explanation is in the list under the diagram.
 */
export function ComponentPanel({ id, onSelect, onClose }: ComponentPanelProps) {
  const component = componentById(id);
  const label = "text-label text-ink-dim";

  // The list folds each component: open the right one before the browser scrolls to it.
  function openDetails() {
    const details = document.getElementById(`component-${id}`);
    if (details instanceof HTMLDetailsElement) details.open = true;
  }

  return (
    <div className="pixel-border flex min-h-full flex-col bg-surface p-5">
      <div className="flex items-start gap-3">
        <ComponentIcon icon={component.icon} size={28} />
        <div className="min-w-0 flex-1">
          <h3
            tabIndex={-1}
            className="text-lg leading-tight font-semibold tracking-[-0.02em] outline-none"
          >
            {component.label}
          </h3>
          <p className="text-sm text-ink-dim">{component.detail}</p>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label={`Close ${component.label}`}
          className="-mt-1 -mr-2 grid size-9 shrink-0 cursor-pointer place-items-center text-lg text-ink-muted transition-colors hover:text-brand"
        >
          ×
        </button>
      </div>
      <p className="mt-4 leading-snug text-pretty">{component.role}</p>

      <h4 className={`${label} mt-6`}>Security</h4>
      <ul className="mt-2.5 grid gap-1.5 text-sm">
        {component.rules.map((rule) => (
          <li key={rule.rule} className="flex gap-2.5">
            <span aria-hidden="true" className="shrink-0 text-emerald-400">
              ✓
            </span>
            {rule.rule}
          </li>
        ))}
      </ul>

      {component.risk ? (
        <>
          <h4 className={`${label} mt-6 text-rose-400/90`}>If it&apos;s compromised</h4>
          <p className="mt-2 text-sm text-pretty text-ink-muted">{component.risk}</p>
        </>
      ) : null}

      <h4 className={`${label} mt-6`}>Talks to</h4>
      <ul className="mt-2.5 flex flex-wrap gap-1.5 text-[13px]">
        {neighbors(component.id).map((neighbor) => (
          <li key={neighbor}>
            <button
              type="button"
              onClick={() => onSelect(neighbor)}
              className="pixel-border cursor-pointer bg-highlight px-2 py-0.5 transition-colors hover:text-brand"
            >
              {componentById(neighbor).label}
            </button>
          </li>
        ))}
      </ul>

      <p className="mt-auto flex flex-wrap gap-x-5 gap-y-1 pt-6 text-sm text-ink-muted">
        <a
          href={`#component-${id}`}
          onClick={openDetails}
          className="border-b border-dashed border-edge-strong pb-0.5 transition-colors hover:text-brand"
        >
          Full details ↓
        </a>
        <a
          href={docsPage(component.docs)}
          className="border-b border-dashed border-edge-strong pb-0.5 transition-colors hover:text-brand"
        >
          Docs ↗
        </a>
      </p>
    </div>
  );
}
