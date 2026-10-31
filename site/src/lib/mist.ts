// Dithered mist for the landing page.
//
// A low resolution fog field, quantized to a fixed blue palette with a 4x4
// Bayer matrix, then scaled up by CSS with `image-rendering: pixelated`. A
// canvas only animates while it is on screen, and never when the visitor
// prefers reduced motion. The pointer stirs it: the fog follows its motion and
// parts where it passes, then settles back. A click breathes a puff of mist,
// which swells and fades.

export type MistShape = "hero" | "band" | "floor";
export type MistPalette = keyof typeof PALETTES;

export interface MistOptions {
  readonly shape: MistShape;
  readonly palette: MistPalette;
  readonly seed: number;
  readonly stars: boolean;
}

/** CSS pixels per mist pixel. */
const PIXEL = 4;
const FRAME_MS = 90;
/** Smoother while the pointer stirs the fog. */
const FRAME_STIRRED_MS = 40;

/** How the pointer stirs the fog, in mist pixels and seconds. */
const STIR_RADIUS = 26;
/** The share of the pointer's motion the fog follows, up to `MAX_SHIFT`. */
const STIR_DRAG = 0.6;
const MAX_SHIFT = 30;
/** The fog parts under the pointer: density taken off at its center. */
const PARTING = 0.75;
/** Time constants of the fog drifting back and closing over the trail. */
const SHIFT_SETTLE_S = 1.1;
const PARTING_SETTLE_S = 0.7;

/**
 * A puff of mist from a click: it swells from `PUFF_RADIUS` to `PUFF_SPREAD`
 * mist pixels while it fades, and adds up to `PUFF_DENSITY` to the fog.
 */
const PUFF_RADIUS = 12;
const PUFF_SPREAD = 46;
const PUFF_DENSITY = 0.95;
const PUFF_SWELL_S = 1.4;
const PUFF_FADE_S = 2.2;
const PUFF_LIFE_S = 8;
/** The oldest puffs go first past this many. */
const MAX_PUFFS = 12;

/**
 * From the page background (`--color-night`) to the brightest fog. Every
 * palette starts on the page background and has the same number of levels.
 */
const PALETTES = {
  night: ["#020818", "#05112e", "#0a1d4a", "#11306f", "#1b4c9e", "#2f74d0", "#5ea8f5", "#b4dcff"],
} as const;
const STAR_COLORS = ["#3b4f7a", "#c9d8ff", "#ffffff"];

/**
 * Stars thin out toward the top edge over this share of the height, so a canvas
 * below another section starts without a visible line.
 */
const STAR_FADE_IN: Record<MistShape, number> = { hero: 0, band: 0, floor: 0.35 };

/** A shooting star crosses the sky every few seconds. */
const METEOR_PERIOD = 7;
const METEOR_DURATION = 0.9;

const clamp01 = (value: number): number => (value < 0 ? 0 : value > 1 ? 1 : value);

/** Density offset added to the noise, by height (0 at the top, 1 at the bottom). */
const SHAPES: Record<MistShape, (v: number) => number> = {
  // Clear sky on top, fog rising from below, fading out at the very bottom so
  // the next section starts on the page background.
  hero(v) {
    const rise = Math.max(0, (v - 0.3) / 0.7);
    const fadeOut = clamp01((v - 0.86) / 0.14);
    return rise * rise * 1.3 - 0.34 - fadeOut * 1.4;
  },
  // A dense layer to cross between two sections.
  band(v) {
    const center = 1 - Math.abs(v - 0.5) * 2;
    return center * center * 1.35 - 0.5;
  },
  // Low fog under the closing call to action. It fades in from the top, so the
  // section starts on the page background, and dims at the very bottom to meet
  // the footer softly.
  floor(v) {
    const rise = Math.max(0, (v - 0.35) / 0.65);
    const fadeIn = clamp01(1 - v / 0.3);
    const fadeOut = clamp01((v - 0.9) / 0.1);
    return rise * rise * 0.95 - 0.25 - fadeIn * 0.8 - fadeOut * 0.5;
  },
};

const BAYER = Float32Array.from(
  [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5],
  (rank) => (rank + 0.5) / 16,
);

/** Packs "#rrggbb" into a little-endian RGBA word, for a Uint32Array view. */
const pack = (hex: string): number => {
  const n = Number.parseInt(hex.slice(1), 16);
  return ((255 << 24) | ((n & 255) << 16) | (n & 0xff00) | ((n >> 16) & 255)) >>> 0;
};
const STAR_WORDS = Uint32Array.from(STAR_COLORS, pack);
const LEVELS = PALETTES.night.length;
/**
 * Clear sky is left transparent: the page background shows through, so the
 * canvas never paints its own copy of it, which browsers can render a shade
 * off from the CSS background.
 */
const SKY = 0;

const mulberry32 = (seed: number) => (): number => {
  seed = (seed + 0x6d2b79f5) | 0;
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

const LATTICE = (() => {
  const random = mulberry32(5);
  return Float32Array.from({ length: 65536 }, random);
})();

// The hot loops below read typed arrays with indexes that are in range by
// construction, hence the non-null assertions.
const valueNoise = (x: number, y: number): number => {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  let u = x - xi;
  let v = y - yi;
  u = u * u * (3 - 2 * u);
  v = v * v * (3 - 2 * v);
  const row0 = (yi & 255) << 8;
  const row1 = ((yi + 1) & 255) << 8;
  const a = LATTICE[row0 | (xi & 255)]!;
  const b = LATTICE[row0 | ((xi + 1) & 255)]!;
  const c = LATTICE[row1 | (xi & 255)]!;
  const d = LATTICE[row1 | ((xi + 1) & 255)]!;
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
};

const fbm = (x: number, y: number, octaves: number): number => {
  let sum = 0;
  let amplitude = 0.5;
  let frequency = 1;
  let norm = 0;
  for (let i = 0; i < octaves; i++) {
    sum += amplitude * valueNoise(x * frequency, y * frequency);
    norm += amplitude;
    amplitude *= 0.5;
    frequency *= 2.03;
  }
  return sum / norm;
};

interface Point {
  readonly x: number;
  readonly y: number;
}

const clampShift = (value: number): number =>
  value < -MAX_SHIFT ? -MAX_SHIFT : value > MAX_SHIFT ? MAX_SHIFT : value;

interface Puff extends Point {
  /** When the click happened, on the mist's clock, in seconds. */
  readonly born: number;
}

interface Star {
  readonly x: number;
  readonly y: number;
  readonly brightness: number;
  readonly phase: number;
  readonly speed: number;
  readonly big: boolean;
  readonly keep: number;
}

class MistField {
  private width = 0;
  private height = 0;
  private image: ImageData | undefined;
  private pixels = new Uint32Array(0);
  private rowShape = new Float32Array(0);
  private stars: Star[] = [];
  private readonly palette: Uint32Array;
  // How far the pointer moved the fog, and how much it parted it, per mist pixel.
  private shiftX = new Float32Array(0);
  private shiftY = new Float32Array(0);
  private parting = new Float32Array(0);
  private stirred = false;
  // The puffs of mist the clicks left, and the density they add, per mist pixel.
  private puffs: Puff[] = [];
  private added = new Float32Array(0);

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly context: CanvasRenderingContext2D,
    private readonly options: MistOptions,
  ) {
    this.palette = Uint32Array.from(PALETTES[options.palette], pack);
    this.palette[0] = SKY;
  }

  /** Matches the canvas to the box it covers, padding included. */
  resize(box: DOMRect): void {
    const width = Math.max(1, Math.ceil(box.width / PIXEL));
    const height = Math.max(1, Math.ceil(box.height / PIXEL));
    if (width === this.width && height === this.height) return;
    this.width = width;
    this.height = height;
    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.style.width = `${width * PIXEL}px`;
    this.canvas.style.height = `${height * PIXEL}px`;
    this.image = this.context.createImageData(width, height);
    this.pixels = new Uint32Array(this.image.data.buffer);
    const shape = SHAPES[this.options.shape];
    this.rowShape = Float32Array.from({ length: height }, (_, y) => shape(y / height));
    this.stars = this.options.stars ? this.makeStars() : [];
    this.shiftX = new Float32Array(width * height);
    this.shiftY = new Float32Array(width * height);
    this.parting = new Float32Array(width * height);
    this.added = new Float32Array(width * height);
    this.stirred = false;
  }

  /** Whether the fog still carries a trace of the pointer, or a puff still lives. */
  get isStirred(): boolean {
    return this.stirred || this.puffs.length > 0;
  }

  /** A click at `at`, in mist pixels, at `time` on the mist's clock. */
  puff(at: Point, time: number): void {
    this.puffs = [...this.puffs.slice(1 - MAX_PUFFS), { ...at, born: time }];
  }

  /** The pointer went from `from` to `to`, in mist pixels, since the last frame. */
  stir(from: Point, to: Point): void {
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const steps = Math.max(1, Math.ceil(Math.hypot(dx, dy) / (STIR_RADIUS / 4)));
    for (let step = 1; step <= steps; step++) {
      this.stamp(
        from.x + (dx * step) / steps,
        from.y + (dy * step) / steps,
        (dx * STIR_DRAG) / steps,
        (dy * STIR_DRAG) / steps,
      );
    }
    this.stirred = true;
  }

  /** Lets the fog drift back and close over the pointer's trail. */
  settle(seconds: number): void {
    if (!this.stirred) return;
    const { shiftX, shiftY, parting } = this;
    const shiftKeep = Math.exp(-seconds / SHIFT_SETTLE_S);
    const partingKeep = Math.exp(-seconds / PARTING_SETTLE_S);
    let left = 0;
    for (let i = 0; i < parting.length; i++) {
      const x = shiftX[i]! * shiftKeep;
      const y = shiftY[i]! * shiftKeep;
      const part = parting[i]! * partingKeep;
      shiftX[i] = x;
      shiftY[i] = y;
      parting[i] = part;
      left = Math.max(left, Math.abs(x), Math.abs(y), part * 10);
    }
    if (left < 0.05) {
      shiftX.fill(0);
      shiftY.fill(0);
      parting.fill(0);
      this.stirred = false;
    }
  }

  /** A soft disc around the pointer: it carries the fog by `push` and parts it. */
  private stamp(cx: number, cy: number, pushX: number, pushY: number): void {
    const { width, height, shiftX, shiftY, parting } = this;
    const r2 = STIR_RADIUS * STIR_RADIUS;
    const top = Math.max(0, Math.floor(cy - STIR_RADIUS));
    const bottom = Math.min(height - 1, Math.ceil(cy + STIR_RADIUS));
    const left = Math.max(0, Math.floor(cx - STIR_RADIUS));
    const right = Math.min(width - 1, Math.ceil(cx + STIR_RADIUS));
    for (let y = top; y <= bottom; y++) {
      for (let x = left; x <= right; x++) {
        const d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
        if (d2 >= r2) continue;
        const falloff = 1 - d2 / r2;
        const weight = falloff * falloff;
        const i = y * width + x;
        shiftX[i] = clampShift(shiftX[i]! + pushX * weight);
        shiftY[i] = clampShift(shiftY[i]! + pushY * weight);
        parting[i] = Math.max(parting[i]!, weight);
      }
    }
  }

  draw(time: number): void {
    if (!this.image) return;
    const { width, height, pixels, rowShape, palette, stirred, shiftX, shiftY, parting, added } =
      this;
    const puffed = this.addPuffs(time);
    const offset = this.options.seed * 37;
    for (let y = 0; y < height; y++) {
      const shape = rowShape[y]!;
      const bayerRow = (y & 3) << 2;
      const row = y * width;
      for (let x = 0; x < width; x++) {
        // Where the pointer carried the fog, it shows the fog from further back.
        const i = row + x;
        const fx = stirred ? x - shiftX[i]! : x;
        const fy = stirred ? y - shiftY[i]! : y;
        const detail = fbm(fx * 0.024 + time * 0.06 + offset, fy * 0.06 + time * 0.01, 3);
        const drift = fbm(fx * 0.009 - time * 0.025 + 40 + offset, fy * 0.03 + 20, 2);
        const parted = stirred ? parting[i]! * PARTING : 0;
        const puff = puffed ? added[i]! : 0;
        const density = clamp01(
          (detail - 0.45) * 1.7 + shape + (drift - 0.5) * 0.5 - parted + puff,
        );
        const level = density * (LEVELS - 1);
        const base = level | 0;
        const index = base + (level - base > BAYER[bayerRow | (x & 3)]! ? 1 : 0);
        pixels[i] = palette[Math.min(index, LEVELS - 1)]!;
      }
    }
    this.drawStars(time);
    if (this.options.stars) this.drawMeteor(time);
    this.context.putImageData(this.image, 0, 0);
  }

  /**
   * Adds up what the living puffs bring to each mist pixel into `added`, and
   * drops the spent ones. A puff follows the fog the pointer carries. Returns
   * whether any puff is left.
   */
  private addPuffs(time: number): boolean {
    this.puffs = this.puffs.filter((puff) => time - puff.born < PUFF_LIFE_S);
    if (this.puffs.length === 0) return false;
    const { width, height, added, shiftX, shiftY, stirred } = this;
    added.fill(0);
    const reach = stirred ? MAX_SHIFT : 0;
    for (const puff of this.puffs) {
      const age = time - puff.born;
      const radius =
        PUFF_RADIUS + (PUFF_SPREAD - PUFF_RADIUS) * (1 - Math.exp(-age / PUFF_SWELL_S));
      const strength = PUFF_DENSITY * Math.exp(-age / PUFF_FADE_S);
      const r2 = radius * radius;
      const top = Math.max(0, Math.floor(puff.y - radius - reach));
      const bottom = Math.min(height - 1, Math.ceil(puff.y + radius + reach));
      const left = Math.max(0, Math.floor(puff.x - radius - reach));
      const right = Math.min(width - 1, Math.ceil(puff.x + radius + reach));
      for (let y = top; y <= bottom; y++) {
        for (let x = left; x <= right; x++) {
          const i = y * width + x;
          const px = stirred ? x - shiftX[i]! : x;
          const py = stirred ? y - shiftY[i]! : y;
          const d2 = (px - puff.x) * (px - puff.x) + (py - puff.y) * (py - puff.y);
          if (d2 >= r2) continue;
          const falloff = 1 - d2 / r2;
          added[i] = Math.min(1.2, added[i]! + strength * falloff * falloff);
        }
      }
    }
    return true;
  }

  private makeStars(): Star[] {
    const random = mulberry32(this.options.seed * 7919);
    const count = Math.floor(this.width * this.height * 0.6 * 0.008);
    const fadeIn = STAR_FADE_IN[this.options.shape] * this.height;
    const stars = Array.from({ length: count }, () => ({
      x: Math.floor(random() * this.width),
      y: Math.floor(random() * this.height * 0.6),
      brightness: 0.25 + random() * random() * 0.75,
      phase: random() * Math.PI * 2,
      speed: 0.5 + random() * 2.5,
      big: random() < 0.05,
      keep: random(),
    }));
    // Fewer stars near the top edge: a dithered fade rather than a line.
    return stars.filter((star) => fadeIn === 0 || star.keep < star.y / fadeIn);
  }

  /** Stars only show through clear sky, never over the fog. */
  private drawStars(time: number): void {
    const { width, height, pixels } = this;
    for (const star of this.stars) {
      const at = star.y * width + star.x;
      if (pixels[at] !== SKY) continue;
      const glow = star.brightness * (0.55 + 0.45 * Math.sin(time * star.speed + star.phase));
      if (glow < 0.15) continue;
      pixels[at] = STAR_WORDS[glow > 0.7 ? 2 : glow > 0.4 ? 1 : 0]!;
      if (!star.big || glow < 0.45) continue;
      pixels[at] = STAR_WORDS[2]!;
      for (const [dx, dy] of [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1],
      ] as const) {
        const nx = star.x + dx;
        const ny = star.y + dy;
        if (nx < 0 || ny < 0 || nx >= width || ny >= height) continue;
        if (pixels[ny * width + nx] === SKY) pixels[ny * width + nx] = STAR_WORDS[0]!;
      }
    }
  }

  /** A short streak moving down and to the left, hidden behind the fog. */
  private drawMeteor(time: number): void {
    const cycle = Math.floor(time / METEOR_PERIOD);
    const progress = (time - cycle * METEOR_PERIOD) / METEOR_DURATION;
    if (cycle === 0 || progress >= 1) return;
    const { width, height, pixels } = this;
    const random = mulberry32(this.options.seed * 131 + cycle);
    const startX = width * (0.35 + random() * 0.6);
    const startY = height * (STAR_FADE_IN[this.options.shape] + random() * 0.3);
    const distance = 28 + random() * 24;
    const headX = startX - progress * distance;
    const headY = startY + progress * distance * 0.45;
    for (let i = 0; i < 7; i++) {
      const x = Math.round(headX + i);
      const y = Math.round(headY - i * 0.45);
      if (x < 0 || y < 0 || x >= width || y >= height) continue;
      const at = y * width + x;
      if (pixels[at] !== SKY) continue;
      pixels[at] = STAR_WORDS[i === 0 ? 2 : i < 3 ? 1 : 0]!;
    }
  }
}

/**
 * Draws the mist on `canvas`, sized to cover its parent, and animates it while
 * it is visible. Returns a function that stops everything.
 */
export function startMist(canvas: HTMLCanvasElement, options: MistOptions): () => void {
  const context = canvas.getContext("2d");
  const parent = canvas.parentElement;
  if (!context || !parent) return () => {};

  const field = new MistField(canvas, context, options);
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  const start = performance.now();
  const elapsed = () => (performance.now() - start) / 1000;

  let visible = false;
  let frame = 0;
  let last = 0;
  // Where the pointer is, and where it was at the last frame, in mist pixels.
  let pointer: Point | undefined;
  let previous: Point | undefined;

  const tick = (timestamp: number) => {
    frame = requestAnimationFrame(tick);
    const moved = pointer !== previous;
    if (timestamp - last < (moved || field.isStirred ? FRAME_STIRRED_MS : FRAME_MS)) return;
    const seconds = last === 0 ? 0 : Math.min(0.2, (timestamp - last) / 1000);
    last = timestamp;
    if (pointer && previous && moved) field.stir(previous, pointer);
    previous = pointer;
    field.settle(seconds);
    field.draw(elapsed());
  };

  // The content above the canvas takes the pointer: listen on the parent, which it bubbles to.
  const onPointerMove = (event: PointerEvent) => {
    const box = canvas.getBoundingClientRect();
    pointer = { x: (event.clientX - box.left) / PIXEL, y: (event.clientY - box.top) / PIXEL };
  };
  // A click breathes a puff of mist where it lands.
  const onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0 || reducedMotion.matches) return;
    const box = canvas.getBoundingClientRect();
    field.puff(
      { x: (event.clientX - box.left) / PIXEL, y: (event.clientY - box.top) / PIXEL },
      elapsed(),
    );
  };
  const onPointerLeave = () => {
    pointer = undefined;
    previous = undefined;
  };

  const update = () => {
    const animate = visible && !reducedMotion.matches;
    if (animate && frame === 0) frame = requestAnimationFrame(tick);
    if (!animate && frame !== 0) {
      cancelAnimationFrame(frame);
      frame = 0;
    }
  };

  const intersection = new IntersectionObserver(([entry]) => {
    visible = entry?.isIntersecting ?? false;
    update();
  });
  // Also fires once on observe, which draws the first frame.
  const resize = new ResizeObserver(() => {
    field.resize(parent.getBoundingClientRect());
    field.draw(elapsed());
  });

  intersection.observe(canvas);
  resize.observe(parent);
  reducedMotion.addEventListener("change", update);
  parent.addEventListener("pointermove", onPointerMove, { passive: true });
  parent.addEventListener("pointerleave", onPointerLeave);
  parent.addEventListener("pointerdown", onPointerDown, { passive: true });

  return () => {
    intersection.disconnect();
    resize.disconnect();
    reducedMotion.removeEventListener("change", update);
    parent.removeEventListener("pointermove", onPointerMove);
    parent.removeEventListener("pointerleave", onPointerLeave);
    parent.removeEventListener("pointerdown", onPointerDown);
    cancelAnimationFrame(frame);
  };
}
