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
- `src/fonts/`: Geist for the text, and JetBrains Mono (subset to Latin, arrows and box drawing) for commands and terminals, under the SIL Open Font License.

## Hosting

Served at https://brume.dev by the Vercel project `brume-site` (team "planchon's projects"), and `www.brume.dev` redirects there. Until the Vercel GitHub app can read `brumecloud/brume`, deploys are manual, from this folder: `vercel deploy --prod`. Once it can, link the project with `site/` as its root directory, and every push to `main` deploys.

`vercel.json` adds what a static export cannot set itself:

- the security headers: `Content-Security-Policy`, `Strict-Transport-Security`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy`;
- `/install.sh`, redirected to the script of the latest GitHub release, so the script always comes from the same place as the binaries.

The Vercel project sets `ENABLE_EXPERIMENTAL_COREPACK=1`, so the build uses the pnpm version pinned in `package.json`.
