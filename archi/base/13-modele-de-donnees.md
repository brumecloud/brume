# Modèle de données

Projet, environnement, service, révision : ce que porte chaque niveau, ce qui est immuable, et ce qu'un rollback restaure.

```
Projet                              variables et secrets communs
└─ Environnement                    réseau interne, réseau egress, cache de build,
   │                                variables et secrets de l'environnement
   └─ Service                       source, slug, type, domaines, egress, volumes,
      │                             variables et secrets du service
      ├─ Révision (immuable)        artefact, commande, port, healthcheck, limites,
      │                             valeurs des variables, noms des secrets
      └─ Déploiement (historique)   révision, versions des secrets, statut, auteur, date
```

## Les niveaux

- **Projet :** les environnements d'une même app (prod, staging), et les variables et secrets communs à tous.
- **Environnement :** un réseau interne et un réseau `egress` à lui, voir [03-reseau.md](03-reseau.md), et les valeurs propres à cet environnement. Deux environnements ne se joignent pas, même dans le même projet.
- **Service :** un container ou un site statique. Le type est fixé à la création et décide de ce que le service peut faire : un site statique est servi par le nginx partagé, hors du réseau de l'environnement, et n'a aucun secret à l'exécution.
- **Le slug d'un service est immuable.** C'est son nom DNS sur le réseau de l'environnement : le renommer casserait les services qui l'appellent. Le nom affiché reste libre. L'engine valide le slug comme un label DNS (`[a-z0-9-]`, 63 caractères au plus).

## Révisions

- **La spec immuable d'un service :** tout ce qu'il faut pour recréer son container. Modifier un de ces champs crée une nouvelle révision.
- **Créée après un build réussi,** jamais avant. Elle pointe donc toujours vers un artefact qui existe : un digest d'image, ou l'empreinte de l'archive d'un site statique. Un build échoué ne crée pas de révision.
- **Jamais les données.** Les volumes et les bases ne sont pas versionnés, voir [Rollback](#rollback).

| Dans la révision : suit le rollback | Sur le service : ne suit pas le rollback |
|---|---|
| artefact (digest), commit d'origine | source (repo, branche), slug, nom affiché |
| commande, port, healthcheck | domaines et routes publiques |
| limites CPU, mémoire, pids | egress |
| valeurs des variables simples et de build | volumes rattachés |
| noms des secrets | valeurs des secrets |

- **La posture de sécurité et les liens avec l'extérieur restent sur le service.** Sinon un rollback rallumerait l'egress coupé après un incident, ou retirerait un domaine dont l'enregistrement DNS existe toujours.

## Déploiements

- **Un déploiement met une révision en service.** Il note la révision, la version de chaque secret injecté, le statut, l'auteur et la date.
- **L'historique ne fait que s'allonger.** Un rollback est un nouveau déploiement d'une ancienne révision : rien n'est réécrit.
- **Changer un secret redéploie la même révision.** La spec n'a pas changé, il n'y a pas de nouvelle révision.
- **On sait quelle version d'un secret tournait à une date donnée.** C'est la première question après une fuite.

## Variables et secrets

### Trois sortes

| Sorte | Dans la révision | Changer la valeur |
|---|---|---|
| Variable simple | la valeur | nouvelle révision, puis déploiement |
| Variable de build | la valeur | rebuild, nouvelle révision, puis déploiement |
| Secret | le nom seulement | redéploiement de la même révision |

- **Les variables simples suivent la révision.** Sinon un rollback casse : la v2 renomme `PAYMENT_URL` en `PAYMENTS_API_URL`, et la v1 restaurée ne trouve plus `PAYMENT_URL`. Comme elles sont injectées à la création du container, en changer une demande de toute façon de le recréer.
- **Les variables de build font partie de l'artefact,** qu'on le veuille ou non : elles finissent dans les couches de l'image ou dans le bundle JS. Pour un site statique, elles sont publiques, voir [05-runtime.md](05-runtime.md#sites-statiques).
- **Les secrets sont résolus au déploiement.** Un rollback ne ramène jamais un secret changé après une fuite. Les secrets de build, passés par `--secret`, suivent la même règle.
- **Les URLs de connexion injectées par un template sont des secrets** (`DATABASE_URL`, `MONGODB_URI`, `REDIS_URL`) : elles contiennent le mot de passe de l'app. Voir [templates.md](templates.md#ce-que-toute-base-reçoit).
- **Avertissement à la saisie** quand une variable simple porte un nom de secret (`*_KEY`, `*_TOKEN`, `*_SECRET`, `*PASSWORD*`) ou que sa valeur contient des identifiants (`scheme://user:pass@host`) : sa valeur resterait dans l'historique des révisions.

### Trois niveaux : projet, environnement, service

- **Variables et secrets se définissent sur le projet, l'environnement ou le service.** Le niveau le plus précis gagne : service, puis environnement, puis projet.
- **L'environnement porte les valeurs qui diffèrent entre prod et staging** (URL publique de l'app, clés d'API de test). Le projet garde ce qui est vraiment identique partout.
- **Un secret de projet atteint aussi le staging,** construit depuis des branches moins relues. Le panel le rappelle à la création d'un secret de projet, et propose de le définir par environnement.
- **Les variables simples partagées sont héritées par tous les services, sauf les bases créées depuis un template.** Une base n'accepte que ses réglages typés, voir [templates.md](templates.md#principe). Un site statique reçoit les variables de build partagées : elles alimentent son build, pas le container nginx.
- **Les secrets partagés ne vont qu'aux services qui les référencent.** Une image tierce (un Redis, un outil d'admin) ne reçoit pas la clé Stripe juste parce qu'elle est dans le projet. Le service déclare le nom du secret, et la valeur est cherchée sur le service, puis l'environnement, puis le projet.
- **Un nom est soit une variable, soit un secret, à tous les niveaux.** Sinon une variable simple de service pourrait remplacer un secret partagé, et sa valeur partirait en clair dans l'historique des révisions.
- **La révision note d'où vient chaque valeur** (projet, environnement ou service), pour l'affichage et l'audit. Elle garde la valeur résolue : un rollback restaure une variable partagée telle qu'elle était, même si elle a changé depuis.

### Modifier une valeur partagée

- **Variable simple ou de build :** avant de confirmer, le panel liste les services touchés, environnement par environnement, prod comprise. Les services qui remplacent la valeur à un niveau plus précis n'en font pas partie. La confirmation crée une révision par service (après un rebuild pour une variable de build), puis la déploie.
- **Secret :** les containers gardent l'ancienne valeur jusqu'à leur prochain déploiement. Le panel propose par défaut de redéployer tout de suite les services qui le référencent.
- **Supprimer un secret encore utilisé est refusé** tant qu'un déploiement actif le référence. Le panel liste les services concernés.

## Rollback

- **Un rollback redéploie une ancienne révision, par son digest, sans rebuild.** Les variables simples et de build reviennent avec elle. Les secrets prennent leur valeur actuelle.
- **Vérifié avant de commencer :** l'artefact existe encore, et chaque secret nommé dans la révision existe, sur le service, l'environnement ou le projet. Sinon le rollback est refusé, avec la liste de ce qui manque.
- **Par service.** Revenir en arrière sur un backend ne touche pas le frontend qui appelle son API. L'état d'un environnement entier à une date donnée pourra se reconstruire plus tard depuis l'historique des déploiements, sans nouvelle entité.
- **Les données ne reviennent jamais en arrière.** Après une migration, l'ancien code tourne sur le nouveau schéma. Le panel avertit avant le rollback d'un service qui a un volume.
- **La version majeure d'une base créée depuis un template ne change pas en 0.1,** donc un rollback ne peut pas la croiser. Voir [12-plus-tard.md](12-plus-tard.md).
- **Le déploiement automatique continue après un rollback :** le prochain push sur la branche configurée redéploie.

## Rétention et purge

- **Immuable ne veut pas dire éternel.** Les images construites n'existent que dans le dockerd local, et le disque du VPS est limité.
- **L'engine garde les artefacts des 10 dernières révisions de chaque service.** Au-delà, la révision reste dans l'historique mais n'est plus rollbackable, et le panel l'affiche.
- **L'engine ne supprime que les artefacts qu'il ne retient plus,** jamais de `docker image prune` à l'aveugle. Si un artefact retenu disparaît quand même (un `docker image prune -a` lancé à la main), il marque la révision non rollbackable et alerte.
- **Une révision peut être purgée définitivement :** son artefact et les valeurs de ses variables sont effacés, la ligne reste dans l'historique. Pour un secret fuité dans un bundle statique, ou une valeur sensible saisie par erreur en variable simple.
- **Une purge n'atteint pas les backups déjà envoyés :** Brume ne peut rien supprimer sur S3, voir [07-secrets-et-backups.md](07-secrets-et-backups.md#le-stockage). Un secret fuité se révoque chez son émetteur, il ne s'efface pas.

## Cache de build

- **Un cache par environnement,** pas par projet. Sinon un build de staging, lancé depuis une branche moins relue, pourrait empoisonner les caches npm ou pip que lit ensuite le build de prod.
