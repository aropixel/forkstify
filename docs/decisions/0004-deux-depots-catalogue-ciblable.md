# 0004 — Deux dépôts ; l'application cible ou importe un catalogue au choix

**Date** : 2026-08-30 · **Statut** : acceptée

## Contexte

Le catalogue est partagé et forkable ([0002](0002-catalogue-partage-forkable.md)).
Application et catalogue ont des cycles de vie et des contributeurs
différents : on ne forke pas une application pour changer ses tops de
The Cure.

## Décision

- **Deux dépôts** : `forkstify` (l'application) et un dépôt de catalogue
  (nom à fixer, `forkstify-catalogue` provisoirement).
- L'application **importe** un dépôt de catalogue — le catalogue de
  référence, son propre fork, ou celui de quelqu'un d'autre « parce qu'on le
  trouve cool ». **Importer = cloner**, puis déclarer « c'est celui-là que
  j'utilise ».
- **Un seul catalogue actif à la fois.** On peut en avoir importé plusieurs
  et **basculer** de l'un à l'autre quand on en a envie.

## Conséquences

- L'application ne contient aucune fiche ; elle connaît une liste de
  catalogues clonés localement et lequel est actif.
- Pas de fusion entre catalogues : basculer, c'est changer de monde. Ce qui
  simplifie beaucoup la résolution des fiches.
- Ouvre une question sur la surcouche personnelle de [0002](0002-catalogue-partage-forkable.md) :
  si le catalogue actif est un clone que l'on modifie et commite, le fork
  *est* la surcouche. Voir [conception/catalogue.md](../conception/catalogue.md).
