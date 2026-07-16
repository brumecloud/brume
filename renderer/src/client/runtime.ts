import mermaid from "mermaid";
import "../theme/index.css";

function initializeTheme() {
  const saved = localStorage.getItem("brume-theme");
  if (saved === "light" || saved === "dark") document.documentElement.dataset.theme = saved;
  document.querySelector<HTMLElement>("[data-brume-theme-toggle]")?.addEventListener("click", () => {
    const current = document.documentElement.dataset.theme;
    const next = current === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    localStorage.setItem("brume-theme", next);
  });
}

function initializeTabs() {
  for (const tabs of document.querySelectorAll<HTMLElement>("[data-brume-tabs]")) {
    const buttons = [...tabs.querySelectorAll<HTMLButtonElement>("[data-brume-tab-button]")];
    const panels = [...tabs.querySelectorAll<HTMLElement>("[data-brume-tab-panel]")];
    buttons.forEach((button, selected) => {
      button.addEventListener("click", () => {
        buttons.forEach((candidate, index) => {
          candidate.setAttribute("aria-selected", String(index === selected));
          panels[index]?.toggleAttribute("hidden", index !== selected);
        });
      });
    });
  }
}

function initializeCodeCopy() {
  for (const pre of document.querySelectorAll<HTMLPreElement>("pre:has(code)")) {
    const button = document.createElement("button");
    button.className = "brume-copy-code";
    button.type = "button";
    button.textContent = "Copy";
    button.addEventListener("click", async () => {
      await navigator.clipboard.writeText(pre.querySelector("code")?.textContent ?? "");
      button.textContent = "Copied";
      window.setTimeout(() => (button.textContent = "Copy"), 1200);
    });
    pre.append(button);
  }
}

function initializeReadTracking() {
  const endpoint = document.querySelector<HTMLMetaElement>('meta[name="brume-read-url"]')?.content;
  if (!endpoint) return;
  let timer: number | undefined;
  const schedule = () => {
    if (document.visibilityState !== "visible") {
      if (timer !== undefined) window.clearTimeout(timer);
      timer = undefined;
      return;
    }
    if (timer !== undefined) return;
    timer = window.setTimeout(() => {
      timer = undefined;
      if (document.visibilityState === "visible") {
        void fetch(endpoint, { credentials: "same-origin", method: "POST", keepalive: true });
      }
    }, 5000);
  };
  document.addEventListener("visibilitychange", schedule);
  schedule();
}

async function initializeMermaid() {
  mermaid.initialize({ securityLevel: "strict", startOnLoad: false, theme: "neutral" });
  await mermaid.run({ querySelector: ".mermaid" });
}

initializeTheme();
initializeTabs();
initializeCodeCopy();
initializeReadTracking();
void initializeMermaid();
