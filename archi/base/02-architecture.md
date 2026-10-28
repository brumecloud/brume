# Architecture

## Vue d'ensemble

```
Internet ─► Cloudflare ─► cloudflared ─► Traefik ─┬─► apps (un réseau interne par environnement)
                                                  ├─► nginx (sites statiques, lecture seule)
                                                  └─► brume webhook ──┐
                                                                      │ webhook.sock
Tailnet ──► tailscale serve ──► brume api ────────────────────────────┤ api.sock
                                                                      ▼
                                 brume logs ◄── flux de logs ── brume engine ──► dockerd
                                                                  │     ▲
                                           S3 (backups, audit) ◄──┘     └── image ◄── BuildKit rootless
```

Brume est un seul binaire signé, `brume`, lancé en plusieurs services systemd. Chaque service a son propre utilisateur système, ses propres droits et son propre durcissement. L'install reste simple, mais une faille dans un service ne donne pas les droits des autres.

## Les composants

| Composant | Rôle | Ce qu'il détient | S'il est compromis |
|---|---|---|---|
| `brume engine` | Seul propriétaire de l'état. Pilote Docker, déchiffre les secrets, lance les backups. | socket Docker, clé maître, base d'état SQLite | Tout. C'est le seul composant dans ce cas, d'où un code minimal. |
| `brume api` | Sert le panel et son API. | les sessions, un socket vers l'engine | Les actions du panel, sans les options dangereuses. Limite en 0.1 : peut changer la source d'un service, donc recevoir ses secrets (voir [12-plus-tard.md](12-plus-tard.md)). |
| `brume webhook` | Reçoit les webhooks GitHub. | le secret HMAC, un socket vers l'engine | Mettre en file le build d'un repo déjà configuré. |
| `brume logs` | Stocke et interroge les logs (DuckDB). | les logs | Lire les logs. Ni réseau, ni secrets, ni Docker. |
| `brume updater` | Installe les mises à jour de Brume. | le droit d'écrire les binaires | Équivaut à root, mais ne tourne que le temps d'une mise à jour et n'accepte que des binaires signés. |
| BuildKit rootless | Construit les images. | un accès Internet sortant, le token GitHub du build | Le build en cours et le cache de son projet. |
| cloudflared | Connecteur du tunnel. | le token du tunnel | Lire et modifier le trafic public. Ne joint que Traefik. |
| Traefik | Routage HTTP. | sa config, en lecture seule | Lire et modifier le trafic public. |
| nginx | Sert les sites statiques. | les fichiers, en lecture seule | Servir du contenu modifié jusqu'au redémarrage. |
| une app | Le code de l'utilisateur. | son réseau d'environnement, ses secrets | Son environnement, rien d'autre. |

## L'engine

- **Seul propriétaire de la base d'état.** Aucun autre service n'y accède : l'API et le webhook passent par l'engine.
- **Réconciliation en boucle,** comme Kubernetes : la base décrit l'état voulu, l'engine fait converger Docker vers cet état. Après un crash ou un reboot, il remet tout en place tout seul.
- **Détection des écarts :** un container, un réseau ou un port publié que Brume n'a pas créé (un `docker run -p` lancé à la main, par exemple) déclenche une alerte.
- **Sa propre politique :** il valide chaque demande contre `/etc/brume/policy.toml`, sans faire confiance à ses clients. Voir [05-runtime.md](05-runtime.md).
- **Aucune donnée non fiable.** Chaque parseur dans l'engine serait une porte vers root. Il ne fait donc ni clone git, ni extraction d'archive, ni parsing de webhook ou de logs. Il ne reçoit que des valeurs typées, déjà validées.
- **Aucun réseau entrant :** il n'écoute que sur des sockets unix.
- **Opérations critiques marquées dès la 0.1 :** révéler un secret, changer la source d'un service, supprimer, ajouter un admin. On pourra y brancher la ré-authentification plus tard sans refondre l'API.

## Les échanges entre services

- **Un socket unix par client :** `/run/brume/api.sock` appartient au groupe `brume-api`, `/run/brume/webhook.sock` au groupe `brume-webhook`. Les permissions du système de fichiers font l'autorisation.
- **Chaque socket a ses propres opérations.** Celui du webhook n'expose que `enqueue_build(repo_id, ref)`.
- **Un protocole typé et versionné,** avec des tailles de message limitées. Aucune opération générique du genre « exécuter une commande Docker ».
- **Les logs vont dans un seul sens :** l'engine lit les flux Docker et les pousse vers `brume logs` sans les interpréter. `brume logs` n'a jamais accès à Docker.
- **Le builder ne touche jamais le runtime :** il produit une image, l'engine la charge dans dockerd et la déploie par digest.

## Le webhook, séparé de l'API

- **C'est le seul composant de Brume joignable depuis Internet.** Dans l'API, une faille dans son parsing donnerait accès à toutes les actions admin.
- **Il n'écoute pas au même endroit.** Le webhook est joignable depuis le réseau d'ingress, l'API seulement par `tailscale serve`. Les fusionner obligerait l'API à écouter côté Internet.
- **Seul, il reste minuscule :** quelques centaines de lignes, une seule opération.

## L'ingress en deux étapes

- **cloudflared n'est que sur le réseau `ingress`,** avec Traefik. Un cloudflared compromis ne joint que Traefik.
- **Traefik est sur `ingress`** et sur les réseaux d'environnement.
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

Sans réseau sortant, du code exécuté dans l'API ou le webhook ne peut ni télécharger d'outils ni sortir de données directement. Et même une injection DuckDB réussie dans `brume logs` n'a ni réseau, ni secrets, ni Docker.

## Les binaires et les mises à jour

- **Les binaires appartiennent à root,** et `ProtectSystem=strict` empêche les services de les modifier.
- **Un updater dédié** vérifie la signature, remplace le binaire et redémarre les services.
- **Conséquence :** un service compromis (hors engine) ne peut pas s'installer durablement en remplaçant son binaire.
