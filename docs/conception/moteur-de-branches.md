# Moteur de branches

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Décidé

- Le moteur raisonne sur les **artistes** ; les morceaux sont choisis dans
  l'artiste via les tops et les signaux d'usage
  ([0003](../decisions/0003-titres-tops-et-portes.md),
  [0010](../decisions/0010-format-revise-links-sans-portes.md)).
- Sa matière première est le **catalogue** de fiches
  ([0002](../decisions/0002-catalogue-partage-forkable.md)), pas une API de
  recommandation.

## Orientations

### Deux représentations complémentaires

1. **Un graphe explicite** — les *connexions* écrites dans les fiches, typées
   (« filiation », « même scène », « même producteur »…) et commentées. C'est
   ce qui **nomme** les branches : une connexion a une étiquette lisible, un
   vecteur n'en a pas.
2. **Un espace vectoriel implicite** — chaque artiste a un vecteur, dérivé de
   sa fiche (description, tags, connexions), éventuellement enrichi de
   co-occurrences (Last.fm, playlists) et de l'historique de l'utilisateur.
   C'est ce qui **remplit les trous** là où personne n'a écrit de connexion,
   et ce qui donne une notion de distance continue.

Les connexions explicites priment quand elles existent ; l'espace vectoriel
prend le relais sinon.

### Les branches comme opérations géométriques

- **Poncer** = rester au même point, tirer d'autres tops.
- **Voisinage** = plus proches voisins dans un rayon *r*.
- **Décalage** = se déplacer dans une direction (même époque autre genre,
  même genre autre époque…).
- **Retour** = se rapprocher du centre de gravité de la bibliothèque.
- **Zone de confort** = distance au centre de gravité de ce que l'utilisateur
  connaît.
- Un **parcours** est un chemin dans l'espace : on peut éviter d'y repasser,
  ou y revenir volontairement.

### Vecteurs : trois sources cumulables, rien à entraîner

1. **Embedding de texte composé par l'application** depuis les champs
   structurés de la fiche (tags, origine, dates, liens), la description en
   nuance. Personne n'écrit « pour l'embedding » — voir
   [catalogue.md](catalogue.md). Capte « Orelsan proche de Casseurs
   Flowters, un peu moins de Stupeflip ».
2. **Co-occurrence** : artistes qui apparaissent ensemble (similaires Last.fm,
   playlists, tags). Capte le « ça s'écoute ensemble » que le texte rate.
3. **Historique de l'utilisateur** : ce qu'il enchaîne réellement. Petit
   volume, très pertinent, c'est ce qui personnalise l'espace.

Les vecteurs sont un **cache régénérable**, jamais une source de vérité.

### Choix du morceau dans l'artiste

Le moteur choisit dans les tops et l'historique selon la zone de confort —
un morceau familier quand on se rassure, un moins écouté quand on explore.
Si la fiche a une **door** dont les tags recoupent la direction prise, ce
morceau reçoit un bonus — critère additionnel, jamais principal
([0011](../decisions/0011-doors-critere-additionnel.md)).

## À trancher

- Longueur d'un segment : fixe (3 morceaux ?), variable selon le type de
  branche, ou selon le confort ?
- Quel modèle d'embedding, local ou distant ? Contrainte : tout dockerisé,
  usage hors-ligne souhaitable pour la partie catalogue.
- Comment on **nomme** une branche issue de l'espace vectoriel et non d'une
  connexion explicite : par les tags dominants des voisins ? par un LLM ?
- Faut-il des directions nommées dans l'espace (« plus ancien », « plus
  électronique ») et comment les construire ?
- Sources de co-occurrence : Last.fm est le candidat évident ; vérifier l'état
  de son API et les conditions d'usage.
