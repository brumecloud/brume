# État, S3 et restauration

Où vit l'état de Brume, comment il part sur S3, et comment on remonte un serveur perdu. L'event sourcing lui-même est décrit dans [14-architecture-logicielle.md](14-architecture-logicielle.md#event-sourcing).

## Principe

- **Le VPS est jetable.** L'état de Brume et les dumps des bases sont sur S3. Un nouveau VPS se remonte depuis le bucket, et reconstruit les images depuis leur commit.
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
backups/…                     dumps des bases
```

- **Les secrets de la plateforme qui doivent survivre à une restauration sont des blobs :** la clé privée de la GitHub App, le secret HMAC du webhook. L'allowlist et les passkeys de l'admin sont dans le log.
- **Restent hors de S3 :** les sessions, la télémétrie et les logs des apps, le cache de build, l'état observé, les images construites, les archives des sites statiques et les images tirées d'un registry.
- **Les images construites et les archives des sites statiques ne sont pas copiées.** Brume n'est pas un registry : elles ne vivent que dans le dockerd local, qui garde celles des 10 dernières révisions ([13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge)). Une restauration les reconstruit depuis leur commit. Décidé le 31 octobre.
- **Les images tirées d'un registry ne sont pas copiées.** Une restauration les retélécharge par leur digest. Si le registry les a supprimées ou s'il est injoignable, le service ne redémarre pas.
- **Le format des objets** (noms, en-têtes, chiffrement) et les règles de leur lecture sont dans [18-formats-s3.md](18-formats-s3.md).

## Le bucket

- **Obligatoire dès l'install.** `brume install` demande le bucket et ses credentials, et ne continue pas sans eux : la clé maître part sur S3 avant la fin de l'install. Voir [08-hote-et-installation.md](08-hote-et-installation.md#brume-install).
- **Un dossier facultatif** permet de partager un bucket existant. Brume écrit alors tout sous ce dossier, qui n'entre jamais dans le chiffrement : voir [18-formats-s3.md](18-formats-s3.md#noms-des-objets).
- **Vide à l'install :** un ancien log de Brume s'y mélangerait au nouveau.
- **Testé à la configuration :** écriture, lecture, refus d'écraser un objet existant (`If-None-Match`, y compris à l'assemblage d'un envoi en plusieurs parties), et échec d'une suppression, d'une règle de cycle de vie et d'une suspension du versioning. Un credential qui réussit l'une de ces trois opérations est refusé. Object Lock et versioning se vérifient avec [`brume verify`](#brume-verify).
- **Il ne se change que sur l'hôte, en root,** comme les clés age ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement)), jamais depuis le panel. Une session volée ne peut ni rediriger l'état, ni couper la restauration sans bruit.
- **Tout changement déclenche une alerte.**
- **Une nouvelle racine reçoit tout le log,** renvoyé depuis SQLite, ses blobs et un nouveau dépôt de la clé maître. Les backups déjà envoyés restent dans l'ancienne racine.

## Chiffrement

- **Seul ce qui est sensible est chiffré.** Décidé le 31 octobre.
- **Les blobs sont chiffrés par la clé maître,** avec XChaCha20-Poly1305 : les valeurs des variables et des secrets, les secrets de la plateforme, les emails. Le format est dans [18-formats-s3.md](18-formats-s3.md#les-blobs).
- **Le log part en clair.** Il ne contient ni valeur ni email ([14-architecture-logicielle.md](14-architecture-logicielle.md#ce-qui-nentre-jamais-dans-un-événement)). Qui lit le bucket voit la structure de l'infra : les noms des projets et des services, les domaines, les repos.
- **Son intégrité tient à la chaîne de hash :** chaque événement porte le hash du précédent. Elle détecte une corruption, un segment manquant ou modifié. Elle n'arrête pas qui a le credential S3 : il peut la recalculer. Voir [Risques acceptés](#risques-acceptés).
- **L'agent ne reçoit aucune clé dérivée de la clé maître.** Il n'envoie que les backups, chiffrés avec age.
- **Les backups restent chiffrés avec age,** pour les clés de l'admin. Le serveur ne peut pas les lire. Voir [07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement).
- **Les empreintes des backups sont dans le log.** Une restauration vérifie chaque dump contre son empreinte.
- **Chaque blob commence par un en-tête en clair :** version du format, algorithme et identifiant de la clé. Les objets sur S3 ne changent jamais : sans cet en-tête, une rotation de la clé maître ou un changement d'algorithme obligerait à deviner quelle clé ouvre quel blob.

## La clé maître et les clés age

- **La clé maître est déposée sur S3, chiffrée avec age** pour chaque clé publique de l'admin. Les destinataires se changent sur l'hôte ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-chiffrement)). Chaque changement dépose un nouvel objet `keys/master.<n>.age`.
- **La clé age privée de l'admin devient la seule chose à garder hors du serveur.** Si elle est perdue, ni les secrets ni les backups ne sont restaurables.
- **L'install conseille plusieurs destinataires,** par exemple une clé papier et une YubiKey (`age-plugin-yubikey`).
- **L'admin prouve qu'il a gardé sa clé avant la fin de l'install.** Brume chiffre un code pour chaque clé publique ; l'admin le déchiffre sur son poste et le recopie.
- **Le panel rappelle régulièrement de faire un exercice de restauration** avec `brume verify`.
- **La rotation de la clé maître viendra plus tard.** Grâce à l'identifiant de clé, les anciens blobs resteront lisibles avec l'ancienne clé. La rotation ne protégera pas les blobs déjà envoyés.

## Les credentials

- **Un seul credential, partagé par l'engine et l'agent : lecture et écriture, jamais de suppression, ni de droit sur le cycle de vie ou le versioning.** Brume ne peut toujours rien supprimer sur S3 ([07-secrets-et-backups.md](07-secrets-et-backups.md#le-stockage)).
- **Il peut abandonner un envoi en plusieurs parties.** Ce droit ne touche aucun objet déjà stocké.
- **La lecture ne révèle aucun secret.** Les blobs sont chiffrés, et les backups le sont pour une clé que le serveur n'a pas. Le log, lui, montre la structure de l'infra. La lecture permet à l'engine de vérifier un envoi douteux, voir [L'envoi](#lenvoi).
- **Le log, les blobs et les clés n'expirent jamais.** Aucune règle de cycle de vie ne doit viser `log/`, `blobs/` ou `keys/` : un blob encore utilisé, écrit il y a deux ans, ressemble à un blob purgé. L'ensemble pèse quelques Mo. Les `backups/` expirent par les règles de [07-secrets-et-backups.md](07-secrets-et-backups.md#le-stockage).

## L'envoi

- **Un abonné de l'engine** lit le log depuis son point de reprise, groupe les événements d'une seconde au plus en un segment et l'envoie. Les blobs qu'il référence partent avant lui.
- **Chaque segment commence juste après le précédent,** et porte le nom de la position de son premier événement.
- **Écriture avec `If-None-Match: *` et une somme de contrôle.** Un objet existant n'est jamais écrasé.
- **Si la réponse se perd,** l'engine relit le segment et le compare au sien. S'il est identique, l'envoi a réussi. S'il diffère, quelqu'un d'autre a écrit sous ce nom : voir la section suivante.
- **Le retard d'envoi est mesuré** ([17-telemetrie.md](17-telemetrie.md#métriques)). Au-delà de 5 minutes, une alerte part, et le panel affiche l'heure du dernier envoi.
- **L'agent envoie les backups.** Un gros dump part en plusieurs parties (multipart upload) : un envoi simple est limité à 5 Go chez la plupart des fournisseurs. `If-None-Match` porte alors sur l'appel qui assemble les parties.
- **Un envoi en plusieurs parties interrompu est abandonné par Brume.** Les parties d'un envoi incomplet sont facturées sans être visibles. `brume install` conseille aussi une règle du bucket qui nettoie les envois incomplets après 7 jours.

## Deux serveurs sur le même bucket

- **Les segments contigus rendent la détection fiable.** Deux serveurs qui divergent écrivent forcément le même nom de segment lors de leur première écriture divergente, et le second reçoit une erreur 412.
- **Le perdant passe en lecture seule :** il refuse les commandes, arrête l'envoi et alerte.
- **Un objet déposé sous le prochain nom par quelqu'un d'autre** a le même effet. L'admin change alors le credential, puis la racine : voir [Le bucket](#le-bucket).
- **Limite :** un serveur qui n'écrit plus rien ne l'apprend jamais. Ses apps continuent de tourner, mais sans trafic public : voir [Restauration](#restauration).
- **`If-None-Match` n'existe pas chez tous les fournisseurs.** Brume le teste à la configuration du bucket. S'il n'est pas supporté, Brume avertit que cette détection ne fonctionne pas.

## Restauration

`brume restore` reprend les étapes de `brume install` ([08-hote-et-installation.md](08-hote-et-installation.md#brume-install)), sauf la création des clés, puis :

1. **L'admin donne les credentials du bucket et sa clé age privée.** Il la tape ; Brume la garde en mémoire le temps de la restauration, sans jamais l'écrire sur le disque.
2. **Brume déchiffre la clé maître** (`keys/`) et la range dans les systemd credentials du nouvel hôte.
3. **Il télécharge la génération courante du log** et vérifie la chaîne de hash, puis reconstruit SQLite : log, blobs, index. Un segment illisible arrête la restauration au dernier événement valide, et l'admin en est prévenu. Un blob manquant est listé, pour que l'admin saisisse la valeur de nouveau. Voir [18-formats-s3.md](18-formats-s3.md#les-règles-du-lecteur).
4. **Il refait l'entrée et l'administration** selon les modes choisis ([14-architecture-logicielle.md](14-architecture-logicielle.md#lentrée-et-ladministration)) :
   - avec Cloudflare, il crée un nouveau tunnel et repointe le CNAME wildcard, comme à l'install : si l'ancien serveur revient, il ne reçoit plus le trafic public ;
   - en entrée directe, l'admin repointe lui-même le DNS vers la nouvelle IP ;
   - avec Tailscale, le nouveau serveur rejoint le tailnet comme un nouvel appareil, et l'admin retire l'ancien.
5. **Il reconstruit depuis leur commit les images et les sites statiques des révisions en service.** Chaque build crée une nouvelle révision, avec le commit et les variables de la révision en service : l'image peut différer de l'ancienne. Une image tirée d'un registry est retéléchargée par son digest. Les révisions plus anciennes restent dans l'historique, mais ne sont plus rollbackables.
6. **Il restaure le dernier dump de chaque base,** déchiffré avec la clé age. Pour chaque base, il affiche l'écart entre l'heure du dump et la fin du log. Il propose de remettre les services qui l'utilisent à la révision qui tournait à l'heure du dump : sinon, un code plus récent peut attendre un schéma que la base restaurée n'a pas.
7. **Il démarre les services.** L'agent fait converger Docker vers l'état voulu, et l'engine reprend l'envoi juste après le dernier segment. Si la restauration s'est arrêtée sur un segment illisible, il ouvre une nouvelle génération : le nom suivant est pris.

- **L'admin se reconnecte au panel :** les sessions ne survivent pas à une restauration. Son allowlist et ses passkeys sont dans l'état.
- **La restauration est testée chaque nuit sur une vraie VM :** installer, écrire, effacer le disque, restaurer. Voir [16-tests.md](16-tests.md#sur-une-vraie-vm).

## `brume verify`

- **Se lance depuis le poste de l'admin,** avec des credentials en lecture et sa clé age privée. Elle n'écrit rien.
- **Elle vérifie que :**
  - chaque segment se lit et la chaîne de hash est complète, d'une génération à l'autre ;
  - une restauration reconstruit un `World` valide ;
  - chaque blob se déchiffre ;
  - chaque backup se déchiffre et correspond à l'empreinte notée dans le log ;
  - aucune règle de cycle de vie ne vise `log/`, `blobs/` ou `keys/`, si ses credentials permettent de lire ces règles.
- **C'est le seul moyen de savoir, avant d'en avoir besoin, que le bucket est restaurable.** Elle détecte une mauvaise règle de cycle de vie, un Object Lock absent ou une clé perdue.

## Ce que la purge n'atteint pas

- **Une purge ([13-modele-de-donnees.md](13-modele-de-donnees.md#rétention-et-purge)) n'atteint pas S3.** Les blobs purgés y restent pour toujours, chiffrés par la clé maître. C'est la règle déjà acceptée pour les backups.
- **Un secret fuité se révoque chez son émetteur :** l'effacer ne suffit pas.

## Risques acceptés

| Risque | Pourquoi on l'accepte |
|---|---|
| La dernière seconde d'état se perd avec le VPS. | En échange, une panne de S3 ne bloque rien. |
| Une restauration rejoue tout le log, y compris les événements écrits par un attaquant qui a eu root. Il n'y a ni restauration à une date, ni rotation de la clé maître. | La 0.1 se concentre sur la prévention et la détection. La restauration à une date viendra plus tard : voir [12-plus-tard.md](12-plus-tard.md). |
| Une image tirée d'un registry n'est pas sur S3. Si le registry l'a supprimée ou s'il est injoignable, une restauration ne relance pas le service. | La copier doublerait le stockage d'une image que son registry garde déjà. Décidé le 31 octobre. |
| Un ancien VPS qui revient garde ses apps en marche, sans trafic public. Leurs tâches de fond (crons, files) peuvent encore écrire ailleurs. | C'est rare, et le nouveau tunnel coupe le trafic public. |
| L'engine est un point unique de défaillance. Sans lui, les apps tournent, mais rien ne se déploie. | Une bascule automatique demanderait du consensus, pas seulement S3. |
| Un attaquant root peut effacer la seconde pas encore envoyée et arrêter l'envoi. L'alerte partirait du serveur qu'il contrôle. | Ce qui est déjà envoyé reste intact. Une alerte externe viendra plus tard, voir [12-plus-tard.md](12-plus-tard.md). |
| Les fournisseurs S3 n'ont pas tous les mêmes fonctions (`If-None-Match`, y compris à l'assemblage d'un envoi en plusieurs parties, Object Lock). | Brume teste le bucket à la configuration. Le coût de sortie dépend du fournisseur. |
| Une restauration refait le build de chaque service en service : elle est plus longue, et l'image obtenue peut différer de l'ancienne. Les révisions plus anciennes ne sont plus rollbackables. Si le commit a disparu de GitHub (branche supprimée, historique réécrit), le service ne redémarre pas. | Brume n'est pas un registry : copier les images sur S3 viendra plus tard ([12-plus-tard.md](12-plus-tard.md)). Décidé le 31 octobre. |
| Le log n'est pas authentifié. Qui a le credential S3 peut écrire de faux événements, en recalculant la chaîne de hash : ajouter son identité à l'allowlist, changer l'image d'un service. Une restauration les rejouerait. Il peut aussi bloquer l'envoi en déposant un objet sous le prochain nom. | Le credential vit sur le serveur, et le compte du fournisseur est protégé par une 2FA matérielle. Des événements authentifiés viendront plus tard ([12-plus-tard.md](12-plus-tard.md)). Décidé le 31 octobre. |
| Qui lit le bucket voit la structure de l'infra : les noms des projets et des services, les domaines, les repos. | Ces noms ne donnent aucun accès. Les valeurs et les emails sont dans les blobs, chiffrés. Décidé le 31 octobre. |
