# Secrets et backups

## Secrets de la plateforme

| Secret | Détenu par | Remarque |
|---|---|---|
| Clé maître des secrets | `brume engine` | systemd credentials, voir plus bas |
| Clé privée de la GitHub App | `brume engine` | génère des tokens d'une heure |
| Credentials S3 | `brume engine` | écriture seule |
| Clé publique age | `brume engine` | la clé privée reste chez l'utilisateur |
| Secret HMAC du webhook | `brume webhook` | |
| Token du tunnel Cloudflare | cloudflared | le seul token Cloudflare stocké |
| Token API Cloudflare | personne | utilisé à l'install puis oublié |
| Clé d'auth Tailscale | personne après l'install | à usage unique |

## Secrets des apps

- **Chiffrés en base avec la clé maître.** Le gain est limité tant que la clé est sur la même machine : la vraie protection porte sur les backups.
- **Clé maître dans les credentials systemd** (`LoadCredentialEncrypted=`). Elle est chiffrée sur le disque, et seul l'engine la reçoit au démarrage, dans un répertoire privé. Les autres services ne peuvent pas la lire, même avec un bug qui permettrait de lire des fichiers. Il faut Debian 12 ou Ubuntu 24.04.
- **Déchiffrés uniquement par l'engine,** et injectés à la création du container.
- **L'API ne voit jamais un secret en clair,** sauf sur une action « révéler » explicite.
- **Dans le code Rust,** les secrets sont stockés dans les types du crate `secrecy`.
- **Variables d'environnement en 0.1.** Les fichiers sur tmpfs viendront plus tard.
- **Logs :** Brume masque les secrets dans les logs de build. Les apps écrivent parfois des secrets dans leurs propres logs : rétention limitée, et pas de logs dans les backups.

## Backups

### Ce qui est sauvegardé

- **Des dumps cohérents** (`pg_dump`, `.backup` pour SQLite), jamais une copie de fichiers en cours d'écriture.
- **Brume fait ses propres dumps :** `pg_dump | age | upload`. Pas de restic : il doit lire, lister et supprimer dans son dépôt, ce qui est incompatible avec des credentials en écriture seule.
- **L'état de Brume lui-même** (config, secrets chiffrés), sinon rien n'est reconstructible.
- **Pas les logs.**

### Le chiffrement

- **age asymétrique :** le serveur n'a que la clé publique. Il peut chiffrer, pas déchiffrer.
- **La clé privée est donnée à l'utilisateur à l'install,** ou l'utilisateur fournit sa propre clé publique. Elle est indispensable pour restaurer.
- **Bonus :** un attaquant qui prend le serveur ne peut pas lire les anciens backups.

### Le stockage

- **Hors du VPS,** sur un stockage compatible S3, configuré depuis le panel.
- **Credentials en écriture seule** (`PutObject`), sans droit de suppression.
- **Versioning du bucket ou Object Lock obligatoire :** `PutObject` peut écraser un objet existant dont on connaît le nom.
- **La rétention se fait par des règles de cycle de vie du bucket,** pas par Brume, qui ne peut rien supprimer.

### La vérification

- **Le serveur ne peut pas déchiffrer ses propres backups.** Le test automatique restaure donc chaque dump dans un container temporaire avant le chiffrement, puis vérifie l'empreinte de l'objet envoyé.
- **Une restauration complète depuis S3 demande la clé privée :** à faire régulièrement depuis le poste de l'admin.
- **Alerte quand un backup échoue.** La panne silencieuse est le cas le plus fréquent.

## Journal d'audit

Envoyé en continu vers S3 avec Object Lock. Voir [09-observabilite.md](09-observabilite.md).
