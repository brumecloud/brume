# Runtime : containers et sites statiques

## Isolation des containers

- **Un réseau interne par environnement,** sans accès Internet par défaut. Voir [03-reseau.md](03-reseau.md).
- **Aucun port publié.**
- **Par défaut :** `no-new-privileges`, capabilities supprimées, limites de mémoire, de CPU et de pids. Sans limites, un seul container peut faire tomber Brume.
- **Profils seccomp et AppArmor par défaut,** jamais `unconfined`.
- **Interdits par défaut :** `--privileged`, réseau host, montages de l'hôte, socket Docker.
- **Accès bloqué** au tailnet, à l'endpoint de métadonnées de l'hébergeur et à Brume. Brume n'écoute que sur des sockets unix et sur `127.0.0.1`.

## `policy.toml` : les options dangereuses

- **Les options interdites ne s'activent que dans `/etc/brume/policy.toml`,** modifiable en root via SSH, jamais depuis le panel.
- **L'engine applique cette politique à chaque demande,** quel que soit le client. Une API compromise ne peut pas la contourner.
- **C'est la réponse aux outils qui demandent le socket Docker :** Portainer, Watchtower, un runner CI.

## Images

- **Le tag est résolu en digest au déploiement,** et le digest est stocké. Un rollback redéploie exactement la même image, et un tag modifié sur le registry ne change pas la prod à l'insu de l'admin.
- **Les composants de Brume** (Traefik, nginx, cloudflared, BuildKit) sont épinglés par digest et mis à jour à chaque release de Brume.

## Secrets des apps

- **Variables d'environnement en 0.1.** Les secrets en fichiers sur tmpfs viendront plus tard.
- Déchiffrés par l'engine et injectés à la création du container. Voir [07-secrets-et-backups.md](07-secrets-et-backups.md).

## Bases de données

- **Templates (Postgres, Redis) :** toujours un mot de passe aléatoire fort, même sans port publié. Un Redis sans mot de passe reste joignable par n'importe quelle app compromise du même environnement.
- **Accès depuis le laptop :** en TCP sur le tailnet uniquement, jamais par un port publié.

## Volumes

- **Supprimer un service ne supprime pas son volume tout de suite.** L'engine applique quelques jours de rétention.

## Sites statiques

- **Servis par un container nginx géré par Brume,** avec un volume en lecture seule et une image nginx sans root.
- **Fichiers cachés refusés** (`.env`, `.git`) : beaucoup de builds les laissent dans le dossier de sortie.
- **Extraction des artefacts** dans un process sans privilège, jamais dans l'engine. Les chemins en `../` et les symlinks sont rejetés, sinon nginx pourrait servir des fichiers situés hors du site.
- **Rollback instantané :** chaque version est conservée. Une version peut être purgée définitivement, par exemple si un secret y a fuité.
- **Les variables lues par le build sont publiques.** `VITE_*`, `NEXT_PUBLIC_*` et tout ce que lit le build finissent dans le bundle JS servi à tout le monde. Brume avertit quand un secret est injecté dans le build d'un site statique.
