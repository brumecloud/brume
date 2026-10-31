# Formats sur S3

**Proposition, à valider avant d'écrire `brume-engine`.** Un objet écrit sur S3 ne change jamais : chaque format décrit ici devra rester lisible par toutes les versions suivantes de Brume. Ce qui va sur S3, la restauration et `brume verify` sont dans [15-etat-et-restauration.md](15-etat-et-restauration.md).

## Principes

- **Des formats documentés, jamais un format interne de Rust.** Pas de bincode ni de format propre à un crate : un autre outil doit pouvoir relire le bucket à partir de ce document.
- **Tout est versionné.** Chaque objet chiffré porte la version de son format. Un lecteur refuse proprement une version qu'il ne connaît pas, et garde le code de lecture de toutes les versions déjà écrites, puisqu'aucun objet n'est jamais réécrit.
- **Des primitives standard, aucune construction maison.** Le chiffrement reprend celle de [age](https://age-encryption.org/v1) : une clé par objet, dérivée par HKDF, puis ChaCha20-Poly1305 par morceaux de 64 Kio.

## Noms des objets

```
log/<génération>/<seq>.seg           génération sur 10 chiffres, seq sur 20 chiffres
blobs/<id>                           128 bits aléatoires, en hexadécimal
keys/master.<n>.age                  n sur 10 chiffres, à partir de 1
artifacts/<nom>                      HMAC-SHA256 du digest, voir plus bas
backups/<service>/<horodatage>.age   identifiant du service, heure UTC (20261031T040000Z)
```

- **Les nombres sont complétés par des zéros.** L'ordre d'une liste S3, alphabétique, est alors l'ordre du log.
- **Le nom d'un artefact ne révèle pas son contenu.** Nommé par son digest, comme le prévoit aujourd'hui [15-etat-et-restauration.md](15-etat-et-restauration.md#ce-qui-va-sur-s3) (`artifacts/sha256/<digest>`), un objet dirait à quiconque lit le bucket quelles couches publiques tu utilises. La proposition nomme l'objet par un HMAC du digest, avec une clé dérivée de la clé maître. Les couches communes restent stockées une seule fois. En contrepartie, relier un objet à une image demande la clé.

## Les objets chiffrés par la clé maître

Les segments du log, les blobs et les artefacts.

- **Une sous-clé par usage,** dérivée de la clé maître par HKDF-SHA256 : `brume/state` (segments et blobs), `brume/artifacts` (artefacts), `brume/artifact-names` (noms des artefacts). L'engine ne transmet à l'agent que les deux dernières : l'agent ne peut pas lire l'état.

### L'en-tête

En clair, 28 octets, entiers en big-endian :

| Champ | Taille | Contenu |
|---|---|---|
| magic | 4 octets | `BRUM` |
| version | 2 octets | version du format, `1` |
| sorte | 1 octet | `1` segment, `2` blob, `3` artefact |
| algorithme | 1 octet | `1` : HKDF-SHA256, puis ChaCha20-Poly1305 par morceaux |
| identifiant de clé | 4 octets | génération de la clé maître, pour la rotation |
| sel | 16 octets | aléatoire, propre à l'objet |

### Le chiffrement

- **La clé de l'objet** est HKDF-SHA256(sous-clé, sel, info = en-tête complet ‖ nom de l'objet). Modifier l'en-tête ou déplacer l'objet sous un autre nom donne une autre clé, et le déchiffrement échoue. C'est ainsi que la clé de l'objet fait partie des données authentifiées.
- **La charge utile est découpée en morceaux de 64 Kio,** chacun chiffré par ChaCha20-Poly1305 avec un nonce de 12 octets : un compteur sur 11 octets, puis un octet qui vaut `1` pour le dernier morceau et `0` sinon. Un objet tronqué, ou dont les morceaux sont réordonnés, ne se déchiffre pas.
- **Un gros artefact se chiffre et se déchiffre en flux,** sans tenir en mémoire.
- **Pas de compression en version 1.** Les couches d'image sont déjà compressées. Un nouvel algorithme pourra l'ajouter.

## Le contenu d'un segment

- **Une première ligne décrit le segment :** `{"segment":1,"generation":…,"first_seq":…,"count":…}`. Le lecteur vérifie qu'elle correspond au nom de l'objet.
- **Puis une ligne par événement, dans l'ordre du log.** Chaque ligne est le JSON compact de l'événement, octet pour octet tel qu'il est stocké dans SQLite : la chaîne de hash porte sur ces octets ([14-architecture-logicielle.md](14-architecture-logicielle.md#la-chaîne-de-hash)). Le JSON compact ne contient aucun retour à la ligne : ceux des chaînes sont échappés.
- **Les octets stockés contiennent `prev_hash`, pas `hash`,** qui ne peut pas faire partie de ce qu'il hache. Le lecteur recalcule le hash de chaque ligne et vérifie qu'il correspond au `prev_hash` de la suivante.

## Les autres objets

- **Un blob** est la valeur brute, sans enveloppe. Le sens de la valeur vient de l'événement qui le référence.
- **La clé maître** (`keys/`) est un fichier age standard, chiffré pour chaque clé publique de l'admin. Il contient `{"format":1,"key_id":…,"key":"<32 octets en base64>"}`.
- **Un backup** est un fichier age standard, chiffré pour les clés de l'admin. Il contient le dump tel que l'outil de la base le produit. L'empreinte SHA-256 de l'objet envoyé est notée dans le log.

## À trancher dans cette proposition

- **La taille des morceaux :** 64 Kio, comme age.
- **Les noms d'artefacts par HMAC :** la confidentialité contre la lisibilité du bucket au débogage.
- **Le format exact de la ligne d'en-tête d'un segment.**
