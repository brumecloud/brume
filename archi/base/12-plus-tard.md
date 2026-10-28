# Hors alpha 0.1

Ce qui est volontairement repoussé, et ce que ça apportera.

| Sujet | Ce que ça apporte |
|---|---|
| Ré-authentification par passkey, vérifiée par l'engine, avant les actions critiques | Ferme la limite de la 0.1 : une API compromise ne pourra plus changer la source d'un service pour recevoir ses secrets. L'engine génère le challenge et vérifie lui-même la signature. |
| Secrets en fichiers sur tmpfs | Les variables d'environnement fuient vers les processus enfants, les rapports d'erreur (Sentry) et `/proc/<pid>/environ`. En option, car presque toutes les apps attendent des variables d'environnement. |
| `systemd-analyze security --offline=true` en CI | Note l'exposition de chaque unité systemd. La CI échoue si le score se dégrade. |
| Shell web dans les containers | La fonctionnalité la plus puissante du panel, donc à concevoir avec la ré-authentification. |
| Tokens d'API, CLI distante, déploiement depuis GitHub Actions | Des accès hors Tailscale, avec leurs propres règles. |
| Previews de PR | Jamais de build avec des secrets pour une PR venant d'un fork. |
| Filtrage de l'accès Internet sortant par domaine | Demande un proxy de sortie. |
| Multi-serveurs | Modèle Edge Agent : chaque nœud se connecte en sortie au panel. Jamais de clé SSH root centrale. |
| Headscale | Self-hosting complet de la couche admin. Proposition : pas en 0.1. |
| Docker rootless, userns-remap, gVisor | Isolation renforcée de l'hôte et des containers. |
