# Développement de Brume

## Règles de code Rust

- **`#![forbid(unsafe_code)]`.**
- **Secrets dans les types du crate `secrecy` :** un `Debug` ou un `tracing::info!` ne peut pas les afficher par accident.
- **Docker piloté par son API** (crate `bollard`), jamais par `sh -c` ni par la CLI `docker`.
- **L'engine n'analyse aucune donnée non fiable.** Voir [02-architecture.md](02-architecture.md).
- **Requêtes SQLite paramétrées** (sqlx). Pour DuckDB, voir [09-observabilite.md](09-observabilite.md).
- **Taille limitée** pour tous les bodies HTTP et tous les messages des sockets.
- **Requêtes vers des URL fournies par l'utilisateur** (notifications, healthchecks) : tailnet et endpoint de métadonnées bloqués.

## Injection dans ce que Brume génère

Si Brume construit une règle Traefik par concaténation, un domaine contenant un backtick peut injecter une condition qui capte tout le trafic :

```
modèle :         Host(`${domain}`)
domaine saisi :  x`) || PathPrefix(`/
règle générée :  Host(`x`) || PathPrefix(`/`)
```

Le même risque existe pour la config nginx, les variables d'environnement (un `\n` dans une valeur) et les commandes `git` (un argument qui commence par `-`).

- **Validation stricte des noms** (domaines, services, variables, branches), par liste blanche.
- **Toute config générée est sérialisée** (serde), jamais construite par templating de chaînes.
- **`--` avant tout argument utilisateur** passé à `git`.

## Frontend

Le code du panel tourne avec les droits de l'admin : un paquet npm compromis donne root.

- **Uniquement des dépendances connues, ou du code écrit par nous.** Mais « connue » ne suffit pas : en septembre 2025, `chalk` et `debug`, parmi les paquets npm les plus utilisés, ont été compromis après le phishing de leur mainteneur.
- **Lockfile, `--ignore-scripts`,** et un délai avant d'adopter une nouvelle version (`minimumReleaseAge` dans pnpm).
- **Compilé en CI et intégré au binaire signé :** le serveur n'exécute jamais npm.
- **Jamais de `dangerouslySetInnerHTML`.**

## CI

- **`cargo deny` et `cargo audit`.**
- **GitHub Actions épinglées par SHA.** L'attaque tj-actions de 2025 passait par un tag modifié.
- **`systemd-analyze security --offline=true` :** plus tard.

## Releases

- **Binaires publiés sur GitHub Releases.**
- **Compte GitHub protégé par une clé 2FA physique.**

### Signature, à trancher

- **minisign :** clé publique intégrée au binaire, clé privée gardée hors ligne. Simple à vérifier dans l'updater.
- **Sigstore via OIDC :** pas de clé longue durée, signature liée au workflow de release. L'updater vérifie l'identité du workflow, ce qui est plus complexe à implémenter.

## Signalement des failles

Un `SECURITY.md` avec une adresse de signalement, publié avant l'annonce publique.
