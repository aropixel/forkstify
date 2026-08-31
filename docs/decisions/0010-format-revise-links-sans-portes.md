# 0010 — Format révisé : champs anglais, liens typés avec proximité, plus de portes

**Date** : 2026-08-31 · **Statut** : acceptée — la suppression des portes
est revenue en critère additionnel via [0011](0011-doors-critere-additionnel.md) · **Amende** [0003](0003-titres-tops-et-portes.md)
(les portes sont retirées) et précise [0007](0007-fiches-en-toml.md)

## Contexte

La relecture des trente premières fiches par Joel a fait émerger trois
demandes : des types de connexion en anglais (ou une proximité chiffrée),
le doute sur les portes (« ce qui me parle, c'est le passage d'un artiste à
un autre, en ciblant les morceaux que j'aime — pas "après A Forest j'ai
envie d'autre chose" »), et une organisation qui rende le paramétrage plus
facile.

## Décision

- **Les clés du format sont en anglais** (`name`, `begin`, `origin`,
  `links`, `to`, `generated`…) : le format est une interface publique,
  au-delà du français. Le *contenu* (descriptions, notes) reste dans la
  langue de chaque catalogue.
- **`links` remplace les connexions en blocs** : un lien = une ligne (table
  TOML en ligne). **Types fermés, en anglais** : `member`, `family`,
  `collab`, `scene`, `similar`, `influence`.
- **Chaque type a une proximité par défaut**, réglée une fois dans
  `catalogue.toml` à la racine du catalogue (`member = 5` … `influence = 2`) ;
  une connexion peut la corriger localement avec `proximity = 1..5`
  (5 = quasi le même univers). Le type nomme et explique la branche, la
  proximité donne la distance au moteur — jamais l'un sans l'autre.
- **Les portes disparaissent du format.** Le choix du morceau chez l'artiste
  d'arrivée revient au moteur : tops + signaux d'usage + zone de confort
  (familier ou découverte), pas une sortie gravée à la main.
- `fiches/` reste **plat** ; les champs sont ordonnés par fréquence
  d'édition (identité compacte, puis tags, tops, links, description).

## Conséquences

- Les 30 fiches du premier lot sont régénérées dans ce format ;
  `catalogue.toml` créé.
- La grille type → proximité par défaut est un réglage du catalogue que
  chacun peut modifier dans son fork : le « paramétrage de son algorithme »
  a une adresse.
