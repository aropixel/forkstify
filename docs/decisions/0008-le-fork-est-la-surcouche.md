# 0008 — Pas de surcouche à part : le fork est la surcouche

**Date** : 2026-08-30 · **Statut** : acceptée · **Remplace** la partie
« deux couches » de [0002](0002-catalogue-partage-forkable.md)

## Contexte

[0002](0002-catalogue-partage-forkable.md) prévoyait deux couches : une base
partagée et une surcouche personnelle appliquée par-dessus. Puis
[0004](0004-deux-depots-catalogue-ciblable.md) a fixé le modèle d'import :
on clone un catalogue, on le déclare actif, un seul actif à la fois.

Avec ce modèle, mes modifications vivent naturellement dans le clone : mes
tops de The Cure sont un commit sur mon fork. Une surcouche séparée
garderait mes tops quand je bascule sur le catalogue de quelqu'un d'autre —
c'est le contraire de « c'est celui-là que j'utilise ».

## Décision

**Il n'y a pas de surcouche à part. Le catalogue actif est un clone git, et
les modifications personnelles sont des commits dedans.** S'approprier le
catalogue, c'est le forker au sens propre.

## Conséquences

- Une seule règle de résolution : la fiche du catalogue actif, point. Pas de
  fusion champ par champ.
- L'application aide à **commiter** (une fiche modifiée ou générée) et à
  **récupérer les mises à jour de l'amont** (`git pull` du dépôt d'origine,
  résolution des conflits). C'est là que passe la complexité qu'on retire
  ailleurs.
- Basculer de catalogue change tout, tops personnels compris. C'est voulu.
