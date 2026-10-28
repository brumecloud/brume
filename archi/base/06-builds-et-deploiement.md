# Builds et déploiement

## Le flux

1. Un push sur une branche configurée déclenche un webhook GitHub.
2. Cloudflare (règle WAF : IP GitHub uniquement), puis cloudflared, puis Traefik (limite de débit), puis `brume webhook`.
3. `brume webhook` vérifie la signature et appelle `enqueue_build(repo_id, ref)` sur l'engine.
4. L'engine vérifie que la ref correspond à la branche configurée, puis lance le build.
5. Le builder clone le repo et construit l'image avec Railpack.
6. L'engine charge l'image dans dockerd et la déploie par digest.

## Le webhook

- **Un process séparé, avec son propre listener,** jamais une route de l'API admin. Les raisons sont dans [02-architecture.md](02-architecture.md).
- **Signature HMAC `X-Hub-Signature-256`** vérifiée en temps constant, avant de parser le JSON.
- **Taille de body limitée.**
- **Anti-rejeu :** GitHub ne signe pas de timestamp, donc on ignore les doublons de `X-GitHub-Delivery`.
- **Le payload n'est qu'un déclencheur.** Le repo vient de la config Brume, et la ref doit correspondre à la branche configurée.
- **Uniquement les pushs sur les branches configurées.** Pas de previews de PR en 0.1. Le jour où elles arrivent : jamais de build avec des secrets pour une PR venant d'un fork.
- **Aucun réseau sortant.**

## L'accès à GitHub

- **Une GitHub App plutôt qu'un token personnel.** L'engine détient la clé privée de l'App et génère pour chaque build un token d'installation, qui expire après une heure.
- **Le token ne doit jamais finir dans l'image.** Un clone via `https://x-access-token:TOKEN@github.com/...` écrit le token dans `.git/config`, et un `COPY . .` le met dans une couche de l'image. Donc : token passé par un credential helper ou un header, et `.git` exclu du contexte de build.
- **Le clone se fait dans le builder, jamais dans l'engine.** Git a eu des failles d'exécution de code au clonage d'un repo piégé, comme CVE-2024-32002.
- **`GIT_ALLOW_PROTOCOL=https:ssh`,** pas de sous-modules arbitraires.
- **`--` avant tout argument utilisateur** passé à `git`, pour qu'une ref commençant par `-` ne devienne pas une option.

## Le builder

- **BuildKit rootless, dans son propre container,** sans socket Docker.
- **Un sandbox affaibli, par nécessité.** Dans un container, BuildKit rootless demande seccomp, AppArmor et `systempaths` en `unconfined`. Selon la doc BuildKit, ce dernier peut permettre de lire et d'écrire des fichiers sensibles du noyau.
- **User namespaces non privilégiés requis.** Sur Ubuntu 24.04, `brume install` désactive `kernel.apparmor_restrict_unprivileged_userns`, pour tout l'hôte. Debian 12 n'a rien à changer.
- **Acceptable, car le code construit est de confiance,** dépendances comprises. Voir [01-modele-de-menace.md](01-modele-de-menace.md#périmètre). Conséquence à connaître : une dépendance piégée aurait de quoi tenter une évasion vers root, par exemple avec les failles runc de novembre 2025, qui se déclenchent depuis un Dockerfile.
- **Image BuildKit v0.25.2 minimum.** Elle embarque son propre runc, qui doit être patché à part de celui de Docker.
- **Avec `userns-remap`, à valider tôt :** rootlesskit mappe la plage `100000:65536` dans le container, au-delà des 65 536 UID que `userns-remap` donne par défaut. Il faudra sans doute une plage `dockremap` plus large.
- **Son propre réseau, avec accès Internet** (npm, pip), mais bloqué vers le tailnet, les plages privées, les endpoints de métadonnées et les réseaux d'environnement.
- **Limites CPU et RAM, un seul build à la fois.** Un build Next.js peut prendre toute la RAM : l'OOM killer tuerait alors les apps ou Brume.
- **Brume et Traefik protégés en priorité de l'OOM killer.**
- **Un cache séparé par projet.** Sinon un build compromis peut empoisonner le cache des autres.
- **Railpack et son frontend BuildKit épinglés par digest.**

## Les secrets pendant le build

- **Passés par `--secret`,** jamais par `ARG` ou `ENV`, sinon ils restent dans les couches de l'image.
- **Masqués dans les logs de build.**
- Pour les sites statiques, les variables du build sont publiques : voir [05-runtime.md](05-runtime.md).

## Déploiement et rollback

- **Le builder ne touche jamais le runtime.** Il produit une image, l'engine la charge dans dockerd et la déploie par digest.
- **Rollback :** redéployer le digest précédent.
