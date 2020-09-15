# API DOCUMENTATION

## Routes existantes

- `GET /session/list_open` : récupère la liste des sessions de jeu actuellement ouvertes,
- `PUT /session/open` : ouvre une nouvelle session de jeu,
- `GET /session/:session_id` : récupère les informations d'une session,
- `PUT /session/:session_id/register_user` : inscrit un nouveau joueur à une session de jeu ouverte,
- `GET /session/:session_id/user/:user_id` : récupère les informations d'un joueur d'une session,
- `PUT /session/:session_id/user/:user_id/ready` : marque un joueur comme prêt à démarrer la partie,
- `PUT /session/:session_id/start` : démarre une partie,
- `PUT /session/:session_id/bid` : soumet une enchère à la partie en cours,
- `PUT /session/:session_id/clearing` : procède aux clearing des enchères.

## Routes à implémenter

### Gestion d'une partie/session
- `GET /sessions/open` Lister les parties que l'on peut rejoindre
- `PUT /session` Créer une nouvelle partie
- `GET /session/:session_id` Récupérer les informations publiques d'une partie (en gros, pas d'infos sur les autres joueurs)

### Gestion des joueurs
- `PUT /session/:session_id/register_user` S'inscrire à une partie
- `PUT /session/:session_id/user/:user_id/ready` Dire qu'un joueur est prêt à commencer une partie
- `GET /session/:session_id/user/:user_id` Récupérer les informations privées d'un joueurs

### Gestion des phases de jeu
- `GET /session/:session_id/user/:user_id/portfolio` Récupérer les informations sur son parc de production
- `GET /session/:session_id/user/:user_id/conso` Récupérer sa prévision de consommation pour la phase en cours
- `POST /session/:session_id/user/:user_id/bid` Poster une enchère
- `GET /session/:session_id/user/:user_id/bids` Voir ses enchères déjà postées
- `DELETE /session/:session_id/user/:user_id/bid/:bid_id` Supprimer une enchère que l'on a postée
- `PUT /session/:session_id/user/:user_id/planning` Poster/mettre à jour son programme d'appel
- `GET /session/:session_id/clearing` Récupérer les infos publiques du clearing si il a eu lieu (sans les échanges qui en résultent)
- `GET /session/:session_id/user/:user_id/clearing` Récupérer les infos privées du clearing si il a eu lieu (avec les échanges qui en résultent)
- `GET /session/:session_id/user/:user_id/results` Récupérer son bilan une fois la phase temps réel jouée
