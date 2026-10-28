# Modèle de menace

## Périmètre

- **Brume est single-tenant, définitivement.** Un VPS, un propriétaire, des admins de confiance. Le multi-tenant n'est pas prévu.
- **Le code déployé appartient à l'utilisateur,** mais ses dépendances, ses images de base et ses entrées ne sont pas de confiance.
- **Docker n'est pas une frontière contre du code hostile venant de plusieurs utilisateurs.** Ce n'est pas notre cas d'usage, et Brume ne le prétend pas.

## Ce qu'on protège

| Actif | Pourquoi c'est critique |
|---|---|
| L'accès root à l'hôte | Contrôle total : apps, données, secrets |
| Les secrets des apps et de la plateforme | Accès aux services tiers : Stripe, bases, GitHub, Cloudflare, S3 |
| Les données (volumes, bases) | Le cœur de valeur de l'utilisateur |
| Les backups | Seul recours en cas de perte ou de rançongiciel |
| Le tailnet de l'admin | Laptop, NAS, autres serveurs : le serveur ne doit pas servir de tremplin |
| L'intégrité des déploiements | Ce qui tourne doit être exactement ce qui a été construit depuis le repo configuré |
| La chaîne de release de Brume | Une compromission toucherait toutes les installations |
| La disponibilité | Un build, des logs ou un container ne doivent pas faire tomber la prod |

## Scénarios d'attaque

1. **Une app publique compromise,** par une faille applicative ou une dépendance piégée. L'attaquant cherche à rebondir vers les autres environnements, l'hôte, le tailnet, Brume ou l'endpoint de métadonnées de l'hébergeur.
2. **Un site malveillant ouvert par l'admin.** Le navigateur est sur le tailnet : CSRF, DNS rebinding, clickjacking, WebSocket détournée vers le panel.
3. **Du contenu contrôlé par l'attaquant affiché dans le panel :** une ligne de log, un nom de branche, un message de commit. Une faille XSS dans le panel donne root.
4. **La supply chain des apps :** un `postinstall` npm qui s'exécute au build, une image qui change derrière un tag.
5. **La supply chain de Brume :** compromission de brume.dev, du compte GitHub, d'une GitHub Action ou d'une dépendance Rust ou npm.
6. **Un accès au tailnet :** appareil compromis, tailnet partagé, nœud ajouté par le serveur de coordination de Tailscale.
7. **La perte des données :** serveur détruit, rançongiciel, suppression des backups par un attaquant.
8. **L'épuisement des ressources :** un build qui prend toute la RAM, des logs qui remplissent le disque.

## Hors périmètre

- **L'hébergeur** a accès à la VM, à son disque et à sa mémoire. On lui fait confiance.
- **Cloudflare** déchiffre tout le trafic public. C'est accepté en échange de zéro port entrant, et à signaler aux utilisateurs (entreprise américaine, RGPD).
- **Tailscale** contrôle le serveur de coordination. Le risque est atténué par l'allowlist d'identités, le second facteur et Tailnet Lock en option.
- **Le DDoS** est délégué à Cloudflare.
- **Le code hostile venant de plusieurs utilisateurs** (voir Périmètre).
