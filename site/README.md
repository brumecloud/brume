# brume.dev

The landing page: a Next.js app exported as a static site (`out/`), with no server and no third-party requests at runtime.

## Commands

```sh
pnpm install
pnpm dev        # http://localhost:3000
pnpm build      # static export in out/
pnpm run ci     # format check, lint, types, build
```

To preview the export: `python3 -m http.server --directory out 4173`.

## Layout

- `src/app/`: the layout (fonts, metadata), the page, the 404, `robots.txt`, `sitemap.xml`, icons and share images.
- `src/components/sections/`: one file per section of the page, in page order in `src/app/page.tsx`.
- `src/components/`: shared pieces: install command, terminal window, cards, mist canvas.
- `src/content.ts`: links and copy.
- `src/lib/mist.ts`: the dithered mist. Its `PALETTE[0]` must stay `--color-night` in `src/app/globals.css`.
- `src/fonts/`: Geist, JetBrains Mono (subset to Latin, arrows and box drawing) and Silkscreen, under the SIL Open Font License.

## Hosting

Any static host works, including Brume's own static sites. The host should send the security headers, since a static export cannot: `Content-Security-Policy`, `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`, `Referrer-Policy` and `Permissions-Policy`.
