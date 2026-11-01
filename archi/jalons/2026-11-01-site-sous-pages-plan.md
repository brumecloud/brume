# Site, sous-pages : plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** Ajouter à brume.dev une page architecture interactive, une page roadmap et douze pages vs, chacune avec son jumeau Markdown.

**Architecture :** Des routes de l'App Router dans `site/src/app/`, en export statique. Les données (composants, liens, parcours, concurrents, roadmap) vivent dans des modules TypeScript typés sous `site/src/`. Pages, jumeaux Markdown et sitemap se construisent à partir de ces données. Le schéma est un SVG rendu par un composant client qui garde la sélection et le parcours en cours. Tout le texte des panneaux est aussi rendu côté serveur, dans une liste sous le schéma.

**Tech Stack :** Next.js (export statique, React Compiler), React 19, Tailwind 4, `@animate-ui/react`, oxlint, oxfmt, TypeScript. Aucune nouvelle dépendance.

**Spec :** [`2026-11-01-site-sous-pages.md`](2026-11-01-site-sous-pages.md)

## Global Constraints

- Commandes depuis `site/`, avec pnpm 12 : `pnpm run ci` (oxfmt --check, oxlint, tsc, build) doit passer à la fin de chaque tâche. Sur le Mac de Paul, pnpm 10 global ne bascule pas vers la 12 : utiliser un shim `exec node ~/Library/pnpm/.tools/pnpm/12.15.0/node_modules/pnpm/bin/pnpm.mjs "$@"` en tête du PATH.
- Ce Next.js a des changements cassants : lire le guide concerné dans `site/node_modules/next/dist/docs/` avant d'écrire une route (`01-app/02-guides/static-exports.md`, `01-app/03-api-reference/03-file-conventions/route.md`).
- Export statique : chaque route handler a `export const dynamic = "force-static"`, et une route dynamique a `generateStaticParams()` et `export const dynamicParams = false`.
- Anglais, US. Pas de mots marketing (« seamless », « powerful », « blazing fast », « simply »).
- Aucun fait sur Brume hors de `docs/content/docs/` ou `archi/base/`. Aucun fait sur un concurrent sans URL officielle.
- « Open source » partout ; la ligne licence des pages vs dit « Free and open source (FSL-1.1-ALv2, Apache 2.0 after two years) ».
- Style : `pixel-border`, `data-light` + variante `lit:`, `reveal`, `text-label`, composants de `site/src/components/`. Les couleurs n'apparaissent qu'au survol ou sous la ligne de lecture.
- Jumeaux Markdown à `<chemin>/index.md`, comme `/index.md` pour `/`. *Écart avec la spec, qui écrit `<url>.md` : Next ne sait pas suffixer un segment dynamique, alors que `index.md` sous le segment fonctionne partout.*
- Liens vers les docs : `docsPage("<section>/<page>")` de `site/src/content.ts`.
- Commits : seulement quand Paul le demande. Chaque tâche finit dans un état relisible.

## Review Focus

1. **Écran étroit (< 640 px) :** le schéma défile à l'horizontale dans son cadre, le panneau s'ouvre en dessous, et chaque composant reste assez grand pour un doigt (au moins 44 px de haut à l'échelle affichée, sinon une liste de boutons sous le schéma). Vérifié dans la tâche 4.
2. **Clavier seul :** Tab atteint chaque composant, Entrée ou Espace ouvre son panneau, Échap le ferme et rend le focus au composant. Vérifié dans la tâche 4.
3. **`prefers-reduced-motion` :** pas de lecture automatique des parcours, pas de pointillés animés. Les étapes avancent à la main. Vérifié dans la tâche 5.
4. **Sans JavaScript, ou pour un robot :** tout le contenu des panneaux (why, how, rules, if compromised) est dans le HTML exporté. Vérifié par grep sur `out/architecture.html` dans la tâche 6.
5. **Un agent ou `curl` sur une sous-page** reçoit le Markdown, comme sur `/`. Vérifié dans `vercel.json` (tâche 6) puis après déploiement : `curl -s https://brume.dev/architecture | head -1` commence par `# `.

---

## Lot 1 : navigation et page architecture

### Tâche 1 : l'enveloppe des sous-pages

**Files :**
- Create : `site/src/components/page-hero.tsx`
- Modify : `site/src/components/site-header.tsx`, `site/src/lib/markdown.ts`, `site/src/app/index.md/route.ts`, `site/src/app/sitemap.ts`, `site/src/content.ts`

**Interfaces :**
- Produces :
  - `PageHero({ title: string, lede: string, badge?: string }): JSX.Element` : un en-tête de sous-page avec `MistCanvas shape="band"`, un H1 et une phrase, avec la place du header absolu au-dessus.
  - `markdownResponse(body: string): Response`, dans `site/src/lib/markdown.ts`, avec `Content-Type: text/markdown; charset=utf-8`. La route `/index.md` s'en sert.
  - `pages: readonly { path: string; title: string }[]` dans `site/src/content.ts` : les sous-pages publiées. Le header, le footer et le sitemap la lisent. Chaque lot y ajoute ses pages.

- [ ] **Step 1 :** écrire `PageHero`, `markdownResponse` et `pages` (vide), et faire passer `src/app/index.md/route.ts` par `markdownResponse`.
- [ ] **Step 2 :** le header affiche `pages` à la place des ancres de la landing (Deploy, Security, Console, Agents, Setup), puis le bouton GitHub. Le footer affiche `pages`, puis GitHub et Issues (son lien « Security » vers `#security` devient `/#security`). Les liens internes utilisent `next/link`.
- [ ] **Step 3 :** `sitemap()` renvoie `/` puis une entrée par élément de `pages` (`${site.url}${path}`, `changeFrequency: "monthly"`, `priority: 0.8`).
- [ ] **Step 4 :** `pnpm run ci` passe, et `grep -c "<loc>" out/sitemap.xml` affiche `1`.

### Tâche 2 : le modèle et les données de l'architecture

**Files :**
- Create : `site/src/architecture/model.ts`, `site/src/architecture/components.ts`, `site/src/architecture/edges.ts`

**Interfaces :**
- Produces, dans `model.ts` :
  ```ts
  type ComponentId = "internet" | "cloudflare" | "traefik" | "caddy" | "apps" | "webhook"
    | "tailnet" | "api" | "engine" | "agent" | "buildkit" | "docker" | "telemetry"
    | "updater" | "s3";
  type Flow = "public" | "admin" | "socket" | "control" | "storage" | "docker";
  interface SecurityRule { readonly rule: string; readonly why: string }
  interface ArchComponent {
    readonly id: ComponentId;
    readonly label: string;            // "brume engine"
    readonly detail: string;           // "state and secrets", under the label in the diagram
    readonly kind: "brume" | "host" | "external";
    readonly icon: NodeIcon;           // the NodeIcon of the old diagram
    readonly column: number;           // grid position, see edges.ts
    readonly row: number;
    readonly why: string;              // one or two sentences
    readonly how: readonly string[];   // two to four points
    readonly rules: readonly SecurityRule[];
    readonly ifCompromised?: string;   // absent for external components where it makes no sense
    readonly docs: string;             // docsPage() path, e.g. "how-it-works/architecture"
  }
  interface ArchEdge {
    readonly id: string;               // "<from>-<to>"
    readonly from: ComponentId;
    readonly to: ComponentId;
    readonly flow: Flow;
    readonly points: readonly (readonly [number, number])[];
    readonly label?: { x: number; y: number; text: string; anchor?: "start" | "middle" | "end" };
  }
  function neighbors(id: ComponentId): readonly ComponentId[];   // from edges, both directions, no duplicates
  function edgesOf(id: ComponentId): readonly ArchEdge[];
  ```
- `components.ts` exporte `archComponents: readonly ArchComponent[]` (les 15, dans l'ordre de lecture du schéma). `edges.ts` exporte `archEdges`, les constantes de grille (`NODE_WIDTH`, `NODE_HEIGHT`, `COLUMNS`, `ROWS`, `VIEW_WIDTH`, `VIEW_HEIGHT`) et les couleurs par flux (`flowStroke`, `flowFill`), reprises de l'ancien schéma (`git show e6184b2:site/src/components/sections/architecture-diagram.tsx`).

- [ ] **Step 1 :** écrire `model.ts` avec les types, et `neighbors` / `edgesOf` qui lisent `archEdges`.
- [ ] **Step 2 :** écrire les 15 composants à partir des pages de `docs/content/docs/how-it-works/`. Pour chacun : `why` vient de son rôle, `how` de ses sections, `rules` des listes de règles (chaque règle garde sa raison), et `ifCompromised` de la colonne « If it is compromised » du tableau « Components » d'`architecture.mdx`, en une phrase. Les composants externes (internet, tailnet, s3, docker) n'ont `ifCompromised` que si la doc en parle (par exemple les droits du credential S3 dans `secrets-and-backups.mdx`).
- [ ] **Step 3 :** placer les composants et tracer les liens. Le chemin public (internet → cloudflare → traefik → apps/caddy/webhook) occupe le haut, le chemin admin (tailnet → api) passe dessous sans le croiser, l'engine est au centre, et l'agent, BuildKit et Docker sont en bas. telemetry, updater et s3 se placent sur les côtés. Les liens suivent `architecture.mdx` (« How the services talk to each other ») : sockets unix webhook→engine et api→engine, état voulu engine→agent et faits agent→engine, engine→s3 et agent→s3, agent→docker, agent→buildkit, docker→telemetry (logs des apps), engine→updater. La grille doit tenir lisiblement dans 1 100 px de large.
- [ ] **Step 4 :** `pnpm exec tsc --noEmit` passe. Un `ComponentId` sans composant, ou un lien vers un id inconnu, est une erreur de type.

### Tâche 3 : les parcours et les idées clés

**Files :**
- Create : `site/src/architecture/tours.ts`, `site/src/architecture/copy.ts`

**Interfaces :**
- Produces :
  ```ts
  interface TourStep { readonly focus: ComponentId; readonly edge?: string; readonly caption: string }
  interface Tour { readonly id: "visit" | "push" | "command" | "state" | "restore"; readonly title: string; readonly steps: readonly TourStep[] }
  const tours: readonly Tour[];
  const architectureCopy: { title: string; lede: string; badge: string; principlesTitle: string; principles: readonly string[] };
  ```

- [ ] **Step 1 :** écrire les cinq parcours de la spec (« A visitor opens your app », « You push to GitHub », « You, or your AI agent, run a command », « Brume saves its state », « The server dies »). Une légende d'une phrase par étape, prise dans `networking.mdx`, `builds-and-deployments.mdx`, `admin-panel.mdx`, `cli-and-ai-agents.mdx`, `secrets-and-backups.mdx`. `edge` désigne un id de `archEdges` qui existe.
- [ ] **Step 2 :** écrire `architectureCopy` : titre « How Brume works. », une phrase de lede, badge « The design of alpha 0.1 », et les six idées clés de la spec, une ligne chacune.
- [ ] **Step 3 :** `pnpm exec tsc --noEmit` passe.

### Tâche 4 : le schéma interactif et son panneau

**Files :**
- Create : `site/src/components/architecture/diagram.tsx`, `site/src/components/architecture/component-panel.tsx`, `site/src/components/architecture/explorer.tsx`
- Modify : `site/src/app/globals.css` (seulement si l'atténuation a besoin d'une classe ; `.flow` existe déjà)

**Interfaces :**
- Consumes : `archComponents`, `archEdges`, `neighbors`, `edgesOf`, `tours`.
- Produces :
  - `Explorer(): JSX.Element` (`"use client"`), qui tient `selected: ComponentId | null` et `tour: { id: Tour["id"]; step: number } | null`. Pendant un parcours, `selected` vaut `steps[step].focus`.
  - `Diagram({ selected, litEdges, onSelect }: { selected: ComponentId | null; litEdges: ReadonlySet<string>; onSelect: (id: ComponentId) => void })`
  - `ComponentPanel({ id, onSelect, onClose }: { id: ComponentId; onSelect: (id: ComponentId) => void; onClose: () => void })`

- [ ] **Step 1 :** `Diagram` reprend le rendu SVG de l'ancien schéma (boîtes à coins pixel, icônes, pointillés `.flow`, flèches par flux). Chaque composant est un `<g role="button" tabIndex={0} aria-pressed aria-label>` qui appelle `onSelect` au clic, sur Entrée et sur Espace. Sans sélection, le schéma est comme l'ancien. Avec une sélection, les composants hors de `{selected, ...neighbors(selected)}` passent à environ 30 % d'opacité, et seuls les liens de `litEdges` gardent leur couleur et leurs pointillés.
- [ ] **Step 2 :** `ComponentPanel` affiche le label et le détail, puis les sections « Why it's here », « How it works », « Security rules » (règle en gras, raison à la suite), « If it's compromised » (si présent), « Talks to » (un bouton par voisin, qui appelle `onSelect`), et un lien « Read the docs ↗ » vers `docsPage(component.docs)`. Un bouton de fermeture appelle `onClose`.
- [ ] **Step 3 :** `Explorer` pose le schéma et le panneau côte à côte à partir de 1 280 px (schéma environ 2/3, panneau 1/3, le schéma garde sa largeur et défile s'il le faut), et le panneau sous le schéma en dessous. Échap ferme le panneau et rend le focus au composant qui l'avait ouvert. Sous 640 px, une rangée de boutons (un par composant) sous le schéma sert de cible tactile.
- [ ] **Step 4 :** `pnpm run ci` passe. Lancer `pnpm exec next dev`, ouvrir une page de test temporaire ou attendre la tâche 6, puis vérifier au clavier : Tab, Entrée, Échap, retour du focus ; et à 375 px de large : défilement, panneau en dessous.

### Tâche 5 : les parcours dans le schéma

**Files :**
- Create : `site/src/components/architecture/tour-bar.tsx`
- Modify : `site/src/components/architecture/explorer.tsx`

**Interfaces :**
- Produces : `TourBar({ tour, onStart, onStep, onStop }: { tour: { id: Tour["id"]; step: number } | null; onStart: (id: Tour["id"]) => void; onStep: (step: number) => void; onStop: () => void })`.

- [ ] **Step 1 :** `TourBar` montre un bouton par parcours. Pendant un parcours, elle montre le titre, « step n of m », la légende de l'étape (`aria-live="polite"`), Précédent, Suivant, Lecture/Pause et Arrêter.
- [ ] **Step 2 :** dans `Explorer`, une étape sélectionne `focus` et allume `edge` s'il existe. La lecture automatique avance toutes les 2,5 s et s'arrête à la dernière étape. Avec `prefers-reduced-motion: reduce` (lu par `matchMedia`), pas de bouton Lecture, et les pointillés `.flow` restent immobiles (déjà le cas dans `globals.css`, à vérifier). Cliquer un composant pendant un parcours arrête le parcours.
- [ ] **Step 3 :** `pnpm run ci` passe ; vérifier à la main les cinq parcours, et le mode mouvement réduit (macOS : Accessibilité → Affichage → Réduire les animations).

### Tâche 6 : la page `/architecture`, son jumeau et le routage des agents

**Files :**
- Create : `site/src/app/architecture/page.tsx`, `site/src/app/architecture/index.md/route.ts`, `site/src/components/architecture/component-list.tsx`, `site/src/lib/architecture-markdown.ts`
- Modify : `site/src/content.ts` (`pages`), `site/vercel.json`, `site/src/components/sections/security.tsx`, `site/README.md`

**Interfaces :**
- Consumes : `PageHero`, `markdownResponse`, `Explorer`, `architectureCopy`, `archComponents`, `tours`, `pages`.
- Produces : `architectureMarkdown(): string`.

- [ ] **Step 1 :** `page.tsx` exporte `metadata` (titre « Architecture · Brume », une description d'une phrase, `alternates: { canonical: "/architecture", types: { "text/markdown": "/architecture/index.md" } }`, Open Graph) et rend, dans l'ordre : `SiteHeader`, `PageHero`, `Explorer`, les idées clés, `ComponentList`, la commande d'install, `SiteFooter`, `ReadingLight`.
- [ ] **Step 2 :** `ComponentList` (composant serveur) rend chaque composant dans un `<section id="component-<id>">` avec les mêmes sections que le panneau. C'est la version lisible sans JavaScript et par les robots.
- [ ] **Step 3 :** `architectureMarkdown()` sort le titre, le lede, le badge, les idées clés, chaque composant (why, how, rules, if compromised, talks to, docs) et chaque parcours (ses étapes numérotées). La route `index.md` renvoie `markdownResponse(architectureMarkdown())`.
- [ ] **Step 4 :** ajouter `{ path: "/architecture", title: "Architecture" }` à `pages`. La section sécurité de la landing renvoie vers `/architecture` (« See how it's built → ») à côté du lien vers le threat model.
- [ ] **Step 5 :** dans `vercel.json`, pour `/architecture` : l'en-tête `Vary: Accept, User-Agent`, et les deux routes `has` (accept `text/markdown`, user-agent des outils et des agents) avec `dest: "/architecture/index.md"`, sur le modèle de celles de `/`. Les placer avant la route catch-all s'il y en a une, et garder `continue` là où les routes de `/` l'ont.
- [ ] **Step 6 :** `site/README.md`, section Layout : `src/architecture/` et `src/components/architecture/`, et la règle des jumeaux Markdown à `<chemin>/index.md`.
- [ ] **Step 7 :** `pnpm run ci` passe, puis :
  - `test -f out/architecture.html && test -f out/architecture/index.md` ;
  - `grep -c 'id="component-' out/architecture.html` affiche `15` ;
  - `grep -q "If it's compromised" out/architecture.html` réussit ;
  - `head -1 out/architecture/index.md` commence par `# ` ;
  - `grep -c "<loc>" out/sitemap.xml` affiche `2`.
- [ ] **Step 8 :** relire chaque fait des données contre sa page de docs : aucun fait sans source. Puis lancer le serveur de dev et laisser Paul tester.

---

## Lot 2 : la roadmap

### Tâche 7 : la page `/roadmap`

**Files :**
- Create : `site/src/roadmap.ts`, `site/src/app/roadmap/page.tsx`, `site/src/app/roadmap/index.md/route.ts`, `site/src/lib/roadmap-markdown.ts`
- Modify : `site/src/content.ts` (`pages`, `milestones` si la timeline détaillée s'en sert), `site/src/components/sections/roadmap.tsx`, `site/vercel.json`, `site/README.md`

**Interfaces :**
- Consumes : `PageHero`, `markdownResponse`, `milestones`, `pages`.
- Produces :
  ```ts
  interface RoadmapItem { readonly title: string; readonly brings: string; readonly when?: "next" | "later" | "ongoing" }
  interface RoadmapTheme { readonly title: string; readonly items: readonly RoadmapItem[] }
  const roadmapThemes: readonly RoadmapTheme[];   // Access and isolation, Data and backups, Detection and recovery, Features, Hosting
  const openDecisions: readonly { readonly title: string; readonly text: string }[];
  function roadmapMarkdown(): string;
  ```

- [ ] **Step 1 :** écrire `roadmap.ts` depuis `docs/content/docs/how-it-works/roadmap.mdx`, tableau par tableau, en raccourcissant chaque « what it brings » à une ou deux phrases. `when` n'est rempli que si la timeline de la landing (`milestones`) ou `archi/base/12-plus-tard.md` place déjà l'élément. Les décisions ouvertes viennent de la partie « Open decisions » de la même page.
- [ ] **Step 2 :** `page.tsx` : `PageHero` avec « the order after 0.1 is a direction, not a promise », la timeline de `milestones` (le rendu de `sections/roadmap.tsx`, extrait dans un composant partagé si besoin), puis un bloc par thème (titre, et pour chaque élément son titre, son badge `when` s'il existe et sa phrase), puis les décisions ouvertes. Même `metadata` que l'architecture, chemin `/roadmap`.
- [ ] **Step 3 :** jumeau `index.md`, `{ path: "/roadmap", title: "Roadmap" }` dans `pages`, routes `vercel.json` comme pour `/architecture`. La section roadmap de la landing finit par « Full roadmap → » vers `/roadmap`.
- [ ] **Step 4 :** `pnpm run ci` passe ; `test -f out/roadmap.html && test -f out/roadmap/index.md` ; `grep -c "<loc>" out/sitemap.xml` affiche `3`. Serveur de dev, test de Paul.

---

## Lot 3 : les pages vs

La recherche est faite : `archi/jalons/2026-11-01-recherche-vs/<slug>.md`, un fichier par concurrent, vérifié le 2026-11-01, avec URL et citation pour chaque fait. Les corrections d'angle sont dans la spec.

### Tâche 8 : les données des concurrents

**Files :**
- Create : `site/src/vs/model.ts`, `site/src/vs/<slug>.ts` (12 fichiers), `site/src/vs/index.ts`
- Modify : `site/src/components/brand-icons.ts`

**Interfaces :**
- Produces :
  ```ts
  type VsSlug = "coolify" | "dokploy" | "caprover" | "portainer" | "kamal" | "dokku" | "easypanel"
    | "railway" | "render" | "fly" | "vercel" | "heroku";
  interface VsRow { readonly label: string; readonly brume: string; readonly them: string; readonly source?: string }
  interface Competitor {
    readonly slug: VsSlug;
    readonly name: string;                 // "Fly.io"
    readonly kind: "self-hosted" | "managed";
    readonly icon?: BrandIconName;
    readonly seoTitle: string;             // under 65 characters
    readonly description: string;          // meta description, under 160 characters, unique
    readonly oneLiner: string;             // line on /vs
    readonly verdict: string;
    readonly rows: readonly VsRow[];       // about eight
    readonly different: readonly string[]; // three, "Where Brume is different"
    readonly better: readonly string[];    // "Where <name> is better today"
    readonly pickThem: string;
    readonly pickBrume: string;
    readonly checked: string;              // "2026-11-01"
  }
  const competitors: readonly Competitor[];  // self-hosted first, in the order of the spec
  function competitor(slug: VsSlug): Competitor;
  ```

- [ ] **Step 1 :** écrire `model.ts` et `index.ts`.
- [ ] **Step 2 :** écrire les douze fichiers depuis les notes de recherche. Les lignes du tableau suivent la spec (self-hosted ou managé). Chaque `them` qui affirme un fait porte la `source` de sa ligne dans la note. La colonne Brume décrit l'alpha 0.1 au présent ; sa ligne maturité dit « Alpha 0.1, in progress ». Pas d'incident ni de panne. Verdict, différences et points forts propres à chaque concurrent : jamais le même texte d'une page à l'autre.
- [ ] **Step 3 :** ajouter à `brand-icons.ts` les logos qui manquent et qui existent dans Simple Icons (CC0) : kamal, dokku, easypanel, railway, render, fly, vercel, heroku, s'ils y sont. Sinon, pas d'`icon`.
- [ ] **Step 4 :** `pnpm exec tsc --noEmit` passe. `grep -L "source:" site/src/vs/*.ts | grep -v "model\|index"` n'affiche rien.

### Tâche 9 : les pages `/vs` et `/vs/[slug]`

**Files :**
- Create : `site/src/app/vs/page.tsx`, `site/src/app/vs/index.md/route.ts`, `site/src/app/vs/[slug]/page.tsx`, `site/src/app/vs/[slug]/index.md/route.ts`, `site/src/components/vs/comparison-table.tsx`, `site/src/lib/vs-markdown.ts`
- Modify : `site/src/content.ts` (`pages`), `site/src/components/site-footer.tsx`, `site/src/components/sections/security.tsx`, `site/vercel.json`, `site/src/app/sitemap.ts`, `site/README.md`

**Interfaces :**
- Consumes : `competitors`, `competitor`, `PageHero`, `markdownResponse`.
- Produces : `vsMarkdown(slug: VsSlug): string`, `vsIndexMarkdown(): string`.

- [ ] **Step 1 :** `vs/[slug]/page.tsx` : `generateStaticParams()` sur `competitors`, `dynamicParams = false`, `generateMetadata()` avec `seoTitle`, `description` et le canonical `/vs/<slug>`. Rendu dans l'ordre de la spec : H1 « Brume vs <name> », badge « Alpha 0.1 · in progress », verdict, tableau (cellule `them` liée à sa source, « Checked <date> » sous le tableau), « Where Brume is different », « Where <name> is better today », « Pick <name> if… / Pick Brume if… », commande d'install, liens vers les autres comparaisons de la même famille.
- [ ] **Step 2 :** `vs/page.tsx` : deux groupes (« Self-hosted platforms », « Managed platforms »), une ligne par concurrent avec son logo, son nom et `oneLiner`.
- [ ] **Step 3 :** jumeaux Markdown (`/vs/index.md` et `/vs/<slug>/index.md`), avec le tableau en Markdown et les sources en liens.
- [ ] **Step 4 :** `{ path: "/vs", title: "Compare" }` dans `pages`. Le sitemap ajoute les douze `/vs/<slug>`. Le footer liste les douze comparaisons. Dans le tableau sécurité de la landing, le nom de chaque outil renvoie vers sa page vs. `vercel.json` : mêmes routes `has` pour `/vs` et `/vs/[a-z]+` vers leur `index.md`.
- [ ] **Step 5 :** `pnpm run ci` passe, puis :
  - `ls out/vs/*.html | wc -l` affiche `12` ;
  - `ls out/vs/*/index.md | wc -l` affiche `12` ;
  - `grep -c "<loc>" out/sitemap.xml` affiche `16` ;
  - `grep -h "<title>" out/vs/*.html | sort | uniq -d` n'affiche rien (titres uniques) ;
  - `grep -h 'name="description"' out/vs/*.html | sort | uniq -d` n'affiche rien.
- [ ] **Step 6 :** relire chaque page vs comme quelqu'un qui hésite entre les deux outils : le verdict est-il juste pour les deux ? Serveur de dev, test de Paul.
