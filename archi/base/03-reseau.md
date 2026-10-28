# Réseau

## Aucun port entrant

- **Le trafic public passe par Cloudflare Tunnel, obligatoire.** `cloudflared` ouvre une connexion sortante vers Cloudflare.
- **L'administration passe par Tailscale,** SSH compris (Tailscale SSH).
- **UFW refuse tout en entrée.** Tailscale fonctionne quand même, grâce à ses connexions sortantes et à ses relais.
- **UFW est statique :** configuré une fois par `brume install`, jamais modifié ensuite. Brume n'a pas besoin de `CAP_NET_ADMIN`. Le flux est géré par Traefik et par les réseaux Docker.
- **Accès de secours :** la console de l'hébergeur.

## Cloudflare Tunnel

### Pourquoi le tunnel plutôt que le proxy classique

Avec le proxy classique, le DNS pointe sur l'IP du serveur et Cloudflare relaie vers le 443. N'autoriser que les IP Cloudflare ne suffit pas : n'importe quel client Cloudflare peut faire pointer sa propre zone sur ton IP et contourner tes règles. Il faudrait Authenticated Origin Pulls avec ton propre certificat, car le certificat global est partagé par tous les clients Cloudflare. Avec le tunnel, rien n'écoute, donc rien à contourner.

### Ce que ça simplifie

- **Plus de Let's Encrypt, d'ACME ni de port 80 :** Cloudflare gère les certificats publics.
- **Traefik ne publie aucun port.** Il tourne sans root, avec un système de fichiers en lecture seule.
- **Une fuite de l'IP du serveur n'a pas d'importance.**

### Ce que ça coûte, à dire aux utilisateurs

- **Cloudflare déchiffre tout le trafic.** Il voit les mots de passe et les données des utilisateurs des apps. C'est une entreprise américaine, ce qui compte pour le RGPD.
- **Le domaine doit être sur le DNS Cloudflare.**
- **Limites du plan gratuit :** 100 Mo par requête et 100 secondes de délai de réponse. Au-delà, l'upload ou la requête échoue.
- **Uniquement du HTTP.** Les bases passent par le tailnet.

### Les tokens

- **Token API Cloudflare :** utilisé par `brume install` pour créer le tunnel et le CNAME wildcard, puis oublié. Un domaine hors de la zone initiale demande un nouveau token, utilisé pour cette seule opération.
- **Token du tunnel :** le seul stocké sur le serveur, détenu par cloudflared.

### Réglages côté Cloudflare

- HSTS et redirection vers HTTPS activés.
- Une règle WAF réserve le chemin du webhook aux plages d'IP de GitHub, en plus de la signature HMAC.

## Domaines

- **Par défaut, un wildcard sur le domaine de l'utilisateur :** `*.example.com` pointe vers le tunnel. Un nouveau service n'a besoin que d'un router Traefik.
- **Un seul niveau de sous-domaine,** par exemple `api-prod.example.com`. Le certificat gratuit de Cloudflare couvre `*.example.com` mais pas `*.apps.example.com`.
- **Avant de créer le router d'un domaine personnalisé,** Brume vérifie qu'il est bien routé vers le tunnel.
- **Le nom du panel** (`brume.xxx.ts.net`) apparaît dans les logs publics de Certificate Transparency : le nom du tailnet devient public. Ce n'est pas grave, mais il faut le savoir.

## Traefik

- **Jamais le socket Docker.** L'engine écrit la config (file provider) dans un dossier monté en lecture seule.
- **Aucun port publié,** pas de root, système de fichiers en lecture seule.
- **Dashboard et API désactivés.** Un router par défaut renvoie une 404.
- **Headers transférés** (`X-Forwarded-For`, `CF-Connecting-IP`) : Traefik ne fait confiance qu'à ceux posés par cloudflared, jamais aux valeurs envoyées par le client. Sinon les apps voient des IP falsifiées.
- **Limite de débit** sur la route du webhook.
- **Règles générées par sérialisation,** à partir de noms validés. Voir l'injection dans [10-developpement.md](10-developpement.md).

## Réseaux Docker

| Réseau | Membres | Accès Internet |
|---|---|---|
| `tunnel` | cloudflared | oui, vers Cloudflare |
| `ingress` (interne) | cloudflared, Traefik | non |
| `static` (interne) | Traefik, nginx | non |
| un par environnement (interne) | les services de l'environnement, Traefik | non |
| `egress` de l'environnement | les services dont l'egress est activé | oui |
| `build` | BuildKit | oui |

- **Aucun container ne publie de port.** Docker contourne UFW : un port publié serait exposé malgré le firewall.
- **Pas d'accès Internet sortant par défaut.** L'egress s'active par service dans le panel, en rattachant le service au réseau `egress` de son environnement. La question est posée à la création du service, avec un message clair : la plupart des apps web appellent des API externes.
- **Docker 26 minimum.** Avant, les réseaux internes laissaient passer les requêtes DNS externes, ce qui permettait d'exfiltrer des données (CVE-2024-29018).
- **Filtrage par domaine** (seulement `api.stripe.com`, par exemple) : plus tard, il demande un proxy de sortie.

## Règles `DOCKER-USER`

Docker active le forwarding IP sur l'hôte. Sans règle, un container compromis peut joindre les machines du tailnet (laptop, NAS, autres serveurs) en passant par le serveur.

- **Bloquer le trafic des réseaux Docker** vers le tailnet (`100.64.0.0/10`) et vers l'endpoint de métadonnées de l'hébergeur (`169.254.169.254`).
- **Écrire dans la chaîne `DOCKER-USER`,** la seule que Docker ne réécrit pas. Docker insère ses propres règles en tête de `FORWARD`, avant celles d'UFW.
- **IPv6 :** si IPv6 est activé sur les réseaux Docker, dupliquer les règles en `ip6tables` avec `fd7a:115c:a1e0::/48`.
- **Persistance :** ces règles disparaissent au reboot. Une unité systemd les réapplique après le démarrage de Docker.
- **Test :** un `curl` vers une machine du tailnet depuis un container doit échouer.

## Tailscale

- **Le serveur est tagué `tag:brume`.** Les ACL du tailnet l'empêchent d'ouvrir des connexions vers les appareils. Les headers d'identité continuent de fonctionner, car ils identifient le client.
- **Les bases sont joignables depuis le laptop** en TCP sur le tailnet uniquement. Sans ce chemin simple, les utilisateurs publieraient le port 5432.
- Le reste (identité, clé d'auth, Tailnet Lock) est dans [04-panel-admin.md](04-panel-admin.md).
