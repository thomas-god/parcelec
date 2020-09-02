# _Serious game_ marchés de l'électricité/gestionnaire d'un parc de production

_06/07/2020_

> Idée générale : s'inspirer du jeu de la demi-journée formation marchés R19 mais en allant plus loin dans la démarche quitte à devoir _gamifier_ certains mécanismes réels.

Chaque joueur possède un portefeuille de production (thermique, nucléaire, hydrauliques (avec ou sans stock), renouvelable, etc.) et de consommation (industriels, ménages) qu'il doit gérer au mieux (ie. gagner le plus d'argent possible) pendant une année. Les différentes actions sont : vendre/acheter de l'énergie sur les marchés, vendre/acheter des éléments du portefeuille, et des actions diverses (actions de maintenance sur les moyens de production, etc.).

Les différents joueurs ont des portefeuilles différents (en tailles et en compositions).

Les joueurs intéragissent entre eux, soit directement, soit indirectement (via la place de marché).

Une année est divisée en différentes phases d'enchères (saisons, moments de la journée) pendant lesquelles les joueurs doivent boucler leur portefeuille. Les phases d'enchères sont entrecoupées de temps morts (résultats de la phase précédente, actions diverses, etc.).

Définir les composants d'un portefeuille (technologies de production, type de consommateurs), les actions d'un joueur, les règles de la place de marché.

Ajouter un concurrent IA qui joue de manière optimale pour inciter les joueurs humains à converger vers une gestion optimale de leur parc.

Comment calculer un score si les portefeuilles sont différents d'un joueur à l'autre, et donc les revenus potentiel. En % du revenu théorique optimal total ?

S'assurer lors de la génération des parcs (prod et conso) des joueurs, que la somme des parc est capable de couvrir la conso max théorique.

## Technologies de production

- Thermique à flamme: puissances moyennes, coût marginal élevé, fortes émissions de CO2. Gaz, charbon, fioul.
- Nucléaire: fortes puissances, coût marginale faible, pas d'émissions de CO2. Planning d'arrêts à gérer ?
- Hydraulique: coût marginal nul, pas de manœuvrabilité pour le fil de l'eau et gestion d'un stock pour les lacs. Pas de CO2.
- Renouvelable: coût marginal nul, pas de manœuvrabilité. Notion de prévision et d'écart réalisé.

## Types de consommateurs

- Industriel: profil connu à l'avance et prédictible.
- Tertiaire: consommation pendant les heures de bureaux.
- Particuliers: pointe le matin et le soir.

## Événements divers

- Casse sur un groupe de production
- Sécheresse
- Variation prix du CO2
- Variations des prix des combustibles (charbon, gaz, pétrole)
- Fermeture centrale(s) par l'ASN
- Vague de froid
- Crise économique
- Publication des résultats trimestriels

## Intéractions entre joueurs

- Achat/vente d'énergie de gré à gré
- Achat/vente d'élement(s) du portefeuille
- Échanges via la place de marché
- Visibilité du portefeuilles des autres acteurs
- Dénonciation à l'autorité de marchés (si MJ présent)
- Annonces publiques et échanges privés

# Élements d'UI

## Élement du portefeuille

- Icône prod/conso (type et puissance)
- Popup avec infos basiques (et actions basiques ?) et clique vers actions avancées
- Actions avancées (entretien, vente à un autre joueur, amélioration, R&D)
- Prévisions éventuelles (prod, conso, précipitations, prix des commodités)
- Prévoir une petite icône pour signifier qu'une info est visible par les autres joueurs

## Portefeuille du joueur

- Container pour les éléments du portefeuilles
- Fonctionnalités basique de tri (par puissance, par type, non alloué, etc)
- Liste des événements divers en cours (icônes à la TWW)
- Liste des dernières actions du joueur, telles que vues par les autres joueurs dans les actions publiques
- Faire une déclaration publiques
- Liste des prévisions pour le portefeuille

## Liste des concurrents

- Liste des dernières actions publiques (du joueur et de ses concurrents)

## Portefeuille d'un concurrent

- Vision limitée des éléments de son portefeuille
- Liste des dernières actions publiques du concurrent
- Liste des actions possibles avec ce concurrent
- Chat avec le concurrent

## Popup pour les éléments divers

- Descriptif de l'événement
- Liens vers les ressources impactées

## Place de marché

- Récapitulatif des offres en achat/vente du joueur
- Temps restant avant fermeture des enchères
- Courbes d'équilibres globales des dernières enchères

## Interface administrateur

- Liste des joueurs
- Modification à la main des parcs générés avant de les attribuer aux joueurs
- Gestion des paramètres de jeu
- Génération manuelle d'événements aléatoires

# Architecture

- Front Vue.js
- Reverse Proxy Nginx (ou Proxy Heroku)
- Back Node.js
- DB Postgres
- Moyen de tester de l'IaaS, par ex. avec les free tiers Heroku

## Gestion de sessions

- Session de jeu (plusieurs joueurs et un MJ optionnel), expire quand la partie est terminée ou un certain temps d'inactivité
- Session d'un joueur liée à une session de jeu

## Base(s) de données

- Une base par session de jeu pour stocker les actions des joueurs, les événements divers, les enchères de la place de marché.
  - Une table par session de jeu ? Et on drop la table quand la session est terminée/a expiré. Ou bien un session_id et une seule table.
- Une base de données "data" avec toutes les données nécessaires au jeu (coûts, etc), ou bien un json.
- SQL probablement plus adapté que du noSQL dans la mesure où l'on va vouloir faire des queries par enchère_id et que les "documents" sont temporellement cohérents entre eux.
