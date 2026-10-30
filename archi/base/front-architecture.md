# Architecture du frontend

Le panel est une SPA React. Elle est compilée en CI et intégrée au binaire signé : le serveur n'exécute jamais npm (voir [10-developpement.md](10-developpement.md#frontend)). `brume api` la sert avec l'API, sur la même origine.

Ce document fixe la stack, l'organisation du code, les règles et l'outillage du panel. Les règles de sécurité de [04-panel-admin.md](04-panel-admin.md) et [10-developpement.md](10-developpement.md) priment.

## Stack

| Rôle | Choix |
|---|---|
| UI | React 19, avec le React Compiler |
| Toolchain | Vite+ (`vp`) : build, tests (Vitest), lint (oxlint), formatage |
| Types | TypeScript officiel (`tsc`), en mode strict |
| Routage | TanStack Router, routes en fichiers |
| Données serveur | TanStack DB (collections synchronisées par TanStack Query), TanStack Query seul pour ce qui n'est pas une entité |
| Composants | [coss ui](https://coss.com/ui) sur Base UI, Tailwind v4, `cva`, `tailwind-merge`, icônes lucide |
| Canvas | React Flow (`@xyflow/react`) |
| Animations | `motion` |
| Données simulées | MSW, en dev uniquement, tant que l'API Rust n'existe pas |
| Tests navigateur | Vitest en mode navigateur, Chromium headless via Playwright, `vitest-browser-react` |

**Pas de store global** (zustand, Redux) en 0.1 : l'état partagé vient des collections TanStack DB, qui ne sont que le cache de l'état serveur, ou de l'URL. Le reste est de l'état local.

## Organisation du code

Trois couches, du haut vers le bas. Une couche n'importe que celles du dessous.

```
web/
├── src/
│   ├── routes/            1. routing et pages : paramètres, loader, assemblage de composants
│   ├── components/ui/     2. primitives coss
│   ├── components/<domaine>/  2. composants : état local, données, rendu
│   ├── hooks/             2. hooks partagés
│   ├── helpers/           3. fonctions pures, testées dans Node : toute la logique
│   ├── api/               le seul code qui parle au serveur : client, collections, requêtes
│   ├── mocks/             l'API simulée (dev et tests uniquement)
│   └── lib/               `cn`, utilisé par components/ui
├── build/                 plugins de build et leurs tests
└── oxlint-plugin/         nos règles de lint
```

- **Une route assemble la page :** `createFileRoute`, la validation des paramètres, le `loader` et un composant de page qui place des composants. Ni état, ni logique : `useState`, `useEffect` et `useRef` y sont interdits par le lint.
- **Un composant gère son état et ses données** (`useState`, `useLiveSuspenseQuery`, mutations), puis lit, affiche et délègue. Il ne calcule rien : un tri, un filtre, un formatage va dans `helpers/`.
- **Un helper est une fonction pure,** sans React, sans TanStack, sans `api/` (sauf ses types) et sans composant. Le lint l'impose.
- **`*.test.ts` tourne dans Node, `*.browser.test.tsx` dans Chromium.**
- **Alias `~/`** vers `src/`.
- **Même règle de commentaires que pour Rust :** en anglais, et pas de commentaire qui répète le code. Le texte de l'interface reste en français.

## TypeScript

```jsonc
{
  "compilerOptions": {
    "target": "ES2023",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "module": "Preserve",
    "moduleResolution": "Bundler",
    "jsx": "react-jsx",
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "verbatimModuleSyntax": true,
    "erasableSyntaxOnly": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "types": ["vite-plus/client", "msw/vite/client"],
    "paths": { "~/*": ["./src/*"] }
  },
  "include": ["src", "test", "build", "oxlint-plugin", "vite.config.ts"]
}
```

`allowImportingTsExtensions` sert à `build/` et `oxlint-plugin/`, que Node exécute directement : leurs imports gardent l'extension `.ts`. Dans `src/`, les imports n'en ont pas.

- **`any` interdit.** Une donnée inconnue est `unknown`, puis validée.
- **Identifiants marqués** (`ServiceId`, `ProjectId`), l'équivalent des newtypes Rust : on ne peut pas passer l'un pour l'autre.
- **Pas d'`enum` ni de `namespace`** (`erasableSyntaxOnly`) : des unions de chaînes.
- **Types inférés** plutôt qu'annotés, sauf aux frontières (props, client d'API).

## React

### Le React Compiler mémoïse à notre place

- **Activé sur tout `src/`.**
- **Pas de `useMemo`, `useCallback`, ni de directive `"use memo"` ou `"use no memo"`** (`brume/no-manual-memo`).
- **Le compilateur saute en silence** une fonction qu'il ne peut pas prouver correcte : une ref lue pendant le rendu, un `setState` dans un effet. Les règles `react/*` du lint font de ces cas des erreurs.
- **Le lint ne voit pas tout** (`try` avec `finally`, `throw` dans un `try`). Un test (`build/react-compiler.test.ts`) compile donc chaque fichier de `src/` et échoue au moindre saut. Le build garde `panicThreshold: "none"`, comme le recommande la doc React.
- **Le plugin Babel officiel,** branché par `@rolldown/plugin-babel` et `reactCompilerPreset()` de `@vitejs/plugin-react`. **Babel 7, pas Babel 8 :** `babel-plugin-react-compiler` 1.0 ne lit pas l'AST de Babel 8 et saute toute fonction qui a une valeur par défaut dans ses props. Le test ci-dessus l'a détecté.

### Chaque effet dit pourquoi il existe

`brume/no-unreasoned-effect` signale chaque `useEffect`, `useLayoutEffect` et `useInsertionEffect`. Avant d'en garder un :

- **Calculable depuis les props ou l'état ?** Le calculer pendant le rendu.
- **Réponse à une action ?** Dans le handler.
- **Réinitialise l'état quand une prop change ?** Une `key` sur le composant.
- **Charge des données ?** TanStack Query.
- **Lit une valeur hors de React ?** `useSyncExternalStore`.

Un effet n'a sa place que s'il synchronise le composant avec quelque chose hors de React : un nœud DOM, un listener, un timer, une connexion SSE. Il porte alors sa raison :

```tsx
// oxlint-disable-next-line brume/no-unreasoned-effect -- opens the log SSE stream and closes it on unmount
useEffect(() => subscribeToLogs(serviceId, append), [serviceId]);
```

**Toute suppression de lint ou de type dit pourquoi** (`brume/require-suppression-reason`). Une suppression devenue inutile est une erreur.

## Composants et styles

- **Les exports de `components/ui` possèdent leur apparence.** On choisit un `variant` ou une `size`, on ne les restyle pas avec `className`. Les classes de layout (largeur, flex, marge, position) vont sur le parent.
- **Une apparence qui manque :** si elle est générique, c'est une nouvelle variante du composant. Si elle est propre à une fonctionnalité, elle reste dans le composant de cette fonctionnalité.
- **Pas de recette de classes empruntée :** `buttonVariants` et consorts ne s'importent pas hors de `components/ui`. On rend le composant (`render={<Button />}`).
- **Pas de `<button>`, `<input>` ou `<select>` brut stylé** comme un contrôle qu'une primitive fournit déjà.
- **Composition par slots :** un composant exporte de petits sous-composants, nommés comme ceux de coss (`Banner`, `BannerTitle`, `BannerActions`). Chacun porte ses classes, accepte `className` et fusionne avec `cn`. On compose ou on étend, on ne copie pas le markup.
- **Tailwind dans le composant qui rend l'élément.** Le CSS global ne sert qu'à ce que les classes ne savent pas exprimer : variables de thème, animations. Pas de `style={{…}}` pour une valeur statique. Le thème passe par `@variant dark`.
- **Deux registres seulement : coss ui (`@coss`) et le registre shadcn officiel.** coss est maintenu par Cal.com, sur Base UI et Tailwind v4 comme notre stack. Le CLI shadcn copie le code source dans le repo : on le relit comme le nôtre. On n'ajoute que les composants utilisés (`pnpm dlx shadcn@<version> add @coss/<composant>`), jamais `@coss/ui` en entier.
- **Le Chart de shadcn ne s'ajoute pas tel quel :** son `ChartStyle` injecte un `<style>` par `dangerouslySetInnerHTML`. On le retire, et les couleurs des séries passent par les variables `--chart-*` du thème.
- **Compatible mobile :** cibles tactiles d'au moins 44 px sur pointeur grossier (`pointer-coarse:`).

## Routes

- **Routes en fichiers, avec `autoCodeSplitting: true`.** `routeTree.gen.ts` est généré, et exclu du lint et du formatage.
- **Un fichier de route est mince :** `createFileRoute`, la validation des paramètres, le `loader`, et un composant de page qui assemble des composants de `components/`.
- **Les paramètres d'URL sont des entrées hostiles :** un lien piégé peut être envoyé à l'admin. `validateSearch` n'accepte que des valeurs connues et borne les chaînes. Les paramètres de chemin (`$serviceId`) passent par `params.parse`. Le parsing est un helper testé ; un paramètre invalide donne un `notFound()`.

```tsx
// routes/projects/$projectId/$env/logs.tsx
export const Route = createFileRoute("/projects/$projectId/$env/logs")({
  validateSearch: parseLogsSearch, // helpers/logs.ts, tested in Node
  component: LogsPage,
});

function LogsPage() {
  const { env } = Route.useParams();
  const filters = Route.useSearch();
  return <LogExplorer environmentName={env} filters={filters} />;
}
```

## Données

- **Un seul client HTTP, dans `src/api/`.** Il appelle des URL relatives (`/api/…`), ajoute `X-Brume: 1` et `Content-Type: application/json`, et transforme les erreurs en un type connu. `fetch` et `EventSource` sont interdits ailleurs.
- **Chaque réponse est décodée** par un helper (`helpers/decode.ts`) avant d'être typée : la forme est vérifiée, pas supposée.
- **Une collection TanStack DB par entité du modèle** (projets, environnements, services, déploiements, variables), avec `queryCollectionOptions`. Le `loader` de la route appelle `collection.preload()`. Les composants lisent par `useLiveSuspenseQuery({ query })`, avec des requêtes nommées dans `api/` (`projectById(id)`, `projectEnvironments(id)`). Les mutations sont optimistes.
- **TanStack Query seul** pour ce qui n'est pas une entité : séries de métriques, statistiques d'une base, logs de build.
- **TanStack DB est encore en 0.x.** La 1.0 arrivera par une mise à jour normale, une fois passé le délai de `minimumReleaseAge`.
- **Révéler un secret est une mutation :** c'est un `POST`, journalisé. Elle tourne avec `gcTime: 0`, et la valeur n'entre jamais dans le cache des queries.
- **Logs en direct par SSE,** comme le prévoit [04-panel-admin.md](04-panel-admin.md). L'onglet garde les 5000 dernières lignes et affiche les 500 dernières qui passent les filtres. Un flux reconnecté renvoie son historique : les lignes déjà reçues sont ignorées.
- **Pas de liste virtualisée :** `useVirtualizer` (TanStack Virtual) renvoie la même instance à chaque rendu, le React Compiler mémoïserait `getVirtualItems()`. Le compilateur la classe d'ailleurs comme incompatible et saute le composant. 500 lignes affichées tiennent sans virtualisation.
- **`EventSource` ne peut pas envoyer `X-Brume`.** Le flux de logs est un `GET` sans effet de bord : le serveur le protège par Fetch Metadata (`Sec-Fetch-Site: same-origin`), et aucun CORS ne le rend lisible depuis un autre site.
- **Couleurs ANSI :** un parseur écrit par nous (`helpers/ansi.ts`) découpe chaque ligne en segments `{ text, style }`, rendus en `<span>` avec des classes. Il ne garde que les codes de couleur et retire les autres séquences (curseur, titres de fenêtre, liens OSC) et les caractères de contrôle. Jamais de HTML, ni de bibliothèque qui en produit.
- **URL venant de l'API** (domaine d'une app, lien de commit) : affichées en lien seulement si leur schéma est `https:`.

### Données simulées

Tant que l'API Rust n'existe pas, MSW la simule dans le navigateur.

- **`src/mocks/` :** un jeu de données (`seed.ts`) et des handlers qui répondent comme l'API, avec une latence. Ils rejettent une requête sans `X-Brume`, comme le vrai serveur. Une route `/api` sans mock répond 501.
- **Hors du build :** le plugin `msw/vite` ne sert le service worker qu'en dev, et `main.tsx` ne charge les mocks que sous `import.meta.env.DEV`. Le build de production ne contient ni le worker ni les handlers.
- **Les tests navigateur utilisent les mêmes handlers.**
- **Les handlers sont le contrat de l'API** en attendant les types partagés avec Rust : voir les décisions ouvertes.

## Canvas

- **React Flow en mode non contrôlé :** il garde l'état des nœuds (positions, mesures, sélection). Le graphe est reconstruit (`key`) seulement quand sa forme change : un service ou une dépendance en plus ou en moins.
- **Chaque nœud lit son service par id** dans la collection TanStack DB. Un changement de statut redessine la carte sans reconstruire le graphe.
- **Placement automatique** (`helpers/canvas.ts`) : un service est une rangée au-dessus des services dont il dépend. Les applications sont en haut, leurs bases en dessous.
- **Les positions déplacées à la main** restent dans le navigateur, dans une collection TanStack DB sur localStorage. Ce sont des préférences d'affichage, pas des données du projet. « Réorganiser » les efface.
- **Les arêtes viennent des dépendances** que l'API calcule à partir des secrets référencés par chaque service.
- **La mention React Flow reste visible :** sa licence demande un abonnement Pro pour la masquer.
- **Le panneau d'un service est une route enfant du canvas** (`$env/_canvas/services/$serviceId/{deployments,variables,settings}`). Le canvas reste monté derrière, le bouton Retour ferme le panneau, et chaque onglet a son URL. Le canvas se décale pour garder le service sélectionné visible à gauche du panneau (`panelFocusCenter`).
- **Supprimer une variable est optimiste** (`onDelete` de la collection) : la ligne disparaît tout de suite et revient si le serveur refuse. Créer une variable passe par l'API puis recharge la collection, pour que la valeur d'un secret n'entre jamais dans le cache.
- React Flow tire `zustand` pour son état interne. La règle « pas de store global » vise notre code, pas celui des dépendances.

## Animations

- **`motion`**, et les règles du skill `improve-animations` (`.claude/skills/improve-animations/AUDIT.md`).
- **Une raison par animation.** Rien sur ce qui sert cent fois par jour : palette de commandes, raccourcis.
- **Sous 300 ms**, `ease-out` à l'entrée. Les courbes et les durées sont des variables du thème, jamais tapées dans un composant.
- **`transform` et `opacity` seulement.** Les popovers partent de leur déclencheur (`--transform-origin` de Base UI).
- **`prefers-reduced-motion` :** on garde l'opacité, on retire le mouvement.

## Sécurité appliquée par l'outillage

| Règle | Comment |
|---|---|
| Jamais de `dangerouslySetInnerHTML` | `react/no-danger` en erreur |
| CSP sans script inline | Aucun `<script>` inline dans `index.html`. Le thème appliqué avant le premier rendu se charge depuis un fichier, sous `'self'`. |
| Trusted Types | Notre code n'insère jamais de HTML brut. Une dépendance qui le fait est refusée. |
| Pas d'`eval` | `eslint/no-eval`, `eslint/no-new-func`, `eslint/no-script-url` |
| Header `X-Brume` sur chaque requête | `fetch` et `EventSource` interdits hors de `src/api/`. Seul le flux SSE des logs s'en passe, voir [Données](#données). |
| Liens externes | `react/jsx-no-target-blank` |

En dev, Vite injecte des scripts inline (HMR, React Refresh). La CSP stricte s'applique au build que sert `brume api`.

## Lint

Dans `vite.config.ts`, sous la clé `lint` de Vite+ :

```ts
lint: {
  plugins: ["eslint", "oxc", "react", "unicorn", "typescript"],
  jsPlugins: ["./oxlint-plugin/index.ts", "@shadcn/lint"],
  settings: { shadcn: { ui: "~/components/ui" } },
  categories: { correctness: "error", suspicious: "warn", perf: "warn" },
  rules: {
    // Security
    "react/no-danger": "error",
    "react/jsx-no-target-blank": "error",
    "eslint/no-eval": "error",
    "eslint/no-new-func": "error",
    "eslint/no-script-url": "error",
    "eslint/no-restricted-globals": [
      "error",
      { name: "fetch", message: "Go through ~/api." },
      { name: "EventSource", message: "Go through ~/api." },
    ],

    // React Compiler
    "react/exhaustive-effect-dependencies": "error",
    "react/globals": "error",
    "react/hooks": "error",
    "react/immutability": "error",
    "react/purity": "error",
    "react/refs": "error",
    "react/set-state-in-effect": "error",
    "react/set-state-in-render": "error",
    "react/static-components": "error",
    "react/unsupported-syntax": "error",
    "react/no-array-index-key": "error",
    "react/no-unstable-nested-components": "error",
    // The compiler memoizes context values.
    "react/jsx-no-constructed-context-values": "off",
    // Automatic JSX runtime: React does not need to be in scope.
    "react/react-in-jsx-scope": "off",

    // TypeScript
    "typescript/no-explicit-any": "error",
    "eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],

    // Our rules
    "brume/no-manual-memo": "error",
    "brume/no-unreasoned-effect": "error",
    "brume/require-suppression-reason": "error",

    // Components
    "shadcn/no-raw-colors": "error",
    "shadcn/no-unknown-classes": "error",
  },
  overrides: [
    {
      files: ["src/**"],
      excludeFiles: ["src/components/ui/**"],
      rules: {
        "shadcn/no-restyle": ["error", { allow: ["layout"] }],
        "shadcn/no-arbitrary-values": "error",
        "shadcn/require-static-classes": "error",
        "eslint/no-restricted-imports": [
          "error",
          {
            patterns: [
              {
                group: ["~/components/ui/*"],
                importNames: ["buttonVariants", "badgeVariants", "toggleVariants"],
                message: "Render the component from components/ui instead of borrowing its recipe.",
              },
            ],
          },
        ],
      },
    },
    { files: ["src/api/**"], rules: { "eslint/no-restricted-globals": "off" } },
  ],
  options: { reportUnusedDisableDirectives: "error" },
},
```

Les couches s'ajoutent par des `overrides` de `eslint/no-restricted-imports`, un par dossier : `routes/` n'importe ni `useState`, ni `useEffect`, ni `useRef` ; `components/`, `hooks/` et `api/` n'importent pas `routes/` ; `helpers/` n'importe ni React, ni TanStack, ni `components/`, ni `api/` (sauf ses types). Une override remplace la règle entière : chaque override reprend donc le motif des recettes de classes. La config complète est dans `web/vite.config.ts`.

**Trois règles maison seulement :** `no-manual-memo`, `no-unreasoned-effect` et `require-suppression-reason`. Elles sont courtes et écrites par nous, dans `web/oxlint-plugin/`, et testées avec le `RuleTester` d'oxlint.

### Revue : deux skills d'audit

`.claude/skills/improve-react` et `.claude/skills/improve-animations` auditent le code à la fin de chaque lot et proposent des plans. Ils ne modifient rien eux-mêmes.

- **Ce document prime.** improve-react s'appuie sur les règles de React Doctor. Celles qui poussent à mémoïser à la main (`jsx-no-new-function-as-prop`, `jsx-no-new-object-as-prop`, `rerender-dependencies`, `no-inline-prop-on-memo-component`) ne s'appliquent pas : le React Compiler s'en charge.
- **React Doctor n'est pas installé.** Il tire `@sentry/node`, un TypeScript inférieur à 6 et son propre oxlint. L'audit se fait sur le catalogue de règles du skill (`AUDIT.md`), sans le scanner.

## Tests

- **Deux projets Vitest :** `unit` dans Node (`src/**/*.test.ts`) et `browser` dans Chromium headless (`src/**/*.browser.test.tsx`). Ni jsdom ni happy-dom : un vrai navigateur.
- **On teste la logique et le comportement observable.** Pas de snapshot de markup, pas de test qui vérifie seulement qu'un callback est branché.
- **Un bug de logique ou de données :** d'abord un test qui échoue, puis le correctif. Un bug purement visuel : une capture avant/après, sans test.
- **Jamais d'attente réelle :** ni `sleep`, ni boucle de polling, ni mesure de durée. Des timers simulés, et des assertions avec `expect.element` ou `vi.waitFor`.
- **`retry: 0` :** un test instable se corrige, il ne se relance pas.
- **`allowOnly: !process.env.CI` :** un `.only` oublié fait échouer la CI.
- **`vitest-browser-react` démonte les composants** avant chaque test, même après un échec. Son `render` n'est pas encore disposable (`await using`).
- **Les tests navigateur passent par les handlers MSW,** comme le dev : un test monte l'app entière sur une URL (`test/render-app.tsx`) et vérifie ce que voit l'admin. Les fichiers de test tournent l'un après l'autre : ils partagent le service worker de MSW, et des iframes en parallèle mélangeraient leurs requêtes.
- **`build/react-compiler.test.ts`** compile chaque fichier de `src/` et échoue si le compilateur en saute une fonction.

## Build

- **Pas de cycle entre chunks.** Un plugin Vite de quelques lignes (`build/no-chunk-cycles.ts`) fait échouer le build si deux chunks s'importent l'un l'autre. Un cycle peut exécuter un chunk avant ses dépendances et laisser une page blanche.
- **Rechargement après une mise à jour.** Les anciens chunks disparaissent du binaire à chaque mise à jour de Brume. Un onglet resté ouvert recharge donc la page une fois sur `vite:preloadError`, avec un drapeau en `sessionStorage` contre les boucles.
- **Sortie dans `web/dist`,** intégrée au binaire par la CI.

## Dépendances

`web/pnpm-workspace.yaml`, extrait :

```yaml
packages:
  - .

catalog:
  react: 19.3.1
  react-dom: 19.3.1
  "@tanstack/react-query": 5.104.5
  "@tanstack/react-router": 1.170.46
  # …

# Vite+ ships its own Vite and Vitest: plugins must resolve the same copies.
overrides:
  vite: npm:@voidzero-dev/vite-plus-core@1.4.0
  vitest: 5.0.5

savePrefix: ""
minimumReleaseAge: 10080
minimumReleaseAgeExclude: []
trustPolicy: no-downgrade
trustPolicyExclude:
  - semver@6.3.1
allowBuilds: {}
```

- **Le workspace pnpm vit dans `web/`,** pas à la racine : la racine reste au code Rust.
- **pnpm 12,** épinglé par `packageManager`. Corepack ne sait pas encore le lancer : `npx pnpm@<version>` en attendant.

- **Le catalogue déclare chaque version une seule fois, en version exacte.** `package.json` référence `catalog:`.
- **`minimumReleaseAge: 10080`,** soit 7 jours. Le défaut de pnpm est d'un jour, et non strict : faute de version assez ancienne, il en installe une récente. Une valeur explicite rend le contrôle strict.
- **`minimumReleaseAgeExclude` reste vide.** Un correctif de sécurité urgent y entre explicitement, puis en sort.
- **`trustPolicy: no-downgrade` :** l'installation échoue si une version a moins de preuves de confiance (trusted publisher, provenance) que les précédentes. `trustPolicyExclude` liste les exceptions relues : `semver@6.3.1` a été publié avant la provenance npm, ce n'est pas un changement d'éditeur.
- **Scripts d'installation :** `strictDepBuilds` (vrai par défaut) fait échouer l'installation si un paquet a un script non relu. `allowBuilds` liste ceux qu'on a relus, à `false` par défaut.
- **`blockExoticSubdeps` garde son défaut** (vrai) : aucune dépendance transitive depuis git ou une URL de tarball.
- **`overrides`** pour garder une seule copie de chaque paquet.
- **`packageManager` épinglé** dans `package.json` ; `trustPolicy` vérifie aussi cette version de pnpm.
- **`knip`** signale les dépendances et les exports inutilisés.
- **Chaque nouvelle dépendance d'exécution se discute en PR :** ce qu'elle apporte, qui la maintient, ce qu'elle tire avec elle.

## CI

- **`pnpm install --frozen-lockfile`.**
- **`vp fmt --check`, `vp lint`, `tsc --noEmit`, `vp test run`, `knip`, `vp build`.**
- **GitHub Actions épinglées par SHA,** voir [10-developpement.md](10-developpement.md#ci).

## Décisions ouvertes

1. **Types partagés entre Rust et TypeScript :** générés depuis les types Rust (`ts-rs`, `specta`) ou depuis un schéma OpenAPI (`utoipa`). Sans génération, les types de `src/api/` s'écrivent à la main et finissent par diverger.
2. **Décodage des réponses :** des décodeurs écrits à la main (`helpers/decode.ts`, aujourd'hui), ou une bibliothèque de schémas (zod, valibot) dont TanStack DB accepte les schémas. À trancher quand le modèle grossit, avec la décision 1.
3. **Graphiques des métriques :** le Chart de shadcn (Recharts, qui tire Redux Toolkit, immer et d3), sans son `ChartStyle`. À confirmer au lot des métriques.

Tranchées : le React Compiler passe par le plugin Babel officiel (Babel 7) ; TanStack DB est adoptée.
