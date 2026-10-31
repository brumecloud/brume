# Formats sur S3

Les noms et le format des objets que Brume écrit sur S3, et les règles de celui qui les relit. Décidé le 31 octobre. Un objet écrit sur S3 ne change jamais : chaque format décrit ici devra rester lisible par toutes les versions suivantes de Brume. Ce qui va sur S3, la restauration et `brume verify` sont dans [15-etat-et-restauration.md](15-etat-et-restauration.md).

## Principes

- **Des formats documentés, jamais un format interne de Rust.** Pas de bincode ni de format propre à un crate : un autre outil doit pouvoir relire le bucket à partir de ce document.
- **Tout est versionné.** Chaque objet porte la version de son format. Un lecteur refuse proprement une version qu'il ne connaît pas, et garde le code de lecture de toutes les versions déjà écrites, puisqu'aucun objet n'est jamais réécrit.
- **Seul ce qui est sensible est chiffré :** les blobs, la clé maître et les backups. Le log part en clair : il ne contient ni valeur ni email ([14-architecture-logicielle.md](14-architecture-logicielle.md#ce-qui-nentre-jamais-dans-un-événement)). Son intégrité tient à la chaîne de hash. Des événements authentifiés viendront plus tard : voir [12-plus-tard.md](12-plus-tard.md).
- **Des primitives standard, aucune construction maison :** XChaCha20-Poly1305 pour les blobs, [age](https://age-encryption.org/v1) pour la clé maître et les backups.

## Noms des objets

```
log/<génération>/<seq>.seg           génération sur 10 chiffres, seq sur 20 chiffres
blobs/<id>                           128 bits aléatoires, en hexadécimal minuscule
keys/master.<n>.age                  n sur 10 chiffres, à partir de 1
backups/<service>/<horodatage>.age   identifiant du service, heure UTC (20261031T040000Z)
```

- **Les nombres sont complétés par des zéros.** L'ordre d'une liste S3, alphabétique, est alors l'ordre du log.
- **Le log commence à la position 1, dans la génération 1 :** son premier segment est `log/0000000001/00000000000000000001.seg`. Le `Genesis` d'une génération prend la position qui suit le dernier événement de la précédente.
- **Ces noms sont relatifs à la racine de Brume.** L'admin peut placer Brume dans un dossier d'un bucket existant ([15-etat-et-restauration.md](15-etat-et-restauration.md#le-bucket)). Le dossier précède alors chaque nom, mais n'entre jamais dans le chiffrement d'un blob : des objets déplacés vers un autre bucket ou un autre dossier restent lisibles.
- **`n` compte les dépôts de la clé maître, pas les clés.** Un changement des destinataires age dépose un nouveau fichier qui contient la même clé. L'identifiant de la clé est dans le fichier.
- **Les images construites et les archives des sites statiques ne vont pas sur S3** en 0.1 : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#ce-qui-va-sur-s3).

## Le contenu d'un segment

- **Un segment est un fichier texte en clair,** en UTF-8, une ligne par enregistrement.
- **Une première ligne décrit le segment :** `{"segment":1,"generation":…,"first_seq":…,"count":…}`. `segment` est la version de ce format. Le lecteur vérifie que la ligne correspond au nom de l'objet, et que le segment contient bien `count` événements.
- **Puis une ligne par événement, dans l'ordre du log.** Chaque ligne est le JSON compact de l'événement, octet pour octet tel qu'il est stocké dans SQLite : la chaîne de hash porte sur ces octets ([14-architecture-logicielle.md](14-architecture-logicielle.md#la-chaîne-de-hash)). Le JSON compact ne contient aucun retour à la ligne : ceux des chaînes sont échappés.
- **Chaque événement porte le hash du précédent** dans `prev_hash`, en hexadécimal minuscule. `prev_hash` reste un champ de premier niveau dans toutes les versions d'un événement : le lecteur vérifie la chaîne sans les upcasters. Celui du premier événement de la génération 1 vaut 64 zéros.
- **Les octets stockés contiennent `prev_hash`, pas `hash`,** qui ne peut pas faire partie de ce qu'il hache. Le lecteur recalcule le hash de chaque ligne et vérifie qu'il correspond au `prev_hash` de la suivante.
- **Pas de compression en version 1.** Les segments sont petits.

## Les blobs

Un blob contient une valeur sensible : la valeur d'une variable ou d'un secret, un secret de la plateforme, un email. Il est chiffré par la clé maître.

- **La clé** est une sous-clé dérivée de la clé maître par HKDF-SHA256, avec un sel vide et l'info `brume/blobs`. L'agent ne reçoit aucune clé dérivée de la clé maître.
- **Un blob tient en mémoire :** il est chiffré en un seul appel de XChaCha20-Poly1305.
- **Il commence par un en-tête en clair,** 35 octets, entiers en big-endian :

| Champ | Taille | Contenu |
|---|---|---|
| magic | 4 octets | `BRUM` |
| version | 2 octets | version du format, `1` |
| algorithme | 1 octet | `1` : XChaCha20-Poly1305 |
| identifiant de clé | 4 octets | génération de la clé maître, pour la rotation |
| nonce | 24 octets | aléatoire, propre au blob |

- **Les données authentifiées** sont l'en-tête complet, suivi du nom du blob, sans le dossier. Modifier l'en-tête ou déplacer le blob sous un autre nom fait échouer le déchiffrement.
- **La charge utile est la valeur brute,** sans enveloppe. Le sens de la valeur vient de l'événement qui la référence.
- **Un blob part sur S3 avant le segment qui le référence.**

## Les autres objets

- **La clé maître** (`keys/`) est un fichier age standard, chiffré pour chaque clé publique de l'admin. Il contient `{"format":1,"key_id":…,"key":"<32 octets en base64>"}`.
- **Un backup** est un fichier age standard, chiffré pour les clés de l'admin. Il contient le dump tel que l'outil de la base le produit. L'empreinte SHA-256 de l'objet envoyé est notée dans le log. Un gros dump part en plusieurs parties : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#lenvoi).

## Les règles du lecteur

Elles valent pour `brume restore`, `brume verify`, et l'engine quand il relit un envoi douteux.

- **Le lecteur lit la version courante de chaque objet.** Brume n'écrit jamais deux fois sous le même nom. Le versioning du bucket garde l'original d'un objet écrasé, à récupérer à la main.
- **Il suit les noms attendus, jamais la liste du bucket.** Le segment suivant porte la position `first_seq + count`, dans la même génération.
- **Une restauration ne lit que la dernière génération,** celle dont le numéro est le plus haut : son `Genesis` contient tout le `World` ([14-architecture-logicielle.md](14-architecture-logicielle.md#générations-et-compaction)). `brume verify` lit toutes les générations, et vérifie que chaque `Genesis` porte le hash du dernier événement de la précédente.
- **Il s'arrête au premier segment illisible :** il ne se lit pas, sa ligne d'en-tête ne correspond pas à son nom, ou il casse la chaîne de hash. Une restauration repart alors du dernier événement valide, et prévient l'admin. L'engine ouvre ensuite une nouvelle génération, numérotée au-dessus de la plus haute présente dans le bucket : le nom suivant est déjà pris.
- **Un blob absent ou qui ne se déchiffre pas n'arrête pas la restauration.** La valeur manque, et la restauration liste les variables et les secrets à saisir de nouveau.
- **Pour la clé maître,** le lecteur essaie chaque fichier de `keys/` que la clé age de l'admin ouvre. L'en-tête d'un blob donne l'identifiant de la clé qui l'ouvre. En 0.1, il n'y a que la clé 1.
