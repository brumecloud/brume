# Secrets et backups

## Secrets de la plateforme

| Secret | Détenu par | Remarque |
|---|---|---|
| Clé maître des secrets | `brume engine` | systemd credentials, voir plus bas |
| Clé privée de la GitHub App | `brume engine` | génère des tokens d'une heure |
| Credentials S3 | `brume engine`, `brume agent` | lecture et écriture, sans suppression |
| Clé publique age | `brume engine` | la clé privée reste chez l'utilisateur |
| Secret HMAC du webhook | `brume webhook` | sauvegardé dans l'état, sur S3 |
| Token du tunnel Cloudflare | cloudflared | le seul token Cloudflare stocké |
| Token API Cloudflare | personne | utilisé à l'install puis oublié |
| Clé d'auth Tailscale | personne après l'install | à usage unique |

## Secrets des apps

- **Définis sur le projet, l'environnement ou le service,** résolus au déploiement, jamais copiés dans une révision. Voir [13-modele-de-donnees.md](13-modele-de-donnees.md#variables-et-secrets).
- **Chiffrés en base avec la clé maître.** Le gain est limité tant que la clé est sur la même machine : la vraie protection porte sur les backups.
- **Clé maître dans les credentials systemd** (`LoadCredentialEncrypted=`). Elle est chiffrée sur le disque, et seul l'engine la reçoit au démarrage, dans un répertoire privé. Les autres services ne peuvent pas la lire, même avec un bug qui permettrait de lire des fichiers. Il faut Debian 12 ou Ubuntu 24.04.
- **La clé maître est aussi sauvegardée sur S3,** chiffrée pour les clés age de l'admin. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#la-clé-maître-et-les-clés-age).
- **Déchiffrés uniquement par l'engine,** puis transmis à l'agent, qui les injecte à la création du container.
- **L'API ne voit jamais un secret en clair,** sauf sur une action « révéler » explicite.
- **Dans le code Rust,** les secrets sont stockés dans les types du crate `secrecy`.
- **Variables d'environnement en 0.1.** Seule exception : la keyfile du replica set MongoDB, écrite dans le volume de la base. Voir [templates.md](templates.md#mongodb). Les fichiers sur tmpfs viendront plus tard.
- **Logs :** Brume masque les secrets dans les logs de build. Les apps écrivent parfois des secrets dans leurs propres logs : rétention limitée, et pas de logs dans les backups.

## Backups

### Ce qui est sauvegardé

- **Des dumps cohérents** (`pg_dump`, `mongodump`, snapshot RDB de Redis, `.backup` pour SQLite), jamais une copie de fichiers en cours d'écriture. Le détail par base est dans [templates.md](templates.md).
- **Brume fait ses propres dumps :** `pg_dump | age | upload`. Pas de restic : il doit lire, lister et supprimer dans son dépôt, ce qui est incompatible avec des credentials sans droit de suppression.
- **L'état de Brume lui-même** part en continu sur S3, par le log d'événements. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md).
- **La clé maître est sauvegardée,** chiffrée pour les clés age de l'admin : un serveur perdu se remonte avec la clé age privée.
- **Pas les logs.**

### Le chiffrement

- **age asymétrique :** le serveur n'a que la clé publique. Il peut chiffrer, pas déchiffrer.
- **La clé privée est donnée à l'utilisateur à l'install,** ou l'utilisateur fournit sa propre clé publique. Elle est indispensable pour restaurer, l'état comme les backups. Plusieurs destinataires sont conseillés, et l'admin prouve qu'il a gardé sa clé avant la fin de l'install : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#la-clé-maître-et-les-clés-age).
- **La clé publique ne se change que sur l'hôte,** en root, jamais depuis le panel. Sinon une session volée rendrait les backups suivants lisibles par l'attaquant.
- **Bonus :** un attaquant qui prend le serveur ne peut pas lire les anciens backups.

### Le stockage

- **Hors du VPS,** sur un stockage compatible S3. Un seul bucket reçoit l'état, les artefacts et les backups : voir [15-etat-et-restauration.md](15-etat-et-restauration.md#ce-qui-va-sur-s3).
- **Obligatoire dès l'install, et modifiable seulement sur l'hôte,** en root, jamais depuis le panel. Une session volée ne peut donc pas envoyer les backups suivants ailleurs.
- **Tout changement de bucket déclenche une alerte.**
- **Credentials en lecture et écriture, sans droit de suppression.** La lecture ne révèle rien : tout est chiffré, et les backups le sont pour une clé que le serveur n'a pas.
- **Versioning du bucket ou Object Lock obligatoire :** `PutObject` peut écraser un objet existant dont on connaît le nom. Brume écrit avec `If-None-Match`, et `brume verify` vérifie le versioning ou l'Object Lock.
- **La rétention se fait par des règles de cycle de vie du bucket,** pas par Brume, qui ne peut rien supprimer. Seuls les backups expirent : le log, les blobs et les clés jamais.

### La vérification

- **Le serveur ne peut pas déchiffrer ses propres backups.** Le test automatique restaure donc chaque dump dans un container temporaire avant le chiffrement, puis vérifie l'empreinte de l'objet envoyé.
- **Une restauration complète depuis S3 demande la clé privée :** à faire régulièrement depuis le poste de l'admin, avec `brume verify`. Voir [15-etat-et-restauration.md](15-etat-et-restauration.md#brume-verify).
- **Alerte quand un backup échoue.** La panne silencieuse est le cas le plus fréquent.

## Journal d'audit

C'est le log d'événements, envoyé en continu sur S3, chiffré et chaîné par hash. Voir [09-observabilite.md](09-observabilite.md#journal-daudit) et [15-etat-et-restauration.md](15-etat-et-restauration.md).
