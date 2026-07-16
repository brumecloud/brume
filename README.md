# Brume

Brume publie des plans Markdown ou MDX sous forme de documentation web lisible.
Le dépôt est un monorepo avec un CLI Rust, un serveur Rust, un serveur MCP et un renderer React/MDX embarqué dans le binaire du CLI.

## Structure

- `crates/brume-cli` contient la commande `brume`.
- `crates/brume-server` contient l’API, l’authentification GitHub et le serveur des plans.
- `crates/brume-mcp` expose les opérations de gestion aux agents via MCP stdio.
- `crates/brume-core` contient le contrat partagé des plans et des bundles.
- `crates/brume-api-client` contient le client HTTP utilisé par le CLI et le MCP.
- `renderer` contient le renderer React 19, MDX 3, Mermaid et Shiki.
- `migrations` contient le schéma PostgreSQL.
- `bruno` contient les tests de bout en bout du backend.

## Prérequis

- Rust 1.97 avec `rustfmt` et `clippy`.
- Bun 1.3.14 ou plus récent.
- Docker pour PostgreSQL local.
- Bruno CLI pour le test de bout en bout.

## Construire

Le script du CLI compile d’abord le renderer web et le worker Bun autonome, puis incorpore les trois artefacts dans le binaire Rust.

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

## Utiliser le CLI

Un dossier de plan peut contenir `index.mdx`, `index.md`, `plan.md` ou `README.md`.
Un fichier `brume.toml` optionnel choisit le titre, le slug, la visibilité et la page d’entrée.

```toml
[plan]
title = "Plan Brume"
entry = "index.mdx"
slug = "brume-v1"
visibility = "private"
```

Construire un plan sans serveur:

```bash
brume plan build ./mon-plan
```

Prévisualiser localement:

```bash
brume plan preview ./mon-plan
```

Se connecter puis publier:

```bash
brume login
brume plan deploy ./mon-plan
```

Les commandes de gestion disponibles sont `plan list`, `plan show`, `plan open`, `plan visibility`, `plan pin`, `plan unpin` et `plan delete`.
Les plans privés nécessitent la session GitHub du propriétaire.
Les plans non listés utilisent une URL secrète.
Les plans publics sont lisibles sans session.

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
Le Cron exécute `garbage-collect` chaque heure et supprime les plans non pin après quinze jours sans lecture significative.
Une lecture est enregistrée après cinq secondes continues avec la page visible et les écritures sont limitées à une fois par heure et par plan.

Variables du service web et du Cron:

```dotenv
BRUME_PUBLIC_URL=https://plan.brume.dev
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

Créer une GitHub OAuth App avec `https://plan.brume.dev/auth/github/callback` comme callback URL.
Le champ `BRUME_GITHUB_ALLOWED_IDS` est une liste d’identifiants GitHub numériques séparés par des virgules.
Une liste vide autorise tous les comptes GitHub.

Configurer `api.brume.dev` et `plan.brume.dev` comme domaines custom du même service web.
Le CLI utilise `api.brume.dev`, tandis que les plans publiés et l’authentification web utilisent `plan.brume.dev`.
Le domaine `brume.dev` reste réservé au site marketing.
Les deux sous-domaines utilisent les routes existantes du serveur sans réécriture.
Le healthcheck Railway est `/health` et vérifie également PostgreSQL.

## Rétention et remplacement

Chaque déploiement charge un nouveau bundle puis bascule la référence active dans une transaction PostgreSQL.
Une republication remplace atomiquement la version visible et réinitialise sa fenêtre de rétention.
L’ancien bundle est supprimé après la bascule.
Un plan pin n’expire pas.
La suppression manuelle demande un challenge court avant d’effacer le bundle et la ligne PostgreSQL.
