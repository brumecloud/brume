# Logs, audit et alertes

## Logs des apps

- **Le pipeline :** l'engine lit les flux Docker et les pousse, sans les interpréter, vers `brume logs` par un socket à sens unique.
- **`brume logs` est isolé :** DuckDB, `PrivateNetwork=yes`, aucun accès à Docker ni aux secrets.
- **Rétention limitée :** les apps écrivent parfois des secrets dans leurs logs.
- **Pas de logs dans les backups.**
- **Affichage :** le contenu des logs est hostile. Voir la section XSS de [04-panel-admin.md](04-panel-admin.md).
- **Disque :** rotation des logs Docker et surveillance de l'espace libre.

### DuckDB durci

Une injection pourrait sinon lire n'importe quel fichier, en écrire avec `COPY ... TO`, ou charger des extensions réseau. Donc :

- requêtes paramétrées ;
- `SET enable_external_access = false` à l'ouverture ;
- chargement automatique des extensions désactivé (`autoinstall_known_extensions`, `autoload_known_extensions`) ;
- puis `SET lock_configuration = true` pour figer ces réglages.

## Logs de Traefik

- **Rétention limitée :** ils contiennent des IP, donc des données personnelles au sens du RGPD.

## Journal d'audit

- **Écrit par l'engine,** puisque toutes les actions passent par lui. Chaque entrée porte l'identité Tailscale de l'admin.
- **Envoyé en continu vers S3 avec Object Lock.** Un attaquant peut arrêter le journal, mais pas effacer ce qui est déjà parti.

## Détection des écarts

Un container, un réseau ou un port publié que Brume n'a pas créé déclenche une alerte. Voir [02-architecture.md](02-architecture.md).

## Alertes

- **Canaux :** email, webhook, ntfy.
- **Événements :** nouveau déploiement, backup en échec, disque presque plein, nouvelle identité sur le panel, écart détecté, tunnel Cloudflare coupé.
- **Les URL de notification sont fournies par l'utilisateur :** les requêtes bloquent le tailnet et l'endpoint de métadonnées, sinon Brume pourrait servir à atteindre des machines internes.
