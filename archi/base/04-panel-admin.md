# Panel d'administration

## Accès réseau

- **`brume api` écoute uniquement sur `127.0.0.1`.** `tailscale serve` l'expose en HTTPS sur le tailnet, avec un certificat `*.ts.net`.
- **HTTPS uniquement,** aucun listener HTTP sur le tailnet.
- **Vérifier le header `Host`** sur chaque requête, contre le DNS rebinding.
- **Jamais de Funnel sur Brume.**

## Tailscale

- **Clé d'auth de l'install :** à usage unique, pré-autorisée, taguée, de courte durée.
- **Nœud tagué `tag:brume` :** un nœud tagué n'a pas d'expiration de clé. Sans tag, la clé expire au bout de 180 jours et on perd l'accès au panel. Les ACL l'empêchent aussi d'ouvrir des connexions vers les appareils du tailnet.
- **Tailnet Lock proposé en option :** il empêche le serveur de coordination de Tailscale d'ajouter un nœud au tailnet.

## Authentification

Deux facteurs indépendants.

### 1. L'identité Tailscale

- `tailscale serve` ajoute le header `Tailscale-User-Login` selon l'appareil qui se connecte.
- **Allowlist d'identités fixée à l'install :** `brume install` demande le login Tailscale de l'admin. Jamais de « le premier qui se connecte devient admin ».
- **Le header n'est fiable que s'il vient de `tailscale serve`.** N'importe quel process local qui atteint `127.0.0.1` peut le forger. Donc : rejeter toute requête sans header, et aucun container en réseau host.
- **Être sur le tailnet ne suffit pas :** un appareil compromis ou un tailnet partagé donnerait accès au panel. D'où le second facteur.

### 2. Un second facteur à la connexion

À trancher : **mot de passe ou passkey.**

| | Passkey (WebAuthn, crate `webauthn-rs`) | Mot de passe |
|---|---|---|
| Phishing | Résiste : liée à l'origine du panel | Phishable |
| UX | Touch ID, Face ID | À taper |
| Côté serveur | Clé publique, rien de secret | Hash argon2id, limite de tentatives |
| Limite | Liée au nom `brume.xxx.ts.net` : à ré-enrôler si la machine ou le tailnet change de nom | Réutilisation de mots de passe |

**Recommandation : passkey.**

- **Les identifiants sont stockés par l'engine,** pas par l'API. Il pourra ainsi vérifier lui-même les actions critiques le jour où on ajoute la ré-authentification.
- **Premier accès :** `brume install` affiche une URL d'enrôlement à usage unique, valable 15 minutes.
- **Récupération :** la CLI `brume` en local, depuis la console de l'hébergeur, via un socket unix en `0600`.

## Session

- **Cookie `__Host-brume_session`,** en `Secure`, `HttpOnly` et `SameSite=Strict`. Un site tiers ne peut pas le faire envoyer.
- **Émis après le second facteur,** sur une requête same-origin.
- **Lié à l'identité Tailscale :** si le header change, la session devient invalide.

## CSRF et requêtes cross-site

L'identité Tailscale est une auth implicite, ajoutée par le réseau, comme un cookie sans `SameSite`. Un site malveillant ouvert sur le laptop de l'admin peut envoyer un `POST` au panel, et la requête arrive avec l'identité Tailscale.

- **Fetch Metadata :** rejeter toute requête d'API dont `Sec-Fetch-Site` n'est pas `same-origin`. C'est plus strict que SameSite : les autres machines en `.ts.net` du tailnet sont « same-site » pour le navigateur.
- **Vérifier `Origin`** sur toute requête qui modifie quelque chose.
- **Exiger `Content-Type: application/json` et un header custom** (`X-Brume: 1`). Un formulaire venant d'un autre site ne peut pas les envoyer sans requête préalable CORS, qui échouera.
- **Aucun effet de bord sur un `GET`.**
- **Aucun CORS.**
- **WebSockets :** vérifier `Origin` au handshake, car les WebSockets ne sont pas soumises à CORS. Pour les logs en direct, préférer SSE, plus sûr par défaut.

## XSS

Une faille XSS dans le panel donne root sur la machine. Or le panel affiche du contenu contrôlé par l'attaquant : lignes de logs, noms de branches, messages de commit.

- **Jamais de `dangerouslySetInnerHTML`,** y compris pour le rendu des couleurs ANSI.
- **CSP stricte :** `default-src 'self'`, pas de script inline.
- **Trusted Types** (`require-trusted-types-for 'script'`), là où le navigateur le supporte : le navigateur bloque lui-même toute insertion de HTML brut.
- **Anti-clickjacking :** `frame-ancestors 'none'`. Sinon, un site peut afficher le panel dans une iframe invisible et faire cliquer l'admin sur « Supprimer ».

## Secrets dans le panel

- **L'API ne renvoie jamais la valeur d'un secret,** sauf sur une action « révéler » explicite.
- **Ré-authentification avant les actions critiques :** plus tard, voir [12-plus-tard.md](12-plus-tard.md).

## Journal d'audit

Chaque action est journalisée avec l'identité Tailscale. Voir [09-observabilite.md](09-observabilite.md).

## Hors 0.1

Shell web dans les containers, tokens d'API, CLI distante, déploiement depuis GitHub Actions.
