# Runtime : containers et sites statiques

## Isolation des containers

- **Un réseau interne par environnement,** sans accès Internet par défaut. Un service exposé rejoint aussi un réseau `edge-<service>`, avec Traefik seul. Voir [03-reseau.md](03-reseau.md).
- **Aucun port publié.**
- **`userns-remap` sur dockerd.** Root dans un container correspond à un UID sans privilège sur l'hôte : une évasion par une faille runc ou noyau ne donne pas root directement. Activé à l'install : l'activer sur un hôte existant oblige à retélécharger toutes les images et à changer le propriétaire de tous les volumes.
- **Par défaut :** `no-new-privileges`, capabilities supprimées, limites de mémoire, de CPU et de pids. Sans limites, un seul container peut faire tomber Brume.
- **Profils seccomp et AppArmor par défaut,** jamais `unconfined`. Seule exception : le builder, voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder).
- **Config construite par liste blanche.** L'agent fixe lui-même chaque champ de la config d'un container ; rien n'est recopié depuis une demande. Une liste d'interdits ne suffit pas : plusieurs options Docker équivalent à un montage de l'hôte ou au réseau d'un autre container.
- **Interdits hors `policy.toml`,** entre autres :
  - `--privileged`, réseau host, montages de l'hôte, socket Docker ;
  - volumes nommés avec des options de driver : `o=bind` en fait un montage de l'hôte ;
  - `--network container:…`, qui rejoint le réseau d'un autre container (celui de Traefik donne tous les réseaux edge) ;
  - `--volumes-from` ;
  - `--pid`, `--ipc`, `--uts` et `--cgroupns` partagés avec l'hôte ou un autre container, et `--userns=host` ;
  - `--cap-add`, `--device`, `--security-opt`, `--sysctl`, `--runtime`.
- **Accès bloqué** au tailnet, aux plages privées, aux endpoints de métadonnées et à Brume. Brume n'écoute que sur des sockets unix, sur `127.0.0.1` (l'API) et sur l'adresse de l'hôte dans le réseau `ingress` (le webhook, et l'API en administration publique), que seuls cloudflared et Traefik joignent.

## `policy.toml` : les options dangereuses

- **Les options interdites ne s'activent que dans `/etc/brume/policy.toml`,** modifiable en root via SSH, jamais depuis le panel.
- **L'agent applique cette politique à chaque container,** quel que soit l'état voulu qu'il reçoit. Ni une API ni un engine compromis ne peuvent la contourner. L'engine la vérifie aussi, pour afficher l'erreur dans le panel.
- **C'est la réponse aux outils qui demandent le socket Docker :** Portainer, Watchtower, un runner CI.
- **Le socket Docker, `--privileged` et le réseau host imposent aussi `--userns=host`.** Docker refuse les deux derniers tant que le container est remappé, et un root remappé ne peut pas ouvrir le socket. Avec `--userns=host`, root dans le container redevient root sur l'hôte.

## Images

- **Le tag est résolu en digest au déploiement,** et le digest est stocké. Un rollback redéploie exactement la même image, et un tag modifié sur le registry ne change pas la prod à l'insu de l'admin.
- **Les composants de Brume** (Traefik, Caddy, cloudflared, BuildKit) sont épinglés par digest et mis à jour à chaque release de Brume.

## Secrets des apps

- **Variables d'environnement en 0.1.** Les secrets en fichiers sur tmpfs viendront plus tard.
- Déchiffrés par l'engine, transmis à l'agent de la machine, et injectés à la création du container. Voir [07-secrets-et-backups.md](07-secrets-et-backups.md).

## Bases de données

- **Templates PostgreSQL, MongoDB et Redis :** identifiants forts générés par l'engine, compte de l'app aux droits minimaux, config en lecture seule. Voir [templates.md](templates.md).
- **Accès depuis le laptop :** en TCP sur le tailnet, par `tailscale serve`, ou par un tunnel SSH sans Tailscale. Jamais par un port publié. Activé base par base. Voir [03-reseau.md](03-reseau.md#tailscale).

## Volumes

- **Supprimer un service ne supprime pas son volume tout de suite.** Brume applique quelques jours de rétention.
- **Un rollback ne restaure jamais un volume.** Voir [13-modele-de-donnees.md](13-modele-de-donnees.md#rollback).

## Sites statiques

- **Servis par un container Caddy géré par Brume,** avec un volume en lecture seule, sans root, depuis l'image officielle épinglée par digest, sans module tiers. Cache, headers et limites des artefacts : voir [templates.md](templates.md#site-statique).
- **HTTPS automatique de Caddy coupé.** Le TLS se termine chez Cloudflare, ou dans Traefik en entrée directe. Sinon Caddy tenterait d'obtenir ses propres certificats.
- **L'API d'admin de Caddy reste sur `localhost`, dans son propre container.** Elle accepte n'importe quelle nouvelle config, mais aucun autre container ne partage le réseau de Caddy, puisque `policy.toml` l'interdit : seul un process de ce container peut la joindre. L'agent recharge la config en lançant `caddy reload` dans le container, sans coupure.
- **Chaque domaine sert un lien `current`** vers le dossier de sa révision, créé par Brume hors de toute archive. Un déploiement ou un rollback bascule ce lien d'un coup, sans toucher à la config de Caddy, qui ne change qu'à l'ajout ou au retrait d'un domaine.
- **Config en JSON, générée par sérialisation** et montée en lecture seule, jamais un Caddyfile construit par concaténation. Voir l'injection dans [10-developpement.md](10-developpement.md#injection-dans-ce-que-brume-génère).
- **Fichiers cachés refusés** (`.env`, `.git`) : beaucoup de builds les laissent dans le dossier de sortie.
- **Extraction des artefacts** dans un process sans privilège, jamais dans l'engine ni dans l'agent. Les chemins en `../` et les symlinks sont rejetés, sinon Caddy pourrait servir des fichiers situés hors du site.
- **Rollback instantané :** les dernières versions restent sur le disque. Une version peut être purgée définitivement, par exemple si un secret y a fuité. Voir [13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge).
- **Les variables lues par le build sont publiques.** `VITE_*`, `NEXT_PUBLIC_*` et tout ce que lit le build finissent dans le bundle JS servi à tout le monde. Brume avertit quand un secret est injecté dans le build d'un site statique.
