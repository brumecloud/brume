# Brume : architecture de base

Brume est une alternative open source et auto-hébergeable à Railway, Coolify et Dokploy, avec une UX et une DevXP plus simples.

Ce dossier rassemble les décisions d'architecture et de sécurité prises pendant la revue de l'alpha 0.1. Elles servent de base aux versions suivantes.

## Périmètre de l'alpha 0.1

- Un seul VPS, un seul propriétaire.
- Installation en une commande : `curl -sSL https://brume.dev/install.sh | bash`.
- Déploiement de n'importe quel container, et de sites statiques avec rollback instantané.
- CI/CD automatique depuis GitHub, build avec Railpack.
- Backups, logs et secrets gérés depuis le panel.
- Panel d'administration joignable uniquement par Tailscale.
- Trafic public uniquement par Cloudflare Tunnel : aucun port entrant sur le serveur.

**Stack :** Rust (tokio, clap), React (TanStack Query, Router et DB, shadcn, Tailwind, compatible mobile), SQLite, DuckDB, Docker Engine, Traefik, nginx, Cloudflare Tunnel, Tailscale.

## Principes

1. **Le plus sûr possible.** Quand une option plus sûre existe, on la prend, tant qu'elle reste simple pour l'utilisateur.
2. **Un élément compromis ne fait pas tomber le reste.** Brume est single-tenant, mais chaque composant est cloisonné comme s'il allait être compromis.
3. **Un seul composant équivaut à root : l'engine.** Son code est minimal, il n'écoute sur aucun réseau et n'analyse aucune donnée non fiable.
4. **Aucun port entrant.** Le public passe par Cloudflare Tunnel, l'administration par Tailscale.
5. **Tout ce qui vient de l'extérieur est hostile :** payloads de webhook, repos git, archives, logs, noms saisis dans le panel.
6. **Les options dangereuses se décident sur l'hôte, jamais dans le panel.**
7. **Détecter autant que prévenir :** journal d'audit hors de portée, alertes, détection des écarts.

## Documents

| Fichier | Contenu |
|---|---|
| [01-modele-de-menace.md](01-modele-de-menace.md) | Ce qu'on protège, contre quoi, et ce qui est hors périmètre |
| [02-architecture.md](02-architecture.md) | Les processus de Brume, leurs privilèges et leurs échanges |
| [03-reseau.md](03-reseau.md) | Cloudflare Tunnel, Traefik, firewall, réseaux Docker, domaines |
| [04-panel-admin.md](04-panel-admin.md) | Tailscale, authentification, session, CSRF, XSS |
| [05-runtime.md](05-runtime.md) | Isolation des containers, politique, images, bases, sites statiques |
| [06-builds-et-deploiement.md](06-builds-et-deploiement.md) | Webhook, accès GitHub, builder, déploiement par digest |
| [07-secrets-et-backups.md](07-secrets-et-backups.md) | Secrets de la plateforme et des apps, backups chiffrés |
| [08-hote-et-installation.md](08-hote-et-installation.md) | install.sh, `brume install`, Docker, SSH, systemd, mises à jour |
| [09-observabilite.md](09-observabilite.md) | Logs, journal d'audit, détection des écarts, alertes |
| [10-developpement.md](10-developpement.md) | Règles de code, dépendances, CI, releases |
| [11-comparatif.md](11-comparatif.md) | Comment font Portainer, Coolify, Dokploy, CapRover et Kamal |
| [12-plus-tard.md](12-plus-tard.md) | Ce qui est volontairement repoussé après la 0.1 |

## Écart avec la spec Notion

La page Notion « Alpha 0.1 » prévoit « TLS only, port 443, Let's Encrypt ». C'est remplacé par Cloudflare Tunnel, obligatoire : plus aucun port entrant, plus de Let's Encrypt. Voir [03-reseau.md](03-reseau.md).

## Décisions ouvertes

1. **Second facteur à la connexion :** mot de passe ou passkey. Recommandation : passkey. Voir [04-panel-admin.md](04-panel-admin.md).
2. **Signature des mises à jour :** clé publique minisign intégrée au binaire, ou identité Sigstore du workflow de release. Voir [10-developpement.md](10-developpement.md).
3. **Propositions par défaut, à confirmer :** pas de Headscale en 0.1, aucune télémétrie, vérification des mises à jour désactivable.
