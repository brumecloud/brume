import { createRoot } from "react-dom/client";
import { ReviewApi, ToolbarApi, fetchOverlayState } from "./api";
import { ReviewLayer } from "./review-layer";
import { ReviewStore } from "./store";
import { applyOverlayStyles, ensureGeistFont } from "./styles";
import { Toolbar } from "./toolbar";

function mountHost(dataset: "brumeOverlayHost" | "brumeReviewHost"): {
  host: HTMLElement;
  mount: HTMLElement;
} {
  const host = document.createElement("div");
  host.dataset[dataset] = "";
  const root = host.attachShadow({ mode: "closed" });
  applyOverlayStyles(root);
  const mount = document.createElement("div");
  // The overlay is dark-only; the components' dark: styles key off this class.
  mount.className = "dark";
  root.append(mount);
  document.documentElement.append(host);
  return { host, mount };
}

(() => {
  const currentUrl = new URL(location.href);
  if (currentUrl.searchParams.get("_brume_auth_complete") === "1" && window.opener) {
    (window.opener as Window).postMessage(
      { type: "brume-owner-authenticated" },
      location.origin,
    );
    window.close();
    return;
  }
  const source = document.currentScript as HTMLScriptElement | null;
  if (!source || document.querySelector("[data-brume-overlay-host]")) return;
  const site = source.dataset.brumeSite;
  const authOrigin = source.dataset.brumeAuthOrigin;
  if (!site || !authOrigin) return;
  ensureGeistFont(authOrigin);
  const toolbarApi = new ToolbarApi(site, authOrigin);
  fetchOverlayState(site)
    .then(async (state) => {
      if (!state.enabled || document.querySelector("[data-brume-overlay-host]")) return;
      let owner = state.owner === true;
      if (!owner) {
        try {
          owner = (await toolbarApi.ownerState()).owner === true;
        } catch {
          /* cross-origin owner check is best-effort */
        }
      }
      let store: ReviewStore | null = null;
      if (state.review) {
        store = new ReviewStore(
          new ReviewApi(site),
          state.review,
          state.identified === true,
          owner,
        );
      }
      if (!owner && !store) return;
      if (document.querySelector("[data-brume-overlay-host]")) return;
      if (store) {
        const { mount } = mountHost("brumeReviewHost");
        createRoot(mount).render(<ReviewLayer store={store} />);
        store.setupListeners();
        void store.refresh().catch(() => {
          /* the review layer stays empty when the fetch fails */
        });
      }
      const { host, mount } = mountHost("brumeOverlayHost");
      let savedSide = "right";
      try {
        savedSide = localStorage.getItem("brume-toolbar-side") === "left" ? "left" : "right";
      } catch {
        /* private mode */
      }
      host.dataset.side = savedSide;
      createRoot(mount).render(
        <Toolbar
          api={toolbarApi}
          owner={owner}
          authMode={state.auth_mode ?? null}
          store={store}
          host={host}
        />,
      );
    })
    .catch(() => {
      /* the overlay stays hidden when its state cannot be loaded */
    });
})();
