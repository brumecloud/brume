# Hôte et installation

## Prérequis

- **Un VPS vierge.** Sur un serveur existant, les ports déjà publiés et les services déjà exposés resteraient ouverts.
- **Debian 12 ou Ubuntu 24.04,** pour les systemd credentials.
- **Docker 26 minimum.**

## install.sh

- **Minuscule, lisible en 30 secondes** avant d'être lancé : télécharger, vérifier, lancer `brume install`.
- **Enveloppé dans une fonction appelée à la dernière ligne :** pas d'exécution partielle si la connexion coupe.
- **Binaires hébergés sur GitHub Releases,** pas sur brume.dev. Si le script, le SHA et le binaire venaient tous de brume.dev, un attaquant qui contrôle le domaine changerait les trois.
- **Vérifie le SHA et la signature du binaire.** Versions épinglées.

## `brume install`

Toute la config de l'hôte vit dans le binaire Rust, pas dans du bash : elle est testable, rejouable sans risque et réappliquée à chaque mise à jour.

Ordre proposé :

1. Installer Docker depuis le dépôt apt officiel, signé par GPG (pas par un second `curl | sh`), et écrire `daemon.json`.
2. Rejoindre le tailnet avec une clé d'auth à usage unique, pré-autorisée et taguée `tag:brume`.
3. Demander le login Tailscale de l'admin pour l'allowlist, puis afficher l'URL d'enrôlement du second facteur (15 minutes).
4. Créer le tunnel Cloudflare et le CNAME wildcard avec un token API, puis oublier le token.
5. Générer la paire age et afficher la clé privée une seule fois, ou importer la clé publique de l'utilisateur.
6. Créer les utilisateurs système, les unités systemd et la clé maître (systemd credentials).
7. Appliquer les règles `DOCKER-USER`, avec l'unité systemd qui les réapplique après Docker.
8. Activer Tailscale SSH et vérifier qu'il fonctionne. Seulement ensuite : SSH par clé uniquement, port 22 fermé au public, UFW fermé en entrée.
9. Activer `unattended-upgrades` pour les patchs de sécurité de l'OS.

## Docker

- **`daemon.json` :** `no-new-privileges: true`, `live-restore: true`, rotation des logs. Par défaut le driver `json-file` grossit sans limite : disque plein, et SQLite et Brume tombent.
- **L'API Docker n'écoute jamais en TCP** (port 2375). C'est la porte d'entrée classique des cryptominers.
- **Profils seccomp et AppArmor par défaut.**

## SSH

- **Authentification par clé uniquement.**
- **Tailscale SSH,** et port 22 fermé au public.
- **Secours :** la console de l'hébergeur.

## Services systemd

- **Un utilisateur dédié par service,** avec le durcissement décrit dans [02-architecture.md](02-architecture.md).
- **Les fichiers de Brume en `0600`,** propriété de leur service.
- **Priorité OOM :** Brume et Traefik protégés en priorité de l'OOM killer.
- **`systemd-analyze security` en CI :** plus tard.

## Mises à jour

- **Brume :** l'updater vérifie la signature avec le matériel de vérification intégré au binaire déjà installé, remplace le binaire et redémarre les services. Le choix minisign ou Sigstore est dans [10-developpement.md](10-developpement.md).
- **Composants** (Traefik, nginx, cloudflared, BuildKit) : épinglés par digest et mis à jour à chaque release de Brume. Dans un an, le risque principal serait un composant non patché.
- **OS :** `unattended-upgrades`.
- **Proposition par défaut, à confirmer :** aucune télémétrie, vérification des mises à jour désactivable.

## Accès de secours

La console de l'hébergeur et la CLI `brume` en local, via un socket unix en `0600`. Elle permet de gérer l'allowlist et de réinitialiser le second facteur si Tailscale ou le panel est inaccessible.
