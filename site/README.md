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
- `src/content.ts`: links and copy, shared by the page and its Markdown version. `pages` lists the sub-pages, which the header, the footer and the sitemap read.
- `src/lib/markdown.ts`: the page in Markdown, for agents and terminals, exported as `/index.md` (`src/app/index.md/route.ts`). The HTML links to it (`<link rel="alternate" type="text/markdown">`). Every sub-page has the same twin at `<path>/index.md`, built from the same data as the page. Every twin ends with links to all the others (`twinFooter()`), so an agent that reads one page finds the rest. `/llms.txt` lists every page with its twin, and `/llms-full.txt` holds them all in one file (`src/lib/llms.ts`).
- `src/lib/json-ld.tsx`: the structured data of every page: Brume as a `SoftwareApplication` on the landing, a `WebPage` and its breadcrumb on the sub-pages. Page titles and descriptions live in `pages` (`src/content.ts`).
- `src/app/architecture/`: the architecture page. Its data lives in `src/architecture/` (the components, the links of the diagram and the tours, every fact taken from `docs/content/docs/how-it-works/`), and its interactive diagram in `src/components/architecture/`. The list under the diagram repeats the panels in the HTML, for readers without JavaScript and for search engines.
- `src/app/roadmap/`: the roadmap page. Its data is `src/roadmap.ts`, taken from `docs/content/docs/how-it-works/roadmap.mdx`. The landing and the page share the timeline (`src/components/roadmap-timeline.tsx`).
- `src/app/vs/`: the comparison pages, `/vs` and one `/vs/<slug>` per competitor. One data file per competitor in `src/vs/`, Brume's side of every table in `src/vs/brume.ts`. Every fact about a competitor links to its official source; the research behind them, with the date it was checked, is in `archi/jalons/2026-11-01-recherche-vs/`.
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

- `Link: <page>; rel="canonical"` on every Markdown twin, so search engines index the HTML page, not its twin;
- the security headers: `Content-Security-Policy`, `Strict-Transport-Security`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy`;
- the same headers for the console demo under `/demo/`, except that the page itself may frame it (`frame-ancestors 'self'`, `X-Frame-Options: SAMEORIGIN`), and that its scripts get no `'unsafe-inline'`;
- a year of caching for `/demo/assets/`, whose file names change with their content;
- `/demo/index.html`, the page the landing frames: the Next.js build on Vercel serves HTML files without their extension, and the demo's service worker only covers URLs under `/demo/`;
- `/install.sh`: until the first release, `public/install.sh`, a placeholder that prints the Brume cloud, says Brume is not out yet, installs nothing and exits 1. It is served as plain text, so it reads in a browser. On the first release, delete `public/install.sh` and put back the redirect to the release's script, so the script always comes from the same place as the binaries: `{ "src": "/install\\.sh", "status": 307, "headers": { "Location": "https://github.com/brumecloud/brume/releases/latest/download/install.sh" } }`;
- `/index.md` instead of the page for agents and terminals, and `<path>/index.md` for each sub-page: a request whose `Accept` asks for `text/markdown`, or whose `User-Agent` is an HTTP tool or library (`curl`, `wget`, `python-requests`, `node`…) or an AI agent (`ChatGPT-User`, `ClaudeBot`, `PerplexityBot`…). Search engines keep the HTML. `/` and the sub-pages answer with `Vary: Accept, User-Agent`. `has` conditions do not work under `vercel dev`: check a deploy with `curl -si https://brume.dev`.

The Vercel project sets `ENABLE_EXPERIMENTAL_COREPACK=1`, so the build uses the pnpm version pinned in `package.json`.
