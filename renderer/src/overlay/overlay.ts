import { fetchOverlayState, ReviewApi } from "./api";
import { type ReviewController, mountReview } from "./review";

const TOOLBAR_STYLES = `
  *{box-sizing:border-box;font-family:inherit}
  :host{font:13px/1.35 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;color:#f5f5f5}
  button,a{font:inherit}
  button{border:1px solid #3a3a3a;border-radius:10px;padding:9px 10px;background:#272727;color:#f5f5f5;cursor:pointer;transition:background .16s ease,border-color .16s ease,transform .16s ease}
  button:hover,a:hover{background:#333;border-color:#484848}
  button:active{transform:scale(.975)}
  button:focus-visible{outline:2px solid #8ab4f8;outline-offset:2px}
  .launcher{position:relative;width:52px;height:52px;padding:0;border:1px solid rgba(255,255,255,.15);border-radius:50%;background:rgba(20,20,20,.94);box-shadow:0 8px 28px rgba(0,0,0,.3);font-size:22px;backdrop-filter:blur(16px);animation:brume-launcher-in .28s cubic-bezier(.2,.8,.2,1) both}
  .launcher:hover{transform:translateY(-2px) scale(1.03);box-shadow:0 12px 34px rgba(0,0,0,.38)}
  .badge{position:absolute;top:-4px;right:-4px;min-width:20px;height:20px;padding:0 5px;border-radius:999px;background:#f5b700;color:#151515;font-size:11.5px;font-weight:650;line-height:20px;display:none}
  .panel{position:absolute;right:0;bottom:0;width:min(390px,calc(100vw - 32px));padding:10px;border:1px solid rgba(255,255,255,.13);border-radius:18px;background:rgba(18,18,18,.96);box-shadow:0 20px 70px rgba(0,0,0,.45);backdrop-filter:blur(22px);opacity:0;visibility:hidden;pointer-events:none;transform:translateY(10px) scale(.97);transform-origin:bottom right;transition:opacity .2s ease,transform .24s cubic-bezier(.2,.8,.2,1),visibility 0s linear .24s}
  :host([data-side="left"]) .panel{right:auto;left:0;transform-origin:bottom left}
  :host([data-open]) .panel{opacity:1;visibility:visible;pointer-events:auto;transform:none;transition-delay:0s}
  :host([data-open]) .launcher{opacity:0;visibility:hidden;pointer-events:none;transform:scale(.84);transition:opacity .14s ease,transform .18s ease,visibility 0s linear .18s}
  header{display:flex;align-items:center;justify-content:space-between;padding:4px 5px 10px;touch-action:none;cursor:grab}
  header strong{font-size:14px;font-weight:620;letter-spacing:-.01em}
  .header-start{display:flex;align-items:center;gap:6px}
  .back{display:none;width:28px;height:28px;padding:0;border:0;background:transparent;color:#aaa;font-size:18px}
  :host([data-view="management"]) .back{display:block}
  .close{width:28px;height:28px;padding:0;border:0;border-radius:8px;background:#292929;color:#aaa}
  .actions{display:grid;grid-template-columns:1fr 1fr;gap:7px}
  .manage{display:block;width:100%;margin-top:7px}
  .owner-tools{display:none}
  :host([data-owner]) .owner-tools{display:block}
  .result{min-height:18px;margin:8px 4px 0;color:#9ee6bc}
  .management{display:none}
  :host([data-view="management"]) .primary{display:none}
  :host([data-view="management"]) .management{display:block;animation:brume-view-in .2s ease both}
  iframe{display:block;width:100%;height:min(520px,calc(100vh - 92px));border:0;background:transparent}
  .review{display:none;margin-top:10px;padding-top:10px;border-top:1px solid rgba(255,255,255,.1)}
  :host([data-review]) .review{display:block}
  .review-status{margin:0 4px 8px;color:#d8d8d8}
  .review-status.done{color:#9ee6bc}
  .review .actions button.finish{background:#e8e8e8;border-color:#e8e8e8;color:#151515;font-weight:600}
  .review .actions button.finish:hover{background:#fff}
  .unplaced{margin:8px 0 0;display:flex;flex-direction:column;gap:6px}
  .unplaced button{display:block;width:100%;text-align:left;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:#cfcfcf}
  @keyframes brume-launcher-in{from{opacity:0;transform:translateY(8px) scale(.82)}to{opacity:1;transform:none}}
  @keyframes brume-view-in{from{opacity:0;transform:translateX(6px)}to{opacity:1;transform:none}}
  @media(prefers-reduced-motion:reduce){*,*::before,*::after{animation-duration:.01ms!important;transition-duration:.01ms!important}}
`;

const TOOLBAR_TEMPLATE = `
  <button class="launcher" type="button" aria-label="Open Brume toolbar">☁️<span class="badge"></span></button>
  <div class="panel">
    <header><span class="header-start"><button class="back" type="button" aria-label="Back">‹</button><strong>Brume</strong></span><button class="close" type="button" aria-label="Close">×</button></header>
    <div class="primary">
      <div class="owner-tools">
        <div class="actions"><button class="share" type="button">Share website</button><button class="copy" type="button">Copy URL</button></div>
        <button class="manage" type="button">Manage access</button>
      </div>
      <div class="review">
        <p class="review-status" aria-live="polite"></p>
        <div class="actions"><button class="element-comment" type="button">Comment on element</button><button class="finish" type="button">Finish review</button></div>
        <div class="unplaced"></div>
      </div>
      <p class="result" aria-live="polite"></p>
    </div>
    <div class="management"><iframe title="Manage website access" allow="clipboard-write" sandbox="allow-forms allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"></iframe></div>
  </div>
`;

interface ToolbarOptions {
  site: string;
  authOrigin: string;
  owner: boolean;
  review: ReviewController | null;
}

function mountToolbar(options: ToolbarOptions): void {
  if (document.querySelector("[data-brume-overlay-host]")) return;
  const host = document.createElement("div");
  host.dataset.brumeOverlayHost = "";
  let savedSide = "right";
  try {
    savedSide = localStorage.getItem("brume-toolbar-side") === "left" ? "left" : "right";
  } catch {
    /* private mode */
  }
  host.dataset.side = savedSide;
  if (options.owner) host.dataset.owner = "";
  if (options.review) host.dataset.review = "";
  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `<style>${TOOLBAR_STYLES}</style>${TOOLBAR_TEMPLATE}`;
  const result = root.querySelector(".result") as HTMLElement;
  const title = root.querySelector("header strong") as HTMLElement;
  const management = root.querySelector("iframe") as HTMLIFrameElement;
  const launcher = root.querySelector(".launcher") as HTMLButtonElement;
  const badge = root.querySelector(".badge") as HTMLElement;
  const managementUrl =
    options.authOrigin +
    "/toolbar/" +
    encodeURIComponent(options.site) +
    "?return_to=" +
    encodeURIComponent(location.href);
  const copy = async () => {
    await navigator.clipboard.writeText(location.href);
    result.textContent = "Copied";
  };
  launcher.onclick = () => (host.dataset.open = "");
  (root.querySelector(".close") as HTMLButtonElement).onclick = () => delete host.dataset.open;
  (root.querySelector(".copy") as HTMLButtonElement).onclick = () =>
    copy().catch(() => (result.textContent = "Could not copy"));
  (root.querySelector(".share") as HTMLButtonElement).onclick = () =>
    navigator.share
      ? navigator.share({ url: location.href }).catch(() => {})
      : copy().catch(() => (result.textContent = "Could not copy"));
  (root.querySelector(".manage") as HTMLButtonElement).onclick = () => {
    if (!management.dataset.loaded) {
      management.src = managementUrl;
      management.dataset.loaded = "true";
    }
    host.dataset.view = "management";
    title.textContent = "Manage access";
  };
  (root.querySelector(".back") as HTMLButtonElement).onclick = () => {
    delete host.dataset.view;
    title.textContent = "Brume";
  };
  addEventListener("message", (event) => {
    if (
      event.origin !== location.origin ||
      (event.data as { type?: string } | null)?.type !== "brume-owner-authenticated"
    ) {
      return;
    }
    management.src = managementUrl + "&refresh=" + Date.now();
  });
  const drag = root.querySelector("header") as HTMLElement;
  let dragging = false;
  drag.onpointerdown = (event) => {
    if ((event.target as Element | null)?.closest("button")) return;
    dragging = true;
    drag.setPointerCapture(event.pointerId);
  };
  drag.onpointerup = (event) => {
    if (!dragging) return;
    dragging = false;
    const side = event.clientX < innerWidth / 2 ? "left" : "right";
    host.dataset.side = side;
    try {
      localStorage.setItem("brume-toolbar-side", side);
    } catch {
      /* private mode */
    }
  };

  const review = options.review;
  if (review) {
    const status = root.querySelector(".review-status") as HTMLElement;
    const finish = root.querySelector(".finish") as HTMLButtonElement;
    const elementComment = root.querySelector(".element-comment") as HTMLButtonElement;
    const unplaced = root.querySelector(".unplaced") as HTMLElement;
    const renderReview = () => {
      const round = review.round();
      const open = round.status === "open";
      badge.style.display = open && round.comment_count > 0 ? "block" : "none";
      badge.textContent = String(round.comment_count);
      status.classList.toggle("done", !open);
      status.textContent = open
        ? `Review round #${round.number} - ${round.comment_count} comment${round.comment_count === 1 ? "" : "s"}`
        : "Review finished";
      finish.style.display = open ? "" : "none";
      elementComment.style.display = open ? "" : "none";
      unplaced.textContent = "";
      const orphans = review.unplacedThreads();
      for (const orphan of orphans) {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = `💬 ${orphan.comments} · ${orphan.preview}`;
        button.onclick = () => {
          delete host.dataset.open;
          review.openThread(orphan.id);
        };
        unplaced.append(button);
      }
    };
    finish.onclick = () => {
      if (!confirm("Finish this review? The site owner will fetch your comments.")) return;
      finish.disabled = true;
      review
        .finish()
        .catch(() => (result.textContent = "Could not finish the review"))
        .finally(() => {
          finish.disabled = false;
          renderReview();
        });
    };
    elementComment.onclick = () => {
      delete host.dataset.open;
      review.startElementMode();
    };
    review.onChange(renderReview);
    renderReview();
  }

  document.documentElement.append(host);
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
  fetchOverlayState(site)
    .then(async (state) => {
      if (!state.enabled || document.querySelector("[data-brume-overlay-host]")) return;
      let owner = state.owner === true;
      if (!owner) {
        const ownerStateUrl =
          authOrigin +
          "/toolbar/" +
          encodeURIComponent(site) +
          "/owner-state?return_to=" +
          encodeURIComponent(location.href);
        try {
          const response = await fetch(ownerStateUrl, { credentials: "include" });
          owner = response.ok && ((await response.json()) as { owner?: boolean }).owner === true;
        } catch {
          /* cross-origin owner check is best-effort */
        }
      }
      let review: ReviewController | null = null;
      if (state.review) {
        review = mountReview(new ReviewApi(site), state.review, {
          identified: state.identified === true,
          owner,
        });
      }
      if (!owner && !review) return;
      if (document.querySelector("[data-brume-overlay-host]")) return;
      mountToolbar({ site, authOrigin, owner, review });
    })
    .catch(() => {
      /* the overlay stays hidden when its state cannot be loaded */
    });
})();
