# Hors alpha 0.1

Ce qui est volontairement repoussé, et ce que ça apportera.

| Sujet | Ce que ça apporte |
|---|---|
| Tailnet Lock | Ni le serveur de coordination de Tailscale ni un compte volé chez le fournisseur d'identité ne peuvent ajouter un appareil au tailnet. Ferme le risque accepté sur Tailscale SSH, voir [01-modele-de-menace.md](01-modele-de-menace.md#risques-acceptés-en-01). |
| Builder jetable sous gVisor, ou build hors du serveur | Un build ne peut plus viser root par une faille noyau ou runc, ni modifier les builds suivants. Ferme le risque accepté sur l'accès en écriture aux branches déployées, voir [01-modele-de-menace.md](01-modele-de-menace.md#risques-acceptés-en-01). |
| Backups durcis | Credentials S3 testés à la configuration : lecture, liste et suppression doivent échouer. En-têtes de rétention Object Lock à chaque envoi, pour qu'un bucket sans Object Lock échoue au lieu d'accepter. Clé maître incluse dans le backup chiffré. Alerte externe quand les backups cessent d'arriver. |
| Mises à jour majeures des bases | Backup, puis dump et restauration dans la nouvelle version. MongoDB : une version à la fois, avec l'étape `setFeatureCompatibilityVersion` faite par Brume. L'engine refuse alors un rollback qui ferait tourner l'ancienne version sur les données de la nouvelle (PostgreSQL, MongoDB, Redis). |
| Réplicas et restauration à un instant précis | Les backups sont des dumps : on peut perdre les données écrites depuis le dernier. L'archivage des WAL de Postgres ramène cette perte à quelques secondes. |
| Secrets en fichiers sur tmpfs | Les variables d'environnement fuient vers les processus enfants, les rapports d'erreur (Sentry) et `/proc/<pid>/environ`. En option, car presque toutes les apps attendent des variables d'environnement. |
| `systemd-analyze security --offline=true` en CI | Note l'exposition de chaque unité systemd. La CI échoue si le score se dégrade. |
| Shell web dans les containers | La fonctionnalité la plus puissante du panel, donc à concevoir avec soin. |
| Tokens d'API, CLI distante, déploiement depuis GitHub Actions | Des accès hors Tailscale, avec leurs propres règles. |
| Previews de PR | Jamais de build avec des secrets pour une PR venant d'un fork. |
| Filtrage de l'accès Internet sortant par domaine | Demande un proxy de sortie. |
| Multi-serveurs | Modèle Edge Agent : chaque nœud se connecte en sortie au panel. Jamais de clé SSH root centrale. |
| Headscale | Self-hosting complet de la couche admin. Proposition : pas en 0.1. |
| Docker rootless, gVisor pour les apps | Isolation renforcée de l'hôte et des containers, au-delà de `userns-remap`. |
