# Brume : architecture de base

Brume est une alternative open source et auto-hébergeable à Railway, Coolify et Dokploy, avec une UX et une DevXP plus simples.

Ce dossier décrit l'architecture et la sécurité de l'alpha 0.1. C'est la base des versions suivantes.

## Périmètre de l'alpha 0.1

- Un seul VPS, un seul propriétaire.
- Installation en une commande : `curl -sSL https://github.com/brumecloud/brume/releases/latest/download/install.sh | bash`.
- Déploiement de n'importe quel container, et de sites statiques avec rollback instantané.
- Templates avec bonnes pratiques intégrées : site statique, PostgreSQL, MongoDB, Redis.
- CI/CD automatique depuis GitHub, build avec Railpack.
- Backups, logs et secrets gérés depuis le panel.
- Panel d'administration joignable uniquement par Tailscale.
- Trafic public uniquement par Cloudflare Tunnel : aucun port entrant sur le serveur.

**Stack :** Rust (tokio, clap), React (TanStack Query, Router et DB, shadcn, Tailwind, compatible mobile), SQLite, DuckDB, Docker Engine, Traefik, nginx, Cloudflare Tunnel, Tailscale.

## Principes

1. **Le plus sûr possible.** Quand une option plus sûre existe, on la prend, tant qu'elle reste simple pour l'utilisateur.
2. **Un élément compromis ne fait pas tomber le reste.** Brume est single-tenant, mais chaque composant est cloisonné comme s'il allait être compromis.
3. **Deux composants de Brume équivalent à root : l'engine et l'updater.** L'engine a un code minimal, n'écoute sur aucun réseau et n'analyse aucune donnée non fiable. L'updater ne tourne que le temps d'une mise à jour et n'accepte que des binaires signés.
4. **Aucun port entrant.** Le public passe par Cloudflare Tunnel, l'administration par Tailscale.
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

## Décisions ouvertes

1. **Second facteur à la connexion :** mot de passe ou passkey. Recommandation : passkey. Dans les deux cas, c'est l'engine qui le vérifie. Voir [04-panel-admin.md](04-panel-admin.md).
2. **Redis : `vm.overcommit_memory=1` sur l'hôte.** Redis le recommande pour le fork de ses snapshots, mais le réglage vaut pour tout l'hôte. Voir [templates.md](templates.md#redis).
3. **Propositions par défaut, à confirmer :** pas de Headscale en 0.1, aucune télémétrie, vérification des mises à jour désactivable.
4. **Frontend :** implémentation du React Compiler, types partagés entre Rust et TypeScript, TanStack DB. Voir [front-architecture.md](front-architecture.md#décisions-ouvertes).
