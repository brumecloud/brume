# Architecture du frontend

Le panel est une SPA React. Elle est compilée en CI et intégrée au binaire signé : le serveur n'exécute jamais npm (voir [10-developpement.md](10-developpement.md#frontend)). `brume api` la sert avec l'API, sur la même origine.

Ce document fixe la stack, l'organisation du code, les règles et l'outillage du panel. Beaucoup de pratiques viennent de [t3code](https://github.com/pingdotgg/t3code). Les règles de sécurité de [04-panel-admin.md](04-panel-admin.md) et [10-developpement.md](10-developpement.md) priment.

## Stack

| Rôle | Choix |
|---|---|
| UI | React 19, avec le React Compiler |
| Toolchain | Vite+ (`vp`) : build, tests (Vitest), lint (oxlint), formatage |
| Types | TypeScript officiel (`tsc`), en mode strict |
| Routage | TanStack Router, routes en fichiers |
| Données serveur | TanStack Query |
| Composants | shadcn sur Base UI, Tailwind v4, `cva`, `tailwind-merge`, icônes lucide |
| Tests navigateur | Vitest en mode navigateur, Chromium headless via Playwright |

**Pas de store global** (zustand, Redux) en 0.1 : l'état partagé vient de TanStack Query ou de l'URL, le reste est de l'état local.

## Organisation du code

```
web/
├── src/
│   ├── routes/            fichiers de route, minces
│   ├── api/               le seul code qui parle au serveur
│   ├── components/ui/     primitives shadcn
│   ├── components/<domaine>/
│   ├── hooks/
│   └── lib/
├── test/                  helpers de test
└── oxlint-plugin/         nos règles de lint
```

- **La logique pure va dans des fichiers `*.logic.ts`** (parsing ANSI, tri, formatage), testés dans Node. Les composants restent simples : ils lisent, affichent et délèguent.
- **`*.test.ts` tourne dans Node, `*.browser.test.tsx` dans Chromium.**
- **Alias `~/`** vers `src/`.
- **Même règle de commentaires que pour Rust :** pas de commentaire qui répète le code.

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
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "verbatimModuleSyntax": true,
    "erasableSyntaxOnly": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "paths": { "~/*": ["./src/*"] }
  },
  "include": ["src", "test", "oxlint-plugin", "vite.config.ts"]
}
```

- **`any` interdit.** Une donnée inconnue est `unknown`, puis validée.
- **Identifiants marqués** (`ServiceId`, `ProjectId`), l'équivalent des newtypes Rust : on ne peut pas passer l'un pour l'autre.
- **Pas d'`enum` ni de `namespace`** (`erasableSyntaxOnly`) : des unions de chaînes.
- **Types inférés** plutôt qu'annotés, sauf aux frontières (props, client d'API).

## React

### Le React Compiler mémoïse à notre place

- **Activé sur tout `src/`.**
- **Pas de `useMemo`, `useCallback`, ni de directive `"use memo"` ou `"use no memo"`** (`brume/no-manual-memo`).
- **Le compilateur saute en silence** une fonction qu'il ne peut pas prouver correcte : une ref lue pendant le rendu, un `setState` dans un effet. Les règles `react/*` du lint font de ces cas des erreurs.
- **Le lint ne voit pas tout** (`try` avec `finally`, `throw` dans un `try`). Un test compile donc chaque fichier de `src/` et échoue au moindre saut. Le build garde `panicThreshold: "none"`, comme le recommande la doc React.

### Chaque effet dit pourquoi il existe

`brume/no-unreasoned-effect` signale chaque `useEffect`, `useLayoutEffect` et `useInsertionEffect`. Avant d'en garder un :

- **Calculable depuis les props ou l'état ?** Le calculer pendant le rendu.
- **Réponse à une action ?** Dans le handler.
- **Réinitialise l'état quand une prop change ?** Une `key` sur le composant.
- **Charge des données ?** TanStack Query.
- **Lit une valeur hors de React ?** `useSyncExternalStore`.

Un effet n'a sa place que s'il synchronise le composant avec quelque chose hors de React : un nœud DOM, un listener, un timer, une connexion SSE. Il porte alors sa raison :

```tsx
// oxlint-disable-next-line brume/no-unreasoned-effect -- ouvre le flux SSE des logs et le ferme au démontage
useEffect(() => subscribeToLogs(serviceId, append), [serviceId]);
```

**Toute suppression de lint ou de type dit pourquoi** (`brume/require-suppression-reason`). Une suppression devenue inutile est une erreur.

## Composants et styles

- **Les exports de `components/ui` possèdent leur apparence.** On choisit un `variant` ou une `size`, on ne les restyle pas avec `className`. Les classes de layout (largeur, flex, marge, position) vont sur le parent.
- **Une apparence qui manque :** si elle est générique, c'est une nouvelle variante du composant. Si elle est propre à une fonctionnalité, elle reste dans le composant de cette fonctionnalité.
- **Pas de recette de classes empruntée :** `buttonVariants` et consorts ne s'importent pas hors de `components/ui`. On rend le composant (`render={<Button />}`).
- **Pas de `<button>`, `<input>` ou `<select>` brut stylé** comme un contrôle qu'une primitive fournit déjà.
- **Composition par slots :** un composant exporte de petits sous-composants (`Banner.Root`, `Banner.Title`, `Banner.Actions`). Chacun porte ses classes, accepte `className` et fusionne avec `cn`. On compose ou on étend, on ne copie pas le markup.
- **Tailwind dans le composant qui rend l'élément.** Le CSS global ne sert qu'à ce que les classes ne savent pas exprimer : variables de thème, animations. Pas de `style={{…}}` pour une valeur statique. Le thème passe par `@variant dark`.
- **Registre shadcn officiel uniquement.** Le CLI copie le code source dans le repo : on le relit comme le nôtre. Aucun registre tiers.
- **Compatible mobile :** cibles tactiles d'au moins 44 px sur pointeur grossier (`pointer-coarse:`).

## Routes

- **Routes en fichiers, avec `autoCodeSplitting: true`.** `routeTree.gen.ts` est généré, et exclu du lint et du formatage.
- **Un fichier de route est mince :** `createFileRoute`, la validation des paramètres, le `loader`, et un composant importé de `components/`.
- **Les paramètres d'URL sont des entrées hostiles :** un lien piégé peut être envoyé à l'admin. `validateSearch` n'accepte que des valeurs connues et borne les chaînes. Les paramètres de chemin (`$serviceId`) passent par `params.parse`.

```tsx
interface LogsSearch {
  readonly level: "all" | "warn" | "error";
  readonly q?: string;
}

export const Route = createFileRoute("/services/$serviceId/logs")({
  validateSearch: (raw: Record<string, unknown>): LogsSearch => ({
    level: raw.level === "warn" || raw.level === "error" ? raw.level : "all",
    ...(typeof raw.q === "string" && raw.q ? { q: raw.q.slice(0, 200) } : {}),
  }),
  component: ServiceLogsPage,
});
```

## Données

- **Un seul client HTTP, dans `src/api/`.** Il appelle des URL relatives (`/api/…`), ajoute `X-Brume: 1` et `Content-Type: application/json`, et transforme les erreurs en un type connu. `fetch` et `EventSource` sont interdits ailleurs.
- **Un `queryOptions` par ressource** (`api/services.ts`), partagé par le `loader` de la route (`ensureQueryData`) et par le composant. Une mutation invalide les clés qu'elle touche.
- **Révéler un secret est une mutation :** c'est un `POST`, journalisé. Elle tourne avec `gcTime: 0`, et la valeur n'entre jamais dans le cache des queries.
- **Logs en direct par SSE,** comme le prévoit [04-panel-admin.md](04-panel-admin.md). Liste virtualisée (TanStack Virtual), et seules les N dernières lignes restent en mémoire.
- **Couleurs ANSI :** un parseur écrit par nous (`ansi.logic.ts`) découpe chaque ligne en segments `{ text, style }`, rendus en `<span>` avec des classes. Il ne garde que les codes de couleur et retire les autres séquences. Jamais de HTML, ni de bibliothèque qui en produit.
- **URL venant de l'API** (domaine d'une app, lien de commit) : affichées en lien seulement si leur schéma est `https:`.

## Sécurité appliquée par l'outillage

| Règle | Comment |
|---|---|
| Jamais de `dangerouslySetInnerHTML` | `react/no-danger` en erreur |
| CSP sans script inline | Aucun `<script>` inline dans `index.html`. Le thème appliqué avant le premier rendu se charge depuis un fichier, sous `'self'`. |
| Trusted Types | Notre code n'insère jamais de HTML brut. Une dépendance qui le fait est refusée. |
| Pas d'`eval` | `eslint/no-eval`, `eslint/no-new-func`, `eslint/no-script-url` |
| Header `X-Brume` sur chaque requête | `fetch` et `EventSource` interdits hors de `src/api/` |
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
    // Sécurité
    "react/no-danger": "error",
    "react/jsx-no-target-blank": "error",
    "eslint/no-eval": "error",
    "eslint/no-new-func": "error",
    "eslint/no-script-url": "error",
    "eslint/no-restricted-globals": [
      "error",
      { name: "fetch", message: "Passer par ~/api." },
      { name: "EventSource", message: "Passer par ~/api." },
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
    // Le compilateur mémoïse les valeurs de contexte.
    "react/jsx-no-constructed-context-values": "off",

    // TypeScript
    "typescript/no-explicit-any": "error",
    "eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],

    // Nos règles
    "brume/no-manual-memo": "error",
    "brume/no-unreasoned-effect": "error",
    "brume/require-suppression-reason": "error",

    // Composants
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
                message: "Rendre le composant de components/ui au lieu d'emprunter sa recette.",
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

**Trois règles maison seulement :** `no-manual-memo`, `no-unreasoned-effect` et `require-suppression-reason`. Elles sont courtes et écrites par nous, dans `web/oxlint-plugin/`.

## Tests

- **Deux projets Vitest :** `unit` dans Node (`src/**/*.test.ts`) et `browser` dans Chromium headless (`src/**/*.browser.test.tsx`). Ni jsdom ni happy-dom : un vrai navigateur.
- **On teste la logique et le comportement observable.** Pas de snapshot de markup, pas de test qui vérifie seulement qu'un callback est branché.
- **Un bug de logique ou de données :** d'abord un test qui échoue, puis le correctif. Un bug purement visuel : une capture avant/après, sans test.
- **Jamais d'attente réelle :** ni `sleep`, ni boucle de polling, ni mesure de durée. Des timers simulés, et des assertions avec `expect.element` ou `vi.waitFor`.
- **`retry: 0` :** un test instable se corrige, il ne se relance pas.
- **`allowOnly: !process.env.CI` :** un `.only` oublié fait échouer la CI.
- **Rendu disposable** (`await using screen = await render(…)`) : un test qui échoue démonte quand même son composant.

## Build

- **Pas de cycle entre chunks.** Un plugin Vite de quelques lignes fait échouer le build si deux chunks s'importent l'un l'autre. Un cycle peut exécuter un chunk avant ses dépendances et laisser une page blanche.
- **Rechargement après une mise à jour.** Les anciens chunks disparaissent du binaire à chaque mise à jour de Brume. Un onglet resté ouvert recharge donc la page une fois sur `vite:preloadError`, avec un drapeau en `sessionStorage` contre les boucles.
- **Sortie dans `web/dist`,** intégrée au binaire par la CI.

## Dépendances

`pnpm-workspace.yaml` :

```yaml
packages:
  - web

catalog:
  react: 19.3.0
  react-dom: 19.3.0
  "@tanstack/react-query": 5.105.0
  "@tanstack/react-router": 1.170.46

minimumReleaseAge: 10080
minimumReleaseAgeExclude: []
trustPolicy: no-downgrade
allowBuilds: {}
```

- **Le catalogue déclare chaque version une seule fois, en version exacte.** `package.json` référence `catalog:`.
- **`minimumReleaseAge: 10080`,** soit 7 jours. Le défaut de pnpm est d'un jour, et non strict : faute de version assez ancienne, il en installe une récente. Une valeur explicite rend le contrôle strict.
- **`minimumReleaseAgeExclude` reste vide.** Un correctif de sécurité urgent y entre explicitement, puis en sort.
- **`trustPolicy: no-downgrade` :** l'installation échoue si une version a moins de preuves de confiance (trusted publisher, provenance) que les précédentes.
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

## Ce qu'on ne reprend pas de t3code

| t3code | Brume |
|---|---|
| `dangerouslySetInnerHTML` pour la coloration syntaxique | Interdit |
| Un script inline dans `index.html` pour le thème | Incompatible avec la CSP : un fichier sous `'self'` |
| GitHub Actions épinglées par tag | Épinglées par SHA |
| Effect et ses atoms pour les données, une trentaine de règles de lint maison | TanStack Query, trois règles maison |
| Environ 80 dépendances d'exécution | Une liste courte, chaque ajout justifié |
| `@tsc-rs/typescript`, en 0.8 | TypeScript officiel |

## Décisions ouvertes

1. **Implémentation du React Compiler :** `babel-plugin-react-compiler`, l'officiel en 1.0, ou `oxc-transform-react`, le port Rust utilisé par t3code. Le port tient en une dépendance, mais `@vitejs/plugin-react` le marque expérimental. Recommandation : l'officiel, tant que le port reste expérimental.
2. **Types partagés entre Rust et TypeScript :** générés depuis les types Rust (`ts-rs`, `specta`) ou depuis un schéma OpenAPI (`utoipa`). Sans génération, les types de `src/api/` s'écrivent à la main et finissent par diverger.
3. **TanStack DB :** prévue dans la stack, mais TanStack Query suffit peut-être pour un panel à un seul admin. À trancher au premier vrai besoin.
