import { MistCanvas } from "~/components/mist-canvas";

/** The descent: a dense layer of mist between the features and the security section. */
export function MistBand() {
  return (
    <div
      aria-hidden="true"
      className="relative isolate grid h-72 place-items-center overflow-hidden"
    >
      <MistCanvas shape="band" seed={2} />
      <p className="text-pixel bg-night px-3 py-1.5 text-brand-soft">under the mist</p>
    </div>
  );
}
