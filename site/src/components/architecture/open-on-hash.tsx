"use client";

import { useEffect } from "react";

/**
 * Unfolds the component the URL points at, such as #component-engine: on arrival, and when a
 * "Talks to" link of the list changes the hash. A folded <details> would otherwise hide it.
 */
export function OpenOnHash() {
  useEffect(() => {
    const open = () => {
      const id = decodeURIComponent(window.location.hash.slice(1));
      const target = id ? document.getElementById(id) : null;
      if (target instanceof HTMLDetailsElement && !target.open) {
        target.open = true;
        target.scrollIntoView();
      }
    };
    open();
    window.addEventListener("hashchange", open);
    return () => window.removeEventListener("hashchange", open);
  }, []);
  return null;
}
