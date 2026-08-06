import { useSyncExternalStore } from "react";
import type { ReviewStore } from "../store";

const subscribeNothing = () => () => {};
const zero = () => 0;

/** Subscribes the component tree to every store change. Accepts null so the
 * toolbar can render without a review store while keeping hook order stable. */
export function useStore(store: ReviewStore | null): void {
  useSyncExternalStore(
    store ? store.subscribe : subscribeNothing,
    store ? store.getVersion : zero,
    store ? store.getVersion : zero,
  );
}
