# 0005 — Les fiches portent un numéro de version

**Date** : 2026-08-30 · **Statut** : acceptée

## Contexte

Le format des fiches est une interface publique ([0002](0002-catalogue-partage-forkable.md)) :
des catalogues écrits par des gens différents, à des moments différents,
seront lus par des versions différentes de l'application ([0004](0004-deux-depots-catalogue-ciblable.md)).

## Décision

Chaque fiche porte un **numéro de version du format** (le schéma de la
fiche), pas de son contenu — git versionne déjà le contenu.

## Conséquences

- L'application sait si elle peut lire une fiche, et comment la migrer si
  elle est plus ancienne que ce qu'elle attend.
- Un champ en tête de fiche, `format: 1` provisoirement ; le nom exact se
  fixe avec le format ([conception/catalogue.md](../conception/catalogue.md)).
