import Image from "next/image";

import logo from "~/assets/logo.png";
import { BrandIcon } from "~/components/brand-icon";
import { logoLit } from "~/components/tone";
import { brumeRows } from "~/vs/brume";
import type { Competitor, VsRow } from "~/vs/model";

import { checkedOn, vsCopy } from "./shared";

const cell = "px-5 py-3.5 align-top";

/** Brume and a competitor, line by line. Each of the competitor's facts links to its source. */
export function ComparisonTable({ competitor }: { readonly competitor: Competitor }) {
  return (
    <div>
      <div className="pixel-border reveal overflow-x-auto bg-surface">
        <table className="w-full min-w-[640px] text-left text-sm">
          <caption className="sr-only">{vsCopy.title(competitor.name)}</caption>
          <thead>
            <tr data-light className="group border-b border-edge">
              <th scope="col" className={`${cell} w-[22%]`}>
                <span className="sr-only">Topic</span>
              </th>
              <th scope="col" className={`${cell} w-[39%] bg-highlight font-semibold`}>
                <span className="flex items-center gap-2.5">
                  <Image
                    src={logo}
                    alt=""
                    width={20}
                    height={20}
                    className={`shrink-0 rounded ${logoLit}`}
                  />
                  Brume
                </span>
              </th>
              <th scope="col" className={`${cell} font-semibold`}>
                <span className="flex items-center gap-2.5">
                  {competitor.icon ? (
                    <BrandIcon name={competitor.icon} className={`size-5 shrink-0 ${logoLit}`} />
                  ) : null}
                  {competitor.name}
                </span>
              </th>
            </tr>
          </thead>
          <tbody>
            {competitor.rows.map((row) => {
              const brume = brumeRows[row.key];
              return (
                <tr
                  key={row.key}
                  data-light
                  className="group border-b border-edge text-ink-muted last:border-b-0"
                >
                  <th scope="row" className={`${cell} font-semibold text-ink`}>
                    {brume.label}
                  </th>
                  <td
                    className={`${cell} bg-highlight transition-colors duration-300 lit:text-ink`}
                  >
                    {brume.brume}
                  </td>
                  <td className={`${cell} transition-colors duration-300 lit:text-ink`}>
                    <Fact row={row} />
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className="mt-3.5 text-[13px] text-ink-dim">
        {vsCopy.checked(checkedOn(competitor.checked))}
      </p>
    </div>
  );
}

/** The competitor's side of a row, linked to the page that says so. */
function Fact({ row }: { readonly row: VsRow }) {
  if (!row.source) return row.them;
  return (
    <a href={row.source} className="transition-colors hover:text-brand">
      {row.them}
      <span
        aria-hidden="true"
        className="ml-1 text-ink-dim transition-colors duration-300 lit:text-brand"
      >
        ↗
      </span>
      <span className="sr-only"> (source)</span>
    </a>
  );
}
