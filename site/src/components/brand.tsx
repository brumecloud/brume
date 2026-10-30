import Image from "next/image";

import logo from "~/assets/logo.png";

interface BrandProps {
  readonly size?: number;
  readonly className?: string;
}

export function Brand({ size = 28, className = "" }: BrandProps) {
  return (
    <a
      href="#top"
      className={`inline-flex items-center gap-2.5 font-semibold tracking-[-0.02em] ${className}`}
    >
      <Image src={logo} alt="" width={size} height={size} className="rounded-md" />
      brume
    </a>
  );
}
