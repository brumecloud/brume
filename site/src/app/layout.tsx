import type { Metadata, Viewport } from "next";
import localFont from "next/font/local";
import type { ReactNode } from "react";

import { site } from "~/content";

import "./globals.css";

const geist = localFont({
  src: "../fonts/geist-variable.woff2",
  variable: "--font-geist",
  weight: "100 900",
});

// Subset to Latin, punctuation, arrows and box drawing: the architecture
// diagram needs the box-drawing characters, missing from the usual Latin subset.
const jetbrainsMono = localFont({
  src: "../fonts/jetbrains-mono-regular.woff2",
  variable: "--font-jetbrains-mono",
  weight: "400",
});

const silkscreen = localFont({
  src: "../fonts/silkscreen.woff2",
  variable: "--font-silkscreen",
  weight: "400",
});

export const metadata: Metadata = {
  metadataBase: new URL(site.url),
  title: site.title,
  description: site.description,
  applicationName: site.name,
  alternates: { canonical: "/" },
  openGraph: {
    type: "website",
    url: "/",
    siteName: site.name,
    title: site.title,
    description: site.description,
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: site.title,
    description: site.description,
  },
};

export const viewport: Viewport = {
  themeColor: "#020818",
  colorScheme: "dark",
};

export default function RootLayout({ children }: { readonly children: ReactNode }) {
  return (
    <html
      lang="en"
      className={`${geist.variable} ${jetbrainsMono.variable} ${silkscreen.variable}`}
    >
      <body className="font-sans text-base">
        <a
          href="#main"
          className="sr-only z-50 bg-brand px-4 py-2 font-medium text-night focus:not-sr-only focus:fixed focus:top-4 focus:left-4"
        >
          Skip to content
        </a>
        {children}
      </body>
    </html>
  );
}
