# Site : pages architecture, vs et roadmap

Trois sortes de sous-pages pour brume.dev, à côté de la landing : une page architecture interactive, une page vs par concurrent, et une roadmap détaillée. Décidé avec Paul le 1er novembre 2026.

## Pourquoi

- **La landing vend, elle n'explique plus.** Elle a été réduite à quatre promesses : deploy anything, secure by design, made for you and your agents, nothing to babysit. Le détail de l'architecture en est sorti (schéma, principes, cartes). Il faut un endroit où un développeur ou un auditeur comprend vraiment les choix, et pourquoi Brume est plus sûr.
- **Les pages vs servent le référencement.** Les gens cherchent « Coolify alternative », « Railway alternative ». Une page par concurrent, sur une URL dédiée.
- **La roadmap de la landing est un résumé.** La page roadmap liste tout ce qui est prévu, avec ce que chaque élément apporte.

## Objectifs

- Un visiteur clique sur un composant du schéma et comprend pourquoi il est là, comment il marche, quelles règles de sécurité il suit, et ce qu'obtient un attaquant qui le compromet.
- Un visiteur lance un parcours et voit, étape par étape, comment une requête ou un déploiement traverse Brume.
- Chaque page vs se classe sur « Brume vs X » et « X alternative », et donne une réponse honnête.
- Chaque page a sa version Markdown, pour les agents, comme `/index.md`.

## Hors périmètre

- Pas de nouvelle page de docs : les pages de `docs/` restent la référence, ces pages-là les vulgarisent et y renvoient.
- Pas de nouvelles images de partage par page : les pages reprennent celles du site.
- Pas de blog, pas de pages par cas d'usage.

## Anti-objectifs

- **Verbeux.** Paul a jugé l'ancienne landing trop bavarde. Même ton que la nouvelle : des titres courts, une phrase par idée. Le détail va dans les panneaux qu'on ouvre, pas dans le flux de la page.
- **Inventer.** Aucun fait sur Brume qui ne vienne de `docs/content/docs/` ou de `archi/base/`. Aucun fait sur un concurrent sans source officielle liée. Mêmes règles que `docs/AGENTS.md`, section « Facts ».
- **Cacher l'alpha.** Brume est une alpha face à des outils matures. Chaque page le dit.
- **Des pages vs clonées.** Douze fois le même texte avec le nom changé : Google les traite comme du contenu dupliqué, et un lecteur le voit. Chaque page a son verdict, ses différences et ses points forts propres.
- **Dénigrer.** On compare des faits sourcés. On dit aussi où le concurrent fait mieux.

## Contraintes

- **Le site actuel :** Next.js en export statique dans `site/`, aucune requête tierce au chargement, CSP de `site/vercel.json`. Ce Next.js a des changements cassants : lire `site/node_modules/next/dist/docs/` avant d'écrire une route.
- **Anglais,** comme la landing.
- **Le style de la landing :** fond nuit, `pixel-border`, couleurs qui n'apparaissent qu'au survol ou sous la ligne de lecture (`data-light`, variante `lit:`), polices Geist et JetBrains Mono. Les composants partagés de `site/src/components/` se réutilisent.
- **« Open source » sur le site.** Décision de Paul, malgré la licence FSL-1.1-ALv2.
- `pnpm run ci` passe dans `site/` (format, lint, types, build).

## Ce qui est décidé

### Les URLs

| Page | URL |
|---|---|
| Architecture | `/architecture` |
| Roadmap | `/roadmap` |
| Index des comparaisons | `/vs` |
| Une comparaison | `/vs/<slug>` pour `coolify`, `dokploy`, `caprover`, `portainer`, `kamal`, `dokku`, `easypanel`, `railway`, `render`, `fly`, `vercel`, `heroku` |

Chaque page a sa version Markdown à `<url>.md` (par exemple `/vs/coolify.md`), construite depuis les mêmes données que la page.

### La navigation

- **Header :** Architecture, Roadmap, Compare (`/vs`), et le bouton GitHub. Les ancres de la landing (Deploy, Security…) en sortent : la landing est assez courte pour se lire au scroll. *Choix de Claude.*
- **Footer :** Architecture, Roadmap, la liste des douze comparaisons, GitHub, Issues. Les liens internes aident le référencement.
- **La landing renvoie aux sous-pages :** la section sécurité vers `/architecture`, la roadmap vers `/roadmap`, le tableau comparatif vers les pages vs de chaque ligne.
- **Sitemap :** toutes les pages.

### La page architecture

En haut, un titre et une phrase, et la mention « the design of alpha 0.1 ».

**Le schéma.** On repart de l'ancien schéma de la landing (`git show e6184b2:site/src/components/sections/architecture-diagram.tsx`) et on le complète. Composants : Internet (les visiteurs et GitHub), Cloudflare Tunnel (cloudflared), Traefik, Caddy, tes apps, `brume webhook`, ton tailnet (toi, la CLI, les agents IA), `brume api`, `brume engine`, `brume agent`, BuildKit, Docker, `brume telemetry`, `brume updater`, le bucket S3. Les liens gardent leurs couleurs par chemin : public, admin, sockets, contrôle, stockage, Docker.

**Un clic sur un composant :**

- les autres s'estompent, ses liens et ses voisins restent allumés ;
- un panneau s'ouvre à côté du schéma (en dessous sur mobile), avec :
  - **Why it's here :** une ou deux phrases ;
  - **How it works :** deux à quatre points ;
  - **Security rules :** chaque règle avec sa raison, en une phrase chacune ;
  - **If it's compromised :** ce qu'obtient un attaquant, d'après le tableau « Components » de `docs/content/docs/how-it-works/architecture.mdx` ;
  - **Talks to :** ses voisins, cliquables pour passer d'un composant à l'autre ;
  - un lien vers la page de docs qui fait foi.
- Les composants externes (Internet, tailnet, S3, Docker) ont un panneau plus court, sans « If it's compromised » quand ça n'a pas de sens.

**Les parcours.** Des boutons au-dessus ou à côté du schéma lancent un parcours. Chaque étape allume un composant ou un lien et affiche une légende d'une phrase. On avance seul (suivant, précédent) ou en lecture automatique. Cinq parcours :

1. **A visitor opens your app :** Internet, Cloudflare Tunnel, Traefik, l'app ou Caddy.
2. **You push to GitHub :** le webhook signé, la file de build de l'engine, l'agent, BuildKit, le déploiement par digest, la bascule du trafic une fois l'app saine.
3. **You, or your AI agent, run a command :** le tailnet, `brume api`, l'engine qui vérifie la session ou la signature et la politique, l'événement, l'état voulu envoyé à l'agent, Docker. Une action irréversible attend l'approbation.
4. **Brume saves its state :** le log et les secrets chiffrés de l'engine vers S3, les dumps chiffrés de l'agent vers S3, les logs des apps vers `brume telemetry`.
5. **The server dies :** un nouveau VPS, l'install, la restauration depuis le bucket.

Les étapes de chaque parcours viennent des pages de docs, sans raccourci qui deviendrait faux.

**Sous le schéma :** les idées clés de l'architecture, une ligne chacune, par exemple « the public path and the admin path never meet », « the engine decides, the agent acts », « only the agent holds Docker, and it listens on nothing », « one user and one socket per service », « Brume can't delete what it sent to S3 », « signed updates, never backwards ».

**Accessibilité et référencement :**

- Les composants sont des boutons, au clavier comme à la souris ; Échap ferme le panneau.
- Les parcours respectent `prefers-reduced-motion` : pas de lecture automatique, pas de paquets animés.
- Tout le contenu des panneaux est dans le HTML servi : par exemple une liste des composants sous le schéma, que le panneau reprend.

**Sources :** les pages anglaises de `docs/content/docs/how-it-works/`, surtout `architecture.mdx`, `networking.mdx`, `admin-panel.mdx`, `runtime.mdx`, `builds-and-deployments.mdx`, `secrets-and-backups.mdx`, `observability.mdx`, `host-and-installation.mdx`, `cli-and-ai-agents.mdx`, `threat-model.mdx`. Les liens vont vers ces pages sur GitHub (`docsPage()` de `site/src/content.ts`), tant que les docs ne sont pas en ligne.

### Les pages vs

**Deux familles :**

- **Self-hosted** (Coolify, Dokploy, CapRover, Portainer, Kamal, Dokku, Easypanel) : le même travail sur ton serveur. L'angle est la sécurité : panel exposé, accès à Docker, ce qu'obtient un attaquant.
- **PaaS managés** (Railway, Render, Fly.io, Vercel, Heroku) : l'angle est ton serveur, tes données, un coût fixe et pas de lock-in. On dit aussi ce qu'ils font que Brume ne fera pas : scaling, régions, zéro serveur à gérer.

**Une page :**

1. Le H1 « Brume vs X », un titre de page pour la recherche (par exemple « Brume vs Coolify: a self-hosted Coolify alternative with zero open ports ») et une meta description propre à la page.
2. Un verdict en une ou deux phrases.
3. Un tableau d'environ huit lignes, Brume contre X, chaque cellule de X liée à sa source.
4. **Where Brume is different :** trois points.
5. **Where X is better today :** les vrais points forts de X, dits franchement. *Accepté par Paul.*
6. **Pick X if… / Pick Brume if…**
7. La commande d'install, et les liens vers les autres comparaisons.

Lignes du tableau. Self-hosted : où ça tourne, le panel admin, l'accès à Docker, ce qu'obtient un attaquant qui a le panel, les ports entrants, l'install, les agents IA (CLI, MCP), la licence et le prix, le multi-serveurs, la maturité. Managé : où tournent tes apps, qui tient tes données, le modèle de prix, le lock-in, les agents IA, le scaling et les régions, la licence, la maturité. Le constructeur ajuste la liste pour qu'elle soit juste pour chaque concurrent.

**La colonne Brume** décrit l'alpha 0.1 : un serveur, un admin. La ligne licence dit « Free and open source (FSL-1.1-ALv2, Apache 2.0 after two years) ». *Choix de Claude : « open source » comme l'a décidé Paul, avec le nom exact de la licence pour rester vérifiable.*

**Brume au présent, comme sur la landing.** Les pages vs parlent de Brume comme la landing validée par Paul : au présent, avec la commande d'install, et un badge « Alpha 0.1 · in progress » visible en haut de page. La ligne maturité dit « alpha, in progress ». *Choix de Claude, à la question du relecteur.*

**Les faits sur les concurrents :** relevés dans leurs docs officielles, avec l'URL et la date de vérification, dans un fichier de données par concurrent. La recherche du 1er novembre 2026 corrige plusieurs angles :

- **Coolify** ne monte pas le socket Docker : il passe par SSH en root, même sur son propre hôte, avec une clé sans passphrase stockée dans l'app. Le tableau de la landing (« Root on every managed server ») reste juste.
- **Easypanel** a déjà Cloudflare Tunnel et la 2FA. Contre lui, l'angle est le panel sur le tailnet seulement et le socket Docker hors du panel, pas les ports fermés.
- **Dokploy** documente un tunnel Cloudflare et un panel sur Tailscale, à la main. Le panel garde le socket.
- **Vercel** fait tourner des containers depuis 2026, en bêta, mais sans volume persistant ni base hébergée. Ne pas écrire « no containers ».
- **Heroku** est en « sustaining engineering » depuis le 6 février 2026. Le dire factuellement, avec la source.
- **Pas d'incident ni de panne** dans les pages, même sourcés : ce serait du dénigrement.

**La page `/vs`** liste les douze en deux groupes, avec une ligne chacun.

**Logos :** ceux qui manquent dans `site/src/components/brand-icons.ts` viennent de Simple Icons (CC0) quand ils y sont. Sinon, le nom seul.

### La page roadmap

- **En haut,** la timeline de la landing (Now, Next, Later, All along), plus détaillée.
- **Ensuite, tout ce qui est prévu, par thème,** repris de `docs/content/docs/how-it-works/roadmap.mdx` : accès et isolation, données et backups, détection et restauration, fonctionnalités, hébergement. Chaque élément dit ce qu'il apporte, en une ou deux phrases.
- **Les décisions ouvertes** dans leur propre partie, décrites comme ouvertes.
- **Pas de priorité inventée :** un élément n'a un « Next » ou un « Later » que si la landing ou `archi/base/` le place déjà. La page reprend « the order after 0.1 is a direction, not a promise ».

### Les versions Markdown et les agents

- Chaque page a son jumeau `.md`, généré comme `site/src/lib/markdown.ts`.
- `site/vercel.json` sert déjà `/index.md` aux agents et aux outils HTTP qui demandent `/`. Les sous-pages suivent la même règle vers leur jumeau.

## Laissé au constructeur

- La disposition du schéma, son découpage en composants React et la façon de piloter l'état (composant sélectionné, parcours, étape).
- Comment les données vivent dans `site/src/` (un fichier par concurrent, un fichier pour les composants du schéma…).
- Le texte exact, dans les limites ci-dessus.
- Le JSON-LD de chaque page.

## Ordre de livraison

Paul teste après chaque lot avant le suivant.

1. **Navigation et page architecture,** le cœur de la demande.
2. **Page roadmap.**
3. **Pages vs,** dont la recherche commence dès le premier lot.
