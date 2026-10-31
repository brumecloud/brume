# brume.dev

The landing page: a Next.js app exported as a static site (`out/`), with no server and no third-party requests at runtime.

## Commands

```sh
pnpm install
pnpm dev        # http://localhost:3000, builds the console demo first if it is missing
pnpm console    # rebuilds the console demo from ../web into public/demo/
pnpm build      # console demo, then static export in out/
pnpm run ci     # format check, lint, types, build
```

The console section embeds the real admin panel of `../web`, built in demo mode with its mocked API: see "Données simulées" in [`archi/base/front-architecture.md`](../archi/base/front-architecture.md). `public/demo/` is generated, not committed. After a change in `../web`, run `pnpm console` again.

To preview the export: `python3 -m http.server --directory out 4173`.

## Layout

- `src/app/`: the layout (fonts, metadata), the page, the 404, `robots.txt`, `sitemap.xml`, `index.md`, icons and share images.
- `src/components/sections/`: one file per section of the page, in page order in `src/app/page.tsx`.
- `src/components/`: shared pieces: install command, terminal window, console studio shot, cards, mist canvas.
- `src/components/reading-light.tsx` and `src/lib/reading-light.ts`: the page stays gray at rest; the block in the middle of the screen takes its colors as the page scrolls, like a hovered one. A block opts in with `data-light`, and colors itself with the `lit:` variant of `src/app/globals.css`. The card grids (`src/components/pinned-grid.tsx`) hold still in the middle of the screen while the scroll lights their cards one by one, where the whole grid fits on screen.
- `src/content.ts`: links and copy, shared by the page and its Markdown version.
- `src/lib/markdown.ts`: the page in Markdown, for agents and terminals, exported as `/index.md` (`src/app/index.md/route.ts`). The HTML links to it (`<link rel="alternate" type="text/markdown">`).
- `src/lib/mist.ts`: the dithered mist. Its `PALETTE[0]` must stay `--color-night` in `src/app/globals.css`.
- `src/fonts/`: Geist for the text, and JetBrains Mono (subset to Latin, arrows and box drawing) for commands and terminals, under the SIL Open Font License.

## Hosting

Served at https://brume.dev by the Vercel project `brume-site` (team "planchon's projects"), and `www.brume.dev` redirects there. Until the Vercel GitHub app can read `brumecloud/brume`, deploys are manual, from this folder. The build reads `../web`, which `vercel deploy` alone does not upload, so build locally and send the result:

```sh
vercel pull --yes --environment=production
vercel build --prod
vercel deploy --prebuilt --prod
```

Once the GitHub app can read the repository, link the project with `site/` as its root directory: Vercel then clones the whole repository, `../web` included, and every push to `main` deploys.

`vercel.json` adds what a static export cannot set itself. It uses the legacy `routes`, not `headers`, `redirects` and `rewrites`: `routes` run before the static files, which is the only way for `/` to answer something else than `index.html`.

- the security headers: `Content-Security-Policy`, `Strict-Transport-Security`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy`;
- the same headers for the console demo under `/demo/`, except that the page itself may frame it (`frame-ancestors 'self'`, `X-Frame-Options: SAMEORIGIN`), and that its scripts get no `'unsafe-inline'`;
- a year of caching for `/demo/assets/`, whose file names change with their content;
- `/install.sh`, redirected to the script of the latest GitHub release, so the script always comes from the same place as the binaries;
- `/index.md` instead of the page for agents and terminals: a request whose `Accept` asks for `text/markdown`, or whose `User-Agent` is an HTTP tool or library (`curl`, `wget`, `python-requests`, `node`…) or an AI agent (`ChatGPT-User`, `ClaudeBot`, `PerplexityBot`…). Search engines keep the HTML. `/` answers with `Vary: Accept, User-Agent`. `has` conditions do not work under `vercel dev`: check a deploy with `curl -si https://brume.dev`.

The Vercel project sets `ENABLE_EXPERIMENTAL_COREPACK=1`, so the build uses the pnpm version pinned in `package.json`.
