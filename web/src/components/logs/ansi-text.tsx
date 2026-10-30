import { type AnsiColor, parseAnsi } from "~/helpers/ansi";
import { cn } from "~/lib/utils";

const COLORS: Record<AnsiColor, string> = {
  black: "text-ansi-black",
  red: "text-ansi-red",
  green: "text-ansi-green",
  yellow: "text-ansi-yellow",
  blue: "text-ansi-blue",
  magenta: "text-ansi-magenta",
  cyan: "text-ansi-cyan",
  white: "text-ansi-white",
  gray: "text-ansi-gray",
};

// Text segments with classes, never HTML: a log line is hostile.
export function AnsiText({ text }: { readonly text: string }) {
  return parseAnsi(text).map(({ text: segment, style }, index) => (
    <span
      // oxlint-disable-next-line react/no-array-index-key -- the segments of a line never move
      key={index}
      className={cn(
        style.color && COLORS[style.color],
        style.bold && "font-semibold",
        style.dim && "opacity-70",
      )}
    >
      {segment}
    </span>
  ));
}
