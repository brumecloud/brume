# Hôte et installation

## Prérequis

- **Un VPS vierge.** Sur un serveur existant, les ports déjà publiés et les services déjà exposés resteraient ouverts.
- **Debian 12 ou Ubuntu 24.04,** pour les systemd credentials.
- **Docker 28.5.2 minimum.** Voir [03-reseau.md](03-reseau.md#réseaux-docker).
- **Un bucket compatible S3,** avec versioning ou Object Lock, et des credentials en lecture et écriture, sans suppression. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#le-bucket).
- **En mode recommandé :** un domaine sur le DNS Cloudflare et un tailnet. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).

## install.sh

- **Minuscule, lisible en 30 secondes** avant d'être lancé : télécharger, vérifier, lancer `brume install`.
- **Enveloppé dans une fonction appelée à la dernière ligne :** pas d'exécution partielle si la connexion coupe.
- **Script et binaires servis par GitHub Releases :** `curl -sSL https://github.com/brumecloud/brume/releases/latest/download/install.sh | bash`. Le script décide quoi télécharger et s'il faut vérifier : il vient donc du même endroit que les binaires.
- **Vérifie la signature minisign du binaire.** Versions épinglées.
- **Limite : la première installation fait confiance à GitHub.** Un compte GitHub compromis peut modifier `install.sh` pour qu'il saute la vérification. La clé hors ligne ne protège que les mises à jour.
- **Vérification manuelle documentée :** la clé publique minisign est publiée dans le README et sur brume.dev, pour vérifier `install.sh` et le binaire avant de les exécuter. Pour tromper cette vérification, il faut compromettre les deux.

## `brume install`

Toute la config de l'hôte vit dans le binaire Rust, pas dans du bash : elle est testable, rejouable sans risque et réappliquée à chaque mise à jour.

Ordre :

1. Choisir les modes d'entrée et d'administration. Recommandés : Cloudflare Tunnel et Tailscale. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).
2. Installer la table nftables de Brume, avec l'unité systemd qui la charge avant Docker. Voir [03-reseau.md](03-reseau.md#règles-nftables).
3. Installer Docker depuis le dépôt apt officiel, signé par GPG (pas par un second `curl | sh`), et écrire `daemon.json`.
4. Avec Tailscale : rejoindre le tailnet avec une clé d'auth à usage unique, pré-autorisée et taguée `tag:brume`.
5. Créer le compte de l'admin : son login Tailscale pour l'allowlist, ou un mot de passe sans Tailscale.
6. Avec Cloudflare : créer le tunnel et le CNAME wildcard avec un token API, puis oublier le token. En entrée directe, l'admin fait pointer le DNS sur l'IP du serveur.
7. Générer la paire age et afficher la clé privée une seule fois, ou importer les clés publiques de l'utilisateur. Plusieurs destinataires sont conseillés. L'admin déchiffre un code pour prouver qu'il a gardé sa clé.
8. Configurer le bucket S3 et le tester : écriture, lecture, `If-None-Match`. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#le-bucket).
9. Créer les utilisateurs système (dont `brume-agent`), les unités systemd et la clé maître (systemd credentials). Déposer la clé maître sur S3, chiffrée pour les clés age, puis démarrer les services.
10. SSH. Avec Tailscale : activer Tailscale SSH et vérifier qu'il fonctionne. Seulement ensuite : SSH par clé uniquement, port 22 fermé au public, UFW fermé en entrée sauf depuis le réseau `ingress` vers le webhook. Sans Tailscale : SSH par clé uniquement sur le port 22, seul port ouvert avec 80 et 443 en entrée directe.
11. Configurer `unattended-upgrades` : dépôts de la distribution, de Docker et de Tailscale, version majeure de Docker épinglée, reboot automatique. Voir [Mises à jour](#mises-à-jour).
12. Afficher l'URL d'enrôlement du second facteur, à usage unique et valable 15 minutes.

## Docker

- **`daemon.json` :** `no-new-privileges: true`, `live-restore: true`, rotation des logs du driver par défaut et de la copie locale. Sans limite, les logs Docker remplissent le disque, et SQLite et Brume tombent.
- **`userns-remap: "default"`** dès l'install. Voir [05-runtime.md](05-runtime.md#isolation-des-containers).
- **`default-address-pools`** épinglés, hors du réseau privé de l'hébergeur, pour les règles nftables.
- **Ubuntu 24.04 :** `kernel.apparmor_restrict_unprivileged_userns=0`, pour le builder. Voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder).
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

- **Brume :** l'updater vérifie la signature minisign avec la clé publique intégrée au binaire déjà installé, remplace le binaire et redémarre les services. Voir [10-developpement.md](10-developpement.md#signature--minisign).
- **Aucun retour en arrière :** le manifeste de release signé porte la version, et l'updater refuse toute version inférieure ou égale à celle installée. Une ancienne version vulnérable reste signée : sans ce contrôle, un attaquant pourrait la réinstaller.
- **Une release qui compacte le log ne se saute pas.** Son manifeste signé le dit, et l'updater l'installe avant toute release plus récente. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#générations-et-compaction).
- **Composants** (Traefik, nginx, cloudflared, BuildKit) : épinglés par digest et mis à jour à chaque release de Brume. Dans un an, le risque principal serait un composant non patché.
- **OS, Docker et Tailscale : `unattended-upgrades`,** configuré avec les dépôts de la distribution, de Docker et de Tailscale. Par défaut, il ne suit que ceux de la distribution.
- **Version majeure de Docker épinglée** par Brume (préférences apt), et relevée à chaque release de Brume, après test. Les versions correctives s'installent seules. Une version majeure peut casser un client de l'API Docker : Docker 29 a relevé la version minimale de son API.
- **Reboot automatique** quand une mise à jour le demande, le noyau notamment : sans reboot, un noyau corrigé ne s'applique pas. Il a lieu à 4 h, heure du serveur, modifiable sur l'hôte. Au démarrage, la table nftables se charge avant Docker, puis l'agent relance tous les containers. Les apps sont coupées le temps du reboot.
- **Mise à jour de dockerd :** avec `live-restore`, les containers continuent de tourner pendant une mise à jour corrective. Une version majeure les redémarre.
- **Mise à jour de tailscaled,** avec Tailscale : le panel et SSH sont coupés quelques secondes.
- **Alertes :** mise à jour en échec, reboot effectué. Voir [09-observabilite.md](09-observabilite.md#alertes).
- **Proposition par défaut, à confirmer :** aucune télémétrie envoyée au projet Brume, vérification des mises à jour désactivable.

## Restauration et vérification

- **`brume restore`** remonte un serveur perdu depuis le bucket. Il reprend les étapes de `brume install`, sauf la création des clés, avec la clé age privée de l'admin.
- **`brume verify`** vérifie, depuis le poste de l'admin, que le bucket est restaurable.
- Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#restauration).

## Accès de secours

La console de l'hébergeur et la CLI `brume` en local, via un socket unix en `0600`. Elle permet de gérer l'allowlist et de réinitialiser le second facteur si Tailscale ou le panel est inaccessible.
