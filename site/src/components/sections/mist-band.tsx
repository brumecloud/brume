import { MistCanvas } from "~/components/mist-canvas";

/** The descent: a layer of aurora mist between the features and the security section. */
export function MistBand() {
  return (
    <div
      aria-hidden="true"
      className="relative isolate grid h-72 place-items-center overflow-hidden"
    >
      <MistCanvas shape="band" palette="aurora" seed={2} />
      <p className="text-label bg-night px-3 py-1.5 text-emerald-200">under the mist</p>
    </div>
  );
}
