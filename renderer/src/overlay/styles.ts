import overlayCss from "./overlay.css?inline";

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
