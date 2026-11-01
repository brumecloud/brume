import { archComponents } from "~/architecture/components";
import { architectureCopy } from "~/architecture/copy";
import { Section } from "~/components/section";

import { ComponentIcon } from "./component-icon";
import { OpenOnHash } from "./open-on-hash";
import { ComponentDetails } from "./component-panel";

/**
 * Every component with its full explanation, in the HTML the server sends: readable without
 * JavaScript, and by search engines. Folded, so the page stays short.
 */
export function ComponentList() {
  return (
    <Section id="components" title={architectureCopy.componentsTitle}>
      <OpenOnHash />
      <ul className="mt-10 border-t-2 border-edge">
        {archComponents.map((component) => (
          <li key={component.id} className="border-b-2 border-edge">
            <details id={`component-${component.id}`} className="group/item scroll-mt-8">
              <summary className="flex cursor-pointer list-none items-baseline gap-4 py-5 transition-colors hover:text-brand [&::-webkit-details-marker]:hidden">
                <span
                  aria-hidden="true"
                  className="w-3 shrink-0 text-ink-dim transition-transform group-open/item:rotate-90"
                >
                  ›
                </span>
                <span className="self-center">
                  <ComponentIcon icon={component.icon} size={20} />
                </span>
                <h3 className="text-lg font-semibold tracking-[-0.015em]">{component.label}</h3>
                <span className="text-sm text-ink-dim">{component.detail}</span>
              </summary>
              <div className="pb-8 pl-7 lg:max-w-[48rem]">
                <ComponentDetails component={component} />
              </div>
            </details>
          </li>
        ))}
      </ul>
    </Section>
  );
}
