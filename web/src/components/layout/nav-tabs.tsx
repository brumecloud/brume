import { createLink } from "@tanstack/react-router";
import { LayoutGroup, motion } from "motion/react";
import { type ComponentProps, type ReactNode, createContext, use, useId, useState } from "react";

import { HIDDEN_PILL, pillHidden, pillOnTab } from "~/helpers/tabs";
import { TAB_TRANSITION } from "~/lib/motion";

const HoverContext = createContext<(tab: HTMLElement) => void>(() => {});

// Without a slide, the pill jumps to its tab and only fades in.
const APPEAR = { ...TAB_TRANSITION, x: { duration: 0 }, width: { duration: 0 } };

// Reference: the tabs of Vercel. One pill per bar follows the hovered tab and the underline
// slides to the active one. Each tab is a URL: links, not ARIA tabs.
export function NavTabs({ children }: { readonly children: ReactNode }) {
  const [pill, setPill] = useState(HIDDEN_PILL);
  const onHover = (tab: HTMLElement) =>
    setPill((current) => pillOnTab(current, tab.offsetLeft, tab.offsetWidth));

  // The group scopes the layout id of the underline: two tab bars can be on screen at once.
  return (
    <LayoutGroup id={useId()}>
      <HoverContext value={onHover}>
        <nav className="relative mt-4 flex border-b px-3" onMouseLeave={() => setPill(pillHidden)}>
          <motion.span
            aria-hidden="true"
            initial={false}
            animate={{ x: pill.left, width: pill.width, opacity: pill.visible ? 1 : 0 }}
            transition={pill.slide ? TAB_TRANSITION : APPEAR}
            className="absolute top-0 bottom-1.5 left-0 rounded-md bg-accent"
          />
          {children}
        </nav>
      </HoverContext>
    </LayoutGroup>
  );
}

function TabAnchor({ children, ...props }: ComponentProps<"a">) {
  const onHover = use(HoverContext);

  return (
    <a
      {...props}
      className="relative px-3 pt-1.5 pb-3 text-muted-foreground text-sm transition-colors hover:text-foreground aria-[current=page]:text-foreground"
      onMouseEnter={(event) => onHover(event.currentTarget)}
    >
      {children}
      {props["aria-current"] === "page" && (
        <motion.span
          layoutId="active-tab"
          aria-hidden="true"
          transition={TAB_TRANSITION}
          className="absolute inset-x-3 -bottom-px h-0.5 bg-foreground"
        />
      )}
    </a>
  );
}

export const NavTab = createLink(TabAnchor);
