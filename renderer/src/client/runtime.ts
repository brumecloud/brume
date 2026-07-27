import mermaid from "mermaid";
import "../theme/index.css";

function initializeTheme() {
  const saved = localStorage.getItem("brume-theme");
  if (saved === "light" || saved === "dark") document.documentElement.dataset.theme = saved;
  for (const toggle of document.querySelectorAll<HTMLButtonElement>("[data-brume-theme-toggle]")) {
    toggle.addEventListener("click", () => {
      const current =
        document.documentElement.dataset.theme ??
        (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
      const next = current === "dark" ? "light" : "dark";
      document.documentElement.dataset.theme = next;
      localStorage.setItem("brume-theme", next);
    });
  }
}

function initializeTabs() {
  for (const tabs of document.querySelectorAll<HTMLElement>("[data-brume-tabs]")) {
    const buttons = [...tabs.querySelectorAll<HTMLButtonElement>("[data-brume-tab-button]")];
    const panels = [...tabs.querySelectorAll<HTMLElement>("[data-brume-tab-panel]")];
    const select = (selected: number, focus = false) => {
      buttons.forEach((candidate, index) => {
        const active = index === selected;
        candidate.setAttribute("aria-selected", String(active));
        candidate.tabIndex = active ? 0 : -1;
        panels[index]?.toggleAttribute("hidden", !active);
      });
      if (focus) buttons[selected]?.focus();
    };

    buttons.forEach((button, selected) => {
      button.addEventListener("click", () => select(selected));
      button.addEventListener("keydown", (event) => {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
        event.preventDefault();
        const next =
          event.key === "Home"
            ? 0
            : event.key === "End"
              ? buttons.length - 1
              : (selected + (event.key === "ArrowRight" ? 1 : -1) + buttons.length) %
                buttons.length;
        select(next, true);
      });
    });
  }
}

function initializeCodeCopy() {
  for (const button of document.querySelectorAll<HTMLButtonElement>("[data-brume-copy-code]")) {
    button.addEventListener("click", async () => {
      const code = button
        .closest<HTMLElement>("[data-brume-code-block]")
        ?.querySelector("code")?.textContent;
      if (code === undefined) return;
      try {
        await navigator.clipboard.writeText(code);
        button.textContent = "Copied";
        button.dataset.copied = "true";
      } catch {
        button.textContent = "Copy failed";
      }
      window.setTimeout(() => {
        button.textContent = "Copy";
        delete button.dataset.copied;
      }, 1400);
    });
  }
}

function initializeAccordions() {
  for (const group of document.querySelectorAll<HTMLElement>("[data-brume-accordions]")) {
    if (group.dataset.type !== "single") continue;
    for (const accordion of group.querySelectorAll<HTMLDetailsElement>(".brume-accordion")) {
      accordion.addEventListener("toggle", () => {
        if (!accordion.open) return;
        for (const sibling of group.querySelectorAll<HTMLDetailsElement>(".brume-accordion")) {
          if (sibling !== accordion) sibling.open = false;
        }
      });
    }
  }

  const target = window.location.hash
    ? document.getElementById(decodeURIComponent(window.location.hash.slice(1)))
    : null;
  if (target instanceof HTMLDetailsElement && target.classList.contains("brume-accordion")) {
    target.open = true;
  }
}

function initializeTableOfContents() {
  const links = [...document.querySelectorAll<HTMLAnchorElement>(".brume-toc a, .brume-mobile-toc a")];
  const headings = links
    .map((link) => document.getElementById(decodeURIComponent(link.hash.slice(1))))
    .filter((heading): heading is HTMLElement => heading !== null);
  if (headings.length === 0) return;

  const update = () => {
    const active =
      [...headings].reverse().find((heading) => heading.getBoundingClientRect().top <= 140) ??
      headings[0];
    for (const link of links) {
      if (link.hash === `#${active?.id}`) link.setAttribute("aria-current", "location");
      else link.removeAttribute("aria-current");
    }
  };
  document.addEventListener("scroll", update, { passive: true });
  update();
}

function initializeMobileNavigation() {
  for (const link of document.querySelectorAll<HTMLAnchorElement>(
    ".brume-mobile-tools details a",
  )) {
    link.addEventListener("click", () => {
      const details = link.closest<HTMLDetailsElement>("details");
      if (details) details.open = false;
    });
  }
}

function initializeAnimations() {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  if (!("IntersectionObserver" in window)) return;
  const elements = [...document.querySelectorAll<HTMLElement>("[data-brume-animate]")];
  if (elements.length === 0) return;
  document.documentElement.classList.add("brume-motion-ready");
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        (entry.target as HTMLElement).dataset.brumeVisible = "true";
        observer.unobserve(entry.target);
      }
    },
    { rootMargin: "0px 0px -8% 0px", threshold: 0.08 },
  );
  elements.forEach((element) => observer.observe(element));
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
initializeAccordions();
initializeTableOfContents();
initializeMobileNavigation();
initializeAnimations();
initializeReadTracking();
void initializeMermaid();
