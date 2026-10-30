# Tests

Comment on teste Brume : d'abord une simulation déterministe de tout le système, puis des tests de conformité et des tests sur une vraie VM. Le frontend a ses propres tests : voir [front-architecture.md](front-architecture.md#tests).

## Les niveaux

| Niveau | Ce qu'il teste | Quand |
|---|---|---|
| Fonctions pures | `decide`, `evolve`, `plan`, config des containers, upcasters | chaque PR |
| Simulation | tout Brume, avec des pannes | chaque PR, et des heures chaque nuit |
| Conformité | chaque faux se comporte comme le vrai | chaque PR, un vrai fournisseur S3 chaque nuit |
| VM réelle | install, templates, réseau, mise à jour, restauration | chaque nuit |

## Les fonctions pures

- **`decide` et `evolve` :** « étant donné ces événements, quand cette commande, alors ces événements, ou ce rejet ».
- **`plan` :** tests de propriété (proptest). Appliquer le plan à l'état observé donne l'état voulu.
- **Config des containers :** un test vérifie la config complète de chaque type de container. Voir [10-developpement.md](10-developpement.md#règles-de-code-rust).
- **Upcasters :** chaque version de chaque événement se lit encore.

## La simulation déterministe

### Le principe

Tout Brume tourne dans un seul thread, avec un temps simulé. Un seul nombre, la graine, décide de tout le reste : l'ordre des événements, les pannes et la charge. Un bug trouvé se rejoue à l'identique avec sa graine.

C'est possible parce que l'engine et l'agent n'ont aucune I/O : voir [14-architecture-logicielle.md](14-architecture-logicielle.md#le-cœur-sans-io).

### Le monde simulé

- **Un engine et un agent en 0.1.** Plusieurs agents viendront avec le placement sur plusieurs machines ([14-architecture-logicielle.md](14-architecture-logicielle.md#les-machines)).
- **Des faux :** `SimDocker`, `SimS3`, `SimLog`, et un réseau simulé entre l'engine et les agents.
- **Une charge aléatoire, qui joue l'admin et GitHub :** créations de projets, déploiements, rollbacks, changements de secrets, purges, webhooks en double, sessions qui expirent.

```rust
fn simulate(seed: u64) -> Result<(), Violation> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let faults = FaultProfile::draw(&mut rng);
    let mut sim = Sim::new(&mut rng, faults); // engine, agents, SimDocker, SimS3, charge
    while sim.now() < DURATION {
        sim.step(&mut rng); // prochain événement, ou une panne
        sim.check_safety()?;
    }
    sim.heal();
    sim.run_until_quiet(&mut rng);
    sim.check_liveness()
}
```

- **Une file d'événements triée par temps simulé.** Quand deux événements tombent au même instant, leur ordre est tiré au hasard : on explore les entrelacements possibles.
- **Un crash efface la mémoire et garde le disque.** Au redémarrage, le composant se reconstruit depuis son disque, comme en vrai.
- **Le chiffrement est un effet comme les autres.** En simulation, il est déterministe ; le vrai se vérifie en conformité.

### Les pannes

| Cible | Pannes |
|---|---|
| engine | crash à n'importe quel moment (après le commit, avant l'envoi, pendant un PUT), disque plein |
| agent | crash, déconnexion, messages retardés |
| S3 | erreurs, timeouts, écriture réussie mais réponse perdue, longue panne, deuxième serveur sur le même bucket |
| Docker | création refusée, container qui meurt, OOM, redémarrage de dockerd, image supprimée à la main, container créé à la main |
| horloge | sauts de l'heure murale |

Chaque graine tire un profil de pannes : beaucoup de crashs, une longue panne S3, ou presque rien. Les combinaisons rares sortent plus souvent qu'avec un taux de panne uniforme.

### Les invariants

**À chaque pas :**

- aucun container hors de `policy.toml` : `SimDocker` vérifie chaque création ;
- **aucune valeur de secret** dans les événements, dans les objets S3 déchiffrés ni dans la télémétrie. La charge connaît toutes les valeurs qu'elle a posées : il suffit de les chercher ;
- aucun email d'admin dans un événement : il ne doit apparaître que dans un blob ;
- le log sur S3 est toujours un préfixe du log local, et la chaîne de hash est valide ;
- la position de l'état voulu appliqué par un agent ne recule jamais ;
- un déploiement enregistre les versions de secrets que `SimDocker` a réellement injectées ;
- aucun artefact encore retenu n'est supprimé ;
- aucune panique.

**Après la fin des pannes :**

- sur chaque machine, ce que Docker fait tourner correspond à l'état voulu ;
- S3 a rattrapé son retard ;
- une restauration depuis S3 redonne le même `World` ;
- rejouer le log local redonne le même `World`.

### Reproduire et faire tourner

- **Un échec affiche sa graine et sa trace.** `BRUME_SEED=0x… cargo test -p brume-sim` le rejoue à l'identique.
- **La trace d'une graine s'exporte vers un Jaeger local,** horodatée en temps simulé. Voir [17-telemetrie.md](17-telemetrie.md#en-développement-et-en-simulation).
- **À chaque PR :** quelques minutes de graines aléatoires, plus les graines qui ont déjà trouvé un bug, gardées dans le repo.
- **Chaque nuit :** des heures de graines.
- **Un test de déterminisme** lance deux fois la même graine et compare l'empreinte des traces. Il échoue dès qu'un `HashMap` ou un `now()` s'est glissé dans le code.
- **Un log de référence de chaque release depuis la dernière compaction** est rejoué par le code courant : les upcasters et `Genesis` restent corrects.

### Pourquoi pas madsim ou turmoil

Ils simulent le temps et le réseau de tokio, en remplaçant tokio au build. Docker, SQLite et la plupart de nos I/O resteraient à simuler nous-mêmes, et le déterminisme dépendrait de leur façon d'intercepter tokio. Avec des cœurs sans I/O, il est garanti par construction.

## Les tests de conformité

- **Une même suite tourne contre chaque faux et contre le vrai :** `SimLog` et rusqlite, `SimDocker` et Docker en CI, `SimS3` et MinIO. Un vrai fournisseur S3 s'y ajoute chaque nuit.
- **Elle couvre ce dont la simulation dépend :** `If-None-Match`, les erreurs de Docker, l'ordre de ses événements, la durabilité d'un commit SQLite.
- **Un faux qui diverge du vrai fait échouer la suite.** Un faux trop optimiste rendrait la simulation inutile.

## Sur une vraie VM

Chaque nuit, sur une VM Debian 12 et une VM Ubuntu 24.04 neuves, avec des comptes Cloudflare et Tailscale de test :

- `install.sh`, puis le déploiement de chaque template, avec ses vérifications ([templates.md](templates.md#vérifications-au-déploiement)) ;
- les règles réseau : un container ne joint ni le tailnet, ni les plages privées, ni les endpoints de métadonnées ([03-reseau.md](03-reseau.md#règles-nftables)) ;
- une mise à jour de Brume par l'updater ;
- une restauration : écrire, effacer le disque, lancer `brume restore`, comparer l'état.
