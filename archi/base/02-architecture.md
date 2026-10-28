# Architecture

## Vue d'ensemble

```
Internet ─► Cloudflare ─► cloudflared ─► Traefik ─┬─► apps exposées (un réseau edge par service)
                                                  ├─► nginx (sites statiques, lecture seule)
                                                  └─► brume webhook ──┐
                                                                      │ webhook.sock
Tailnet ──► tailscale serve ──► brume api ────────────────────────────┤ api.sock
                                                                      ▼
                                        S3 (backups, audit) ◄── brume engine ──► dockerd ──► brume logs
                                                                  │     ▲                        │
                                                                  │     └── image ◄── BuildKit   │
                                                                  └────────── lecture ───────────┘
```

Brume est un seul binaire signé, `brume`, lancé en plusieurs services systemd. Chaque service a son propre utilisateur système, ses propres droits et son propre durcissement. L'install reste simple, mais une faille dans un service ne donne pas les droits des autres.

## Les composants

| Composant | Rôle | Ce qu'il détient | S'il est compromis |
|---|---|---|---|
| `brume engine` | Seul propriétaire de l'état. Pilote Docker, déchiffre les secrets, lance les backups. | socket Docker, clé maître, base d'état SQLite | Tout. Avec l'updater, c'est le seul composant de Brume dans ce cas, d'où un code minimal. |
| `brume api` | Sert le panel et son API. | un socket vers l'engine | Rien sans session ouverte par l'admin : l'engine vérifie chaque session. Pendant une session, les actions du panel, sans les options dangereuses. |
| `brume webhook` | Reçoit les webhooks GitHub. | le secret HMAC, un socket vers l'engine | Mettre en file le build d'un repo déjà configuré. |
| `brume logs` | Reçoit les logs de dockerd, les stocke et les interroge (DuckDB). | les logs | Lire les logs. Ni réseau, ni secrets, ni Docker. |
| `brume updater` | Installe les mises à jour de Brume. | le droit d'écrire les binaires | Équivaut à root, mais ne tourne que le temps d'une mise à jour et n'accepte que des binaires signés. |
| BuildKit rootless | Construit les images. | un accès Internet sortant, le token GitHub et les secrets du build | Le build en cours, ses secrets et le token GitHub. Son sandbox est affaibli, car il n'exécute que du code de confiance : voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder). |
| cloudflared | Connecteur du tunnel. | le token du tunnel | Lire et modifier le trafic public. Ne joint que Traefik et le webhook. |
| Traefik | Routage HTTP. | sa config, en lecture seule | Lire et modifier le trafic public, joindre les services exposés. Pas les bases. |
| nginx | Sert les sites statiques. | les fichiers, en lecture seule | Servir du contenu modifié jusqu'au redémarrage. |
| une app | Le code de l'utilisateur. | son réseau d'environnement, ses secrets | Son environnement, rien d'autre. |

## L'engine

- **Seul propriétaire de la base d'état.** Aucun autre service n'y accède : l'API et le webhook passent par l'engine.
- **Réconciliation en boucle,** comme Kubernetes : la base décrit l'état voulu, l'engine fait converger Docker vers cet état. Après un crash ou un reboot, il remet tout en place tout seul.
- **Détection des écarts :** un container, un réseau ou un port publié que Brume n'a pas créé (un `docker run -p` lancé à la main, par exemple) déclenche une alerte.
- **Sa propre politique :** il construit la config de chaque container par liste blanche et valide chaque demande contre `/etc/brume/policy.toml`, sans faire confiance à ses clients. Voir [05-runtime.md](05-runtime.md#isolation-des-containers).
- **Aucune donnée non fiable.** Chaque parseur dans l'engine serait une porte vers root. Il ne fait donc ni clone git, ni extraction d'archive, ni parsing de webhook ou de logs. Il ne reçoit que des valeurs typées, déjà validées.
- **Aucun réseau entrant :** il n'écoute que sur des sockets unix.
- **Propriétaire des sessions.** L'engine vérifie le second facteur, émet la session et la valide à chaque demande de l'API. Une API compromise ne peut rien faire sans une session ouverte par l'admin. Voir [04-panel-admin.md](04-panel-admin.md#session).

## Les échanges entre services

- **Un socket unix par client :** `/run/brume/api.sock` appartient au groupe `brume-api`, `/run/brume/webhook.sock` au groupe `brume-webhook`. Les permissions du système de fichiers font l'autorisation.
- **Chaque socket a ses propres opérations.** Celui du webhook n'expose que `enqueue_build(repo_id, ref)`.
- **Un protocole typé et versionné,** avec des tailles de message limitées. Aucune opération générique du genre « exécuter une commande Docker ».
- **La collecte des logs ne passe pas par l'engine :** dockerd envoie les logs de chaque container directement à `brume logs`. Un flot de logs ne peut donc pas saturer l'engine. Voir [09-observabilite.md](09-observabilite.md#logs-des-apps).
- **La lecture passe par l'engine :** l'API demande les logs à l'engine, qui vérifie la session puis relaie la réponse de `brume logs` sans l'interpréter. L'API les diffuse au panel en SSE. `brume logs` n'a jamais accès à Docker.
- **Le builder ne touche jamais le runtime :** il produit une image, l'engine la charge dans dockerd et la déploie par digest.

## Le webhook, séparé de l'API

- **C'est le seul composant de Brume joignable depuis Internet.** Dans l'API, une faille dans son parsing donnerait accès à toutes les actions admin.
- **Il n'écoute pas au même endroit.** Le webhook est joignable depuis le réseau d'ingress, l'API seulement par `tailscale serve`. Les fusionner obligerait l'API à écouter côté Internet.
- **Seul, il reste minuscule :** quelques centaines de lignes, une seule opération.

## L'ingress en deux étapes

- **cloudflared est sur `tunnel`,** pour sortir vers Cloudflare, **et sur `ingress`,** avec Traefik. Compromis, il ne joint que Traefik et le webhook.
- **Traefik est sur `ingress`** et sur un réseau `edge-<service>` par service exposé. Jamais sur les réseaux d'environnement : il ne joint pas les bases.
- **Traefik n'écoute que sur son IP fixe du réseau `ingress`.** Sur les réseaux edge, il ne fait qu'ouvrir des connexions vers les apps. Une app ne peut donc pas l'appeler pour joindre une autre app sans passer par Cloudflare.
- **Traefik n'a jamais le socket Docker.** L'engine écrit sa config (file provider) dans un dossier que Traefik monte en lecture seule.
- **Monter le socket Docker en `:ro` ne protège rien :** un socket en lecture seule accepte quand même tous les appels de l'API Docker.

## Durcissement des services

Tous les services ont un utilisateur dédié, `NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateTmp=yes`, et `ReadWritePaths=` limité à leur dossier dans `/var/lib/brume`.

| Service | Réseau autorisé |
|---|---|
| `brume engine` | sockets unix, sortant vers S3, GitHub et les canaux de notification |
| `brume api` | localhost uniquement (`IPAddressDeny=any`, `IPAddressAllow=localhost`) |
| `brume webhook` | le réseau ingress uniquement |
| `brume logs` | aucun (`PrivateNetwork=yes`) |

`brume logs` a en plus des plafonds de mémoire et de CPU (`MemoryMax=`, `CPUQuota=`) : un flot de logs ne peut pas affamer le serveur.

Sans réseau sortant, du code exécuté dans l'API ou le webhook ne peut ni télécharger d'outils ni sortir de données directement. Et même une injection DuckDB réussie dans `brume logs` n'a ni réseau, ni secrets, ni Docker.

## Les binaires et les mises à jour

- **Les binaires appartiennent à root,** et `ProtectSystem=strict` empêche les services de les modifier.
- **Un updater dédié** vérifie la signature minisign, remplace le binaire et redémarre les services.
- **Conséquence :** un service compromis (hors engine) ne peut pas s'installer durablement en remplaçant son binaire.
