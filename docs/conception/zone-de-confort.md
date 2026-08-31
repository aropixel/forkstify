# Zone de confort

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Décidé

- Réglage de **0 à 5**, défini **à l'ouverture**.
- Mesure la **familiarité** ([0001](../decisions/0001-confort-familiarite.md)) :
  0 = cocon, 5 = exploration.
- Sert à **choisir seul** la branche à un embranchement quand l'utilisateur ne
  choisit pas activement. L'application ne bloque jamais.

## Orientations

- **Le réglage structure l'éventail proposé**, pas seulement le choix par
  défaut : à chaque embranchement, les branches sont étalées sur l'axe — une
  plus rassurante que le réglage, une au niveau, une plus aventureuse.
  L'utilisateur acquiert un modèle mental stable (« à gauche je me rassure,
  à droite je m'aventure ») et choisit sans lire.
- **Le réglage est ajustable en cours de parcours.** La valeur à l'ouverture
  n'est qu'un point de départ.
- **Techniquement, le confort est une distance** entre un artiste candidat et
  le centre de gravité de ce que l'utilisateur connaît, dans l'espace
  vectoriel décrit dans [moteur-de-branches.md](moteur-de-branches.md).

## À trancher

- Comment l'application sait ce que l'utilisateur **connaît** : bibliothèque
  Spotify (titres et artistes suivis), historique d'écoute, parcours passés,
  fiches modifiées dans son fork du catalogue ? Probablement tout ça, avec
  des poids.
- Combien de temps l'application attend à un embranchement avant de choisir
  seule : jusqu'à la fin du segment, un délai fixe, ou pas d'attente du tout
  (elle enchaîne et l'utilisateur peut dévier à tout moment) ?
- Le confort est-il un curseur réglé par l'utilisateur, un indicateur affiché
  par l'application, ou les deux ?
