# Brume : architecture de base

Brume est une alternative auto-hébergeable à Railway, Coolify et Dokploy, avec une UX et une DevXP plus simples. Son code source est disponible sous licence FSL : voir [10-developpement.md](10-developpement.md#licence).

Ce dossier décrit l'architecture et la sécurité de l'alpha 0.1. C'est la base des versions suivantes.

## Périmètre de l'alpha 0.1

- Un seul VPS, un seul propriétaire.
- Debian 12 ou 13, Ubuntu 24.04 ou 26.04, en amd64 ou arm64.
- Installation en une commande : `curl -sSL https://github.com/brumecloud/brume/releases/latest/download/install.sh | bash`.
- Déploiement de n'importe quel container, et de sites statiques avec rollback instantané.
- Templates avec bonnes pratiques intégrées : site statique, PostgreSQL, MongoDB, Redis.
- CI/CD automatique depuis GitHub, build avec Railpack.
- Backups, logs et secrets gérés depuis le panel.
- Panel d'administration joignable par Tailscale (recommandé), par un tunnel SSH ou en public.
- Trafic public par Cloudflare Tunnel (recommandé : aucun port entrant sur le serveur), ou en direct sur les ports 80 et 443.
- Tout l'état de Brume sur S3, secrets chiffrés : un VPS perdu se remonte depuis le bucket.

**Stack :** Rust (tokio, clap), React (TanStack Query, Router et DB, coss ui, Tailwind, React Flow, compatible mobile), SQLite, DuckDB, S3, OpenTelemetry, Docker Engine, Traefik, Caddy, Cloudflare Tunnel, Tailscale.

## Principes

1. **Le plus sûr possible.** Quand une option plus sûre existe, on la prend, tant qu'elle reste simple pour l'utilisateur.
2. **Un élément compromis ne fait pas tomber le reste.** Brume est single-tenant, mais chaque composant est cloisonné comme s'il allait être compromis.
3. **Deux composants de Brume équivalent à root : l'agent et l'updater.** L'agent a un code minimal, n'écoute rien et n'analyse aucune donnée non fiable. L'updater ne tourne que le temps d'une mise à jour et n'accepte que des binaires signés. L'engine, qui tient l'état et les secrets, n'est pas équivalent à root sans une faille du noyau ou de runc : voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lengine--le-plan-de-contrôle).
4. **Aucun port entrant, par défaut.** Le public passe par Cloudflare Tunnel, l'administration par Tailscale. Les autres modes, moins sûrs, se choisissent sur l'hôte : voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).
5. **Le code du propriétaire est de confiance, l'extérieur ne l'est pas.** Est hostile tout ce qui vient de l'extérieur : requêtes vers les apps, payloads de webhook, sites ouverts par l'admin. Est hostile aussi tout ce qui en garde la trace : logs, en-têtes. Tout nom saisi dans le panel est validé par liste blanche.
6. **Les options dangereuses se décident sur l'hôte, jamais dans le panel.**
7. **Détecter autant que prévenir :** journal d'audit hors de portée, alertes, détection des écarts.

## Documents

| Fichier | Contenu |
|---|---|
| [01-modele-de-menace.md](01-modele-de-menace.md) | Ce qu'on protège, contre quoi, ce qui est hors périmètre et les risques acceptés en 0.1 |
| [02-architecture.md](02-architecture.md) | Les processus de Brume, leurs privilèges et leurs échanges |
| [03-reseau.md](03-reseau.md) | Cloudflare Tunnel, Traefik, firewall, réseaux Docker, domaines |
| [04-panel-admin.md](04-panel-admin.md) | Tailscale, authentification, session, CSRF, XSS |
| [05-runtime.md](05-runtime.md) | Isolation des containers, politique, images, bases, sites statiques |
| [templates.md](templates.md) | Les templates (site statique, PostgreSQL, MongoDB, Redis) et les pratiques qu'ils imposent |
| [06-builds-et-deploiement.md](06-builds-et-deploiement.md) | Webhook, accès GitHub, builder, déploiement par digest |
| [07-secrets-et-backups.md](07-secrets-et-backups.md) | Secrets de la plateforme et des apps, backups chiffrés |
| [08-hote-et-installation.md](08-hote-et-installation.md) | install.sh, `brume install`, Docker, SSH, systemd, mises à jour |
| [09-observabilite.md](09-observabilite.md) | Logs, journal d'audit, détection des écarts, alertes |
| [10-developpement.md](10-developpement.md) | Style et règles de code, dépendances, CI, releases |
| [front-architecture.md](front-architecture.md) | Stack, organisation du code, règles et outillage du panel |
| [11-comparatif.md](11-comparatif.md) | Comment font Portainer, Coolify, Dokploy, CapRover et Kamal |
| [12-plus-tard.md](12-plus-tard.md) | Ce qui est volontairement repoussé après la 0.1 |
| [13-modele-de-donnees.md](13-modele-de-donnees.md) | Projets, environnements, services, révisions, variables et secrets, rollback |
| [14-architecture-logicielle.md](14-architecture-logicielle.md) | Engine et agent, event sourcing, cœurs sans I/O, protocole entre l'engine et l'agent, crates |
| [15-etat-et-restauration.md](15-etat-et-restauration.md) | L'état sur S3, chiffrement et clés, envoi asynchrone, restauration, `brume verify` |
| [16-tests.md](16-tests.md) | Simulation déterministe, tests de conformité, tests sur une vraie VM |
| [17-telemetrie.md](17-telemetrie.md) | OpenTelemetry : traces, métriques et logs de Brume, sans secret |
| [18-formats-s3.md](18-formats-s3.md) | Noms et formats des objets sur S3, chiffrement des blobs, règles du lecteur |

## Décisions ouvertes

1. **Accord de contribution (CLA) :** en demander un aux contributeurs, pour garder la possibilité de changer de licence, ou non. Voir [10-developpement.md](10-developpement.md#licence).
2. **Mises à jour automatiques, avant la première release :** l'heure d'installation (proposition : la fenêtre de 4 h du reboot), le retour local au binaire précédent si le nouveau ne démarre pas, et le déploiement progressif par un pourcentage dans `latest.json`. Voir [08-hote-et-installation.md](08-hote-et-installation.md#mises-à-jour).
3. **Canal de vérification de la clé de signature hors de GitHub,** puisque brume.dev est déployé depuis le même dépôt. Voir [10-developpement.md](10-developpement.md#signature--minisign).
4. **Config locale du tunnel Cloudflare, avant d'écrire l'install :** vérifier dans la documentation de Cloudflare qu'elle empêche bien le dashboard d'ajouter des routes, et qu'un tel tunnel se crée par l'API. Voir [03-reseau.md](03-reseau.md#la-config-du-tunnel).
5. **Pas de Headscale en 0.1 :** proposition à confirmer, plus tard.
6. **Un domaine dédié aux apps,** séparé du domaine qui porte les emails : un compte Cloudflare volé ne pourrait plus détourner les MX. Conseil à l'install, ou rien. Plus tard. Voir [03-reseau.md](03-reseau.md#domaines).
7. **Frontend :** décodage des réponses de l'API, graphiques des métriques. Voir [front-architecture.md](front-architecture.md#décisions-ouvertes).
