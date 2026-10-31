# Jalon 1 de la 0.1 : déployer des images publiques depuis le panel, en local

Premier jalon de l'alpha 0.1. Il pose les vraies fondations de Brume (Rust, engine et agent, event sourcing, cœurs sans I/O) et les branche sur le panel existant, sur le Mac de Paul, sans rien exposer à Internet.

Les décisions de [`archi/base/`](../base/README.md) s'appliquent, sauf là où ce document dit explicitement qu'un point est repoussé ou adapté au Mac.

## Intention

- **La 0.1 complète est un gros chantier** : installeur Linux, nftables, Tailscale, Cloudflare, updater signé, simulation déterministe, état chiffré sur S3, passkeys, templates et backups. Il faut une première ligne d'arrivée qui marche vraiment, de bout en bout.
- **Ce jalon n'est pas un prototype jetable.** Son code est la base de la 0.1 : chaque raccourci pris ici doit pouvoir être remplacé plus tard sans réécrire le reste.
- **Le panel plaît déjà.** Il tourne aujourd'hui sur des mocks MSW ; ce jalon le branche sur la vraie API.

## La ligne d'arrivée

Le jalon est fini quand Paul, sur son Mac, depuis le panel :

1. crée un projet, puis un environnement en plus de celui créé par défaut ;
2. ajoute à un environnement un service à partir d'une image publique (`nginx:1.29`, `redis:8`, `postgres:17`…) ;
3. lui donne des variables simples et des secrets, aux niveaux projet, environnement et service ;
4. déploie, et le container tourne réellement dans le Docker de sa machine, avec ces variables et ces secrets ;
5. voit les logs du container en direct dans le panel ;
6. voit la liste des événements qui ont produit cet état, et retrouve ce log dans un bucket local.

## Objectifs

### Fonctionnels

- **Projets :** création (vide, ou avec un premier service depuis une image), liste, tableau de bord existant.
- **Environnements :** `production` créé avec le projet ; création d'un environnement de plus, vide (nom validé comme un label DNS). Chaque environnement a son propre réseau Docker.
- **Services depuis une image publique :** création dans un environnement existant (depuis le canvas) et à la création d'un projet. Slug immuable, validé comme un label DNS.
- **Réglages d'un service :** image, commande de démarrage, port, chemin de healthcheck, limites CPU et mémoire, egress. Changements mis en attente, puis appliqués par « Déployer », comme le panel le fait déjà (`/api/changes`).
- **Variables et secrets** sur les trois niveaux, avec les règles de [13-modele-de-donnees.md](../base/13-modele-de-donnees.md#variables-et-secrets) :
  - les variables simples suivent la révision, avec la valeur résolue et son origine ;
  - les secrets sont résolus au déploiement, la révision ne garde que leur nom ;
  - les secrets partagés ne vont qu'aux services qui les référencent ;
  - un nom est soit une variable, soit un secret, à tous les niveaux ;
  - « révéler » un secret est une action explicite, journalisée (`SecretRevealed`).
- **Un écran de variables partagées** par projet et par environnement, qui n'existe pas encore dans le panel :
  - modifier une variable simple partagée suit le flux de [13-modele-de-donnees.md](../base/13-modele-de-donnees.md#modifier-une-valeur-partagée) : le panel liste les services touchés, environnement par environnement, production comprise, et la confirmation crée une révision par service, puis la déploie ;
  - modifier un secret partagé l'enregistre tout de suite ; les containers gardent l'ancienne valeur jusqu'à leur prochain déploiement, et le panel propose de redéployer les services qui le référencent ;
  - un service référence un secret partagé par son nom, dans son onglet Variables : une ligne secrète sans valeur, résolue au déploiement sur le service, puis l'environnement, puis le projet ;
  - supprimer un secret encore référencé par un déploiement actif est refusé, avec la liste des services concernés ;
  - les arêtes du canvas (`dependencies`) restent calculées par l'API depuis les secrets référencés.
- **Déploiement, redéploiement, rollback, suppression d'un service :**
  - le tag de l'image est résolu en digest avant de créer la révision ; la révision stocke le digest ;
  - un service sans volume se déploie sans coupure : le nouveau container démarre à côté de l'ancien, et l'alias DNS bascule après son healthcheck ;
  - un service avec un volume s'arrête puis redémarre. **Ses volumes sont ceux que déclare son image** (`VOLUME` de `postgres:17` ou `redis:8`) : l'agent les crée en volumes nommés gérés par Brume, qui survivent aux déploiements. Les données d'un Postgres ne sont donc pas remises à zéro à chaque déploiement. Supprimer le service garde ses volumes ;
  - un rollback redéploie le digest d'une ancienne révision, avec la valeur actuelle des secrets. **Après un rollback, la spec modifiable du service est celle de la révision en service** : les réglages et les variables du service affichés dans le panel reviennent avec elle, et le prochain « Déployer » part de là, sans réappliquer en silence ce que le rollback a défait. Les variables partagées restent à leur valeur actuelle ;
  - un healthcheck en échec fait échouer le déploiement, et l'ancien container continue de tourner.
- **Statut des services** (`online`, `deploying`, `crashed`, `stopped`) calculé depuis le dernier déploiement et l'état observé par l'agent.
- **Logs :** les logs d'exécution d'un environnement en direct (SSE) dans l'explorateur de logs existant ; les logs du déploiement (étapes, téléchargement de l'image) dans le panneau du déploiement.
- **Activité :** une vue du panel qui liste les événements du log, dans l'ordre, avec leur type, leur entité, leur acteur et leur date. Jamais de valeur de secret.

### Techniques

- **Rust, le workspace de crates de [14-architecture-logicielle.md](../base/14-architecture-logicielle.md#les-crates),** limité aux crates utiles à ce jalon : `brume-domain`, `brume-proto`, `brume-engine-core`, `brume-agent-core`, `brume-engine`, `brume-agent`, `brume-runtime`, `brume-api`, `brume-telemetry`, `brume`. `brume-webhook`, `brume-updater` et `brume-sim` viendront avec leurs jalons.
- **Un binaire, une sous-commande par service** (`brume engine`, `brume agent`, `brume api`, `brume telemetry`), plus `brume dev`, qui les lance tous en process séparés sur le Mac et les arrête ensemble.
- **Les services se parlent par des sockets unix,** un par client, dans le dossier d'exécution. La machine de l'agent est identifiée par la connexion (identifiants du pair), jamais par le message.
- **L'engine est le seul propriétaire de l'état :**
  - log d'événements dans SQLite (rusqlite, WAL, `synchronous=FULL`), métadonnées et chaîne de hash de [14-architecture-logicielle.md](../base/14-architecture-logicielle.md#event-sourcing) ;
  - `World` en mémoire, reconstruit au démarrage en rejouant le log ;
  - `decide(&World, Command)` et `evolve(&mut World, &Event)` purs, dans `brume-engine-core` ;
  - les opérations longues (déploiement, rollback) sont des machines à états explicites, reprises après un crash de l'engine, avec leurs délais ;
  - valeurs des variables dans la table `blobs`, valeurs des secrets dans `secret_versions`, chiffrées par la clé maître. Jamais dans un événement ;
  - **les changements en attente vivent hors du log,** dans une table à part, comme les sessions : un brouillon n'est pas une décision. C'est leur déploiement qui produit des événements. La valeur d'un secret en attente y est chiffrée par la clé maître ;
  - **aucun type d'événement n'est publié avant la première release :** le log et le bucket de ce jalon sont des données de dev, qu'on efface quand un type change (`brume dev --reset`). Chaque événement porte quand même sa `schema_version`, mais il n'y a ni upcaster ni log de référence à rejouer avant la release.
- **L'agent est le seul à tenir le socket Docker** (bollard). Il reçoit l'état voulu complet de sa machine, observe Docker, calcule `plan(voulu, observé)` et exécute. Il redémarre lui-même les containers arrêtés : pas de politique de redémarrage Docker.
- **Config des containers construite par liste blanche dans `brume-agent-core`,** avec un test qui vérifie la config complète d'un container d'app. Aucun port publié, `no-new-privileges`, limites de mémoire, de CPU et de pids, aucune option de `policy.toml`.
- **Réseaux Docker :** un réseau interne par environnement (sans Internet), et un réseau `egress` par environnement, rejoint par les services dont l'egress est activé. Le slug du service est son nom DNS sur le réseau de l'environnement.
- **Le bucket local :** un abonné de l'engine envoie le log en segments vers un stockage d'objets, avec les noms de [18-formats-s3.md](../base/18-formats-s3.md#noms-des-objets) (`log/<génération>/<seq>.seg`) et le contenu de segment qu'il décrit (une ligne d'en-tête, puis une ligne par événement, octets identiques à SQLite). Écriture sans jamais écraser un objet existant, point de reprise dans SQLite, envoi toujours asynchrone.
- **`brume telemetry`** reçoit les logs des containers et les stocke dans DuckDB, durci comme le prévoit [09-observabilite.md](../base/09-observabilite.md#duckdb-durci). L'engine vérifie la session, puis relaie la lecture sans l'interpréter ; l'API la diffuse en SSE.
- **Tests :** `decide`/`evolve` en « étant donné, quand, alors » ; `plan` en tests de propriété ; la config complète d'un container ; un test de bout en bout contre le vrai Docker (créer un projet, déployer `nginx`, lire un log, redéployer, rollback).
- **Le panel** appelle la vraie API en dev, par le proxy de Vite. Le mode démo de brume.dev et les tests navigateur gardent MSW. Les types de `src/api/` sont générés depuis les types Rust, comme décidé dans [front-architecture.md](../base/front-architecture.md#décisions-ouvertes).
- **`brume api` sert aussi le build du panel,** intégré au binaire, avec la CSP stricte de [04-panel-admin.md](../base/04-panel-admin.md#xss).

## Hors du jalon

Repoussés, pas abandonnés : chacun garde sa place dans l'architecture.

- Build depuis GitHub (GitHub App, webhook, BuildKit, Railpack), sites statiques, templates de bases (Postgres, MongoDB, Redis avec identifiants générés et vérifications).
- Domaines, Traefik, Caddy, Cloudflare Tunnel, Tailscale. Les apps ne sont joignables ni depuis Internet ni depuis le Mac : les logs suffisent à vérifier qu'un container tourne.
- Passkeys et allowlist : la connexion est simplifiée en local (voir plus bas).
- `brume install`, systemd, utilisateurs dédiés, nftables, `userns-remap`, `policy.toml` lu sur le disque.
- Vrai S3, chiffrement des segments, envoi des blobs, backups, `brume restore`, `brume verify`.
- Updater, `latest.json`, minisign.
- Simulation déterministe, tests de conformité, VM réelle.
- Détection des écarts, alertes, notifications, métriques des containers, télémétrie OTLP de Brume elle-même.
- Purge des révisions, rétention des artefacts et des volumes, volumes choisis par l'admin (seuls ceux que déclare l'image existent dans ce jalon).
- Copie d'un environnement : un nouvel environnement part vide.

## Anti-objectifs

Le jalon serait raté, même s'il marche, si :

- **l'architecture est contournée :** l'API ou l'engine parle à Docker, l'état voulu vit dans des tables SQL modifiées en place au lieu du log, un composant autre que l'engine écrit l'état ;
- **une valeur de secret apparaît** dans un événement, dans le bucket, dans les logs de Brume, dans une réponse d'API hors « révéler », ou dans le cache du panel ;
- **le panel change d'apparence** sans raison, ou ses conventions (couches, lint, React Compiler) sont contournées ;
- **la démo de brume.dev ou les tests existants du panel cassent.**

## Contraintes

- **Mac Apple Silicon, Docker Desktop 29.4** (dockerd dans une VM Linux arm64), Rust stable 1.92, nightly pour `cargo fmt`.
- **Docker Desktop n'est pas le VPS cible :** pas de systemd, pas de nftables, pas de `userns-remap`, et dockerd ne joint pas un socket unix du Mac. Ces écarts restent confinés à la coquille (`brume dev`, la collecte des logs), jamais dans les cœurs.
- **Règles de code** de [10-developpement.md](../base/10-developpement.md) et du `AGENTS.md` : style Google Rust, `#![forbid(unsafe_code)]`, `secrecy`, newtypes validés, aucune I/O ni `HashMap` dans les crates du cœur, `cargo +nightly fmt`, `cargo clippy -- -D warnings`.
- **Règles du panel** de [front-architecture.md](../base/front-architecture.md) : couches, lint, React Compiler, `vp fmt`, `tsc`, `vp test`, `knip`.
- **Protections CSRF gardées** malgré la connexion simplifiée, car elles ne coûtent rien : `X-Brume`, Fetch Metadata, `Origin`, aucun CORS, API sur `127.0.0.1` uniquement.
- **Livraison par lots.** Après chaque lot qui touche le panel, le travail s'arrête : Paul lance le panel et teste avant le lot suivant.

## Ce qui est décidé, et par qui

| Décision | Par |
|---|---|
| La ligne d'arrivée : images publiques, variables, secrets, projets, environnements, logs visibles dans le panel | Paul |
| Pas de Tailscale ni de Cloudflare, rien d'exposé à Internet | Paul |
| Le code est la vraie base de la 0.1 : Rust, engine et agent, event sourcing, cœurs sans I/O, panel sur la vraie API | Paul |
| Un log d'événements envoyé dans un bucket local (dossier ou S3 local) | Paul |
| Les logs dans le panel suffisent : les apps n'ont pas besoin d'être joignables depuis le Mac | Paul |
| Simulation déterministe, updater, install Linux : plus tard | Paul |
| Les trois niveaux de variables et de secrets dans ce jalon, avec un écran de variables partagées | Paul |
| Un nouvel environnement part vide | Paul |
| Un service référence un secret partagé par son nom, sans valeur, dans son onglet Variables | Claude, d'après 13-modele |
| Les volumes déclarés par l'image deviennent des volumes nommés de Brume ; ces services se déploient par arrêt puis redémarrage | Claude, d'après 14-architecture |
| Après un rollback, la spec modifiable du service est celle de la révision en service | Claude |
| Les changements en attente vivent dans une table hors du log, comme les sessions | Claude |
| Aucun type d'événement publié avant la première release : le log de dev s'efface | Claude, d'après 14-architecture (« un type publié ne change jamais ») |
| Le bucket local est un dossier, derrière une interface de stockage d'objets ; un vrai S3 sera une seconde implémentation | Claude |
| Segments du log en clair, avec la chaîne de hash, comme tranché pour les formats S3 ; seuls les secrets sont chiffrés | Claude, d'après la décision de Paul sur les formats S3 |
| Vue « Activité » des événements dans le panel | Claude (supposition non contestée) |
| Templates de bases hors du jalon ; « dépôt GitHub », « site statique » et « base de données » visibles mais désactivés (« Coming soon ») | Claude (supposition non contestée) |
| Domaines : le réglage reste affiché, désactivé | Claude |
| Connexion simplifiée : `brume dev` affiche un lien de connexion à usage unique ; l'engine émet la session et la vérifie à chaque demande, cookie `HttpOnly`, `SameSite=Strict` | Claude |
| Logs des containers : collectés par le moyen qui marche sur Docker Desktop, sans passer par l'engine ; le choix est confirmé par un essai au premier lot | Claude, accepté par Paul |
| Capabilities : toutes retirées, puis une courte liste rendue, celle dont les images publiques courantes ont besoin (nginx, postgres, redis) ; `NET_RAW`, `MKNOD`, `SYS_*` jamais rendues | Claude |
| Un « healthcheck » sans chemin = le container tourne encore après un court délai | Claude |

## Laissé au builder

- La sérialisation et le framing du protocole entre l'engine et l'agent, tant qu'ils passent par serde avec des tailles bornées.
- La façon de faire basculer l'alias DNS sur le réseau de l'environnement pendant un déploiement sans coupure.
- La façon de sonder un `healthcheckPath` HTTP depuis le réseau de l'environnement.
- Le mécanisme exact de collecte des logs sur Docker Desktop.
- L'emplacement des données locales (`BRUME_HOME`), les ports, le format du lien de connexion.
- `ts-rs` ou `specta` pour générer les types du panel.
- Le découpage en lots, tant que chaque lot qui touche le panel se termine par un arrêt pour les tests de Paul.
