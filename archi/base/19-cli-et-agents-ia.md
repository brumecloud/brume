# CLI, MCP et agents IA

Brume est agent first. On doit pouvoir déployer n'importe quoi sur son serveur en une commande dans le terminal, ou en une phrase à un agent IA. Tout ce que fait le panel, la CLI le fait aussi, et un agent IA s'en sert directement ou par MCP.

## Vocabulaire

- **« L'agent » seul désigne `brume agent`,** le service qui pilote Docker sur chaque machine. Un agent de code (Claude Code, Codex, Cursor…) s'appelle toujours un **agent IA**, dans le code comme dans la doc.

## Principes

1. **Le panel, la CLI et le MCP passent par la même API.** Une action disponible dans le panel mais pas dans la CLI est un bug. Seules exceptions : ce qui demande le second facteur, c'est-à-dire approuver une action, révéler un secret et valider une connexion de la CLI.
2. **La CLI est écrite d'abord pour les agents IA.** Un humain s'en sert aussi bien, l'inverse n'est pas vrai.
3. **Le modèle de sécurité est celui du panel :** le tailnet, deux facteurs, et l'engine qui vérifie chaque demande. Une API compromise ne peut toujours rien faire sans l'admin.
4. **Un agent IA lit du contenu hostile.** Les actions irréversibles attendent donc le second facteur de l'admin, dans le panel. Voir [Les approbations](#les-approbations).

## Une commande, une phrase

```bash
brume deploy                         # le dossier courant
brume deploy ghcr.io/acme/api:1.4    # une image
brume deploy github.com/acme/site    # un repo GitHub, par la GitHub App
brume deploy postgres                # un template
```

À un agent IA, la même chose s'écrit en une phrase : « déploie ce dossier sur mon serveur Brume, avec une base Postgres ».

- **`brume deploy` attend que le service soit sain,** puis affiche son URL. `--no-wait` rend la main tout de suite.
- **Il faut d'abord `brume login`, une seule fois par appareil.** Voir [La connexion](#la-connexion--brume-login).

### Ce que `deploy` déploie

| Argument | Source | Build |
|---|---|---|
| aucun, ou un chemin | le dossier, envoyé par la CLI | Railpack, sur le serveur |
| `github.com/<owner>/<repo>` | le repo, par la GitHub App | Railpack, puis à chaque push sur la branche configurée |
| `postgres`, `mongodb`, `redis` | le template | aucun |
| une référence d'image | le registry | aucun : le tag est résolu en digest, voir [05-runtime.md](05-runtime.md#images) |

- **L'argument est lu dans cet ordre :** un chemin qui existe, puis un nom de template, puis un repo GitHub, puis une image. Sinon `deploy` échoue et dit ce qu'il a essayé.
- **`--dir`, `--repo`, `--template` et `--image` lèvent toute ambiguïté.** Le skill demande aux agents IA de toujours les utiliser.
- **Le type d'un service se choisit à sa création :** un container par défaut, un site statique avec `--static`. Voir [templates.md](templates.md#site-statique).
- **Un service qui écoute sur un port est exposé** sur un sous-domaine du wildcard, et `deploy` affiche son URL. `--private` le garde sur le réseau de son environnement. Une base n'est jamais exposée. Voir [03-reseau.md](03-reseau.md#domaines).

### La cible

- **Au premier `deploy` d'un dossier,** la CLI crée le projet (nommé d'après le dossier), l'environnement `production` et le service. Elle note ce lien dans sa config locale, sur le laptop : rien n'est écrit dans le repo.
- **Les `deploy` suivants depuis ce dossier** visent le même service. `--project`, `--env` et `--service` choisissent une autre cible.
- **`deploy` ne touche pas aux changements en attente du panel.** Il applique les siens, et ceux de l'admin restent en attente.
- **Relancer `deploy` sans rien changer ne crée pas de révision :** même archive, ou même digest, et même spec.

### Le dossier envoyé par la CLI

Une deuxième source à côté de GitHub, pour déployer sans repo ni push.

- **La CLI respecte `.gitignore`.** Elle n'envoie jamais `.git`, ni `.env` et `.env.*` : un `COPY . .` les mettrait dans une couche de l'image. Les secrets passent par `brume secrets`.
- **Une archive de taille limitée,** avec son empreinte SHA-256.
- **Personne d'autre que le builder n'ouvre l'archive.** L'API l'écrit telle quelle dans un dossier de dépôt. L'engine n'en reçoit que l'empreinte et la taille. L'agent la monte en lecture seule dans le builder, qui vérifie l'empreinte puis l'extrait. Ni l'engine, ni l'agent, ni l'API ne l'ouvrent : voir [02-architecture.md](02-architecture.md#lengine).
- **L'extraction est bornée :** les chemins en `../` et les symlinks qui sortent du dossier sont rejetés, et la taille décompressée comme le nombre de fichiers sont limités.
- **Le même builder que pour GitHub,** avec le même cache par environnement et les mêmes secrets de build. Voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder).
- **L'archive est supprimée après le build.** Un rollback repart de l'image, comme pour GitHub.
- **La révision note son origine :** l'empreinte de l'archive et, si le dossier est un repo git, le commit et le fait qu'il ait ou non des modifications non commitées. La CLI déclare ces deux derniers champs : ils servent à l'affichage, pas à la sécurité.
- **Sur un service relié à GitHub,** un `deploy` depuis un dossier est accepté. Le prochain push redéploie depuis la branche, comme après un rollback.

## La connexion : `brume login`

`brume login <adresse du panel>` relie un appareil au serveur, une fois.

1. La CLI génère une paire de clés Ed25519 sur le laptop. La clé privée ne quitte jamais l'appareil : elle va dans le trousseau de macOS, ou dans un fichier en `0600` ailleurs.
2. La CLI envoie la clé publique à l'API par le tailnet, avec le nom de l'appareil. Elle ouvre le panel sur la demande et affiche un code court.
3. Dans le panel, l'admin vérifie que le code est le même et valide avec son second facteur, même si sa session est déjà ouverte.
4. L'engine enregistre la clé, liée à l'identité Tailscale de l'admin, et `brume login` rend la main.

### Des requêtes signées plutôt qu'un jeton

- **Chaque requête de la CLI est signée** par la clé de l'appareil. La signature couvre la méthode, le chemin, l'empreinte du body, un horodatage et un nonce.
- **C'est l'engine qui vérifie la signature, pas l'API.** Il vérifie aussi la fraîcheur de l'horodatage et refuse un nonce déjà vu. La commande exécutée est celle qui a été signée. Le format exact est laissé au builder.
- **Pourquoi pas un simple jeton :** un jeton passe par l'API à chaque requête. Une API compromise le copierait et agirait seule tant qu'il est valide. Dans le panel, une session courte borne cette fenêtre. Une clé de la CLI dure des semaines : elle ne doit donc jamais transiter. Une API compromise peut au pire retenir ou bloquer une requête signée, pas la modifier ni en forger une autre.
- **Une requête porte soit le cookie de session, soit une signature, jamais les deux,** sinon elle est rejetée. Les protections CSRF du panel (Fetch Metadata, `Origin`, `X-Brume`) visent le cookie, que le navigateur envoie seul. Un navigateur n'ajoute jamais de signature de lui-même : de ce côté, il n'y a pas de CSRF possible. Voir [04-panel-admin.md](04-panel-admin.md#csrf-et-requêtes-cross-site).

### La clé

- **Liée à l'identité Tailscale :** une requête dont l'en-tête `Tailscale-User-Login` n'est pas celui de la clé est rejetée. Un appareil tagué n'a pas d'identité Tailscale : une CI ou un serveur ne peut donc pas utiliser la CLI en 0.1. Voir [12-plus-tard.md](12-plus-tard.md).
- **Expire après 30 jours sans usage,** et se révoque à tout moment depuis le panel, qui liste les clés avec leur appareil et leur dernier usage. `brume logout` révoque la clé, puis l'efface du laptop.
- **Hors du log d'événements,** comme les sessions : une table SQLite, jamais envoyée sur S3. Une restauration révoque donc toutes les clés. Enregistrer ou révoquer une clé est un événement d'audit. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#ce-qui-est-un-événement).
- **La passkey reste dans le panel.** La CLI ne la demande jamais. Elle est liée à l'origine du panel et exige Touch ID, Face ID ou le code de l'appareil : ni un terminal ni un agent IA ne peuvent valider à la place de l'admin. Voir [04-panel-admin.md](04-panel-admin.md#2-une-passkey-à-la-connexion).
- **Sans Tailscale,** la CLI passe par le tunnel SSH ou par l'adresse publique du panel, comme le navigateur. La clé n'est alors liée à aucune identité réseau, comme la session à son cookie. La connexion passe toujours par le second facteur dans le panel. Voir [04-panel-admin.md](04-panel-admin.md#sans-tailscale).

## Ce qu'une clé de la CLI permet

| Action | Avec une clé de la CLI |
|---|---|
| Lire les projets, services, déploiements, logs et métriques, les variables, et le nom des secrets | oui |
| Créer et modifier des projets, environnements, services et variables | oui |
| Définir la valeur d'un secret | oui, en écriture seule |
| Déployer, rollback, redémarrer | oui, sauf dans un [environnement protégé](#les-environnements-protégés) |
| Supprimer un service, un environnement, un projet ou un volume ; purger une révision | après approbation |
| Ouvrir l'egress d'un service, ouvrir l'accès d'une base par le tailnet, ajouter un domaine personnalisé | après approbation |
| Révéler la valeur d'un secret | jamais : dans le panel uniquement |
| Enregistrer une nouvelle clé | jamais : `brume login` passe par le panel |
| Activer une option dangereuse | jamais : sur l'hôte uniquement, voir [05-runtime.md](05-runtime.md#policytoml--les-options-dangereuses) |

- **Les valeurs de secrets ne ressortent jamais par la CLI.** Une valeur qui arrive dans le terminal arrive aussi dans le contexte de l'agent IA, et chez son fournisseur.
- **`brume secrets import <fichier>`** lit un `.env` et envoie ses valeurs comme secrets, sans les afficher. Un agent IA peut donc migrer un `.env` sans que les valeurs passent par son contexte.

### Les approbations

- **Une action soumise à approbation n'est pas refusée, elle attend.** La CLI rend un statut `approval_required`, avec un code de sortie dédié et l'URL de la demande dans le panel. `--wait` attend la décision.
- **Le panel montre exactement ce qui sera fait :** le diff, l'appareil, et le client (CLI ou MCP). L'admin approuve avec son second facteur, même si sa session est ouverte.
- **Une demande expire après 15 minutes et ne s'exécute qu'une fois.** Elle porte la version attendue de son `stream` : si l'état a changé entre-temps, elle est rejetée. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#le-cycle-dune-commande).
- **La demande, la décision et l'expiration sont des événements du log.** Une demande survit donc à un redémarrage de l'engine, et son échéance se déduit de l'état, comme les autres délais.

### Les environnements protégés

- **Dans un environnement protégé, tout changement venu de la CLI attend une approbation :** déploiement, rollback, variable ou secret.
- **Désactivé par défaut,** pour garder le déploiement en une commande. Le panel le recommande pour la prod dès qu'un autre environnement existe.
- **Seul le panel active ou lève la protection.**

## Le serveur MCP

- **`brume mcp` est un serveur MCP local, en stdio,** lancé par l'agent IA sur le laptop. Il utilise la clé de `brume login` et ne demande aucune autre configuration.
- **Un outil MCP par commande de la CLI,** avec les mêmes paramètres et les mêmes réponses JSON. Les deux appellent le même client, et la CI échoue si une commande n'a pas d'outil.
- **Les annotations MCP** (`readOnlyHint`, `destructiveHint`, `idempotentHint`) décrivent chaque outil. Elles aident l'agent IA, mais ne protègent rien : c'est l'engine qui décide.
- **Le contenu venu de l'extérieur est marqué comme non fiable dans les réponses :** lignes de logs, messages de commit, noms de branches. Ça aide aussi l'agent IA, sans le protéger. La barrière, ce sont les approbations.
- **Pas de MCP distant en 0.1.** Servi par `brume api`, il demanderait OAuth et une autre façon de lier un agent IA à l'admin. Voir [12-plus-tard.md](12-plus-tard.md).

## Le skill

- **Un skill au format Agent Skills (`SKILL.md`),** publié avec la doc. Il apprend à un agent de code à déployer sur Brume avec la CLI.
- **Il fixe les bonnes habitudes :** toujours `--json` et les flags explicites de `deploy`, attendre avec `--wait`, transmettre l'URL d'une approbation à l'humain au lieu de chercher un contournement, et passer les secrets par `brume secrets import` plutôt que de les lire.

## Une CLI écrite pour les agents IA

- **`--json` sur toutes les commandes,** avec un schéma stable et versionné. Le résultat va sur stdout, la progression sur stderr.
- **Jamais de question interactive hors d'un terminal.** Une valeur manquante est une erreur qui nomme le flag à ajouter.
- **Des codes de sortie stables et documentés :** succès, erreur d'usage, refus de la politique, approbation requise, échec du build ou du déploiement, serveur injoignable.
- **Chaque erreur a un code stable, un message, et la commande suivante à lancer.**
- **On peut relancer une commande sans risque.** Les créations se font par nom : relancer une commande qui a réussi ne crée pas de doublon.
- **Les commandes longues attendent par défaut,** avec un délai (`--timeout`). Hors d'un terminal, une commande sans fin, comme `brume logs --follow`, s'arrête au bout de ce délai.

## Le code

- **`brume-client` :** le client typé de l'API, la signature des requêtes et la config locale. Les commandes de la CLI et `brume mcp` s'en servent toutes les deux.
- **`brume-mcp` :** le serveur MCP, sur le SDK Rust officiel de MCP (`rmcp`).
- **Un seul binaire, `brume`.** Sur le serveur, il contient les services et la CLI. Pour le laptop (macOS, Linux), il est compilé sans les services, par une feature cargo : la CLI ne dépend ni de `bollard`, ni de `rusqlite`, ni de `duckdb`. `cargo deny` le vérifie. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#les-crates).
- **Signé et publié comme le reste :** minisign, dans la même release GitHub.
- **La CLI de secours ne change pas.** Sur l'hôte, en root, elle passe par `admin.sock`, sans l'API ni clé. Les commandes de récupération n'existent que là. Voir [04-panel-admin.md](04-panel-admin.md#2-une-passkey-à-la-connexion).

## Les risques

- **Un agent IA détourné par du contenu qu'il lit :** une ligne de log, un message de commit, une page web, une issue GitHub. C'est un nouveau scénario d'attaque : voir [01-modele-de-menace.md](01-modele-de-menace.md#scénarios-dattaque).
- **Déployer, c'est exécuter du code avec les secrets du service.** Un agent IA détourné qui peut déployer peut donc faire sortir ces secrets, sans jamais les révéler. C'est le même risque que l'accès en écriture à une branche déployée, et il est accepté pour la même raison. Les environnements protégés et l'egress fermé par défaut le réduisent. Voir [01-modele-de-menace.md](01-modele-de-menace.md#risques-acceptés-en-01).
- **Le fournisseur de l'agent IA voit ce que l'agent lit :** logs, configuration, noms des secrets, jamais leurs valeurs. À savoir avant de lui donner la CLI, comme pour Cloudflare et le trafic public.
