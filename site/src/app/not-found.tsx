import type { Metadata } from "next";
import Link from "next/link";

import { MistCanvas } from "~/components/mist-canvas";

export const metadata: Metadata = {
  title: "Lost in the mist · Brume",
  robots: { index: false },
};

export default function NotFound() {
  return (
    <main
      id="main"
      className="relative isolate grid min-h-svh place-items-center overflow-hidden px-6 text-center"
    >
      <MistCanvas shape="floor" palette="dawn" seed={4} stars />
      <div>
        <p className="text-label text-brand">404</p>
        <h1 className="mt-4 text-[clamp(2rem,5vw,3.5rem)] font-semibold tracking-[-0.04em]">
          Lost in the mist.
        </h1>
        <p className="mt-4 text-ink-muted">This page doesn&apos;t exist, or it drifted away.</p>
        <Link
          href="/"
          className="pixel-border mt-8 inline-block bg-surface px-4 py-2 text-sm transition-colors hover:text-brand"
        >
          Back to brume.dev
        </Link>
      </div>
    </main>
  );
}
