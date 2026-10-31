import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // A static site: plain files that any host can serve, Brume included.
  output: "export",
  reactCompiler: true,
  // Resizing images needs a server. The few images here ship as they are.
  images: { unoptimized: true },
};

export default nextConfig;
