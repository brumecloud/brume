# Télémétrie

Les métriques, logs et traces de Brume lui-même, en OpenTelemetry. Le but : comprendre ce que fait Brume, en production comme en simulation.

## Périmètre

- **Seulement la télémétrie de Brume.** Les apps n'envoient pas d'OTLP à Brume. Leurs logs sont collectés comme avant ([09-observabilite.md](09-observabilite.md#logs-des-apps)), et l'agent mesure leur consommation (CPU, mémoire, réseau).
- **Elle reste chez l'admin.** Rien ne part vers le projet Brume : la décision « aucune télémétrie » du [README](README.md#décisions-ouvertes) porte sur cet envoi-là.
- **Ce n'est pas l'audit.** L'audit, c'est le log d'événements ([14-architecture-logicielle.md](14-architecture-logicielle.md#event-sourcing)). La télémétrie peut perdre des données, et un service compromis peut en fabriquer.
- **Aucune décision ni alerte ne repose sur la télémétrie.** Les alertes partent des faits et de l'état observé que l'engine reçoit des agents.

## Le pipeline

```
engine, agent, api, webhook ── OTLP, otlp.sock ──┐
dockerd ── fluentd (logs des apps) ──────────────┤
                                                 ▼
                                         brume telemetry ── DuckDB : logs, métriques, traces
                                           │        ▲
                                           │        └── panel, via l'engine
                                           └──► collector OpenTelemetry (option) ──► backend de l'admin
```

- **Notre code ne connaît que `tracing`.** La coquille de chaque service installe le SDK OpenTelemetry (`tracing-opentelemetry`, `opentelemetry-appender-tracing`, métriques OTEL) et exporte en OTLP. Les cœurs ne dépendent pas d'OpenTelemetry.
- **`brume telemetry` remplace `brume logs` et reçoit tout :** l'OTLP des services sur `/run/brume/otlp.sock`, et les logs des apps par fluentd, comme avant. Il stocke logs, métriques et traces dans DuckDB. Il garde les protections de [09-observabilite.md](09-observabilite.md#duckdb-durci) : aucun réseau, DuckDB durci, plafonds de mémoire et de CPU.
- **La source est identifiée par le noyau** (`SO_PEERCRED`), jamais par l'attribut `service.name`. Une API compromise ne peut pas se faire passer pour l'engine.
- **La lecture passe par l'engine,** comme pour les logs des apps : le panel ne parle jamais directement à `brume telemetry`.
- **La télémétrie ne bloque jamais.** Les files sont bornées : quand elles débordent, les données sont jetées et comptées.
- **Journald sert de secours.** Chaque service écrit aussi ses logs sur stderr : si `brume telemetry` tombe, `journalctl -u brume-engine` fonctionne toujours.
- **Rétention limitée,** par type de signal et par taille sur le disque. La télémétrie ne va ni sur S3, ni dans les backups.

### L'export

- **En option,** vers le backend de l'admin en OTLP : Grafana, Honeycomb, un Jaeger…
- **Par le collector OpenTelemetry officiel,** en container épinglé par digest, sur son propre réseau de sortie. `brume telemetry` lui transmet les données par un socket unix, et reste sans réseau.
- **Changer la destination déclenche une alerte,** comme pour les backups.

## Traces

- **Une trace suit une action de bout en bout :** l'API, l'engine, l'agent, puis Docker, S3 ou GitHub. Le `traceparent` W3C voyage dans l'enveloppe des messages de `brume-proto`.
- **La trace commence dans `brume api`** pour une commande de l'admin, **et dans `brume webhook`** pour un push. Un contexte de trace venu de l'extérieur est ignoré.
- **Le `trace_id` est la `correlation_id` des événements.** Depuis une ligne d'audit, on ouvre la trace, et inversement.
- **Un déploiement forme une seule trace,** du webhook à `DeploymentSucceeded`. Son contexte est stocké dans les métadonnées des événements : la trace continue même si l'engine redémarre en plein milieu.
- **La réconciliation ne crée un span que si le plan contient des actions.** Sinon, un span vide apparaîtrait toutes les quelques secondes.
- **C'est la coquille qui crée les spans,** autour de chaque entrée et de chaque effet. Le cœur n'émet que des événements `tracing` sur ses décisions, par exemple le motif d'un rejet.

## Métriques

| Où | Métriques clés |
|---|---|
| engine | commandes par type et par résultat, durée de `decide`, **retard de l'envoi vers S3** (en secondes et en événements), erreurs S3, opérations longues en cours |
| agent | **écart entre la position voulue et la position appliquée**, durée de réconciliation, actions Docker par résultat, écarts détectés |
| containers | CPU, mémoire, réseau, redémarrages et OOM, mesurés par l'agent. C'est la source de l'onglet métriques de chaque service dans le panel. |
| builds, backups | durée, résultat, temps écoulé depuis le dernier backup réussi de chaque base |
| télémétrie | données reçues et jetées, taille de DuckDB |

- **Les labels ne contiennent que des identifiants et des `enum`,** jamais une chaîne saisie : leur nombre de valeurs reste borné.

## Logs

- **Les événements `tracing` deviennent des logs OpenTelemetry,** avec le `trace_id` et le `span_id` de leur span.
- **Niveau `info` par défaut.** Le niveau `debug` s'active sur l'hôte, service par service.

## Aucun secret dans la télémétrie

Les traces finissent souvent par contenir des en-têtes, des URLs avec mot de passe ou des payloads. Brume l'empêche par construction :

- **`secrecy` masque déjà les secrets** dans `Debug`.
- **Seuls les types sûrs entrent dans la télémétrie.** Le trait `Telemetry`, dans `brume-domain`, n'est implémenté que par les identifiants, les `enum`, les durées, les tailles et les newtypes validés. Ni `String`, ni `&str`, ni les types de `secrecy`.
- **Nos macros `event!` et `span!` exigent ce trait,** et `clippy` interdit les macros de `tracing` ailleurs dans nos crates (`disallowed-macros`). Un champ dangereux ne compile pas.
- **Ni en-têtes ni corps de requête** dans les spans de l'API et du webhook : ni cookie, ni signature HMAC, ni payload GitHub.
- **La simulation le vérifie :** elle cherche chaque valeur de secret posée par la charge dans toute la télémétrie produite ([16-tests.md](16-tests.md#les-invariants)).
- **Les logs des apps restent du contenu hostile,** affichés selon les règles de [04-panel-admin.md](04-panel-admin.md).

## En développement et en simulation

- **En dev,** `BRUME_OTLP_ENDPOINT` envoie toute la télémétrie vers un Jaeger ou un Grafana local.
- **En simulation,** une couche de capture horodate en temps simulé. La trace d'une graine qui échoue s'exporte vers Jaeger : on voit ce qui s'est passé, pas à pas. La même capture sert au test de déterminisme et à la recherche de secrets ([16-tests.md](16-tests.md#reproduire-et-faire-tourner)).
