# 0016 — Une base large **et** la génération à la volée

**Date** : 2026-09-06 · **Statut** : acceptée

## Contexte

`docs/conception/premiere-installation.md` (06/09/2026) a relevé un point
dur que rien n'avait écrit : **la base actuelle n'est pas neutre, c'est
l'univers de Joel**. Les 214 fiches sont nées de son classement Spotify puis
de proche en proche. Comme **une graine sans fiche ne peut pas démarrer** —
les branches viennent des liens, des tags et du vecteur de la fiche — un
utilisateur au goût éloigné ne pourrait presque rien lancer.

Trois sorties étaient proposées : une base large et neutre (a), une base
mince avec génération obligatoire (b), ou plusieurs bases par famille de
goût (c).

## Décision

**(a) et (b), ensemble.**

- **Le catalogue de référence vise la largeur.** Il n'est pas la
  bibliothèque de son auteur mais une base utilisable par quelqu'un d'autre :
  on l'élargit en amont, hors session, avec l'outillage. C'est lui que
  l'application propose d'importer au premier lancement.
- **Et l'application génère une fiche à la volée** quand on arrive chez un
  artiste qui n'en a pas. Le catalogue de chacun grandit ainsi vers son
  univers dès la première écoute, ce que
  [conception/catalogue.md](../conception/catalogue.md) appelait déjà « le
  mécanisme central ».

Les deux se complètent au lieu de s'opposer : la largeur fait que
l'installation marche tout de suite, la génération fait qu'elle ne reste
jamais étrangère. Aucune des deux seule ne suffit — une base large finit
toujours par manquer quelqu'un, une base mince rend l'application
inutilisable hors ligne le premier jour.

**(c) n'est pas écartée**, elle devient inutile à trancher :
[0004](0004-deux-depots-catalogue-ciblable.md) permet déjà d'importer
« celui de quelqu'un d'autre parce qu'on le trouve cool ». Des bases par
famille de goût pourront exister sans nouvelle décision.

## Conséquences

- **Le chemin du catalogue actif est un réglage** (`[catalogue] path`), et
  non plus un chemin en dur. L'argument de ligne de commande le surcharge.
- **Un fork par utilisateur, pas un dépôt de différences.** Le catalogue de
  quelqu'un est un **fork du dépôt de référence** — il en contient donc
  *tout*, plus ses commits. C'est ce qui permet à la fois `git pull` depuis
  l'amont et une PR vers lui. Un dépôt qui ne contiendrait que les
  modifications ne saurait faire ni l'un ni l'autre, et contredirait
  [0008](0008-le-fork-est-la-surcouche.md) : « il n'y a pas de surcouche à
  part ».
- **Une fiche générée est une édition** au sens de
  [0013](0013-affinage-clavier-mesure-ou-edition.md) : elle produit un commit
  et porte `generated = true` jusqu'à relecture, comme les fiches d'amorce.
- **La génération demande le réseau.** Hors ligne, arriver chez un artiste
  sans fiche reste un cul-de-sac — l'application le dit plutôt que d'échouer.
- Restent à écrire, et à trancher au fil : l'**import au premier lancement**
  (forkstify clone-t-il, ou demande-t-il une URL ?), le **scan de la
  bibliothèque** de l'utilisateur, et **quand** la génération se déclenche —
  à l'arrivée chez l'artiste, ou sur demande.
