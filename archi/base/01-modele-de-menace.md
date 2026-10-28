# Modèle de menace

## Périmètre

- **Brume est single-tenant, définitivement.** Un VPS, un propriétaire, des admins de confiance. Le multi-tenant n'est pas prévu.
- **Le code déployé est celui du propriétaire du VPS, et Brume lui fait confiance :** son code, ses dépendances et ses images de base.
- **L'extérieur n'est pas de confiance :** les requêtes vers les apps, les webhooks, les sites ouverts par l'admin, les registries, et tout ce qui en garde la trace (logs, en-têtes). Une requête venue de l'extérieur peut exploiter une faille d'une app : le code qui tourne alors dans le container n'est plus celui du propriétaire.
- **Docker n'est pas une frontière contre du code hostile venant de plusieurs utilisateurs.** Ce n'est pas notre cas d'usage, et Brume ne le prétend pas. Les containers tournent quand même avec `userns-remap`, pour le cas d'une app compromise depuis l'extérieur : root dans un container n'est pas root sur l'hôte. Voir [05-runtime.md](05-runtime.md).

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

1. **Une app publique compromise** par une faille applicative, exploitée depuis l'extérieur. L'attaquant cherche à rebondir vers les autres environnements, l'hôte, le tailnet, Brume ou l'endpoint de métadonnées de l'hébergeur.
2. **Un site malveillant ouvert par l'admin.** Le navigateur est sur le tailnet : CSRF, DNS rebinding, clickjacking, WebSocket détournée vers le panel.
3. **Du contenu contrôlé par l'attaquant affiché dans le panel :** une ligne de log, un nom de branche, un message de commit. Une faille XSS dans le panel donne ses actions : toutes les apps, leurs secrets et leurs données.
4. **Un registry hors du contrôle du propriétaire :** une image qui change derrière un tag. D'où le déploiement par digest.
5. **La supply chain de Brume :** compromission de brume.dev, du compte GitHub, d'une GitHub Action ou d'une dépendance Rust ou npm.
6. **Un accès au tailnet :** appareil compromis, tailnet partagé, nœud ajouté par le serveur de coordination de Tailscale.
7. **La perte des données :** serveur détruit, rançongiciel, suppression des backups par un attaquant.
8. **L'épuisement des ressources :** un build qui prend toute la RAM, des logs qui remplissent le disque, un flot de logs provoqué depuis l'extérieur.

## Hors périmètre

- **L'hébergeur** a accès à la VM, à son disque et à sa mémoire. On lui fait confiance. Le compte de l'utilisateur chez l'hébergeur équivaut donc à root (console, mode rescue) : à protéger par une 2FA.
- **Cloudflare** déchiffre tout le trafic public. C'est accepté en échange de zéro port entrant, et à signaler aux utilisateurs (entreprise américaine, RGPD). Le compte Cloudflare peut aussi ajouter des routes privées au tunnel : voir [03-reseau.md](03-reseau.md#ce-que-ça-coûte-à-dire-aux-utilisateurs).
- **Tailscale** contrôle le serveur de coordination. Pour le panel, le risque est atténué par l'allowlist d'identités et le second facteur. Pour SSH, voir les risques acceptés.
- **Le poste de l'admin.** Compromis, il détient l'identité Tailscale et le second facteur. Brume ne s'en protège pas.
- **Le code du propriétaire, dépendances comprises.** Un paquet npm piégé est traité comme du code du propriétaire. Le builder l'exécute avec un sandbox affaibli : voir [06-builds-et-deploiement.md](06-builds-et-deploiement.md#le-builder).
- **Le DDoS** est délégué à Cloudflare.
- **Le code hostile venant de plusieurs utilisateurs** (voir Périmètre).

## Risques acceptés en 0.1

| Risque | Pourquoi on l'accepte | Plus tard |
|---|---|---|
| Une app compromise depuis l'extérieur lit ses secrets et joint les services de son environnement. | Toutes les plateformes ont ce problème : une app a besoin de ses secrets et de ses bases. | Secrets en fichiers sur tmpfs |
| Un accès en écriture à une branche déployée donne l'exécution de code en prod, avec les secrets du service. Il donne aussi l'exécution de code dans le builder, dont le sandbox est affaibli : avec une faille noyau ou runc, jusqu'à root. | C'est le principe du déploiement continu : qui peut pousser est traité comme le propriétaire. À protéger côté GitHub : 2FA, protection de branche, aucun bot qui fusionne seul. | Builder jetable sous gVisor |
| Traefik analyse tout le HTTP public. Compromis, il lit le trafic et joint les services exposés. | Il faut un proxy. Il est épinglé par digest, sans root, en lecture seule, et ne joint pas les bases. Voir [03-reseau.md](03-reseau.md#traefik). | |
| L'identité Tailscale seule donne un shell root par Tailscale SSH : le second facteur du panel ne couvre pas SSH. Un compte volé chez le fournisseur d'identité peut ajouter un appareil au tailnet. | Suffisant pour l'alpha. | Tailnet Lock |
