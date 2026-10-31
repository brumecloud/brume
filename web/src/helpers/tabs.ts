// The pill behind the hovered tab, in pixels from the left of the tab bar.
export interface TabPill {
  readonly left: number;
  readonly width: number;
  readonly visible: boolean;
  // Slides to its new tab; otherwise it appears there, without crossing the bar.
  readonly slide: boolean;
}

export const HIDDEN_PILL: TabPill = { left: 0, width: 0, visible: false, slide: false };

// The pointer enters a tab: from another tab the pill slides, from outside it appears.
export function pillOnTab(current: TabPill, left: number, width: number): TabPill {
  return { left, width, visible: true, slide: current.visible };
}

// The pointer leaves the bar: the pill fades where it is.
export function pillHidden(current: TabPill): TabPill {
  return { ...current, visible: false, slide: false };
}
