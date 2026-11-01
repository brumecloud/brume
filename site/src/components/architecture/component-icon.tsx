import Image from "next/image";

import type { Glyph, NodeIcon } from "~/architecture/model";
import logo from "~/assets/logo.png";
import { brandIcons } from "~/components/brand-icons";
import { logoLit } from "~/components/tone";

function GlyphMark({ name }: { readonly name: Glyph }) {
  switch (name) {
    case "globe":
      return (
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M3 12h18M12 3c3 3.2 3 14.8 0 18M12 3c-3 3.2-3 14.8 0 18" />
        </>
      );
    case "storage":
      return (
        <>
          <ellipse cx="12" cy="6" rx="8" ry="3" />
          <path d="M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3" />
        </>
      );
    case "apps":
      return (
        <>
          <rect x="4" y="4" width="7" height="7" />
          <rect x="13" y="4" width="7" height="7" />
          <rect x="4" y="13" width="7" height="7" />
          <rect x="13" y="13" width="7" height="7" />
        </>
      );
  }
}

interface IconProps {
  readonly icon: NodeIcon;
  readonly size: number;
}

/** A component's mark inside the diagram's SVG, at (x, y). */
export function SvgComponentIcon({ icon, size, x, y }: IconProps & { x: number; y: number }) {
  switch (icon.kind) {
    case "brume":
      return <image href={logo.src} x={x} y={y} width={size} height={size} className={logoLit} />;
    case "brand": {
      const brand = brandIcons[icon.name];
      if ("image" in brand) {
        return (
          <image
            href={brand.image.src}
            x={x}
            y={y}
            width={size}
            height={size}
            className={logoLit}
          />
        );
      }
      const { path, color, viewBox, fillRule } = brand;
      return (
        <svg x={x} y={y} width={size} height={size} viewBox={viewBox} className={logoLit}>
          <path d={path} fillRule={fillRule} style={{ fill: color }} />
        </svg>
      );
    }
    case "glyph":
      return (
        <svg
          x={x}
          y={y}
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          strokeWidth="1.75"
          strokeLinejoin="round"
          className="stroke-ink"
        >
          <GlyphMark name={icon.name} />
        </svg>
      );
  }
}

/** A component's mark in HTML, in color: the panel and the list show it next to the name. */
export function ComponentIcon({ icon, size }: IconProps) {
  switch (icon.kind) {
    case "brume":
      return <Image src={logo} alt="" width={size} height={size} className="shrink-0 rounded" />;
    case "brand": {
      const brand = brandIcons[icon.name];
      if ("image" in brand) {
        return (
          <Image
            src={brand.image}
            alt=""
            width={size}
            height={size}
            className="shrink-0 object-contain"
          />
        );
      }
      const { path, color, viewBox, fillRule } = brand;
      return (
        <svg aria-hidden="true" width={size} height={size} viewBox={viewBox} className="shrink-0">
          <path d={path} fillRule={fillRule} style={{ fill: color }} />
        </svg>
      );
    }
    case "glyph":
      return (
        <svg
          aria-hidden="true"
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          strokeWidth="1.75"
          strokeLinejoin="round"
          className="shrink-0 stroke-ink"
        >
          <GlyphMark name={icon.name} />
        </svg>
      );
  }
}
