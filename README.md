# Brume

Brume permet de partager un plan Markdown ou MDX, de publier un site HTML statique ou d’exposer temporairement un serveur local.

Le CLI couvre ces trois usages:

| Commande | Usage |
| --- | --- |
| `brume plan` | Prévisualiser, construire, publier et gérer un plan Markdown ou MDX. |
| `brume deploy` | Publier tel quel le contenu d’un dossier HTML statique. |
| `brume tunnel` | Donner une URL publique temporaire à un serveur HTTP local. |

## Démarrage rapide

Les publications, les tunnels et les commandes de gestion nécessitent une connexion.

```bash
brume login
```

La commande ouvre GitHub sur `auth.brume.dev`, puis conserve dans le trousseau système un access token d’une heure et un refresh token rotatif de 90 jours.
Le CLI renouvelle automatiquement l’access token.
La prévisualisation et la construction locale d’un plan fonctionnent sans connexion.

Pour afficher toutes les options disponibles:

```bash
brume --help
brume plan --help
brume deploy --help
brume tunnel --help
```

## 1. Plans Markdown ou MDX

La commande `brume plan` transforme un dossier de documents en site navigable.
Le dossier doit contenir une page d’entrée nommée `index.mdx`, `index.md`, `plan.md` ou `README.md`.

### Prévisualiser un plan

```bash
brume plan preview ./mon-plan
```

La prévisualisation démarre un serveur local et ouvre le plan dans le navigateur.
Le dossier courant est utilisé si aucun chemin n’est fourni.

```bash
brume plan preview
brume plan preview ./mon-plan --port 3000
brume plan preview ./mon-plan --no-open
```

Sans `--port`, Brume choisit un port disponible.
L’option `--no-open` empêche l’ouverture automatique du navigateur.

### Construire un plan localement

```bash
brume plan build ./mon-plan
```

Par défaut, le résultat est écrit dans `./mon-plan/.brume/dist`.
Utiliser `--destination` pour choisir un autre dossier.

```bash
brume plan build ./mon-plan --destination ./dist
```

### Publier un plan

```bash
brume plan deploy ./mon-plan
```

Un nouveau plan est privé par défaut.
La visibilité peut être `private`, `unlisted` ou `public`.

```bash
brume plan deploy ./mon-plan --slug architecture-v2
brume plan deploy ./mon-plan --visibility public
brume plan deploy ./mon-plan --visibility unlisted --pin
```

- `--slug` choisit la dernière partie de l’URL.
- `--visibility` choisit qui peut lire le plan.
- `--pin` empêche l’expiration automatique du plan.

Sans `--slug`, Brume utilise le slug défini dans `brume.toml`, puis le nom du dossier en dernier recours.
Une republication avec le même slug remplace atomiquement la version active.

### Configurer un plan

Un fichier `brume.toml` optionnel placé à la racine du plan évite de répéter les mêmes options.

```toml
[plan]
title = "Plan Brume"
entry = "index.mdx"
slug = "brume-v1"
visibility = "private"
```

Les options passées dans la commande remplacent les valeurs de `brume.toml`.

### Gérer les plans publiés

Les commandes suivantes utilisent le slug du plan:

```bash
brume plan list
brume plan show brume-v1
brume plan open brume-v1
brume plan visibility brume-v1 public
brume plan pin brume-v1
brume plan unpin brume-v1
brume plan delete brume-v1
```

`plan list` affiche notamment la visibilité, la dernière lecture et la date d’expiration.
Dans un terminal interactif, les dates sont raccourcies et les URL complètes des plans sont cliquables.
`plan delete` demande une confirmation avant de supprimer définitivement le plan et ses fichiers.

### Sortie JSON pour les agents

L'option globale `--output json` produit des données structurées sur la sortie standard et fonctionne avec toutes les commandes.
Elle peut être placée avant ou après les sous-commandes.

```bash
brume plan list --output json
brume plan show brume-v1 --output json
brume deploy ./dist --output json
```

Les commandes longues comme `plan preview` et `tunnel` produisent un objet JSON par événement, séparé par une nouvelle ligne.
Les diagnostics et les confirmations interactives restent sur la sortie d'erreur afin de ne pas corrompre la sortie JSON.

Les plans privés nécessitent la session GitHub du propriétaire.
Les plans non listés utilisent une URL secrète.
Les plans publics sont lisibles sans session.

## 2. Sites HTML statiques

La commande `brume deploy` publie directement le contenu d’un dossier sans passer par le renderer Markdown et sans modifier son HTML.
Le dossier doit contenir un fichier `index.html` à sa racine.

```bash
brume deploy ./dist
```

Le dossier courant est utilisé si aucun chemin n’est fourni.
Sans `--url`, le serveur génère un identifiant public aléatoire.

```bash
brume deploy ./dist --url mon-app
brume deploy ./dist --url mon-spa --spa
brume deploy ./dist --url documentation --pin
```

Avec `--url mon-app`, le déploiement est public à l’adresse `https://mon-app-<handle>.brume.dev/`.

- `--url` choisit le slug utilisé dans l’URL publique.
- `--spa` active le fallback vers `index.html` pour une application monopage.
- `--pin` empêche l’expiration automatique du déploiement.

L’option `--spa` sert `index.html` pour les routes GET inexistantes sans extension, tout en conservant une réponse 404 pour les assets inexistants.
Les chemins de fichiers exacts et les fichiers `index.html` des sous-dossiers sont servis normalement.
Chaque fichier est limité à 20 MiB, avec une limite de 100 MiB et 5 000 fichiers par déploiement.
Les liens et les chemins d’assets ne sont pas réécrits.
Le site est servi depuis la racine de son propre sous-domaine, donc les chemins absolus comme `/assets/app.js` fonctionnent directement.
Comme les plans, les déploiements non pin expirent après quinze jours sans lecture et une republication remplace atomiquement la version active.

## 3. Tunnel vers un serveur local

La commande `brume tunnel` rend accessible un serveur qui écoute sur `127.0.0.1`.

Démarrer d’abord le serveur local, puis indiquer son port et le slug public à Brume.

```bash
brume tunnel 3000 --url mon-app
```

`--url` est optionnel.
Sans cette option, le serveur génère un identifiant public aléatoire.

La commande affiche l’URL publique une fois la connexion établie:

```text
https://mon-app-<handle>.brume.dev
```

Le tunnel reste actif tant que la commande tourne.
Utiliser `Ctrl-C` pour l’arrêter.
Les requêtes HTTP, les corps en streaming et les WebSockets sont relayés vers `http://127.0.0.1:3000`.
Le chemin public est transmis tel quel au serveur local, sans préfixe ajouté ou retiré.
L'URL est publique et ne doit pas exposer un service local contenant des données sensibles.

## Règles communes pour les URLs

Les slugs passés à `--slug` contiennent entre 1 et 80 caractères.
Pour `--url`, le slug et le handle doivent tenir ensemble dans la limite DNS de 63 caractères du sous-domaine.
Ils acceptent uniquement les lettres ASCII minuscules, les chiffres et les tirets internes.

Exemples valides: `documentation`, `mon-app` et `api-v2`.

## Autres commandes

```bash
brume version
brume mcp config
```

`brume version` affiche la version du CLI, le SHA court, le titre et le corps du message du commit utilisé pour le construire.
`brume mcp config` affiche la configuration MCP à copier dans Codex.

## Renderer MDX sûr

Le renderer accepte Markdown, GFM, Mermaid et un ensemble fermé de composants MDX: `Callout`, `Card`, `CardGrid`, `CodeGroup`, `Decision`, `FileTree`, `Mermaid`, `Risk`, `Step`, `Steps`, `Tab` et `Tabs`.
Les imports, exports, expressions JavaScript et spread props MDX sont refusés.
Le serveur assainit une seconde fois le HTML avant de le stocker.

Les documents du dossier deviennent automatiquement des pages.
Les liens Markdown vers d’autres fichiers `.md` ou `.mdx` sont réécrits vers leur route publiée.
Les images PNG, JPEG, GIF et WebP sont copiées comme assets du bundle.

## MCP

Après `brume login`, la commande suivante affiche la configuration MCP à copier dans Codex:

```bash
brume mcp config
```

Le serveur MCP permet de lister les plans et leur dernière lecture, inspecter un plan, déployer un dossier, changer sa visibilité, le pin, l’unpin et le supprimer avec une confirmation en deux étapes.

## Structure du dépôt

- `crates/brume-cli` contient la commande `brume`.
- `crates/brume-server` contient l’API, l’authentification GitHub et les serveurs des plans et déploiements statiques.
- `crates/brume-mcp` expose les opérations de gestion aux agents via MCP stdio.
- `crates/brume-core` contient le contrat partagé des plans et des bundles.
- `crates/brume-api-client` contient le client HTTP utilisé par le CLI et le MCP.
- `renderer` contient le renderer React 19, MDX 3, Mermaid et Shiki.
- `migrations` contient le schéma PostgreSQL.
- `bruno` contient les tests de bout en bout du backend.

## Construire le projet

Le développement nécessite Rust 1.97 avec `rustfmt` et `clippy`, Bun 1.3.14 ou plus récent, Docker et Bruno CLI.

Chaque commit doit incrémenter la version SemVer du workspace dans `Cargo.toml` et synchroniser `Cargo.lock`.
Les scripts de build refusent une version invalide, inchangée ou inférieure à celle du commit parent.
Ils incorporent également le SHA complet, le titre et le corps du message du dernier commit dans les binaires CLI et serveur.

Le script du CLI compile le renderer web et le worker Bun autonome, puis incorpore les trois artefacts dans le binaire Rust.

```bash
./scripts/build-cli.sh --release
```

Le serveur incorpore uniquement le runtime web et la feuille de style.

```bash
./scripts/build-server.sh --release
```

Pour tout construire:

```bash
./scripts/build-all.sh --release
```

Le cross-build du CLI accepte aussi `--target <triple-rust>` pour les cibles macOS, Linux et Windows reconnues par `scripts/build-renderer.sh`.

## Développement backend local

Copier `.env.example` vers `.env`, puis démarrer PostgreSQL:

```bash
docker compose up -d postgres
```

Construire et démarrer le serveur:

```bash
./scripts/build-server.sh
set -a
source .env
set +a
target/debug/brume-server serve
```

Pour créer un token local sans GitHub OAuth:

```bash
target/debug/brume-server create-dev-token --github-id 1 --login paul
```

Cette commande est destinée au développement local et utilise la même base configurée que le serveur.

Le test backend complet démarre PostgreSQL, lance le serveur, publie la fixture via le CLI puis vérifie les routes de gestion et la suppression avec Bruno:

```bash
./scripts/e2e-bruno.sh
```

## Déployer sur Railway

Créer quatre ressources dans le même projet Railway:

1. Un service web lié à ce dépôt et configuré avec `railway.toml`.
2. Un service PostgreSQL.
3. Un Storage Bucket Railway.
4. Un service Cron lié au même dépôt, avec `/railway.gc.toml` comme chemin Config as Code.

Le service web exécute automatiquement les migrations au démarrage.
Le Cron exécute `garbage-collect` chaque heure et supprime les plans et déploiements non pin après quinze jours sans lecture.
Une lecture est enregistrée après cinq secondes continues avec la page visible et les écritures sont limitées à une fois par heure et par plan.
Pour un déploiement statique, une réponse HTML enregistre directement la lecture, au maximum une fois par heure.

Variables du service web et du Cron:

```dotenv
BRUME_API_PUBLIC_URL=https://api.brume.dev
BRUME_AUTH_PUBLIC_URL=https://auth.brume.dev
BRUME_PLAN_PUBLIC_URL=https://plan.brume.dev
BRUME_PUBLIC_DOMAIN=brume.dev
BRUME_DATABASE_URL=${{Postgres.DATABASE_URL}}
BRUME_STORAGE_BACKEND=s3
BRUME_GITHUB_CLIENT_ID=...
BRUME_GITHUB_CLIENT_SECRET=...
BRUME_GITHUB_ALLOWED_IDS=123456
```

Injecter aussi les credentials du Bucket dans les deux services.
Brume accepte les variables Railway natives `ENDPOINT`, `REGION`, `BUCKET`, `ACCESS_KEY_ID`, `SECRET_ACCESS_KEY`, ou les variantes AWS produites par `railway bucket credentials`.
La valeur `AWS_S3_URL_STYLE=virtual` correspond aux buckets Railway récents.
Pour un ancien bucket path-style, utiliser `AWS_S3_URL_STYLE=path`.

Créer une GitHub OAuth App avec `https://auth.brume.dev/auth/github/callback` comme callback URL.
Le champ `BRUME_GITHUB_ALLOWED_IDS` est une liste d’identifiants GitHub numériques séparés par des virgules.
Une liste vide autorise tous les comptes GitHub.

Configurer `api.brume.dev`, `auth.brume.dev`, `plan.brume.dev` et `*.brume.dev` vers le même service web.
Le CLI et le MCP utilisent `api.brume.dev`, GitHub OAuth utilise `auth.brume.dev`, et les plans utilisent `plan.brume.dev`.
Tous les autres sous-domaines sont réservés aux tunnels et aux déploiements statiques.
Les cookies d’authentification restent limités à `auth.brume.dev` ou `plan.brume.dev` et ne sont jamais partagés avec le wildcard.
Le service doit rester sur une seule réplique tant que le registre des tunnels n’est pas distribué.
Le domaine `brume.dev` reste réservé au site marketing.
Le serveur sélectionne le routeur à partir de l’en-tête `Host`, sans réécriture de chemin par le proxy.
Le healthcheck Railway est `/health` et vérifie également PostgreSQL.

## Rétention et remplacement

Chaque déploiement charge un nouveau bundle puis bascule la référence active dans une transaction PostgreSQL.
Une republication remplace atomiquement la version visible et réinitialise sa fenêtre de rétention.
L’ancien bundle est supprimé après la bascule.
Un plan pin n’expire pas.
La suppression manuelle demande un challenge court avant d’effacer le bundle et la ligne PostgreSQL.
