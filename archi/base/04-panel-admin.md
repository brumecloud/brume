# Panel d'administration

## Accès réseau

Ce document décrit l'administration par Tailscale, le mode recommandé. Les autres modes sont à la fin : voir [Sans Tailscale](#sans-tailscale).

- **`brume api` écoute uniquement sur `127.0.0.1`.** `tailscale serve` l'expose en HTTPS sur le tailnet, avec un certificat `*.ts.net`.
- **HTTPS uniquement,** aucun listener HTTP sur le tailnet.
- **Vérifier le header `Host`** sur chaque requête, contre le DNS rebinding.
- **Jamais de Funnel sur Brume.**

## Tailscale

- **Clé d'auth de l'install :** à usage unique, pré-autorisée, taguée, de courte durée.
- **Nœud tagué `tag:brume` :** un nœud tagué n'a pas d'expiration de clé. Sans tag, la clé expire au bout de 180 jours et on perd l'accès au panel. La policy du tailnet doit aussi lui interdire d'ouvrir des connexions vers les appareils : voir [03-reseau.md](03-reseau.md#tailscale).
- **Tailscale SSH ne passe pas par le second facteur.** En 0.1, l'identité Tailscale seule donne un shell root. Risque accepté, voir [01-modele-de-menace.md](01-modele-de-menace.md#risques-acceptés-en-01).
- **Tailnet Lock : plus tard.** Il empêchera le serveur de coordination de Tailscale, ou un compte volé chez le fournisseur d'identité, d'ajouter un nœud au tailnet. Voir [12-plus-tard.md](12-plus-tard.md).

## Authentification

Deux facteurs indépendants.

### 1. L'identité Tailscale

- **`tailscale serve` pose le header `Tailscale-User-Login`** selon l'appareil qui se connecte, après avoir supprimé les headers `Tailscale-User-*` envoyés par le client. Il ne le pose pas pour un appareil tagué : toute requête sans ce header est rejetée.
- **Allowlist d'identités fixée à l'install :** `brume install` demande le login Tailscale de l'admin. Jamais de « le premier qui se connecte devient admin ».
- **Un process local qui atteint `127.0.0.1` peut forger le header.** Aucun container n'est en réseau host, et sans session émise par l'engine, un header forgé ne donne rien.
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

- **Vérifié par l'engine, pas par l'API.** L'engine stocke les identifiants, génère le challenge, vérifie la réponse et émet la session.
- **Premier accès :** `brume install` affiche une URL d'enrôlement à usage unique, valable 15 minutes. Elle porte un jeton affiché seulement dans le terminal de l'install : en administration publique, cette URL est joignable depuis Internet.
- **Récupération :** la CLI `brume` en local, depuis la console de l'hébergeur, via un socket unix en `0600`.

## Session

- **Émise par l'engine après le second facteur,** sur une requête same-origin. L'engine garde la session ; l'API ne fait que transmettre son jeton.
- **Hors du log d'événements :** une table SQLite, jeton haché, jamais envoyée sur S3. Une restauration déconnecte donc l'admin. Ouvrir ou fermer une session produit un événement d'audit. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#ce-qui-est-un-événement).
- **Validée par l'engine à chaque demande.** L'API joint le jeton et l'identité Tailscale à chaque appel, et l'engine vérifie que la session est valide et liée à cette identité. Une API compromise ne peut rien faire sans une session ouverte par l'admin.
- **Durée courte, fixée par l'engine :** c'est la fenêtre pendant laquelle une API compromise peut agir.
- **Cookie `__Host-brume_session`,** en `Secure`, `HttpOnly` et `SameSite=Strict`. Un site tiers ne peut pas le faire envoyer.
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

Une faille XSS dans le panel donne ses actions pendant la session de l'admin : toutes les apps, leurs secrets et leurs données. Or le panel affiche du contenu contrôlé par l'attaquant : lignes de logs, noms de branches, messages de commit.

- **Jamais de `dangerouslySetInnerHTML`,** y compris pour le rendu des couleurs ANSI.
- **CSP stricte :** `default-src 'self'`, pas de script inline, plus `base-uri 'none'` et `form-action 'self'`, que `default-src` ne couvre pas.
- **Trusted Types** (`require-trusted-types-for 'script'`), là où le navigateur le supporte : le navigateur bloque lui-même toute insertion de HTML brut.
- **Anti-clickjacking :** `frame-ancestors 'none'`. Sinon, un site peut afficher le panel dans une iframe invisible et faire cliquer l'admin sur « Supprimer ».

## Secrets dans le panel

- **L'API ne renvoie jamais la valeur d'un secret,** sauf sur une action « révéler » explicite.
- **« Révéler » passe par la session vérifiée par l'engine,** comme toutes les actions, et est journalisé.

## Journal d'audit

Chaque action est un événement du log, avec l'identifiant de l'admin. Son identité (login Tailscale ou email) vit dans un blob qu'on peut purger. Voir [09-observabilite.md](09-observabilite.md#journal-daudit).

## Sans Tailscale

Deux modes, choisis à l'install sur l'hôte : le tunnel SSH et l'administration publique. Voir [03-reseau.md](03-reseau.md#administration-sans-tailscale) pour le réseau.

- **Le premier facteur devient un mot de passe :** hash argon2id, limite de tentatives, vérifié par l'engine. Il n'y a plus d'identité Tailscale ni d'allowlist : le compte de l'admin est créé à l'install.
- **Le second facteur est une passkey,** quelle que soit la décision ouverte sur le mode Tailscale. Un mot de passe plus un mot de passe ne font pas deux facteurs.
- **La session n'est liée qu'à son cookie.** Tout le reste s'applique : Fetch Metadata, `Origin`, header custom, CSP, Trusted Types.
- **Tunnel SSH :** le panel est servi sur `http://localhost`. Les navigateurs récents y acceptent les cookies `Secure` et les passkeys, à vérifier sur chacun.
- **Public :** le panel a son propre domaine, et `Host` est vérifié contre ce domaine. Traefik et l'engine limitent les tentatives de connexion. N'importe qui sur Internet peut atteindre la page de connexion et l'API : le second facteur et la vérification des sessions par l'engine sont la seule barrière.

## Hors 0.1

Shell web dans les containers, tokens d'API, CLI distante, déploiement depuis GitHub Actions.
