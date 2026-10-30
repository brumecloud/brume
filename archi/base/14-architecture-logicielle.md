# Architecture logicielle

Comment le code de Brume est organisé : les processus et leurs responsabilités, l'event sourcing, le protocole entre l'engine et l'agent, et les crates. L'état sur S3 et la restauration sont dans [15-etat-et-restauration.md](15-etat-et-restauration.md), les tests dans [16-tests.md](16-tests.md), la télémétrie dans [17-telemetrie.md](17-telemetrie.md).

## Principes

1. **L'état voulu est event-sourcé.** Chaque changement est un événement ajouté à un log, jamais une mise à jour en place. L'historique et l'audit en découlent.
2. **Un seul écrivain.** L'engine décide seul, une commande après l'autre, dans un ordre global.
3. **Le code qui décide ne fait aucune I/O.** L'engine et l'agent sont des machines à états pilotées par leurs entrées. C'est ce qui permet de simuler tout Brume de façon déterministe.
4. **Le code équivalent à root est le plus petit possible.** L'engine tient l'état et les secrets, l'agent tient le socket Docker : seul l'agent est équivalent à root. L'engine ne l'est pas sans une faille du noyau ou de runc, voir [L'engine](#lengine--le-plan-de-contrôle).
5. **SQLite fait foi pendant l'exécution, S3 quand le disque est perdu.** L'envoi vers S3 est toujours asynchrone.

## Vue d'ensemble

```
Internet ─► Cloudflare ─► cloudflared ─► Traefik ─┬─► apps, nginx
                                                  └─► brume webhook ──┐
                                                                      │ webhook.sock
Tailnet ──► tailscale serve ──► brume api ────────────────────────────┤ api.sock
                                                                      ▼
                      GitHub, notifications ◄─────────────────── brume engine ──► S3 : log, blobs, clés
                                                                   ▲      │
                                             observations, faits   │      │   état voulu, jobs
                                                                   │      ▼   (agent.sock)
                      S3 : backups, artefacts ◄───────────────── brume agent ──► dockerd ──► containers
                                                                                   │
                                                       brume telemetry ◄───────────┘ logs des apps
```

Le schéma montre les modes recommandés, voir [L'entrée et l'administration](#lentrée-et-ladministration). Tous les services envoient aussi leur télémétrie à `brume telemetry` : voir [17-telemetrie.md](17-telemetrie.md).

## L'engine : le plan de contrôle

- **Seul propriétaire de l'état :** le log d'événements dans SQLite, et le `World` en mémoire.
- **Décide chaque commande, une à la fois.** Les invariants qui touchent plusieurs entités se vérifient donc sur l'état complet, sans course : slug unique dans un environnement, un nom est soit une variable soit un secret à tous les niveaux, refus de supprimer un secret encore référencé.
- **Propriétaire des sessions et du second facteur,** comme avant : voir [04-panel-admin.md](04-panel-admin.md#session).
- **Pilote les opérations longues** (builds, déploiements, backups, rollbacks) et leurs délais.
- **Détient** la clé maître, la clé privée de la GitHub App et les credentials S3 de l'état.
- **Envoie le log sur S3** en arrière-plan : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#lenvoi).
- **Alerte** à partir des faits que lui remontent les agents.
- **Écoute** sur `api.sock`, `webhook.sock`, `agent.sock` et `admin.sock` (la CLI de secours, root seulement). En sortie : S3, GitHub et les canaux de notification.
- **N'a pas le socket Docker.** Compromis, il donne les apps, leurs secrets et leurs données, dans les limites que l'agent applique. Il ne donne pas root sur l'hôte sans une faille du noyau ou de runc.
- **Risque accepté : le builder raccourcit ce chemin.** Un engine compromis peut faire construire n'importe quel repo, et le builder tourne avec un sandbox affaibli ([06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder)). Avec une faille du noyau ou de runc, ça mène à root. C'est la même famille que le risque déjà accepté sur l'accès en écriture à une branche déployée ([01-modele-de-menace.md](01-modele-de-menace.md#risques-acceptés-en-01)). Le builder jetable sous gVisor le fermera ([12-plus-tard.md](12-plus-tard.md)).

## L'agent : un par machine

- **Seul composant de Brume qui tient le socket Docker.** Avec l'updater, c'est le seul équivalent à root.
- **Applique l'état voulu de sa machine :** il observe Docker, calcule `plan(voulu, observé)` et exécute les actions.
- **Construit la config de chaque container par liste blanche et applique `/etc/brume/policy.toml`.** Voir [05-runtime.md](05-runtime.md#isolation-des-containers). L'engine vérifie aussi la politique, pour afficher une erreur claire dans le panel, mais c'est le contrôle de l'agent qui protège.
- **Vérifie aussi la forme de l'état voulu.** Une route Traefik ne vise qu'un service exposé de sa machine, et un port `tailscale serve` qu'une base dont l'accès par le tailnet est activé.
- **Démarre les containers au boot et les redémarre s'ils s'arrêtent,** après avoir vérifié la table nftables. Les containers n'ont pas de politique de redémarrage Docker : voir [03-reseau.md](03-reseau.md#règles-nftables).
- **Écrit la config de Traefik** (file provider) **et celle de `tailscale serve`.**
- **Lance les containers ponctuels :** builder, dumps de backup, vérifications des templates ([templates.md](templates.md#vérifications-au-déploiement)), extraction des sites statiques. Il charge les images construites dans dockerd.
- **Envoie les backups et les artefacts sur S3 :** voir [15-etat-et-restauration.md](15-etat-et-restauration.md).
- **Détecte les écarts** et les signale à l'engine, qui alerte.
- **N'écoute rien :** c'est lui qui se connecte à l'engine.

## Les autres services

`brume api`, `brume webhook` et `brume updater` ne changent pas : voir [02-architecture.md](02-architecture.md). `brume logs` devient `brume telemetry`, et reçoit aussi la télémétrie de Brume : voir [17-telemetrie.md](17-telemetrie.md).

## L'entrée et l'administration

Cloudflare Tunnel et Tailscale sont recommandés, mais plus obligatoires.

| | Recommandé | Autres modes |
|---|---|---|
| Trafic public | **Cloudflare Tunnel :** aucun port entrant. | **Entrée directe :** Traefik publie les ports 80 et 443, avec des certificats Let's Encrypt. |
| Administration | **Tailscale :** le panel et SSH passent par le tailnet. | **Tunnel SSH :** le panel n'écoute que sur localhost. **Public :** Traefik sert le panel sur le 443. |

- **L'admin choisit à l'install, sur l'hôte,** comme les autres options dangereuses (principe 6 du [README](README.md#principes)). Un changement de mode se fait aussi sur l'hôte, en root, et déclenche une alerte. Une restauration redemande les modes, comme l'install.
- **Le panel signale en permanence qu'un mode non recommandé est actif.**
- **Pour le code, ce sont des adaptateurs.** L'état voulu de l'agent dit quelle entrée faire tourner : cloudflared, ou Traefik avec ses ports publiés. Le mode d'administration ne change que l'adresse où écoute `brume api`. Le reste de Brume ne dépend pas des modes.
- **Sans Cloudflare,** Traefik réserve le chemin du webhook aux plages d'IP de GitHub, à la place de la règle WAF. L'IP du serveur devient publique, et la protection contre le DDoS n'est plus déléguée.
- **Sans Tailscale :**
  - la connexion au panel passe par un mot de passe et une passkey, vérifiés par l'engine, sans identité Tailscale ;
  - SSH reste ouvert sur le port 22, par clé uniquement ;
  - l'accès aux bases depuis le laptop passe par un tunnel SSH.
- **En mode public, n'importe qui sur Internet peut atteindre la page de connexion et l'API du panel.** Le second facteur et la vérification des sessions par l'engine deviennent la seule barrière. C'est le mode le plus exposé.
- **Le détail des règles réseau et de la connexion de chaque mode** va dans [03-reseau.md](03-reseau.md) et [04-panel-admin.md](04-panel-admin.md).

## Le cœur sans I/O

L'engine et l'agent ont chacun un cœur sans I/O, et une coquille tokio qui fait les I/O à sa place.

```rust
// brume-engine-core : ni tokio, ni horloge, ni hasard, ni I/O
pub enum Input {
    Command { client: ClientId, cmd: Command },
    FromAgent { node: NodeId, msg: AgentMsg },
    EffectDone { id: EffectId, result: EffectResult },
    Tick,
}

pub enum Output {
    Reply { client: ClientId, reply: Reply },
    ToAgent { node: NodeId, msg: EngineMsg },
    Effect { id: EffectId, effect: Effect }, // PUT S3, appel GitHub, notification
    WakeAt(Timestamp),
}

impl<L: EventLog> Engine<L> {
    pub fn handle(&mut self, ctx: &mut Ctx, input: Input, out: &mut Vec<Output>);
}
```

- **`Ctx` fournit l'heure et le hasard.** En production, l'horloge système et `OsRng`. En simulation, l'horloge simulée et un générateur initialisé par la graine. Les UUIDv7 et les nonces de chiffrement en dépendent.
- **Le log local est un trait synchrone, `EventLog`.** En production, il est écrit avec rusqlite ; en simulation, c'est un faux.
- **Tout ce qui passe par le réseau est un effet :** S3, GitHub, les notifications et, pour l'agent, les appels Docker. Le cœur émet l'effet, la coquille l'exécute, et le résultat revient au cœur comme une entrée `EffectDone`.
- **Les décisions sont des fonctions pures :** `decide(&World, Command)` et `evolve(&mut World, &Event)` pour l'engine, `plan(&Voulu, &Observé)` pour l'agent.
- **Les opérations longues sont des machines à états explicites,** pas des `async fn`. C'est plus verbeux, mais chaque étape survit à un crash et se teste seule.
- **La coquille reste mince :** elle lit les sockets, réveille le cœur aux échéances qu'il demande (`WakeAt`), exécute les effets et renvoie leurs résultats.
- **Les flux de lecture ne traversent pas le cœur.** Les logs en SSE et les requêtes de télémétrie ne passent par `handle()` que pour vérifier la session. Ensuite, la coquille relaie les octets. Sinon, un gros flux de logs ralentirait toutes les commandes, puisque le cœur est le seul écrivain.

## Event sourcing

### Ce qui est un événement

| Sorte | Exemples | Modifie le `World` |
|---|---|---|
| Décision | `ProjectCreated`, `ServiceCreated`, `RevisionCreated`, `DeploymentRequested`, `SecretVersionAdded`, `EgressEnabled` | oui |
| Fait rapporté | `BuildSucceeded`, `DeploymentSucceeded`, `DeploymentFailed`, `DriftDetected`, `BackupCompleted`, `ArtifactMissing` | oui |
| Audit | `LoginSucceeded`, `LoginFailed`, `SessionRevoked`, `SecretRevealed`, `CommandRejected` | non |

- **L'état observé n'est pas event-sourcé.** Le statut des containers, les healthchecks et la consommation changent tout le temps : l'engine les garde en mémoire, tels que les agents les rapportent. Seuls les faits qui comptent deviennent des événements.
- **Les sessions non plus.** Elles vivent dans une table SQLite hors du log, avec le jeton haché, jamais envoyée sur S3.
- **Une action sensible sans effet sur l'état est un événement d'audit :** révéler un secret, ouvrir une session, voir une commande refusée par la politique.

### Les métadonnées

Chaque événement porte :

- `seq` : sa position dans le log, globale et sans trou ;
- `generation` : la génération du log, voir [Générations et compaction](#générations-et-compaction) ;
- `stream` : l'entité concernée (projet, environnement, service, déploiement) ;
- `type` et `schema_version` ;
- `at` : l'heure de l'engine ;
- `actor` : l'identifiant opaque de l'admin, `github` pour un webhook, l'agent pour un fait rapporté, `engine` pour un délai expiré. L'identité de l'admin (son email) n'est pas dans l'événement, voir la section suivante ;
- `session`, quand il y en a une ;
- `correlation_id` : le `trace_id` OpenTelemetry de l'action, voir [17-telemetrie.md](17-telemetrie.md#traces) ;
- `causation_id` : la commande ou l'événement qui l'a provoqué ;
- `prev_hash` et `hash`, voir [La chaîne de hash](#la-chaîne-de-hash).

### Ce qui n'entre jamais dans un événement

- **Les valeurs des secrets.** Elles vivent dans une table `secret_versions`, chiffrées par la clé maître. L'événement porte le nom du secret et sa version : on sait quelle version tournait à une date donnée sans que la valeur apparaisse dans le log.
- **Les valeurs des variables.** Elles vivent dans une table `blobs`, sous un identifiant aléatoire. Pas d'empreinte de la valeur à la place : celle d'une valeur courte se retrouve par force brute. Purger une révision ([13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge)) efface ses blobs, et ses événements restent.
- **Les données personnelles.** L'email d'un admin ou l'auteur d'un commit vivent dans un blob, et l'événement ne porte qu'un identifiant opaque. Le log part sur S3 et n'y est jamais réécrit : purger le blob suffit pour répondre à une demande d'effacement (RGPD).
- **Les jetons :** sessions, tokens GitHub, mots de passe.

### Le `World` et SQLite

- **Le `World` est la seule copie de l'état courant.** L'engine le garde en mémoire et le reconstruit au démarrage en rejouant la génération courante du log. Quelques dizaines de milliers d'événements se rejouent en moins d'une seconde.
- **SQLite contient** le log (`events`), les tables hors log (`secret_versions`, `blobs`, `sessions`, les points de reprise) et des index d'historique : déploiements par service, audit par acteur. Ces index ne reçoivent que des ajouts, dans la transaction qui ajoute les événements.
- **Aucune projection de l'état courant en SQL :** il n'y a rien à garder synchronisé avec le `World`.
- **SQLite en WAL, avec `synchronous=FULL` :** un commit est durable.

### Le cycle d'une commande

1. L'engine reçoit une commande typée et validée, avec la session qui la porte.
2. `decide(&World, cmd)` rend des événements, ou un rejet. Un rejet par la politique produit un événement `CommandRejected`.
3. Une transaction SQLite ajoute les événements, chaînés, et les lignes d'index, puis le commit les rend durables.
4. `evolve` applique les événements au `World`. Il ne peut pas échouer, puisqu'un événement est un fait : s'il ne s'applique pas, c'est un bug, donc une panique.
5. Le cœur réveille ce qui dépend du log : l'envoi vers S3, les opérations longues, l'état voulu des agents.

- **Une commande peut porter la version attendue de son `stream`.** Si une autre commande l'a modifié entre-temps, elle est rejetée. Le panel s'en sert pour ne pas écraser un formulaire modifié ailleurs.

### Les opérations longues et les délais

- **Une opération longue est une machine à états pure,** qui réagit aux événements et émet des commandes : build, déploiement, backup, rollback. Par exemple : `BuildRequested`, `BuildSucceeded`, `RevisionCreated`, `DeploymentStarted`, puis `DeploymentSucceeded` ou `DeploymentFailed`.
- **Son état se déduit du log.** Après un crash, l'engine la reprend où elle en était.
- **Les délais se déduisent aussi de l'état.** Un déploiement commencé à T expire à T plus son délai : au démarrage, l'engine recalcule ses échéances depuis le `World`. Une échéance atteinte produit une commande, puis un événement, par exemple `DeploymentFailed { reason: Timeout }`.
- **Les relances aussi :** le nombre de tentatives est dans l'état, et le délai avant la suivante se calcule.

### Le déploiement

- **Un service sans volume se déploie sans coupure.** Le nouveau container démarre à côté de l'ancien. Une fois son healthcheck passé, la route Traefik et l'alias DNS de l'environnement basculent vers lui, puis l'ancien s'arrête. Si le healthcheck échoue, le nouveau container s'arrête et l'ancien continue de tourner.
- **Un service avec un volume s'arrête, puis redémarre.** Deux containers ne partagent jamais un volume : la coupure dure le temps du démarrage, et le panel le signale.
- **Un site statique bascule instantanément,** voir [05-runtime.md](05-runtime.md#sites-statiques).
- **C'est l'engine qui pilote, par une suite d'états voulus.** Chaque étape est un événement et survit donc à un crash. L'agent n'a pas à se souvenir d'un déploiement en cours.
- **`DeploymentSucceeded` porte ce que l'agent a observé :** le container démarré et le healthcheck réussi.
- **La façon de faire basculer l'alias DNS** sur le réseau de l'environnement est laissée au builder.

### Les versions des événements

- **Stockés en JSON,** avec `#[serde(tag = "type")]` et une `schema_version`. Le JSON reste lisible des années plus tard, sans le code qui l'a écrit.
- **Un type publié ne change jamais.** Une nouvelle forme est une nouvelle version, et un upcaster convertit l'ancienne à la lecture.
- **La CI rejoue un log de référence de chaque release :** voir [16-tests.md](16-tests.md).

### Générations et compaction

- **À une mise à jour qui casse le format, l'engine compacte le log.** Il ouvre une nouvelle génération, dont le premier événement, `Genesis`, contient tout le `World`. Le code n'a plus à rejouer les générations précédentes, et leurs upcasters peuvent disparaître.
- **Les anciennes générations restent,** sur S3 et dans SQLite. L'audit les affiche telles quelles, en JSON.
- **`Genesis` porte le hash du dernier événement de la génération précédente :** la chaîne continue d'une génération à l'autre.
- **Une release qui compacte ne se saute pas.** Son manifeste signé le dit, et l'updater l'installe avant toute release plus récente. Sinon, une install restée sur l'ancienne génération ne pourrait plus lire son log.

### La chaîne de hash

- **`hash = SHA-256(prev_hash ‖ octets stockés de l'événement)`.** On hache les octets tels qu'ils sont stockés, jamais une nouvelle sérialisation.
- **Vérifiée au démarrage, à la restauration et par `brume verify`.** Un trou ou une modification se voit. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#brume-verify).
- **Limite :** un attaquant root sur le serveur détient la clé maître et peut prolonger la chaîne. Elle protège ce qui est déjà parti sur S3, pas le présent.

## Les machines

- **En 0.1, une seule machine.** L'install enregistre la machine du master (`MachineRegistered`), et tous les services y tournent.
- **En v1, le placement se fera par service.** Un environnement s'étendra donc sur plusieurs machines, et son réseau devra les traverser (overlay ou WireGuard) : c'est un chantier à part entière. Le placement sera un nouvel événement, sans modifier les anciens.
- **L'entrée du trafic public avec plusieurs machines se décidera à ce moment-là :** soit le master relaie tout le trafic, soit chaque machine a son propre tunnel.
- **Avec plusieurs machines, une restauration repartira d'une position du log que les agents distants auront déjà dépassée.** La position devra alors inclure un compteur de restaurations.

## Le protocole entre l'engine et l'agent

- **En 0.1, un socket unix :** `/run/brume/agent.sock`, ouvert par l'engine, groupe `brume-agent`. **Plus tard,** les agents des autres machines se connecteront en sortie au master par le tailnet, avec les mêmes messages. Le master n'aura aucun accès aux machines.
- **Messages typés et versionnés** dans `brume-proto`, avec une taille limitée. Le format de sérialisation est laissé au builder, tant qu'il passe par serde.
- **De l'engine vers l'agent :**
  - `DesiredState` : l'état voulu complet de la machine, jamais un diff. Containers, réseaux, routes Traefik, ports `tailscale serve`, artefacts à garder dans dockerd, et les secrets résolus de ses containers. Il porte la position du log dont il découle.
  - `Job` : une tâche ponctuelle avec un identifiant (build, backup, vérification d'un template). L'engine y joint ce qu'il faut : le token GitHub d'une heure, les identifiants admin d'une base.
- **De l'agent vers l'engine :**
  - `Hello` : sa machine, sa version, la position du dernier état voulu appliqué ;
  - `Observed` : l'état observé résumé, à chaque changement et à intervalle régulier ;
  - `Fact` et `JobResult` : ce que l'engine enregistre comme événements.
- **La position ne recule jamais.** L'agent ignore un état voulu plus ancien que celui qu'il applique, et confirme chaque position appliquée.
- **L'engine ne croit pas l'agent sur parole.** Il vérifie que l'agent ne parle que des ressources de sa propre machine. La machine est identifiée par la connexion, jamais par le message : `SO_PEERCRED` en 0.1, l'identité Tailscale plus tard.
- **Les secrets voyagent dans les types de `secrecy`,** et ne sont jamais journalisés.
- **Si l'engine tombe,** l'agent continue d'appliquer le dernier état voulu reçu, redémarrages compris, sans rien changer d'autre. **Si l'agent tombe,** l'engine accepte toujours les commandes, qui s'appliqueront à son retour.
- **Au boot, en 0.1, l'agent attend l'engine** avant de démarrer les containers. Garder l'état voulu sur la machine, pour démarrer sans le master, viendra avec le multi-machines.
- **Versions :** sur un seul VPS, l'engine et l'agent viennent du même binaire. Avec plusieurs machines, le protocole devra accepter la version N et la version N-1 pendant une mise à jour.

## Les crates

```
crates/
├── brume-domain        newtypes validés, commandes, événements, decide, evolve
├── brume-proto         messages des sockets, versionnés, tailles bornées
├── brume-engine-core   machine à états de l'engine
├── brume-agent-core    machine à états de l'agent, plan, config des containers
├── brume-engine        coquille de l'engine : sockets, log (rusqlite), S3, GitHub
├── brume-agent         coquille de l'agent : bollard, nftables, S3
├── brume-runtime       commun aux coquilles : framing des sockets, client S3, chiffrement, OTEL
├── brume-api           HTTP, SSE, sert le panel
├── brume-webhook       HMAC, payload GitHub
├── brume-telemetry     réception OTLP et logs des apps, DuckDB
├── brume-updater       minisign
├── brume-sim           simulateur et faux, pour les tests seulement
└── brume               le binaire unique : clap, une sous-commande par service
```

- **Les crates du cœur ne font aucune I/O.** `brume-domain`, `brume-proto`, `brume-engine-core` et `brume-agent-core` ne dépendent ni de tokio, ni de rusqlite, ni de bollard. Leur `clippy.toml` interdit `Instant::now`, `SystemTime::now`, `thread_rng` et `HashMap` (`disallowed-methods`, `disallowed-types`). On utilise `BTreeMap`, dont l'ordre de parcours est stable.
- **Seul `brume-agent` dépend de `bollard`, seul `brume-telemetry` de `duckdb`, seul `brume-engine` de `rusqlite`.**
- **Seul le binaire `brume` dépend de `brume-webhook`.** L'engine ne voit jamais le code qui lit les payloads GitHub : « l'engine n'analyse aucune donnée non fiable » devient une propriété du graphe de dépendances.
- **`brume-sim` n'entre jamais dans le binaire.**
- **`cargo deny` vérifie ces règles en CI,** avec `bans` et `wrappers`.
- **Les types de `brume-proto` deviennent des types de `brume-domain` par `TryFrom`,** donc validés. Une valeur non validée n'atteint jamais `decide`.
- **Des traits seulement là où il existe deux implémentations :** le log, et les effets que la simulation remplace.
- **rusqlite plutôt que sqlx pour le log.** Le trait `EventLog` est synchrone, et sqlx n'a qu'une API asynchrone. Les requêtes restent paramétrées, comme le demande [10-developpement.md](10-developpement.md#règles-de-code-rust).

## Ce qui change dans les autres documents

Ces décisions modifient les documents existants. Les mises à jour à faire :

| Document | Changement |
|---|---|
| [README.md](README.md) et le README à la racine du repo | Principe 3 : l'agent et l'updater sont les deux composants équivalents à root, l'engine ne l'est pas sans une faille du noyau ou de runc. Périmètre et principe 4 : Cloudflare Tunnel et Tailscale sont recommandés, plus obligatoires. Décision ouverte 3 : « aucune télémétrie » ne concerne que l'envoi au projet Brume. |
| [01-modele-de-menace.md](01-modele-de-menace.md) | La clé age privée de l'admin devient un actif. Les risques acceptés de [15-etat-et-restauration.md](15-etat-et-restauration.md#risques-acceptés), celui du builder atteint par un engine compromis, et ceux des modes non recommandés. |
| [02-architecture.md](02-architecture.md) | L'agent dans le tableau des composants, le schéma, les sockets et le durcissement. L'engine n'est plus équivalent à root. L'adresse d'écoute de `brume api` dépend du mode d'administration. |
| [03-reseau.md](03-reseau.md) | La config de Traefik et de `tailscale serve`, la vérification nftables et le démarrage au boot passent à l'agent. L'entrée directe, SSH ouvert sans Tailscale, l'accès aux bases par tunnel SSH. |
| [04-panel-admin.md](04-panel-admin.md) | Les sessions vivent hors du log, et les connexions sont des événements d'audit. L'administration par tunnel SSH ou publique : connexion par mot de passe et passkey, sans identité Tailscale. |
| [05-runtime.md](05-runtime.md), [06-builds-et-deploiement.md](06-builds-et-deploiement.md) | La config des containers, `policy.toml`, le builder et le chargement des images passent à l'agent. L'engine génère le token GitHub. |
| [07-secrets-et-backups.md](07-secrets-et-backups.md) | La clé maître est sauvegardée. L'état de Brume est sauvegardé par le log. Le bucket est obligatoire et ne se change que sur l'hôte, plus depuis le panel. Il reçoit aussi le log et les artefacts. Credentials en lecture et écriture, toujours sans suppression. Chiffrement obligatoire. |
| [08-hote-et-installation.md](08-hote-et-installation.md) | Utilisateur `brume-agent`, choix des modes d'entrée et d'administration, bucket demandé et testé à l'install, vérification de la clé age, `brume restore`, `brume verify`. L'updater n'enjambe pas une release qui compacte le log. |
| [templates.md](templates.md) | Plus d'avertissement « sans S3 configuré » : le bucket est obligatoire. |
| [09-observabilite.md](09-observabilite.md) | Le journal d'audit est le log d'événements. `brume logs` devient `brume telemetry`. Les alertes partent des faits, jamais de la télémétrie. |
| [10-developpement.md](10-developpement.md) | Crates et règles de dépendances, rusqlite pour le log, simulation en CI. |
| [12-plus-tard.md](12-plus-tard.md) | Multi-machines : agents distants, placement par service et réseau d'environnement entre machines, entrée publique à décider, état voulu gardé par l'agent, haute disponibilité du master (Raft sur le log). La sauvegarde de la clé maître est faite. |
| [13-modele-de-donnees.md](13-modele-de-donnees.md) | La rétention des artefacts est exécutée par l'agent, et ne vaut que pour dockerd : sur S3, rien n'est supprimé. La purge n'atteint pas S3. |
