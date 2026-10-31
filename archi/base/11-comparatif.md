# Comment font les autres

D'après leurs docs d'installation :

| Outil | Panel | Accès à Docker | Si le panel est compromis |
|---|---|---|---|
| Portainer | container exposé sur le 9443 | socket monté dans le même container que l'UI | root sur l'hôte |
| Coolify | app Laravel exposée sur le 8000 | SSH en root vers l'hôte, même en local : la clé privée est dans l'app | root sur tous les serveurs gérés |
| Dokploy | app Next.js exposée sur le 3000 | socket monté, Docker Swarm | root sur l'hôte |
| CapRover | app Node exposée, mot de passe par défaut `captain42` | socket monté, Docker Swarm | root sur l'hôte |
| Kamal | pas de panel | SSH depuis le poste du dev, seul un petit proxy tourne sur le serveur | pas de panel à attaquer |

## Leçons pour Brume

- **Tous sauf Kamal mettent l'UI web et le socket Docker dans le même process.** C'est leur faiblesse de fond. La séparation engine/API est ce qui distingue Brume.
- **Monter le socket en `:ro` ne protège rien.** Coolify le fait pour son Traefik, mais un socket monté en lecture seule accepte quand même tous les appels de l'API Docker.
- **Le premier démarrage est toujours le point faible.** Portainer coupe l'instance si personne ne crée l'admin en 5 minutes, CapRover démarre avec un mot de passe connu. Chez Brume, le compte de l'admin est fixé à l'install, l'URL d'enrôlement porte un jeton à usage unique, et le panel n'est public que si l'admin le choisit sur l'hôte.
- **Une clé SSH root centrale vers chaque serveur,** comme chez Coolify, est un point unique de compromission. Brume copie plutôt l'Edge Agent de Portainer : l'agent de chaque machine se connecte en sortie au master, qui ne détient aucun accès aux machines. Voir [14-architecture-logicielle.md](14-architecture-logicielle.md#le-protocole-entre-lengine-et-lagent).
- **La pratique courante pour limiter le socket est un proxy filtrant,** comme `docker-socket-proxy` de Tecnativa. L'engine de Brume en est une version beaucoup plus étroite : il expose des opérations métier, pas l'API Docker.
- **Kamal montre la surface minimale :** pas de control plane permanent, un proxy piloté par commandes et sans socket. Brume a besoin d'un panel et de webhooks, mais garde l'idée d'un proxy configuré par fichiers, jamais par le socket.
