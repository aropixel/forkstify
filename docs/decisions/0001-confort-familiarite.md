# 0001 — La zone de confort mesure la familiarité

**Date** : 2026-08-30 · **Statut** : acceptée

## Contexte

Le réglage « zone de confort » (0 à 5) doit mesurer une distance. Deux
définitions étaient possibles :

- la proximité avec le **morceau courant** (cohérence de l'enchaînement) ;
- la proximité avec **ce que l'utilisateur connaît** (bibliothèque, historique).

Elles divergent : un grand saut vers un artiste qu'on adore est loin du morceau
courant mais parfaitement confortable.

## Décision

**Confort = familiarité.** La zone de confort mesure à quel point on accepte
de s'éloigner de ce qu'on connaît. La cohérence de l'enchaînement avec le
morceau courant est un paramètre distinct, implicite, géré par le moteur de
branches.

## Conséquences

- Le moteur a besoin de savoir ce que l'utilisateur connaît : sa bibliothèque
  Spotify, son historique d'écoute, ses parcours passés.
- Une branche « Décalage » peut être confortable si elle atterrit en terrain
  connu ; une branche « Voisinage » peut être inconfortable si les voisins
  sont inconnus. Le type de branche et son niveau de confort sont deux axes
  indépendants.
