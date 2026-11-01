# Hôte et installation

## Prérequis

- **Un VPS vierge.** Sur un serveur existant, les ports déjà publiés et les services déjà exposés resteraient ouverts.
- **Debian 12 ou 13, Ubuntu 24.04 ou 26.04,** en amd64 ou arm64. Les systemd credentials demandent au moins Debian 12 ou Ubuntu 24.04. La famille Red Hat viendra plus tard : voir [12-plus-tard.md](12-plus-tard.md).
- **Docker 28.5.2 minimum.** Voir [03-reseau.md](03-reseau.md#réseaux-docker).
- **Un bucket compatible S3,** avec versioning ou Object Lock, et des credentials en lecture et écriture, sans suppression ni droit sur le cycle de vie ou le versioning. Brume peut s'installer dans un dossier d'un bucket existant. `brume install` le vérifie. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#le-bucket).
- **En mode recommandé :** un domaine sur le DNS Cloudflare et un tailnet. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).
- **Avec Cloudflare, un token API** qui a ces droits sur la zone et son compte : Zone → Zone → Read, Zone → DNS → Edit, Zone → Zone Settings → Edit, Account → Cloudflare Tunnel → Edit.

## install.sh

- **Servi par brume.dev :** `curl -sSL https://brume.dev/install.sh | sudo bash -s -- --domain <zone>`. Ses arguments passent tels quels à `brume install`.
- **Minuscule, lisible en 30 secondes** avant d'être lancé : détecter l'architecture, télécharger l'archive depuis la release GitHub, vérifier son empreinte, l'installer dans `/usr/local/lib/brume` avec le lien `/usr/local/bin/brume`, lancer `brume install`.
- **Enveloppé dans une fonction appelée à la dernière ligne :** pas d'exécution partielle si la connexion coupe.
- **Version et empreintes SHA-256 des archives écrites dans le script.** Le script vient de brume.dev, les archives d'une release GitHub : remplacer une archive sur GitHub ne suffit pas, il faut aussi changer `install.sh` sur brume.dev.
- **Une archive par architecture,** amd64 et arm64 : le binaire `brume`, avec le panel intégré, et `libduckdb.so` à côté.
- **`libduckdb.so` vient dans l'archive de Brume,** plutôt que d'être téléchargée chez DuckDB par `install.sh` : la build officielle de la version de DuckDB que lie Brume, vérifiée par ses empreintes à la construction de la release. DuckDB n'est jamais compilé dans Brume : le binaire se lie dynamiquement à cette librairie, qu'il trouve à côté de lui. Sans elle, aucun service ne démarre, `brume telemetry` étant le seul à s'en servir.
- **Les alphas sont des pré-releases GitHub,** qui ne sont jamais servies comme `latest`.
- **minisign arrive avec l'updater.** D'ici là, l'empreinte épinglée dans `install.sh` est la seule vérification. Voir [Mises à jour](#mises-à-jour).
- **Limite : la première installation fait confiance à brume.dev.** Qui contrôle brume.dev peut servir un `install.sh` qui télécharge autre chose. La clé hors ligne ne protégera que les mises à jour.
- **Vérification manuelle, avec minisign :** la clé publique minisign sera publiée dans le README et sur brume.dev, pour vérifier `install.sh` et l'archive avant de les exécuter. Mais brume.dev est construit depuis le même dépôt GitHub : un compte GitHub compromis peut atteindre les deux. Un canal de vérification hors de GitHub reste à trouver : voir les [décisions ouvertes](README.md#décisions-ouvertes).

## `brume install`

Toute la config de l'hôte vit dans le binaire Rust, pas dans du bash : elle est testable, rejouable sans risque et réappliquée à chaque mise à jour.

- **Les modes se choisissent par les options,** sur l'hôte : `--admin tailscale` (défaut) ou `--admin ssh-tunnel`, et `--domain <zone>` pour les domaines publics par Cloudflare Tunnel. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).
- **Le token API Cloudflare** est lu dans `CLOUDFLARE_API_TOKEN`, que `sudo` ne garde qu'avec `--preserve-env=CLOUDFLARE_API_TOKEN`, ou demandé dans le terminal, sans écho. Il reste en mémoire : jamais écrit, ni dans un fichier, ni dans un log.
- **Rien ne change avant la confirmation.** L'install affiche les étapes qui restent à faire, puis demande de continuer, sauf avec `--yes`.
- **Rejouable :** chaque étape constate d'abord si elle est déjà faite. Après un échec, la même commande reprend là où elle s'est arrêtée, sans recréer ni tunnel, ni utilisateur, ni clé maître.
- **Les attentes sont bornées.** Un lien à cliquer est rappelé toutes les 2 minutes. Au bout de 15 minutes, l'install s'arrête et dit de relancer la même commande.

Ordre :

1. Vérifier l'hôte : root, systemd, Ubuntu 24.04 ou 26.04, Debian 12 ou 13, amd64 ou arm64, SSH sur le port 22, le seul que le pare-feu ouvre à SSH, aucun autre paquet Docker. Avec `--domain` : le token a tous ses droits sur la zone, et aucun autre enregistrement ne tient `*.<zone>`. Sinon, l'install s'arrête avant tout changement, en nommant le droit ou l'enregistrement en cause.
2. Installer la table nftables de Brume, avec l'unité systemd qui la charge avant Docker. Voir [03-reseau.md](03-reseau.md#règles-nftables).
3. Installer Docker depuis le dépôt apt officiel, signé par GPG (pas par un second `curl | sh`), après avoir écrit `daemon.json`. Épingler sa version majeure (préférences apt), et vérifier qu'il remappe les containers.
4. Avec Tailscale : l'installer depuis son dépôt apt, puis rejoindre le tailnet par `tailscale up --ssh`, sans clé d'auth : l'install affiche le lien de connexion et attend que l'admin le suive. Le login Tailscale du propriétaire du nœud, celui qui a cliqué, devient l'allowlist du panel. Voir [04-panel-admin.md](04-panel-admin.md#tailscale). Sans Tailscale : créer le compte de l'admin, avec un mot de passe.
5. Avec Tailscale : servir le panel par `tailscale serve`, en HTTPS, vers `127.0.0.1:7878`. Si le tailnet n'a pas activé HTTPS, l'install affiche le lien de Tailscale qui l'active, et attend.
6. Générer la paire age et afficher la clé privée une seule fois, ou importer les clés publiques de l'utilisateur. Plusieurs destinataires sont conseillés. L'admin déchiffre un code pour prouver qu'il a gardé sa clé.
7. Configurer le bucket S3, et son dossier facultatif, qui doit être vide, et le tester : écriture, lecture, `If-None-Match`, y compris à l'assemblage d'un envoi en plusieurs parties, puis l'échec d'une suppression, d'une règle de cycle de vie et d'une suspension du versioning. Un credential qui réussit l'une de ces trois opérations est refusé. Conseiller une règle du bucket qui nettoie les envois incomplets après 7 jours. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#le-bucket).
8. Créer les utilisateurs système, un par service, leurs dossiers, `/etc/brume/api.toml`, les unités systemd et la clé maître (systemd credentials). Déposer la clé maître sur S3, chiffrée pour les clés age, puis démarrer les services.
9. Avec Cloudflare, une fois l'engine démarré : créer le tunnel en config locale, ou retrouver celui de ce serveur, le CNAME wildcard `*.<zone>` proxifié vers le tunnel, et activer « Always Use HTTPS » sur la zone, puis oublier le token API. Confier le token du tunnel à l'engine, qui le scelle par la clé maître. Demander un second token, en lecture seule (tunnel et DNS), que l'engine garde pour détecter un connecteur ou un enregistrement inconnu. En entrée directe, l'admin fait pointer le DNS sur l'IP du serveur.
10. SSH. Avec Tailscale : Tailscale SSH est activé à l'étape 4. Vérifier qu'il fonctionne, et seulement ensuite : SSH par clé uniquement, port 22 fermé au public. Sans Tailscale : SSH par clé uniquement sur le port 22, seul port ouvert avec 80 et 443 en entrée directe.
11. Configurer `unattended-upgrades` : dépôts de la distribution, de Docker et de Tailscale, version majeure de Docker épinglée, reboot automatique. Voir [Mises à jour](#mises-à-jour).
12. Afficher l'URL du panel et un lien de connexion à usage unique, valable 15 minutes. `sudo brume admin login` en affiche un nouveau. Avec la passkey : l'URL d'enrôlement du second facteur, à usage unique et valable 15 minutes.

**Pas encore livré** (repoussé par le [jalon 2](../jalons/2026-11-01-jalon-2-vps.md#hors-du-jalon)) :

- **Les étapes 6 et 7, la paire age et le bucket S3.** Le bucket reste un dossier local, `/var/lib/brume/bucket`.
- **La clé maître dans les systemd credentials.** L'engine la crée dans son dossier à son premier démarrage, et elle ne part pas sur S3.
- **À l'étape 9, HSTS et le token en lecture seule.** HSTS est un réglage de toute la zone, qui toucherait les autres sites qu'elle sert.
- **Le reste de l'étape 10.** SSH reste ouvert sur le port 22, et `brume install` ne touche pas à la config de sshd.
- **L'étape 11.** La config d'`unattended-upgrades` de la distribution reste active, sans reboot automatique.
- **La passkey de l'étape 12.** La connexion se fait par le lien à usage unique.
- **Le tag `tag:brume`,** le mot de passe sans Tailscale, l'entrée directe et l'administration publique.

Au premier lancement du panel, l'admin crée la GitHub App de l'installation : voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#laccès-à-github).

## Docker

- **`daemon.json` :** `no-new-privileges: true`, `live-restore: true`, rotation des logs du driver par défaut et de la copie locale. Sans limite, les logs Docker remplissent le disque, et SQLite et Brume tombent.
- **`userns-remap: "default"`** dès l'install. Voir [05-runtime.md](05-runtime.md#isolation-des-containers).
- **`default-address-pools`** épinglés, hors du réseau privé de l'hébergeur, pour les règles nftables : `10.213.0.0/17`, découpé en `/24`.
- **`"bridge": "none"` :** pas de pont par défaut. Chaque container rejoint un réseau de Brume.
- **Ubuntu :** `kernel.apparmor_restrict_unprivileged_userns=0`, pour le builder. À confirmer sur Ubuntu 26.04 et Debian 13. Voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder).
- **L'API Docker n'écoute jamais en TCP** (port 2375). C'est la porte d'entrée classique des cryptominers.
- **Profils seccomp et AppArmor par défaut.**

## SSH

- **Authentification par clé uniquement.**
- **Avec Tailscale :** Tailscale SSH, et port 22 fermé au public. Pas encore livré : en attendant, le port 22 reste ouvert, et `brume install` refuse un SSH qui écoute ailleurs, que le pare-feu couperait.
- **Sans Tailscale :** port 22 ouvert, par clé uniquement. Voir [03-reseau.md](03-reseau.md#administration-sans-tailscale).
- **Secours :** la console de l'hébergeur.

## Services systemd

- **Un utilisateur dédié par service,** avec le durcissement décrit dans [02-architecture.md](02-architecture.md) : `brume-engine`, `brume-agent`, seul dans le groupe `docker`, `brume-api` et `brume-telemetry`. Ils partagent le groupe `brume`, celui des sockets.
- **Chemins :** le binaire et `libduckdb.so` dans `/usr/local/lib/brume`, avec le lien `/usr/local/bin/brume` ; les données dans `/var/lib/brume`, un dossier en `0700` par service ; les sockets dans `/run/brume` ; la config dans `/etc/brume`.
- **Les fichiers de Brume en `0600`,** propriété de leur service. Sauf ce que d'autres doivent lire, sans secret : `/etc/brume/api.toml`, et la config de Traefik, lue par son utilisateur remappé.
- **Priorité OOM :** Brume et Traefik protégés en priorité de l'OOM killer.
- **`systemd-analyze security` en CI :** plus tard.

## Mises à jour

- **Brume se met à jour tout seul.** L'engine lit `https://brume.dev/latest.json` à intervalle régulier. Ce fichier est le manifeste de la dernière release, signé hors ligne avec minisign : version du format, version de la release, date d'expiration, release de compaction ou non, empreinte et adresse du binaire sur GitHub Releases. Voir [10-developpement.md](10-developpement.md#signature--minisign).
- **Signature vérifiée avant toute lecture.** L'engine vérifie la signature sur les octets bruts, avant de parser le JSON, comme le webhook vérifie son HMAC : il n'analyse aucune donnée non signée venue d'Internet.
- **L'updater ne croit pas l'engine.** Lancé par l'engine quand une version plus récente existe, il refait toutes les vérifications : signature avec l'une des deux clés publiques intégrées au binaire déjà installé, version, compaction, empreinte du binaire téléchargé. Puis il remplace le binaire et redémarre les services.
- **Un manifeste expiré déclenche une alerte.** brume.dev, hébergé sur Vercel et déployé depuis le même dépôt GitHub, peut geler les mises à jour en servant toujours le même fichier, mais pas en faire installer une fausse ni une ancienne. La date d'expiration détecte le gel : le mainteneur re-signe donc `latest.json` régulièrement, même sans release.
- **Ce que la vérification apprend au projet Brume :** l'IP du serveur, et la version, l'OS et l'architecture qu'elle envoie, de quoi compter les installations par version. Ni domaine, ni compte, ni identifiant d'installation : le comptage reste approximatif. C'est la seule télémétrie envoyée au projet. Elle est annoncée à l'install et dans les docs, et les logs de brume.dev ont une rétention courte : une IP est une donnée personnelle au sens du RGPD.
- **La vérification se coupe sur l'hôte,** en root. Les mises à jour de Brume deviennent alors manuelles.
- **Aucun retour en arrière :** le manifeste de release signé porte la version, et l'updater refuse toute version inférieure ou égale à celle installée. Une ancienne version vulnérable reste signée : sans ce contrôle, un attaquant pourrait la réinstaller.
- **Une release qui compacte le log ne se saute pas.** Son manifeste signé le dit, et l'updater l'installe avant toute release plus récente. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#générations-et-compaction).
- **Composants** (Traefik, Caddy, cloudflared, BuildKit) : épinglés par digest et mis à jour à chaque release de Brume. `libduckdb.so` est dans l'archive de Brume : l'updater la remplace avec le binaire. Dans un an, le risque principal serait un composant non patché.
- **OS, Docker et Tailscale : `unattended-upgrades`,** configuré avec les dépôts de la distribution, de Docker et de Tailscale. Par défaut, il ne suit que ceux de la distribution.
- **Version majeure de Docker épinglée** par Brume (préférences apt), et relevée à chaque release de Brume, après test. Les versions correctives s'installent seules. Une version majeure peut casser un client de l'API Docker : Docker 29 a relevé la version minimale de son API.
- **Reboot automatique** quand une mise à jour le demande, le noyau notamment : sans reboot, un noyau corrigé ne s'applique pas. Il a lieu à 4 h, heure du serveur, modifiable sur l'hôte. Au démarrage, la table nftables se charge avant Docker, puis l'agent relance tous les containers. Les apps sont coupées le temps du reboot.
- **Mise à jour de dockerd :** avec `live-restore`, les containers continuent de tourner pendant une mise à jour corrective. Une version majeure les redémarre.
- **Mise à jour de tailscaled,** avec Tailscale : le panel et SSH sont coupés quelques secondes.
- **Alertes :** mise à jour en échec, reboot effectué, manifeste de mise à jour expiré. Voir [09-observabilite.md](09-observabilite.md#alertes).
- **Repoussé, à trancher avant la première release :** l'heure d'installation (proposition : la fenêtre de 4 h du reboot), le retour local au binaire précédent si le nouveau ne démarre pas, et le déploiement progressif par un pourcentage dans `latest.json`. Voir les [décisions ouvertes](README.md#décisions-ouvertes).

## Restauration et vérification

- **`brume restore`** remonte un serveur perdu depuis le bucket. Il reprend les étapes de `brume install`, sauf la création des clés, avec la clé age privée de l'admin.
- **`brume verify`** vérifie, depuis le poste de l'admin, que le bucket est restaurable.
- Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#restauration).

## Accès de secours

La console de l'hébergeur et la CLI `brume` en local, en root, via `admin.sock`, que l'engine réserve à root. Elle permet de gérer l'allowlist et de réinitialiser le second facteur si Tailscale ou le panel est inaccessible. Pour l'instant, elle donne un lien de connexion au panel : `sudo brume admin login`.
