import overlayCss from "./overlay.css?inline";

const GOOGLE_FONTS_CSS =
  "https://fonts.googleapis.com/css2?family=Geist:wght@100..900&display=swap";

/**
 * Loads Geist from the Google Fonts CDN (family "Geist"), with the self-hosted
 * copy (family "Geist Variable", registered by the injected page markup or
 * here as a backstop) as fallback in the :host font stack. Fonts must be
 * registered at document level - shadow trees can't own @font-face - and
 * loading is lazy, so both loads are kicked off explicitly.
 */
export function ensureGeistFont(authOrigin: string): void {
  if (!document.querySelector('link[data-brume-geist]')) {
    const preconnect = document.createElement("link");
    preconnect.rel = "preconnect";
    preconnect.href = "https://fonts.gstatic.com";
    preconnect.crossOrigin = "anonymous";
    const stylesheet = document.createElement("link");
    stylesheet.rel = "stylesheet";
    stylesheet.href = GOOGLE_FONTS_CSS;
    stylesheet.dataset.brumeGeist = "";
    document.head.append(preconnect, stylesheet);
  }
  const registered = [...document.fonts].some(
    (face) => face.family.replace(/["']/g, "") === "Geist Variable",
  );
  if (!registered) {
    try {
      const face = new FontFace(
        "Geist Variable",
        `url("${authOrigin}/_brume/geist.woff2") format("woff2")`,
        { style: "normal", weight: "100 900", display: "swap" },
      );
      document.fonts.add(face);
    } catch {
      /* the injected @font-face remains the only source */
    }
  }
  void document.fonts.load("13px Geist").catch(() => {});
  void document.fonts.load('13px "Geist Variable"').catch(() => {});
}

let sheet: CSSStyleSheet | null = null;
let propertiesHoisted = false;

/**
 * Adopts the compiled Tailwind sheet into a shadow root. One shared sheet is
 * parsed once and reused by both overlay roots.
 *
 * Browsers ignore `@property` rules inside shadow trees, but registrations are
 * per-document, so those rules are hoisted into a document-level sheet once.
 * Only `--tw-*`/shadcn internals are registered, which is safe even when the
 * host page uses Tailwind itself (the definitions are identical).
 */
export function applyOverlayStyles(root: ShadowRoot): void {
  if (!sheet) {
    sheet = new CSSStyleSheet();
    sheet.replaceSync(overlayCss);
  }
  root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
  if (!propertiesHoisted) {
    propertiesHoisted = true;
    const properties = overlayCss.match(/@property\s+--[\w-]+\s*\{[^}]*\}/g);
    if (properties?.length) {
      const documentSheet = new CSSStyleSheet();
      documentSheet.replaceSync(properties.join("\n"));
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, documentSheet];
    }
  }
}
