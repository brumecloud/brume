/**
 * The Brume mark: a cloud shedding two drifting lines of mist.
 * Stroke-based on currentColor so it sits next to lucide icons; the
 * .brume-logo-mist classes let the launcher animate the mist on entrance.
 */
export function BrumeLogo({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242" />
      <path className="brume-logo-mist brume-logo-mist-1" d="M16 17H7" />
      <path className="brume-logo-mist brume-logo-mist-2" d="M17 21H9" />
    </svg>
  );
}
