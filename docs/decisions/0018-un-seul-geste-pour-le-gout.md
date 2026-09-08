# 0018 — Un seul geste pour le goût : aimer prime sur les tops

- **Date** : 2026-09-08
- **Statut** : accepté

## Contexte

Après une journée d'écoute, Joel : « en tant que développeur, je comprends
le besoin d'avoir des tops (points d'entrée). En tant qu'utilisateur,
j'avais du mal à savoir s'il valait mieux que je like ou que je mette en
top : pour moi c'était la même chose. »

Deux touches faisaient, à l'oreille, la même chose — `tl` écrivait dans
l'appris, `tt` dans la fiche, pour tout le monde — et la première n'avait
**aucun effet sur un top** : un aimé n'entrait dans le réservoir que s'il
n'était pas déjà un top (poids 0,8 contre 1,0). L'utilisateur qui aimait un
top ne l'entendait pas davantage, et basculait sur `tt`, promouvant son
goût en connaissance partagée sans passer par une PR.

## Décision

- **À l'écoute, un seul geste simple dit « je veux entendre ce morceau
  plus souvent »** : `tl`. Son contraire, `ts`, dit « ce morceau ne
  m'intéresse pas » : il note, retire l'aimé, et passe. `tb` reste le
  « plus jamais ». Aimer efface les « moins souvent » ; l'un défait l'autre.
- **Les aimés priment sur les tops.** Dans le réservoir (0012 §1), un aimé
  pèse **dix tops au cocon et deux grand ouvert**, le curseur de confort
  entre les deux — jamais moins qu'un top. Un top aimé prend le poids de
  l'aimé et porte sa marque `♥`.
- **Les tops ne sont plus que les portes d'entrée d'un fork vierge** : ce
  par quoi on entre chez un artiste qu'on n'a pas encore écouté. Ils
  restent modifiables sur son fork — dans la discographie (`ad`, qui les
  corrige en fournée) ou à la main dans la fiche — mais **il n'y a plus de
  raccourci `tt` / `tT` à l'écoute**.

## Conséquences

- 0013 n'est pas révisée : chaque touche reste une mesure ou une édition ;
  seule la table change, et elle vit dans `keybindings.md` (0015). Les
  éditions restantes à l'écoute sont `td` et `aL`.
- Le goût ne devient jamais silencieusement de la connaissance : la
  **promotion** (« aimé trois fois, hors tops — promouvoir en top ? »)
  reste à concevoir, et demandera une question, jamais un automatisme.
- La distinction se lit là où on choisit : l'aide à la saisie dit « plus
  souvent : j'aime » et « moins souvent : il ne m'intéresse pas ».
