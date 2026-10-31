# Réseau

## Aucun port entrant

Ce document décrit les modes recommandés. L'entrée directe et l'administration sans Tailscale sont possibles, choisies sur l'hôte : voir [Les autres modes](#les-autres-modes).

- **Le trafic public passe par Cloudflare Tunnel.** `cloudflared` ouvre une connexion sortante vers Cloudflare.
- **L'administration passe par Tailscale,** SSH compris (Tailscale SSH).
- **UFW refuse tout en entrée,** sauf depuis le réseau `ingress` vers le port du webhook. Tailscale fonctionne quand même, grâce à ses connexions sortantes et à ses relais.
- **UFW est statique :** configuré une fois par `brume install`, jamais modifié ensuite. Brume n'a pas besoin de `CAP_NET_ADMIN`. Le flux est géré par Traefik et par les réseaux Docker.
- **Accès de secours :** la console de l'hébergeur.

## Cloudflare Tunnel

### Pourquoi le tunnel plutôt que le proxy classique

Avec le proxy classique, le DNS pointe sur l'IP du serveur et Cloudflare relaie vers le 443. N'autoriser que les IP Cloudflare ne suffit pas : n'importe quel client Cloudflare peut faire pointer sa propre zone sur ton IP et contourner tes règles. Il faudrait Authenticated Origin Pulls avec ton propre certificat, car le certificat global est partagé par tous les clients Cloudflare. Avec le tunnel, rien n'écoute, donc rien à contourner.

### Ce que ça simplifie

- **Ni Let's Encrypt, ni ACME, ni port 80 :** Cloudflare gère les certificats publics.
- **Traefik ne publie aucun port.** Il tourne sans root, avec un système de fichiers en lecture seule.
- **Une fuite de l'IP du serveur n'a pas d'importance.**

### Ce que ça coûte, à dire aux utilisateurs

- **Cloudflare déchiffre tout le trafic.** Il voit les mots de passe et les données des utilisateurs des apps. C'est une entreprise américaine, ce qui compte pour le RGPD.
- **Le compte Cloudflare contrôle le trafic et tout le DNS du domaine,** emails compris. La config du tunnel est locale, pour que le dashboard ne puisse pas changer ses règles : voir [La config du tunnel](#la-config-du-tunnel). Si une route privée s'ajoutait quand même, les appareils WARP de l'organisation joindraient, par `cloudflared`, tout ce que son container peut joindre : les règles nftables limitent ça à Traefik et au webhook. À protéger par une 2FA.
- **Le domaine doit être sur le DNS Cloudflare.**
- **Limites du plan gratuit :** 100 Mo par requête et 100 secondes de délai de réponse. Au-delà, l'upload ou la requête échoue.
- **Uniquement du HTTP.** Les bases passent par le tailnet.

### Les tokens

- **Token API Cloudflare :** utilisé par `brume install` pour créer le tunnel et le CNAME wildcard, puis oublié. Un domaine hors de la zone initiale demande un nouveau token, utilisé pour cette seule opération.
- **Secret du tunnel :** détenu par cloudflared. Quiconque le détient peut lancer un second connecteur du même tunnel depuis sa propre machine, et Cloudflare lui envoie une part du trafic public. C'est le secret le plus sensible de l'entrée.
- **Token en lecture seule** (tunnel et DNS) : détenu par l'engine, demandé à l'install. Il ne peut rien modifier. L'engine liste les connecteurs du tunnel et les enregistrements DNS de la zone, et alerte sur un connecteur ou un enregistrement inconnu.

### La config du tunnel

- **Gérée en local, jamais depuis le dashboard.** L'agent écrit la config de cloudflared : tout le trafic va à Traefik, qui répond 404 aux domaines inconnus. Une route ajoutée dans le dashboard ne doit pas pouvoir détourner le trafic.
- **À vérifier avant l'implémentation** dans la documentation de Cloudflare : qu'un tunnel configuré en local refuse bien les routes ajoutées depuis le dashboard. Voir les [décisions ouvertes](README.md#décisions-ouvertes).

### Réglages côté Cloudflare

- **HSTS et redirection vers HTTPS,** posés par `brume install` avec le token API : ces réglages ne changent plus ensuite.
- **Aucune règle WAF sur le webhook.** Elle listerait les IP de GitHub, qui changent, alors que le token API est oublié après l'install : personne ne la tiendrait à jour. Traefik fait ce filtrage avec une liste à jour : voir [Traefik](#traefik).

## Domaines

- **Par défaut, un wildcard sur le domaine de l'utilisateur :** `*.example.com` pointe vers le tunnel. Un nouveau service ne demande aucun changement DNS.
- **Un seul niveau de sous-domaine,** par exemple `api-prod.example.com`. Le certificat gratuit de Cloudflare couvre `*.example.com` mais pas `*.apps.example.com`.
- **Avant de créer le router d'un domaine personnalisé,** Brume vérifie qu'il est bien routé vers le tunnel.
- **Le nom du panel** (`brume.xxx.ts.net`) apparaît dans les logs publics de Certificate Transparency : le nom du tailnet devient public. Ce n'est pas grave, mais il faut le savoir.

## Traefik

- **Jamais le socket Docker.** L'agent écrit la config (file provider) dans un dossier monté en lecture seule.
- **Aucun port publié,** pas de root, système de fichiers en lecture seule.
- **N'écoute que sur son IP fixe du réseau `ingress`.** Par défaut, Traefik écoute sur toutes ses interfaces : une app pourrait l'appeler avec le `Host` d'une autre app et contourner Cloudflare (Access, WAF).
- **Un réseau `edge-<service>` par service exposé,** partagé avec Traefik seul. Traefik n'est sur aucun réseau d'environnement et ne joint jamais les bases.
- **Dashboard et API désactivés.** Un router par défaut renvoie une 404.
- **Headers transférés** (`X-Forwarded-For`, `CF-Connecting-IP`) : Traefik ne fait confiance qu'à ceux posés par cloudflared, jamais aux valeurs envoyées par le client. Sinon les apps voient des IP falsifiées. Les headers `CF-*` qui ne viennent pas de cloudflared sont supprimés.
- **Le chemin du webhook est réservé aux plages d'IP de GitHub,** dans tous les modes, avec une limite de débit. L'engine récupère ces plages (`api.github.com/meta`) et les tient à jour, et l'agent les écrit dans la config de Traefik. Avec Cloudflare, Traefik juge l'IP posée par cloudflared (`CF-Connecting-IP`). Ces plages sont partagées par tous les utilisateurs de GitHub : la vraie barrière reste la signature HMAC.
- **Règles générées par sérialisation,** à partir de noms validés. Voir l'injection dans [10-developpement.md](10-developpement.md).

## Réseaux Docker

| Réseau | Membres | Accès Internet |
|---|---|---|
| `tunnel` | cloudflared | oui, vers Cloudflare |
| `ingress` (interne) | cloudflared, Traefik | non |
| `static` (interne) | Traefik, Caddy | non |
| un par environnement (interne) | les services de l'environnement | non |
| `edge-<service>` (interne) | Traefik, un service exposé | non |
| `egress` de l'environnement | les services dont l'egress est activé | oui |
| `build` | BuildKit | oui |

- **Aucun container ne publie de port.** Docker contourne UFW : un port publié serait exposé malgré le firewall.
- **Pas d'accès Internet sortant par défaut.** L'egress s'active par service dans le panel, en rattachant le service au réseau `egress` de son environnement. La question est posée à la création du service, avec un message clair : la plupart des apps web appellent des API externes.
- **Docker 28.5.2 minimum :**
  - Docker 26 ferme la fuite DNS des réseaux internes, qui permettait d'exfiltrer des données (CVE-2024-29018).
  - Docker 28 bloque l'accès direct aux IP des containers depuis les voisins du réseau de l'hébergeur.
  - Docker 28.5.2 embarque runc 1.3.3, qui corrige les évasions de container de novembre 2025 (CVE-2025-31133, CVE-2025-52565, CVE-2025-52881).
- **Filtrage par domaine** (seulement `api.stripe.com`, par exemple) : plus tard, il demande un proxy de sortie.

## Règles nftables

Docker active le forwarding IP sur l'hôte. Sans règle, un container compromis peut joindre, en passant par le serveur, les machines du tailnet (laptop, NAS, autres serveurs), le réseau privé de l'hébergeur ou son endpoint de métadonnées.

- **Une table nftables propre à Brume** (`inet brume`), chargée par une unité systemd avant `docker.service`. Elle ne dépend pas des chaînes de Docker : un `drop` dans cette table est définitif, quelles que soient les règles que Docker ajoute.
- **Le trafic des réseaux Docker est refusé vers :**
  - le tailnet : `100.64.0.0/10` ;
  - les plages privées : `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, sauf les sous-réseaux Docker de Brume ;
  - le link-local, donc les endpoints de métadonnées : `169.254.0.0/16` ;
  - l'endpoint d'Azure, hors link-local : `168.63.129.16` ;
  - en IPv6 : `fc00::/7` (tailnet et métadonnées AWS compris) et `fe80::/10`.
- **Sous-réseaux Docker épinglés** dans `default-address-pools`, hors du réseau privé de l'hébergeur : les règles les connaissent à l'avance.
- **Tous les réseaux sont concernés,** `tunnel` et `build` compris.
- **L'agent refuse de démarrer un container si la table manque.** Les containers n'ont pas de politique de redémarrage Docker : c'est l'agent qui les démarre au boot, après cette vérification.
- **Test :** un `curl` depuis un container vers une machine du tailnet ou une IP privée doit échouer.

## Tailscale

- **Le serveur est tagué `tag:brume`.** La policy du tailnet doit lui interdire d'ouvrir des connexions vers les appareils : la policy par défaut d'un tailnet autorise tout. Les headers d'identité continuent de fonctionner, car ils identifient le client.
- **Les bases sont joignables depuis le laptop** en TCP sur le tailnet uniquement. Sans ce chemin simple, les utilisateurs publieraient le port 5432.
  - **Par `tailscale serve` en TCP :** tailscaled transfère un port du nœud vers le container de la base. Docker ne publie aucun port.
  - **Activé base par base** dans le panel. Chaque base exposée a son propre port sur le nœud.
  - **L'agent tient la config de `tailscale serve` à jour** par l'API locale de tailscaled, à chaque réconciliation : l'IP d'un container peut changer à son redémarrage. Il garde l'entrée HTTPS du panel.
  - **La policy du tailnet n'ouvre ces ports qu'aux identités de l'admin.**
- Le reste (identité, clé d'auth, Tailnet Lock) est dans [04-panel-admin.md](04-panel-admin.md).

## Les autres modes

Moins sûrs, choisis à l'install sur l'hôte, et signalés en permanence dans le panel. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration).

### Entrée directe, sans Cloudflare

- **Traefik publie les ports 80 et 443.** C'est le seul container qui publie des ports. UFW les ouvre en entrée.
- **Plus de cloudflared ni de réseau `tunnel`.** Le reste ne change pas : réseaux Docker, règles nftables, aucun autre port publié.
- **Certificats Let's Encrypt, un par domaine,** obtenus par Traefik (HTTP-01 ou TLS-ALPN-01). Aucun token DNS à garder sur le serveur. Les certificats vivent dans le seul dossier de Traefik qui n'est pas en lecture seule.
- **Le DNS pointe sur l'IP du serveur,** avec un wildcard. Après une restauration, l'admin repointe le DNS vers la nouvelle IP.
- **Le chemin du webhook reste réservé aux plages d'IP de GitHub** par Traefik, comme avec Cloudflare. Traefik juge alors l'IP de la connexion.
- **Traefik ne fait confiance à aucun header `X-Forwarded-*` reçu :** il pose lui-même l'IP du client. Il pose aussi HSTS, que Cloudflare posait.
- **L'IP du serveur est publique,** et la protection contre le DDoS n'est plus déléguée.

### Administration sans Tailscale

- **Tunnel SSH :** `brume api` reste sur `127.0.0.1`, et l'admin le joint par `ssh -L`. Aucun port d'administration n'est ouvert sur Internet.
- **Public :** Traefik sert le panel sur un domaine dédié, par le tunnel Cloudflare ou en entrée directe. `brume api` écoute alors sur le réseau `ingress`, comme le webhook.
- **SSH reste ouvert sur le port 22,** par clé uniquement, puisque Tailscale SSH n'est plus là.
- **Les bases se joignent depuis le laptop par un tunnel SSH,** base par base. Le mécanisme est laissé au builder, tant qu'aucun port Docker n'est publié sur une interface publique.
- **La connexion au panel** passe par un mot de passe et une passkey : voir [04-panel-admin.md](04-panel-admin.md#sans-tailscale).
