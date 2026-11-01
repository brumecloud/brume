# Audit de sécurité de la version 0.1

Audit du code de **Brume 0.1**, tel qu'il existe à la fin du jalon 1 : commit `e6184b2`, le 1er novembre 2026. Les numéros de ligne cités renvoient à ce commit.

L'audit suit la méthode du skill [cloudflare/security-audit-skill](https://github.com/cloudflare/security-audit-skill) : reconnaissance, chasse guidée par une grille de couverture, deux vérifications indépendantes de chaque piste, puis un rapport tiré des seuls constats vérifiés.

## En bref

- **Aucune faille critique ni haute.** 7 constats confirmés (1 moyen, 6 bas) et 8 pistes à valider, sans sévérité tant qu'un fait réel n'est pas observé.
- **Les frontières que le modèle de menace place en tête tiennent.** Une page web malveillante ne pilote pas l'API, une ligne de log hostile ne s'exécute pas dans le panel, les secrets restent chiffrés au repos, et un container n'obtient ni privilège ni port publié.
- **Ce qui reste :**
  - la disponibilité : un container peut remplir le disque par ses logs, et un client local non authentifié peut faire grossir le log ou occuper l'API ;
  - une fuite de portée des secrets au redéploiement (C3) ;
  - un flux de logs qui survit à la déconnexion (C7) ;
  - la confiance accordée aux ports de `127.0.0.1`, qui ne compte que si un autre compte tourne sur la machine.

## Périmètre

- **Audité :** le code du jalon 1, lancé par `brume dev` sur un Mac : engine, agent, telemetry, API, panel, plus `site/` et `docs/`. L'API n'écoute que sur `127.0.0.1` et rien n'est exposé à Internet.
- **Pas audité, faute de code :** ce que la 0.1 ne prévoit encore que dans [`archi/base/`](../base/README.md) : l'installeur, Tailscale, Traefik, nftables, `userns-remap`, Cloudflare Tunnel, S3 et les backups, les passkeys, l'updater, la CLI et le serveur MCP. Ces pièces devront être auditées quand elles seront codées.
- **Attaquants retenus**, d'après le [modèle de menace](../base/01-modele-de-menace.md) :
  - une page web malveillante ouverte par l'admin ;
  - une app compromise, donc du code hostile dans un container ;
  - un registry ou une image hors contrôle ;
  - du contenu hostile affiché dans le panel ;
  - un client local non authentifié sur `127.0.0.1`.

  Un processus du même utilisateur que l'admin équivaut à l'admin dans le jalon 1 : ce cas n'est pas un constat.

## Méthode et limites

- **Grille de couverture :** 64 unités (surface × frontière × sous-système × classe d'attaque), chassées par 16 agents en trois vagues, entre lesquelles des critiques de couverture ont ajouté des unités.
- **Vérification :** chaque piste est passée par un vérificateur qui cherchait à la réfuter, puis chaque constat final par un second. Toute correction importante d'un constat a été revérifiée par un vérificateur de plus. 60 agents en tout.
- **Exécution :** en plus de la lecture du code, des tests minimaux ont tourné sur une copie jetable du code, dans un sandbox macOS : sans réseau, écritures limitées au dossier de travail, environnement vide, limites de CPU, de fichiers et de durée. Ce sandbox n'impose ni plafond mémoire ni quota disque. Aucun Docker réel, aucun navigateur, aucun second compte, aucun service distant.
- **Couverture partielle :**
  - **Une unité reportée** : le Mac, un autre compte ou une page web peuvent-ils joindre les containers par le routage d'OrbStack ou de Docker Desktop ? Elle demande un vrai Docker.
  - **Une unité bloquée** : la recherche de CVE dans `Cargo.lock` et les `pnpm-lock.yaml`. Il n'y avait pas de base d'avis hors ligne : à faire avec `cargo audit` et `pnpm audit --prod`.
  - **Pas de passe finale de couverture.** Un seul passage ne couvre jamais toute la cible.

## Ce qui tient

| Frontière | Contrôle vérifié |
|---|---|
| Page web → API | Host en liste blanche, `Sec-Fetch-Site`, `X-Brume`, `Origin`, JSON seul, aucun CORS, CSP avec `frame-ancestors 'none'` et Trusted Types (`crates/brume-api/src/csrf.rs`) |
| Client non authentifié → actions | session vérifiée par l'engine à chaque appel ; jetons de 32 octets aléatoires stockés en SHA-256 ; lien de connexion consommé dans la même transaction que la création de session |
| Contenu de log → panel | rendu en texte React, filtre ANSI, aucune sink HTML (`web/src/helpers/ansi.ts`) |
| Secrets au repos | ChaCha20-Poly1305 avec une AAD liée à la ligne, clé maître en 0600, `Debug` masqués (`crates/brume-proto/src/redact.rs`), logs de bollard plafonnés à `info` |
| Stockage | tout le SQL paramétré ; DuckDB sans accès externe, avec sa configuration verrouillée |
| Container → hôte | config par liste blanche : `cap_drop ALL`, `no-new-privileges`, aucun port publié, volumes nommés seulement, limites de mémoire, de CPU et de pids |
| Image → déploiement | déploiement par digest, sonde busybox épinglée par digest |
| Process locaux → engine | uid du pair vérifié (`SO_PEERCRED`) sur les sockets de l'engine, sockets en 0600 |
| Protocole interne | trames bornées à 4 MiB, erreurs de décodage qui ne recopient pas l'entrée |
| Code | aucun `unsafe` (`unsafe_code = "forbid"`) |

## Constats confirmés

| # | Sévérité | Constat |
|---|---|---|
| C1 | moyenne | Un container peut remplir le disque de l'hôte par ses logs |
| C2 | basse | L'API garde sans limite de temps les connexions qui n'avancent plus |
| C3 | basse | Un redéploiement donne un secret partagé à un service qui ne le référence plus |
| C4 | basse | Chaque connexion ratée ajoute un événement permanent au log |
| C5 | basse | Un registry lent bloque les déploiements des autres environnements |
| C6 | basse | La file des appels de l'API vers l'engine n'a pas de borne |
| C7 | basse | Un flux de logs ouvert survit à la déconnexion et à l'expiration |

Chaque constat a été reproduit localement par un test minimal, puis revérifié par un second agent.

### C1. Un container peut remplir le disque de l'hôte par ses logs

- **Où :** les logs de container suivis par l'agent (`crates/brume-agent/src/logs.rs:124`) et la rétention de telemetry (`crates/brume-telemetry/src/lib.rs:102`).
- **Quoi :**
  - Chaque ligne d'un container finit dans `logs.duckdb`, et la seule purge efface les lignes de plus de 7 jours, une fois par heure.
  - Ni l'agent ni telemetry ne bornent le volume ou le débit d'un container.
  - Docker plafonne sa propre copie à 30 Mo par container ; celle de Brume, non.
- **Qui :** du code hostile dans un container, sans réseau ni privilège, qui écrit en boucle sur sa sortie standard.
- **Impact :** un seul container a fait conserver 81,9 Mo, et rien n'a été purgé. Au bout, le disque est plein : l'engine et telemetry ne peuvent plus écrire, et la reprise est manuelle. Ni secret ni intégrité touchés.
- **Correctif :** un plafond global de taille dans la passe de rétention (par exemple 2 Gio), qui supprime les lignes les plus anciennes et tourne chaque minute. En plus, un budget d'octets par container dans `follow`. C'est la « rétention par taille » que prévoit déjà [17-telemetrie.md](../base/17-telemetrie.md).
- **Test :** insérer plus que le plafond, lancer la passe, vérifier que les messages restent sous le plafond et que les lignes récentes des autres environnements sont toujours là.

### C2. L'API garde sans limite de temps les connexions qui n'avancent plus

- **Où :** `brume_api::run` (`crates/brume-api/src/lib.rs:77-86`), qui confie chaque connexion à `axum::serve` sans réglage.
- **Quoi :**
  - axum 0.8 sert les connexions sans minuterie. Le délai de lecture des en-têtes de hyper est donc désactivé, et hyper n'a aucun délai d'écriture.
  - Une connexion muette, une tête partielle, un corps jamais envoyé ou une réponse jamais lue restent ouverts indéfiniment.
  - Il n'y a pas de plafond de connexions.
- **Qui :** un client local non authentifié qui n'est pas un navigateur.
- **Impact :** une fois la limite de descripteurs atteinte, le panel ne répond plus aux nouvelles connexions. L'engine, l'agent et les containers continuent de tourner.
- **Correctif :**
  1. Une boucle d'acceptation explicite avec `hyper::server::conn::http1::Builder`, une minuterie et un `header_read_timeout` de 10 s.
  2. Un délai de lecture du corps (10 s, réponse 408).
  3. Un délai de progression de l'écriture.
  4. Un plafond de connexions.

  Le correctif complet, testé, est dans les records de l'audit. Derrière un Traefik bien réglé, l'essentiel du risque disparaît.

### C3. Un redéploiement donne un secret partagé à un service qui ne le référence plus

- **Où :** `resolve_secrets` (`crates/brume-domain/src/decide.rs:401-415`), appelé par `Command::Redeploy` (`decide.rs:115-137`) ; la valeur arrive dans l'env du container par `crates/brume-engine-core/src/desired.rs:195`.
- **Quoi :**
  - Un déploiement qui retire un secret commit la nouvelle spec avant de connaître son issue. S'il échoue, la révision active nomme encore le secret, alors que la spec ne le déclare plus.
  - « Redéployer » relance cette révision. Pour un nom que la spec ne déclare pas, `resolve_secrets` retombe sur le secret partagé du même nom, celui de l'environnement ou du projet.
  - Exemple : un service avait son propre `PAYMENT_KEY`, retiré par un déploiement qui a échoué. Au redéploiement, il reçoit le `PAYMENT_KEY` du projet, qu'il n'a jamais référencé.
  - Autre exemple : une référence qu'on vient de couper revient avec la valeur tournée.
- **Qui :** le code d'un service coupé d'un secret, ou qui n'a jamais eu droit à ce secret partagé.
- **Impact :** cela casse une règle du jalon : un secret partagé ne va qu'aux services qui le référencent. L'effet reste borné à un secret, dans le même projet.
- **Correctif :** ne résoudre vers un secret partagé que les noms que la spec courante déclare comme référence (`ServiceVariableValue::SharedSecret`). Un redéploiement d'une révision périmée est alors refusé, avec la liste des secrets manquants. Ce correctif a été appliqué et testé sur une copie : les deux redéploiements fautifs sont refusés, et les suites existantes passent.
- **Décision du 1er novembre :** prévoir aussi une alerte dans le panel quand un redéploiement relance une révision qui ne correspond plus à la spec du service. Ses réglages et ses variables simples reviennent eux aussi en silence.

### C4. Chaque connexion ratée ajoute un événement permanent au log

- **Où :** la branche d'échec de `Engine::login` (`crates/brume-engine-core/src/sessions.rs:130-131`).
- **Quoi :**
  - `POST /api/session` n'exige pas de session.
  - Chaque jeton inconnu écrit un `LoginFailed` de 250 octets environ dans le log chaîné, copié dans le bucket et rejoué à chaque démarrage.
  - Aucune limite de débit, et le log n'est jamais compacté.
- **Qui :** un client local non authentifié, sans navigateur.
- **Impact :** le log, la base et le bucket grossissent sans fin, et la vue Activité comme le démarrage ralentissent. Mesuré : 10 000 essais donnent 10 000 événements.
- **Correctif :** au plus un `LoginFailed` par fenêtre (par exemple une minute), écrit par l'engine. Ne pas ajouter de seau à jetons qui répond 429 : il refuserait aussi le vrai lien de l'admin pendant une attaque. Testé : 10 000 essais donnent 1 événement, et un lien valide fonctionne toujours.

### C5. Un registry lent bloque les déploiements des autres environnements

- **Où :** `jobs::serve` (`crates/brume-agent/src/jobs.rs`), avec ses 3 créneaux de résolution d'image partagés par tous les environnements.
- **Quoi :**
  - Un pull garde son créneau jusqu'à 10 minutes, alors que le déploiement échoue au bout de 5.
  - Aucun message n'annule un job.
  - L'engine renvoie chaque job à la reconnexion de l'agent, sans déduplication : une reconnexion fait trois pulls pour un seul déploiement.
- **Qui :** un registry hors contrôle qui distille ses couches.
- **Impact :** les déploiements des autres environnements échouent par délai. Les containers en marche, les rollbacks, les secrets et l'intégrité ne sont pas touchés.
- **Correctif :**
  1. Dédupliquer par `job_id`.
  2. Borner l'ensemble attente et pull à 4 minutes depuis l'arrivée du job.
  3. Limiter un même registry à 2 des 3 créneaux.

  Abaisser seulement `PULL_TIMEOUT` ne suffit pas : c'est mesuré.

### C6. La file des appels de l'API vers l'engine n'a pas de borne

- **Où :** `EngineClient` (`crates/brume-api/src/engine_client.rs:118,199,218-222`).
- **Quoi :**
  - Les appels passent par un canal non borné.
  - Un appel dont le client HTTP est parti reste dans la file, est écrit et est exécuté par l'engine.
- **Qui :** un client local non authentifié qui envoie des `POST /api/session` en rafale et part sans attendre.
- **Impact :** les appels de l'admin attendent derrière l'arriéré, et la mémoire de l'API croît. Tout se résorbe quand la rafale cesse.
- **Correctif :** une file bornée qui répond 503, l'abandon des appels dont le client est parti, et un plafond de `Login` non authentifiés en vol.

### C7. Un flux de logs ouvert survit à la déconnexion et à l'expiration

- **Où :** le relais des logs (`crates/brume-engine/src/relay.rs:52`), dont la session n'est vérifiée qu'à l'ouverture (`crates/brume-engine-core/src/api.rs:31-41`).
- **Quoi :** après un logout ou à l'échéance des 12 h, l'ancien flux SSE continue de recevoir les nouvelles lignes de l'environnement.
- **Qui :** quelqu'un qui détenait un jeton valide à l'ouverture du flux.
- **Impact :** une lecture des logs d'un environnement qui se prolonge. Aucune écriture, aucun secret de Brume.
- **Correctif :** lier chaque flux au hash de sa session. L'engine émet `SessionClosed` au logout, et le relais se ferme à `expires_at`.

## Pistes à valider

Ces pistes reposent sur le code, mais un fait décisif n'a pas pu être observé : un navigateur, un second compte, ou un vrai Docker. Elles n'ont pas de sévérité. Les cinq premières ne comptent que si un autre compte, ou un processus d'un autre uid, tourne sur la machine. Elles visent surtout le Mac du jalon 1.

| # | Piste | Fait à observer | Correctif simple si elle se confirme |
|---|---|---|---|
| NV1 | Le cookie `brume_session` vaut pour tous les ports de `127.0.0.1` : une page servie sur un autre port par un autre compte recevrait la session (`crates/brume-api/src/session.rs:33`) | Le navigateur envoie-t-il le cookie à un autre port ? Un autre compte tourne-t-il sur la machine ? | Révoquer les sessions au démarrage, sessions plus courtes |
| NV2 | `localhost` est accepté et documenté, mais Brume n'écoute que sur `127.0.0.1` : un autre compte peut prendre `[::1]:7878` (`crates/brume-api/src/lib.rs:41-44,78`) | Ordre de connexion du navigateur pour `localhost`, avec un second compte | Retirer `localhost` de la liste blanche et de la doc |
| NV3 | Les sessions et les liens de connexion survivent à l'arrêt de `brume dev` : un compte qui prend le port 7878 entre deux exécutions recevrait un jeton valide (`crates/brume/src/dev.rs`) | Prise du port par un autre uid juste après Ctrl-C, avec un navigateur | Révoquer sessions et liens inutilisés au démarrage de l'engine |
| NV4 | Un service worker posé sur `127.0.0.1:7878` par un autre compte, pendant que Brume est arrêté, survivrait à son retour (`crates/brume-api/src/csrf.rs`) | Comportement du navigateur (enregistrement, survie, prise de la navigation) | `Clear-Site-Data` à la connexion, `worker-src 'none'` dans la CSP |
| NV5 | Le panel servi par `vp dev` sur 5173 n'a aucune protection anti-encadrement (`web/build/real-api.ts:30-50`) ; dev seulement | Envoi du cookie depuis une iframe dans un vrai navigateur | `server.headers` en mode dev, avec la même CSP que `brume api` |
| NV6 | Le home et `run/` déjà existants ne sont jamais vérifiés : un autre compte qui contrôle le home lit `master.key` par une ACL héritée, ou remplace `run/` (`crates/brume-runtime/src/peer.rs`) | Brume est-il lancé dans un dossier écrit par un autre compte (`/tmp`, `/Users/Shared`) ? | Au démarrage, exiger propriétaire = uid effectif, pas de lien, pas d'ACL, mode 0700 |
| NV7 | Un container sans egress, seulement sur un réseau `internal`, joint peut-être la boucle locale de l'hôte par `host.docker.internal` ou la passerelle de Docker Desktop ou d'OrbStack (`crates/brume-engine-core/src/desired.rs:70`) | Depuis un container sur `brume-env-*` seul, tenter l'hôte et ses passerelles | nftables, déjà prévu pour le VPS |
| NV8 | Une image qui déclare environ 48 000 volumes rend l'état voulu trop gros pour une trame, pour toute la machine et de façon durable (`crates/brume-engine-core/src/desired.rs`) | Un vrai dockerd accepte-t-il une telle image ? | Plafonner les volumes par image, vérifier la taille de l'état avant l'envoi |

## Durcissement

Ce ne sont pas des failles : aucun principal touché n'a été démontré. Ce sont les notes les plus utiles relevées pendant l'audit.

- **Fichiers locaux :**
  - `logs.duckdb` et le bucket local sont créés au umask (644/755), alors que `state.sqlite` est forcé en 0600.
  - `MasterKey::load_or_create` ne vérifie ni le propriétaire ni un lien symbolique.
  - Si `master.key` disparaît alors que des secrets scellés existent, une nouvelle clé est créée en silence.
- **`.gitignore`** n'ignore `/.brume-dev/` qu'à la racine. Lancé depuis `web/`, `brume dev` crée `engine/master.key` dans un dossier suivi par git, que Vite sert aussi.
- **Labels d'image :** une image avec `LABEL brume.probe=1` fait supprimer son propre container à chaque démarrage de l'agent. Il faut écraser toutes les clés `brume.*` héritées de l'image.
- **`pick_digest`** retombe sur la première `RepoDigest` pour les images `docker.io/library/...`. Elle peut appartenir à un autre dépôt, et l'engine ne recoupe pas `ImageResolved` avec l'image demandée.
- **AAD des secrets en attente :** elle ne lie que l'identifiant du changement, pas l'environnement, le service ni le nom.
- **Liste blanche de l'API :** elle accepte toujours `127.0.0.1:5173` et `localhost:5173`, même hors dev.
- **Session :**
  - le panel n'a pas de bouton de déconnexion ;
  - une nouvelle connexion ne révoque pas les anciennes ;
  - les sessions expirées ne sont jamais purgées ;
  - le cookie n'a ni `__Host-` ni `Secure`, que la conception annonce.
- **Agent :**
  - aucune version minimale de Docker, alors que la doc exige la 28.5.2 ;
  - `DOCKER_HOST=tcp://` est ignoré, contrairement à ce que dit la doc ;
  - les sondes HTTP retiennent le tour de réconciliation jusqu'à 15 s.
- **Protocole :**
  - une réponse de plus de 4 MiB ferme la connexion multiplexée de l'API ;
  - `desired_state` est tout-ou-rien : une valeur introuvable gèle aussi une coupure d'egress.
- **Panel :**
  - `parseAnsi` garde les contrôles C1 et bidi, qui permettent un réordonnancement visuel d'une ligne de log ;
  - les champs de secret n'ont pas `autoComplete="new-password"`.
- **Supply chain :**
  - ni CI ni `cargo deny` dans le dépôt ;
  - `docs/` n'a pas `trustPolicy: no-downgrade` ;
  - la libduckdb de Homebrew n'est pas épinglée.

## Décisions prises après l'audit

- **En 0.1, le panel n'est jamais public.** L'administration passe par Tailscale ou par un tunnel SSH, et `brume api` reste sur `127.0.0.1`. Le mode d'administration publique est retiré de la 0.1. [04-panel-admin.md](../base/04-panel-admin.md), [03-reseau.md](../base/03-reseau.md), [14-architecture-logicielle.md](../base/14-architecture-logicielle.md), le [README de `archi/base/`](../base/README.md) et le README du dépôt doivent encore être alignés.
- **C3 :** corriger `resolve_secrets` et ajouter une alerte dans le panel quand un redéploiement relance une révision qui ne correspond plus à la spec.

## Avant d'exposer quoi que ce soit à Internet

L'audit s'est fait sur un Mac, sans rien d'exposé. Sur le VPS, le trafic des apps sera public : c'est le scénario 1 du modèle de menace, l'app compromise. Avant cela, il faut :

- **nftables.** Sans lui, un container avec egress joint l'endpoint de métadonnées de l'hébergeur (`169.254.169.254`), l'hôte et le LAN.
- **`userns-remap`.** Sans lui, root dans un container n'est pas séparé de root sur l'hôte en cas d'évasion.
- **Traefik**, avec des timeouts.
- **Un filtrage dans `DOCKER-USER` ou nftables.** Les ports publiés par Docker contournent UFW et les règles `INPUT`.
- **C1 corrigé**, pour qu'une app compromise ne remplisse pas le disque.

Si un panel public revient plus tard, il faudra en plus :
- un mot de passe et une passkey ;
- des limites de tentatives dans Traefik et dans l'engine ;
- C2, C4 et C6 corrigés : ils deviendraient attaquables depuis Internet ;
- un cookie `__Host-` en `Secure`, un bouton de déconnexion et la révocation des autres sessions ;
- un second facteur pour « révéler » un secret.

## Suites

1. Corriger C1 et C3, avec l'alerte de C3, avant le premier déploiement sur un VPS.
2. Corriger C4 et C2, simples et testés, ou les accepter pour la 0.1 avec une ligne dans [12-plus-tard.md](../base/12-plus-tard.md). Avec un panel derrière Tailscale ou un tunnel SSH, seul un processus de la machine peut les déclencher.
3. Lancer `cargo audit` et `pnpm audit --prod`, et vérifier NV7 sur un vrai Docker Linux.
4. Repasser l'audit sur chaque nouvelle pièce (installeur, Tailscale, Traefik, nftables, S3, passkeys, updater, CLI et MCP), en repartant de la grille de couverture de ce passage.
