# État, S3 et restauration

Où vit l'état de Brume, comment il part sur S3, et comment on remonte un serveur perdu. L'event sourcing lui-même est décrit dans [14-architecture-logicielle.md](14-architecture-logicielle.md#event-sourcing).

## Principe

- **Le VPS est jetable.** L'état de Brume et les artefacts sont sur S3, et un nouveau VPS se remonte depuis le bucket.
- **SQLite fait foi pendant l'exécution, S3 quand le disque est perdu.** SQLite contient tout, car l'état de contrôle tient en quelques Mo. Brume ne va jamais chercher sur S3 une donnée qui manquerait en local.
- **L'envoi vers S3 est toujours asynchrone,** jamais sur le chemin d'une commande. Une panne de S3 ne bloque ni les commandes, ni les déploiements, ni les webhooks.
- **Le prix : la dernière seconde d'état** peut se perdre si le VPS disparaît.
- **Les données des apps ne sont sur S3 que sous forme de dumps,** voir [07-secrets-et-backups.md](07-secrets-et-backups.md#backups). Une restauration les ramène au dernier dump. Les volumes hors templates ne sont pas sauvegardés en 0.1.

## Ce qui va sur S3

Un seul bucket, avec versioning ou Object Lock :

```
log/<génération>/<seq>.seg    segments d'événements
blobs/<id>                    valeurs des variables et des secrets, secrets de la plateforme
keys/master.<n>.age           clé maître, chiffrée pour les clés age de l'admin
artifacts/sha256/<digest>     couches des images, archives des sites statiques
backups/…                     dumps des bases
```

- **Les secrets de la plateforme qui doivent survivre à une restauration sont des blobs :** la clé privée de la GitHub App, le secret HMAC du webhook. L'allowlist et les passkeys de l'admin sont dans le log.
- **Restent hors de S3 :** les sessions, la télémétrie et les logs des apps, le cache de build, l'état observé.

## Le bucket

- **Obligatoire dès l'install.** `brume install` demande le bucket et ses credentials, et ne continue pas sans eux : la clé maître part sur S3 avant la fin de l'install. Voir [08-hote-et-installation.md](08-hote-et-installation.md#brume-install).
- **Testé à la configuration :** écriture, lecture, et refus d'écraser un objet existant (`If-None-Match`). Object Lock et versioning se vérifient avec [`brume verify`](#brume-verify).
- **Il ne se change que sur l'hôte, en root,** comme les clés age ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement)), jamais depuis le panel. Une session volée ne peut ni rediriger l'état, ni couper la restauration sans bruit.
- **Tout changement déclenche une alerte.**

## Chiffrement

- **Tout est chiffré, sans option.**
- **L'état** (segments et blobs) **est chiffré par la clé maître,** avec un chiffrement authentifié (AEAD, par exemple XChaCha20-Poly1305). Sans la clé, personne ne peut fabriquer un segment, même avec les credentials S3. La clé de l'objet fait partie des données authentifiées : un segment ne peut pas être déplacé sous un autre nom.
- **Chiffrer l'état avec age seul ne suffirait pas :** n'importe qui avec la clé publique fabrique un fichier valide.
- **Les artefacts sont chiffrés par une clé dérivée de la clé maître,** que l'engine transmet à l'agent. L'agent ne peut donc pas lire l'état.
- **Les backups restent chiffrés avec age,** pour les clés de l'admin. Le serveur ne peut pas les lire. Voir [07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement).
- **Les empreintes des backups et des artefacts sont dans le log.** Leur intégrité se vérifie contre le log, qui est authentifié.
- **Chaque objet chiffré commence par un en-tête en clair :** version du format, algorithme et identifiant de la clé. Les objets sur S3 ne changent jamais : sans cet en-tête, une rotation de la clé maître ou un changement d'algorithme obligerait à deviner quelle clé ouvre quel objet.

## La clé maître et les clés age

- **La clé maître est déposée sur S3, chiffrée avec age** pour chaque clé publique de l'admin. Les destinataires se changent sur l'hôte ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement)). Chaque changement dépose un nouvel objet `keys/master.<n>.age`.
- **La clé age privée de l'admin devient la seule chose à garder hors du serveur.** Si elle est perdue, rien n'est restaurable : ni l'état, ni les secrets, ni les backups.
- **L'install conseille plusieurs destinataires,** par exemple une clé papier et une YubiKey (`age-plugin-yubikey`).
- **L'admin prouve qu'il a gardé sa clé avant la fin de l'install.** Brume chiffre un code pour chaque clé publique ; l'admin le déchiffre sur son poste et le recopie.
- **Le panel rappelle régulièrement de faire un exercice de restauration** avec `brume verify`.
- **La rotation de la clé maître viendra plus tard.** Grâce à l'identifiant de clé, les anciens objets resteront lisibles avec l'ancienne clé. La rotation ne protégera pas les segments déjà envoyés.

## Les credentials

- **Un seul credential, partagé par l'engine et l'agent : lecture et écriture, jamais de suppression.** Brume ne peut toujours rien supprimer sur S3 ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-stockage)).
- **La lecture ne révèle rien.** Tout est chiffré, et les backups le sont pour une clé que le serveur n'a pas. Elle permet à l'engine de vérifier un envoi douteux, voir [L'envoi](#lenvoi).
- **Le log, les blobs et les clés n'expirent jamais.** Aucune règle de cycle de vie ne doit viser `log/`, `blobs/` ou `keys/` : un blob encore utilisé, écrit il y a deux ans, ressemble à un blob purgé. L'ensemble pèse quelques Mo. Les `backups/` expirent par les règles de [07-secrets-et-backups.md](07-secrets-et-backups.md#le-stockage).
- **Brume ne supprime jamais d'artefact sur S3.** La rétention des 10 dernières révisions ([13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge)) ne vaut que pour dockerd. Une couche commune à plusieurs images n'est stockée qu'une fois, et le panel affiche la taille de `artifacts/`.
- **L'admin peut ajouter une règle d'expiration sur `artifacts/`.** Une restauration reconstruit alors depuis son commit un artefact expiré, sans garantie d'obtenir exactement la même image.

## L'envoi

- **Un abonné de l'engine** lit le log depuis son point de reprise, groupe les événements d'une seconde au plus en un segment, le chiffre et l'envoie.
- **Chaque segment commence juste après le précédent,** et porte le nom de la position de son premier événement.
- **Écriture avec `If-None-Match: *` et une somme de contrôle.** Un objet existant n'est jamais écrasé.
- **Si la réponse se perd,** l'engine relit le segment et le compare au sien. S'il est identique, l'envoi a réussi. S'il diffère, un autre serveur écrit dans ce bucket : voir la section suivante.
- **Le retard d'envoi est mesuré** ([17-telemetrie.md](17-telemetrie.md#métriques)). Au-delà de 5 minutes, une alerte part, et le panel affiche l'heure du dernier envoi.
- **L'agent envoie les artefacts** après chaque build réussi. Une révision dont l'artefact n'est pas encore sur S3 reste rollbackable en local, mais pas après une perte du VPS.

## Deux serveurs sur le même bucket

- **Les segments contigus rendent la détection fiable.** Deux serveurs qui divergent écrivent forcément le même nom de segment lors de leur première écriture divergente, et le second reçoit une erreur 412.
- **Le perdant passe en lecture seule :** il refuse les commandes, arrête l'envoi et alerte.
- **Limite :** un serveur qui n'écrit plus rien ne l'apprend jamais. Ses apps continuent de tourner, mais sans trafic public : voir [Restauration](#restauration).
- **`If-None-Match` n'existe pas chez tous les fournisseurs.** Brume le teste à la configuration du bucket. S'il n'est pas supporté, Brume avertit que cette détection ne fonctionne pas.

## Restauration

`brume restore` reprend les étapes de `brume install` ([08-hote-et-installation.md](08-hote-et-installation.md#brume-install)), sauf la création des clés, puis :

1. **L'admin donne les credentials du bucket et sa clé age privée.** Il la tape ; Brume la garde en mémoire le temps de la restauration, sans jamais l'écrire sur le disque.
2. **Brume déchiffre la clé maître** et la range dans les systemd credentials du nouvel hôte.
3. **Il télécharge la génération courante du log.** Il vérifie chaque segment (AEAD) et la chaîne de hash, puis reconstruit SQLite : log, blobs, index.
4. **Il refait l'entrée et l'administration** selon les modes choisis ([14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration)) :
   - avec Cloudflare, il crée un nouveau tunnel et repointe le CNAME wildcard, comme à l'install : si l'ancien serveur revient, il ne reçoit plus le trafic public ;
   - en entrée directe, l'admin repointe lui-même le DNS vers la nouvelle IP ;
   - avec Tailscale, le nouveau serveur rejoint le tailnet comme un nouvel appareil, et l'admin retire l'ancien.
5. **Il charge dans dockerd les artefacts des révisions en service,** après avoir vérifié leurs empreintes contre le log. Un artefact absent est reconstruit depuis son commit.
6. **Il restaure le dernier dump de chaque base,** déchiffré avec la clé age. Pour chaque base, il affiche l'écart entre l'heure du dump et la fin du log. Il propose de remettre les services qui l'utilisent à la révision qui tournait à l'heure du dump : sinon, un code plus récent peut attendre un schéma que la base restaurée n'a pas.
7. **Il démarre les services.** L'agent fait converger Docker vers l'état voulu, et l'engine reprend l'envoi juste après le dernier segment.

- **L'admin se reconnecte au panel :** les sessions ne survivent pas à une restauration. Son allowlist et ses passkeys sont dans l'état.
- **La restauration est testée chaque nuit sur une vraie VM :** installer, écrire, effacer le disque, restaurer. Voir [16-tests.md](16-tests.md#sur-une-vraie-vm).

## `brume verify`

- **Se lance depuis le poste de l'admin,** avec des credentials en lecture et sa clé age privée. Elle n'écrit rien.
- **Elle vérifie que :**
  - chaque segment se déchiffre et la chaîne de hash est complète ;
  - une restauration reconstruit un `World` valide ;
  - chaque backup se déchiffre et correspond à l'empreinte notée dans le log ;
  - les artefacts des révisions retenues existent ;
  - aucune règle de cycle de vie ne vise `log/`, `blobs/` ou `keys/`, si ses credentials permettent de lire ces règles.
- **C'est le seul moyen de savoir, avant d'en avoir besoin, que le bucket est restaurable.** Elle détecte une mauvaise règle de cycle de vie, un Object Lock absent ou une clé perdue.

## Ce que la purge n'atteint pas

- **Une purge ([13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge)) n'atteint pas S3.** Les blobs purgés y restent pour toujours, chiffrés par la clé maître. C'est la règle déjà acceptée pour les backups.
- **Un secret fuité se révoque chez son émetteur :** l'effacer ne suffit pas.

## Risques acceptés

| Risque | Pourquoi on l'accepte |
|---|---|
| La dernière seconde d'état se perd avec le VPS. | En échange, une panne de S3 ne bloque rien. |
| Un ancien VPS qui revient garde ses apps en marche, sans trafic public. Leurs tâches de fond (crons, files) peuvent encore écrire ailleurs. | C'est rare, et le nouveau tunnel coupe le trafic public. |
| L'engine est un point unique de défaillance. Sans lui, les apps tournent, mais rien ne se déploie. | Une bascule automatique demanderait du consensus, pas seulement S3. |
| Un attaquant root peut effacer la seconde pas encore envoyée et arrêter l'envoi. L'alerte partirait du serveur qu'il contrôle. | Ce qui est déjà envoyé reste intact. Une alerte externe viendra plus tard, voir [12-plus-tard.md](12-plus-tard.md). |
| Les fournisseurs S3 n'ont pas tous les mêmes fonctions (`If-None-Match`, Object Lock), et une restauration télécharge toutes les images. | Brume teste le bucket à la configuration. Le coût de sortie dépend du fournisseur. |
| Les artefacts s'accumulent sur S3, faute de droit de suppression. | Les couches communes ne sont stockées qu'une fois, et l'admin peut ajouter une règle d'expiration. |
