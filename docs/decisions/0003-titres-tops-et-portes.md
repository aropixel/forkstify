# 0003 — Les titres se choisissent par tops, avec ciblage manuel possible

**Date** : 2026-08-30 · **Statut** : acceptée — la partie « portes » est
retirée par [0010](0010-format-revise-links-sans-portes.md)

## Contexte

Le moteur de branches raisonne sur les **artistes** (quelques centaines, on
peut les décrire et les relier). Il faut ensuite choisir quel **morceau**
jouer. Décrire chaque morceau individuellement rendrait les fiches trop
lourdes.

## Décision

- **Les tops en priorité** : chaque fiche liste les morceaux qui forment le
  réservoir par défaut de l'artiste. C'est ce qu'on joue quand on le ponce ou
  qu'on arrive chez lui.
- **Un ciblage manuel de morceaux précis reste possible** : une fiche peut
  désigner des **portes**, morceaux choisis à la main comme sortie vers une
  direction donnée (*A Forest* → post-punk, *Friday I'm in Love* → pop). Une
  porte est optionnelle, un top est la norme.

## Conséquences

- Les fiches restent légères : une liste de titres, plus quelques portes
  annotées quand on a quelque chose à dire.
- Le moteur peut préférer une porte à un top quand il change de direction, et
  un top quand il reste sur place.
- Seuls les artistes sont vectorisés ; les morceaux ne le sont pas.
