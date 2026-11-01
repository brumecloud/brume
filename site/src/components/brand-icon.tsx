import Image from "next/image";

import { type BrandIconName, brandIcons } from "./brand-icons";

interface BrandIconProps {
  readonly name: BrandIconName;
  readonly className?: string;
}

/** A decorative brand mark: name the brand in text next to it. */
export function BrandIcon({ name, className = "size-5" }: BrandIconProps) {
  const icon = brandIcons[name];
  if ("image" in icon) {
    return (
      <Image src={icon.image} alt="" aria-hidden="true" className={`object-contain ${className}`} />
    );
  }
  const { path, color, viewBox, fillRule } = icon;
  return (
    <svg aria-hidden="true" viewBox={viewBox} className={className}>
      <path d={path} fill={color} fillRule={fillRule} />
    </svg>
  );
}
