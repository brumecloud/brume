import { describe, expect, it } from "vite-plus/test";

import { parseAnsi } from "./ansi";

const ESC = "\u001B";

describe("parseAnsi", () => {
  it("keeps plain text as a single unstyled segment", () => {
    expect(parseAnsi("GET /health 200")).toEqual([{ text: "GET /health 200", style: {} }]);
  });

  it("applies colors until a reset", () => {
    expect(parseAnsi(`${ESC}[32mINFO${ESC}[0m server started`)).toEqual([
      { text: "INFO", style: { color: "green" } },
      { text: " server started", style: {} },
    ]);
  });

  it("combines bold and color in one sequence", () => {
    expect(parseAnsi(`${ESC}[1;31mERROR`)).toEqual([
      { text: "ERROR", style: { color: "red", bold: true } },
    ]);
  });

  it("maps bright colors to their base color, and 90 to gray", () => {
    expect(parseAnsi(`${ESC}[92mok${ESC}[90mdim`)).toEqual([
      { text: "ok", style: { color: "green" } },
      { text: "dim", style: { color: "gray" } },
    ]);
  });

  it("resets the color alone with 39 and the weight with 22", () => {
    expect(parseAnsi(`${ESC}[1;33mwarn${ESC}[39m bold${ESC}[22m plain`)).toEqual([
      { text: "warn", style: { color: "yellow", bold: true } },
      { text: " bold", style: { bold: true } },
      { text: " plain", style: {} },
    ]);
  });

  it("ignores 256 and true colors without losing the text", () => {
    expect(parseAnsi(`${ESC}[38;5;196mred${ESC}[38;2;1;2;3m too`)).toEqual([
      { text: "red too", style: {} },
    ]);
  });

  it("strips cursor moves and other non-color sequences", () => {
    expect(parseAnsi(`${ESC}[2K${ESC}[1Gprogress 50%`)).toEqual([
      { text: "progress 50%", style: {} },
    ]);
  });

  it("strips hyperlinks and window titles (OSC)", () => {
    const link = `${ESC}]8;;https://evil.example${ESC}\\click${ESC}]8;;${ESC}\\`;
    const title = `${ESC}]0;pwned\u0007`;

    expect(parseAnsi(`${title}${link}`)).toEqual([{ text: "click", style: {} }]);
  });

  it("drops control characters but keeps tabs", () => {
    expect(parseAnsi("a\u0008b\rc\td\u0000")).toEqual([{ text: "abc\td", style: {} }]);
  });

  it("keeps HTML as text", () => {
    expect(parseAnsi('<img src=x onerror="alert(1)">')).toEqual([
      { text: '<img src=x onerror="alert(1)">', style: {} },
    ]);
  });

  it("returns nothing for an empty line", () => {
    expect(parseAnsi("")).toEqual([]);
  });
});
