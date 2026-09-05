# 0015 — Grammaire clavier : quatre namespaces

**Date** : 2026-09-05 · **Statut** : acceptée

## Contexte

Les premières sessions longues d'`ecouter` (05/09/2026) ont produit onze
retours d'usage, dont celui-ci : « je commence à me mélanger, je ne veux
pas rajouter et que ça devienne inutilisable ».

L'inventaire a donné raison à l'inquiétude. La table des touches vivait en
trois exemplaires (le code, `forme-de-l-application.md`,
`retours-usage.md`), **onze gestes sur une trentaine** étaient câblés, et
les rassembler a révélé **huit collisions** : `u` voulait dire deux choses,
`n` trois, `d` et `p` étaient à la fois des actions et des préfixes, `dt`
et `da` doublonnaient des gestes déjà décidés, `.` doublonnait `e`, `h`/`l`
doublonnaient `1 2 3`.

La décision [0013](0013-affinage-clavier-mesure-ou-edition.md) fixait déjà
le *principe* (chaque touche est une mesure ou une édition, chacune est le
raccourci d'une commande `:`) mais laissait la table en orientation. Elle
n'a pas tenu à l'épreuve de l'ajout.

## Décision

**Le premier caractère dit sur quoi on agit, le second ce qu'on fait.**
Quatre namespaces :

> **`f` la branche · `e` encore · `t` le morceau · `a` l'artiste**

Le reste du clavier ne sert qu'à naviguer et à piloter la session
(`h`/`l` et les flèches, espace, `u`, `q`, `Q`, `.`, `?`, `/`, `:`).

Trois règles :

1. **Une touche vient d'un mot anglais**, à la vim (`y` yank, `c` change) —
   `f` fork, `e` encore, `t` track, `a` artist, puis `l` like, `s` skip,
   `b` ban, `m` mark, `t` top, `d` door, `e` edit, `L` link, `p` peek,
   `r` reroll, `w` wander, `u` undo.
2. **La cible se préfixe, elle ne se suffixe pas.** `tl` aime le morceau,
   `al` aime l'artiste. Deux gestes ne peuvent donc se croiser que dans un
   même namespace, où l'on maîtrise les lettres.
3. **Un geste fréquent a une touche, un réglage a une commande `:`.**
   `:size`, `:comfort`, `:sync`, `:fork`.

**La table fait foi dans [`docs/keybindings.md`](../keybindings.md)** — un
seul exemplaire, plus jamais trois.

## Conséquences

- **Les huit collisions tombent**, et par construction : le namespace les
  rend impossibles ailleurs que chez lui.
- **Le clavier nu se vide** — il ne garde que `h l e f t a u q Q` — ce qui
  laisse dix-huit lettres pour le mode file d'attente et la suite.
- **`u` et `fu` se séparent** : `u` annule le dernier geste (0013 est donc
  respectée, pas révisée), `fu` remonte d'un cran dans le parcours.
- **Les gestes fréquents coûtent deux frappes** (`tl`, `ts`) là où vim en
  garde une. C'est le prix de la régularité, et il ne se jugera qu'à
  l'usage. `1` `2` `3` restent le raccourci de `f1` `f2` `f3` — l'exception
  assumée, choisir une branche étant *le* geste du produit. Si un autre
  geste se révèle constant, le clavier nu est assez vide pour l'y promouvoir.
- **Le compte se met après le verbe** : `f3` la branche 3, `e3` trois
  encores. La règle vim (`3dd`) voudrait l'inverse, mais elle est
  **incompatible** avec le raccourci `1 2 3` : une frappe de `3` serait à
  la fois « branche 3 » et « début d'un compte », et l'analyseur ne peut
  pas trancher sans attendre la touche suivante — ce qui rendrait lent le
  geste le plus fréquent. Le compte suit donc le namespace, ce qui a
  l'avantage de rendre `f` et `e` symétriques. Corollaire : `e` seul n'est
  pas une commande, le compte est obligatoire.
- **La saisie passe en mode brut** (sans Entrée), sauf `/` et `:` qui
  ouvrent une ligne — c'est le retour n° 1, et c'est ce qui rend la
  grammaire tapable.
- 0013 n'est pas révisée : son principe (mesure / édition, `u`, commandes
  `:`) est repris tel quel. Seule sa **table**, qui n'était qu'une
  orientation, est remplacée.
