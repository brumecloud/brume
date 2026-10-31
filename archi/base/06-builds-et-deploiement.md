# Builds et déploiement

## Le flux

1. Un push sur une branche configurée déclenche un webhook GitHub.
2. Cloudflare, puis cloudflared, puis Traefik (IP de GitHub uniquement, limite de débit), puis `brume webhook`. En entrée directe, la requête arrive directement à Traefik : voir [03-reseau.md](03-reseau.md#traefik).
3. `brume webhook` vérifie la signature et appelle `enqueue_build(repo_id, ref)` sur l'engine.
4. L'engine vérifie que la ref correspond à la branche configurée, puis demande le build à l'agent.
5. L'agent lance le builder, qui clone le repo et construit l'image avec Railpack.
6. L'agent charge l'image dans dockerd. Elle ne part pas sur S3 : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#ce-qui-va-sur-s3). L'engine crée la révision, puis la déploie par digest.

## Les trois sources

- **Un push sur une branche configurée,** par la GitHub App : le flux ci-dessus.
- **Un dossier envoyé par la CLI** (`brume deploy`) : ni clone ni token GitHub. Le builder vérifie l'empreinte de l'archive, l'extrait, puis construit avec Railpack, avec le même sandbox, le même cache et les mêmes secrets de build. Voir [19-cli-et-agents-ia.md](19-cli-et-agents-ia.md#le-dossier-envoyé-par-la-cli).
- **Une image d'un registry :** aucun build. Le tag est résolu en digest au déploiement, voir [05-runtime.md](05-runtime.md#images).

Les trois finissent de la même façon : une révision qui pointe vers un digest, que l'agent déploie.

## Le webhook

- **Un process séparé, avec son propre listener,** jamais une route de l'API admin. Les raisons sont dans [02-architecture.md](02-architecture.md).
- **Signature HMAC `X-Hub-Signature-256`** vérifiée en temps constant, avant de parser le JSON.
- **Taille de body limitée.**
- **Anti-rejeu :** GitHub ne signe pas de timestamp, donc on ignore les doublons de `X-GitHub-Delivery`.
- **Le payload n'est qu'un déclencheur.** Le repo vient de la config Brume, et la ref doit correspondre à la branche configurée.
- **Uniquement les pushs sur les branches configurées.** Pas de previews de PR en 0.1. Le jour où elles arrivent : jamais de build avec des secrets pour une PR venant d'un fork.
- **Aucun réseau sortant.**

## L'accès à GitHub

- **Une GitHub App par installation de Brume, plutôt qu'un token personnel.** Elle est créée au premier lancement, depuis le panel, par le flux « manifest » de GitHub. Le panel envoie le navigateur de l'admin sur GitHub avec un manifeste : nom, URL du webhook de cette installation, permissions, événement `push`. L'admin valide, et GitHub renvoie le navigateur vers le panel avec un code. L'engine échange ce code contre l'identifiant de l'App, sa clé privée et le secret du webhook. Le retour passe par le navigateur : le flux fonctionne avec un panel joignable seulement par le tailnet.
- **Aucune App partagée par le projet Brume.** Elle imposerait une seule URL de webhook, donc un relais central, et une clé privée commune à toutes les installations.
- **En lecture seule :** permissions `contents: read` et `metadata: read`, et installée sur les seuls repos que l'admin choisit.
- **Un token par build, limité au repo construit,** en `contents: read`, qui expire après une heure. L'engine le génère avec la clé privée de l'App, et l'agent le transmet au builder. Une dépendance piégée qui le vole ne lit que ce repo, pendant une heure.
- **Le token ne doit jamais finir dans l'image.** Un clone via `https://x-access-token:TOKEN@github.com/...` écrit le token dans `.git/config`, et un `COPY . .` le met dans une couche de l'image. Donc : token passé par un credential helper ou un header, et `.git` exclu du contexte de build.
- **Le clone se fait dans le builder, jamais dans l'engine ni dans l'agent.** Git a eu des failles d'exécution de code au clonage d'un repo piégé, comme CVE-2024-32002.
- **`GIT_ALLOW_PROTOCOL=https:ssh`,** pas de sous-modules arbitraires.
- **`--` avant tout argument utilisateur** passé à `git`, pour qu'une ref commençant par `-` ne devienne pas une option.

## Le builder

- **BuildKit rootless, dans son propre container lancé par l'agent,** sans socket Docker.
- **Un sandbox affaibli, par nécessité.** Dans un container, BuildKit rootless demande seccomp, AppArmor et `systempaths` en `unconfined`. Selon la doc BuildKit, ce dernier peut permettre de lire et d'écrire des fichiers sensibles du noyau.
- **User namespaces non privilégiés requis.** Sur Ubuntu, `brume install` désactive `kernel.apparmor_restrict_unprivileged_userns`, pour tout l'hôte. Debian n'a rien à changer. À confirmer sur Ubuntu 26.04 et Debian 13.
- **Acceptable, car le code construit est de confiance,** dépendances comprises. Voir [01-modele-de-menace.md](01-modele-de-menace.md#périmètre). Conséquence à connaître : une dépendance piégée aurait de quoi tenter une évasion vers root, par exemple avec les failles runc de novembre 2025, qui se déclenchent depuis un Dockerfile.
- **Un engine compromis peut lui faire construire n'importe quel repo.** Le même chemin vers root s'ouvre alors : risque accepté, voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lengine--le-plan-de-contrôle).
- **Image BuildKit v0.25.2 minimum.** Elle embarque son propre runc, qui doit être patché à part de celui de Docker.
- **Avec `userns-remap`, à valider tôt :** rootlesskit mappe la plage `100000:65536` dans le container, au-delà des 65 536 UID que `userns-remap` donne par défaut. Il faudra sans doute une plage `dockremap` plus large.
- **Son propre réseau, avec accès Internet** (npm, pip), mais bloqué vers le tailnet, les plages privées, les endpoints de métadonnées et les réseaux d'environnement.
- **Limites CPU et RAM, un seul build à la fois.** Un build Next.js peut prendre toute la RAM : l'OOM killer tuerait alors les apps ou Brume.
- **Brume et Traefik protégés en priorité de l'OOM killer.**
- **Un cache séparé par environnement.** Sinon un build compromis peut empoisonner le cache des autres, et un build de staging celui de la prod. Voir [13-modele-de-donnees.md](13-modele-de-donnees.md#cache-de-build).
- **Railpack et son frontend BuildKit épinglés par digest.**

## Les secrets pendant le build

- **Passés par `--secret`,** jamais par `ARG` ou `ENV`, sinon ils restent dans les couches de l'image.
- **Masqués dans les logs de build.**
- Pour les sites statiques, les variables du build sont publiques : voir [05-runtime.md](05-runtime.md).

## Déploiement et rollback

- **Le builder ne touche jamais le runtime.** Il produit une image, l'agent la charge dans dockerd et la déploie par digest.
- **Sans coupure pour un service sans volume :** le nouveau container démarre à côté de l'ancien, et le trafic bascule après son healthcheck. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#le-déploiement).
- **Rollback :** redéployer le digest d'une révision précédente, sans rebuild. Ce qui revient avec elle et ce qui ne revient pas : voir [13-modele-de-donnees.md](13-modele-de-donnees.md#rollback).
