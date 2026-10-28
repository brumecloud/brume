# Logs, audit et alertes

## Logs des apps

- **La collecte :** dockerd envoie les logs de chaque container à `brume logs`, par le driver `fluentd` sur un socket unix. Le socket est en `0600` : seul root, donc dockerd, peut y écrire. L'engine fixe ce driver dans la config de chaque container.
- **Connexion asynchrone** (`fluentd-async`) : un `brume logs` arrêté n'empêche pas un container de démarrer.
- **Mode non bloquant, tampon borné** (`mode=non-blocking`, `max-buffer-size`) : sous un flot de logs, Docker perd des lignes au lieu de bloquer l'app ou de grossir en mémoire. L'engine ne voit jamais ce flot.
- **`docker logs` fonctionne toujours :** Docker garde une copie locale bornée des logs (dual logging).
- **La lecture :** l'API demande les logs à l'engine, qui vérifie la session et relaie la réponse de `brume logs` sans l'interpréter. L'API les diffuse au panel en SSE. Sans session, une API compromise ne lit aucun log.
- **Lecture plafonnée :** nombre de flux par session, nombre de lignes par réponse, durée d'une requête DuckDB.
- **`brume logs` est isolé :** DuckDB, `PrivateNetwork=yes`, plafonds de mémoire et de CPU, aucun accès à Docker ni aux secrets.
- **Rétention limitée :** les apps écrivent parfois des secrets dans leurs logs.
- **Pas de logs dans les backups.**
- **Affichage :** le contenu des logs est hostile, car les apps y recopient des requêtes venues de l'extérieur. Voir la section XSS de [04-panel-admin.md](04-panel-admin.md).
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
- **Événements :** nouveau déploiement, backup en échec, changement de destination des backups, mise à jour système en échec, reboot automatique, healthcheck en échec, disque presque plein, nouvelle identité sur le panel, écart détecté, tunnel Cloudflare coupé.
- **Les URL de notification sont fournies par l'utilisateur :** les requêtes bloquent le tailnet, les plages privées et les endpoints de métadonnées, sinon Brume pourrait servir à atteindre des machines internes.
