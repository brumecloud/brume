import { createLink } from "@tanstack/react-router";
import { AnimatePresence, LayoutGroup, motion } from "motion/react";
import { type ComponentProps, type ReactNode, createContext, use, useId, useState } from "react";

import { TAB_TRANSITION } from "~/lib/motion";

interface HoverState {
  readonly hovered: string | undefined;
  readonly setHovered: (tab: string | undefined) => void;
}

const HoverContext = createContext<HoverState>({ hovered: undefined, setHovered: () => {} });

// Reference: the tabs of Vercel. A pill follows the hovered tab and the underline slides to
// the active one. Each tab is a URL: links, not ARIA tabs.
export function NavTabs({ children }: { readonly children: ReactNode }) {
  const [hovered, setHovered] = useState<string>();

  // The group scopes the shared layout ids: two tab bars can be on screen at once.
  return (
    <LayoutGroup id={useId()}>
      <HoverContext value={{ hovered, setHovered }}>
        <nav className="mt-4 flex border-b px-3" onMouseLeave={() => setHovered(undefined)}>
          {children}
        </nav>
      </HoverContext>
    </LayoutGroup>
  );
}

function TabAnchor({ children, ...props }: ComponentProps<"a">) {
  const tab = useId();
  const { hovered, setHovered } = use(HoverContext);

  return (
    <a
      {...props}
      className="relative px-3 pt-1.5 pb-3 text-muted-foreground text-sm transition-colors hover:text-foreground aria-[current=page]:text-foreground"
      onMouseEnter={() => setHovered(tab)}
    >
      <AnimatePresence>
        {hovered === tab && (
          <motion.span
            layoutId="hovered-tab"
            aria-hidden="true"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={TAB_TRANSITION}
            className="absolute inset-x-0 top-0 bottom-1.5 rounded-md bg-accent"
          />
        )}
      </AnimatePresence>
      <span className="relative">{children}</span>
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
