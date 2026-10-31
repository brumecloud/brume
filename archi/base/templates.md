# Templates

Brume propose quatre templates : site statique, PostgreSQL, MongoDB et Redis. Chacun applique des bonnes pratiques que le panel ne peut pas désactiver.

## Principe

- **Un template est une recette typée, compilée dans le binaire signé.** Aucun fichier YAML à modifier. Chaque template a une version : une nouvelle version arrive avec une release de Brume, et la réconciliation l'applique avec un court redémarrage du service.
- **Paramètres typés et bornés :** taille, mode Redis, accès par le tailnet, et les réglages listés pour chaque template. Chaque réglage a un nom, un type et des bornes. Brume le traduit dans la config qu'il génère : aucune valeur n'est passée telle quelle.
- **Aucun flag, variable d'environnement ou fichier de config libre.** Un réglage qui affaiblit la sécurité n'est proposé que s'il est listé ici, désactivé par défaut.
- **Une base ne reçoit ni les variables ni les secrets partagés** du projet ou de l'environnement. Un site statique reçoit les variables de build partagées, qui alimentent son build et non le container nginx. Voir [13-modele-de-donnees.md](13-modele-de-donnees.md#trois-niveaux--projet-environnement-service).
- **Config générée par sérialisation,** montée en lecture seule dans le container. Voir [10-developpement.md](10-developpement.md#injection-dans-ce-que-brume-génère).
- **Dernière version majeure uniquement** pour une nouvelle base. Elle ne change plus ensuite : les mises à jour majeures ne sont pas en 0.1, voir [12-plus-tard.md](12-plus-tard.md). Le panel signale une version majeure en fin de support. Les versions mineures suivent les releases de Brume.

## Ce que toute base reçoit

- **Image officielle, épinglée par digest.**
- **Lancée directement sous l'utilisateur de la base,** sans l'entrypoint root de l'image. Toutes les capabilities supprimées, système de fichiers en lecture seule, tmpfs pour les sockets et `/tmp`.
- **Réseau d'environnement uniquement.** Jamais de réseau edge, d'egress ni de domaine. L'accès depuis le laptop, par le tailnet ou par un tunnel SSH, s'active base par base : voir [03-reseau.md](03-reseau.md#tailscale).
- **Deux identifiants, générés par l'engine :**
  - un compte admin, gardé par l'engine pour les backups et la maintenance, révélable ;
  - un compte pour l'app, avec le minimum de droits.

  Un service lié reçoit `DATABASE_URL`, `MONGODB_URI` ou `REDIS_URL` avec le compte de l'app. Une injection dans l'app obtient les droits de l'app, pas ceux de la base. Ces URLs contiennent le mot de passe : ce sont des secrets, résolus au déploiement et jamais copiés dans une révision.
- **Une taille de mémoire choisie dans une liste fixe.** Le cache de la base en découle. Sinon, la base se dimensionne sur la RAM de l'hôte et l'OOM killer la tue.
- **Arrêt propre :** le bon signal d'arrêt, et 60 secondes avant `SIGKILL`. Par défaut, Docker tue le container au bout de 10 secondes, et la base doit se réparer au démarrage suivant.
- **Healthcheck dans le container.** Une base en échec déclenche une alerte.
- **Backups activés par défaut,** avec le test de restauration de [07-secrets-et-backups.md](07-secrets-et-backups.md#la-vérification). Le bucket S3 est obligatoire dès l'install.
- **Logs envoyés à `brume telemetry`,** sans jamais journaliser les requêtes.
- **Pas de TLS entre les apps et la base :** le trafic reste sur le bridge de l'hôte, ou dans WireGuard par le tailnet.

## PostgreSQL

- **Authentification `scram-sha-256` uniquement,** depuis le sous-réseau de l'environnement. `POSTGRES_HOST_AUTH_METHOD` n'est jamais posé.
- **Le rôle de l'app possède sa base,** sans être superuser ni pouvoir créer de rôle ou de base. Il ne peut donc ni lancer `COPY … PROGRAM`, ni utiliser un langage non sûr, ni `ALTER SYSTEM`. Il peut créer les extensions marquées `trusted`.
- **`allow_alter_system = off` :** même le superuser ne peut pas modifier la config durablement par SQL.
- **`log_statement = none` :** un `ALTER ROLE … PASSWORD` écrirait le mot de passe dans les logs.
- **`idle_in_transaction_session_timeout` posé :** une transaction oubliée ne garde pas ses verrous indéfiniment.
- **`shared_buffers` à 25 % de la limite mémoire.**
- **Arrêt par `SIGINT`,** l'arrêt rapide de Postgres.
- **Backups :** `pg_dump` par base, plus les rôles (`pg_dumpall --globals-only`).
- **Réglages :** `max_connections`, `work_mem`, `statement_timeout`, `idle_in_transaction_session_timeout`.

## MongoDB

- **Authentification activée,** en SCRAM-SHA-256. L'utilisateur de l'app a le rôle `readWrite` sur sa base uniquement.
- **Replica set à un nœud :** les transactions, les change streams et Prisma en ont besoin. Brume l'initialise au premier démarrage.
- **Keyfile du replica set,** générée par l'engine et écrite dans le volume de la base, en `0400`. MongoDB ne la lit que depuis un fichier : c'est la seule exception aux secrets en variables d'environnement.
- **Depuis le laptop, connexion avec `directConnection=true` :** le nom du membre du replica set n'existe que dans le réseau de l'environnement.
- **`security.javaScriptEnabled: false` :** aucun JavaScript côté serveur (`$where`, mapReduce). Une voie d'injection NoSQL en moins.
- **Cache WiredTiger fixé** par la formule de MongoDB, appliquée à la limite du container : 50 % de (limite − 1 Go), 256 Mo minimum.
- **Backups :** `mongodump --archive --oplog`.
- **Réglages :** JavaScript côté serveur, désactivé par défaut.

## Redis

- **Utilisateurs ACL,** dans un fichier généré par Brume et monté en lecture seule :
  - `default` désactivé ;
  - l'app : toutes les commandes sauf celles d'administration (`CONFIG`, `DEBUG`, `MODULE`, `ACL`, `SHUTDOWN`, `REPLICAOF`, `MONITOR`, `SAVE`, `BGSAVE`, `BGREWRITEAOF`, `SYNC`, `PSYNC`, `FAILOVER`, `MIGRATE`). `INFO` et `CLIENT` restent permis : des librairies comme BullMQ en ont besoin ;
  - un admin, pour Brume ;
  - un utilisateur limité à `PING`, pour le healthcheck.
- **Un mode, choisi à la création :**
  - **cache :** aucune persistance, `allkeys-lru` ;
  - **données :** AOF (`appendfsync everysec`) et snapshots RDB, `noeviction`. Une file comme BullMQ ou Sidekiq ne perd jamais un job à cause d'une éviction.
- **`maxmemory` à 75 % de la limite du container :** il reste de la place pour le fork des snapshots.
- **Les mises à jour comptent plus que la config.** La faille Lua d'octobre 2025 (CVE-2025-49844) n'exigeait qu'un utilisateur authentifié capable de lancer des scripts, et les files de jobs ont besoin des scripts.
- **Backups :** snapshot RDB en mode données, aucun en mode cache.
- **Réglages :** `timeout`, pour les connexions inactives.

## Site statique

Le container nginx, l'extraction des artefacts et le rollback sont décrits dans [05-runtime.md](05-runtime.md#sites-statiques). Le template ajoute :

- **Cache :** `Cache-Control: immutable` pendant un an pour les fichiers dont le nom contient une empreinte, `no-cache` pour le HTML. Un rollback reste instantané, même à travers le cache de Cloudflare.
- **Headers :** `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Content-Security-Policy: frame-ancestors 'self'` contre le clickjacking. HSTS vient de Cloudflare.
- **Requêtes :** `GET` et `HEAD` uniquement, aucun listing de dossier, aucune version du serveur dans les réponses.
- **Vraies 404 par défaut.** Le repli vers `index.html` des SPA est un réglage.
- **Fichiers `.map` non servis par défaut :** ils exposent le code source.
- **Taille totale et nombre de fichiers des artefacts limités,** vérifiés pendant l'extraction, contre les bombes d'archive.
- **Réglages :** repli SPA, fichiers `.map`, Content-Security-Policy complète (valeur validée, sans retour à la ligne).

## Vérifications au déploiement

Dans un container jetable sur le réseau de l'environnement, avec un code de sortie pour seul résultat :

- une connexion sans mot de passe est refusée ;
- le compte de l'app échoue sur une opération d'admin (`CREATE ROLE`, `createUser`, `CONFIG GET`) ;
- le healthcheck passe.

Un échec fait échouer le déploiement : la version précédente continue de tourner, ou la nouvelle base n'est pas marquée prête.
