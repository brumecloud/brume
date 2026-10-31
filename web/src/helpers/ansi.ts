// Log lines are hostile (archi/base/04-panel-admin.md#xss): keep the colors, drop every
// other escape sequence, and hand back plain text segments. Never HTML.

export type AnsiColor =
  | "black"
  | "red"
  | "green"
  | "yellow"
  | "blue"
  | "magenta"
  | "cyan"
  | "white"
  | "gray";

interface AnsiStyle {
  readonly color?: AnsiColor;
  readonly bold?: boolean;
  readonly dim?: boolean;
}

export interface AnsiSegment {
  readonly text: string;
  readonly style: AnsiStyle;
}

const COLORS: readonly AnsiColor[] = [
  "black",
  "red",
  "green",
  "yellow",
  "blue",
  "magenta",
  "cyan",
  "white",
];

// CSI (ESC [ params final), OSC (ESC ] … BEL or ESC \), or any other two-character escape.
// oxlint-disable-next-line no-control-regex -- ANSI sequences start with the ESC control character
const ESCAPE = /\u001B(?:\[([0-9;?]*)([@-~])|\][^\u0007\u001B]*(?:\u0007|\u001B\\)?|[@-Z\\-_])/g;
// Every C0 control character but the tab.
// oxlint-disable-next-line no-control-regex -- stripping control characters is the point
const CONTROL = /[\u0000-\u0008\u000A-\u001F\u007F]/g;

export function parseAnsi(input: string): AnsiSegment[] {
  const segments: AnsiSegment[] = [];
  let style: AnsiStyle = {};
  let last = 0;

  const push = (raw: string) => {
    const text = raw.replace(CONTROL, "");
    if (!text) {
      return;
    }
    const previous = segments.at(-1);
    if (previous && sameStyle(previous.style, style)) {
      segments[segments.length - 1] = { text: previous.text + text, style };
    } else {
      segments.push({ text, style });
    }
  };

  for (const match of input.matchAll(ESCAPE)) {
    push(input.slice(last, match.index));
    last = match.index + match[0].length;
    if (match[2] === "m") {
      style = applySgr(style, match[1] ?? "");
    }
  }
  push(input.slice(last));
  return segments;
}

// Select Graphic Rendition: only the codes that change the color or the weight.
function applySgr(style: AnsiStyle, params: string): AnsiStyle {
  const codes = params === "" ? [0] : params.split(";").map(Number);
  let next = style;
  for (let index = 0; index < codes.length; index += 1) {
    const code = codes[index] ?? 0;
    if (code === 38 || code === 48) {
      // 256 colors (5;n) or true colors (2;r;g;b): skipped, without breaking the rest.
      index += codes[index + 1] === 5 ? 2 : 4;
    } else if (code === 0) {
      next = {};
    } else if (code === 1) {
      next = { ...next, bold: true };
    } else if (code === 2) {
      next = { ...next, dim: true };
    } else if (code === 22) {
      next = withoutWeight(next);
    } else if (code === 39) {
      next = withoutColor(next);
    } else if (code === 90) {
      next = { ...next, color: "gray" };
    } else if (code >= 30 && code <= 37) {
      next = { ...next, color: COLORS[code - 30] ?? "white" };
    } else if (code >= 91 && code <= 97) {
      next = { ...next, color: COLORS[code - 90] ?? "white" };
    }
  }
  return next;
}

function withoutColor({ color: _color, ...rest }: AnsiStyle): AnsiStyle {
  return rest;
}

function withoutWeight({ bold: _bold, dim: _dim, ...rest }: AnsiStyle): AnsiStyle {
  return rest;
}

function sameStyle(a: AnsiStyle, b: AnsiStyle): boolean {
  return a.color === b.color && a.bold === b.bold && a.dim === b.dim;
}
