import { type BrandIconName, brandIcons } from "./brand-icons";

interface BrandIconProps {
  readonly name: BrandIconName;
  readonly className?: string;
}

/** A decorative brand mark: name the brand in text next to it. */
export function BrandIcon({ name, className = "size-5" }: BrandIconProps) {
  const { path, color, viewBox, fillRule } = brandIcons[name];
  return (
    <svg aria-hidden="true" viewBox={viewBox} className={className}>
      <path d={path} fill={color} fillRule={fillRule} />
    </svg>
  );
}
