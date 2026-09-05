# Raccourcis

**Référence unique des touches de forkstify.** Ce document fait foi : la
table a vécu en trois exemplaires (le code, `forme-de-l-application.md`,
`retours-usage.md`) jusqu'au 05/09/2026, et c'est ce qui avait laissé
s'installer huit collisions. Les autres notes y renvoient désormais.

La grammaire est fixée par la décision
[0015](decisions/0015-grammaire-clavier-namespaces.md) ; ce document est
seul juge de la table elle-même, qui s'ajuste sans rouvrir la décision.

## La règle, en une ligne

> **`f` la branche · `e` encore · `t` le morceau · `a` l'artiste** —
> le reste du clavier ne sert qu'à naviguer et à piloter la session.

Le premier caractère dit *sur quoi* on agit, le second *ce qu'on fait*.
**Espace est le leader** : hors grammaire, il montre ce qu'on peut taper —
tout, ou seulement le namespace en cours de frappe, comme which-key dans
LazyVim.

Chaque touche vient d'un **mot anglais**, à la vim (`y` yank, `c` change) :
`f` fork, `e` encore, `t` track, `a` artist, puis `l` like, `s` skip,
`b` ban, `m` mark, `t` top, `d` door, `e` edit, `L` link, `p` peek/pause,
`r` reroll, `w` wander, `u` undo.

## Légende

| Marque | Sens |
|---|---|
| ✅ | Câblé, utilisable dans `ecouter` |
| 📋 | Décidé (0015), **pas encore câblé** — la touche répond « pas encore câblé » au lieu de ne rien faire |

## `f` — la branche

| Touche | Mot | Action | |
|---|---|---|---|
| `f<n>` | **fork** | Branche n, **après la branche en cours** | ✅ |
| `fn<n>` | fork **now** | …après le morceau en cours, le reste conservé | ✅ |
| `f!<n>` | fork now, **force** | …après le morceau en cours, le reste retiré | ✅ |
| `fp` | fork **peek** | Prévoir les branches sans attendre le dernier morceau | ✅ |
| `fr` | fork **reroll** | Reproposer trois autres branches | ✅ |
| `fu` | fork **undo** | Revenir à la branche précédente | ✅ |
| `fw` | fork **wander** | Partir loin, hors de l'univers courant | 📋 |
| `1`…`9` | | Raccourci de `f1`…`f9` | ✅ |
| entrée | | Auto : tire au sort parmi les branches affichées | ✅ |

Le `!` est le *force* de vim (`:w!`) : « et tant pis pour ce qui suivait ».
Le `n` est *now*. Ça se lit à voix haute : « fork now 3 », « fork force 3 ».

Après `f`, un **chiffre** désigne une branche, une **lettre** une opération.

## `e` — encore

| Touche | Mot | Action | |
|---|---|---|---|
| `e<n>` | **encore** | n morceaux de plus de l'artiste, **en fin de branche** | ✅ |
| `en<n>` | encore **now** | …après le morceau en cours, le reste conservé | ✅ |
| `e!<n>` | encore now, **force** | …après le morceau en cours, le reste retiré | ✅ |

`e` seul n'est pas une commande : le compte est obligatoire.

`f` et `e` sont les namespaces de **lecture** — ils décident de ce qui va
sonner. `t` et `a` sont ceux de l'**affinage** — ils décident de ce que le
moteur retient.

## `t` — le morceau en cours

| Touche | Mot | Action | Nature | |
|---|---|---|---|---|
| `tl` | track **like** | Aimer (reflété en titre aimé Spotify) | mesure | 📋 |
| `ts` | track **skip** | « Pas celui-là, pas maintenant » | mesure | 📋 |
| `tb` | track **ban** | « Plus jamais celui-là » | mesure | 📋 |
| `tm` | track **mark** | Mettre dans une récolte à trier plus tard | mesure | 📋 |
| `tt` | track **top** | Promouvoir en top | édition | 📋 |
| `tT` | track **untop** | Retirer des tops | édition | 📋 |
| `td` | track **door** | En faire une door vers la dernière direction prise | édition | 📋 |

## `a` — l'artiste en cours

| Touche | Mot | Action | Nature | |
|---|---|---|---|---|
| `al` | artist **like** | Cet artiste, plus souvent | mesure | 📋 |
| `as` | artist **skip** | Cet artiste, moins souvent | mesure | 📋 |
| `ab` | artist **ban** | Plus jamais cet artiste | mesure | 📋 |
| `ae` | artist **edit** | Ouvrir la fiche dans `$EDITOR`, recommit, vecteur recalculé | édition | 📋 |
| `aL` | artist **link** | Lier à un autre artiste (format [0010](decisions/0010-format-revise-links-sans-portes.md)) | édition | 📋 |

Les trois verbes forment sur l'artiste une **échelle lisible** : `al` plus
souvent, `as` moins souvent, `ab` plus jamais.

**Tout ce namespace et le précédent attendent `learned/`** — la boucle
d'apprentissage ([0014](decisions/0014-forme-de-l-appris.md)), étape 2 de
[`avancement.md`](avancement.md). Les mesures n'ont nulle part où
s'écrire tant qu'elle n'est pas fermée.

## Navigation et session

| Touche | Mot | Action | |
|---|---|---|---|
| `h` / `l` | | Morceau **précédent / suivant** — vim, axe horizontal | ✅ |
| ← / → | | Idem, pour les doigts hors de la rangée d'accueil | ✅ |
| `p` | **pause** | Pause / lecture | ✅ |
| espace | | **Le leader** : les touches disponibles, ou celles du namespace en cours de frappe | ✅ |
| `/texte` | | Chercher — catalogue + Spotify, un chiffre choisit | ✅ |
| `q` | **quit** | Quitter (affiche le parcours) | ✅ |
| `u` | **undo** | Annuler la dernière action : mesure ou édition (0013) | 📋 |
| `.` | | Répéter la dernière action (son sens vim) | 📋 |
| `?` | **why** | La phrase qui explique le morceau ou la branche | 📋 |
| `Q` | **queue** | Entrer en mode file d'attente | 📋 |

**`u` et `fu` ne sont pas la même chose** : `u` annule le dernier *geste*
(un top posé de travers, un ban), `fu` remonte d'un cran dans le *parcours*.

## Commandes `:`

0013 veut que chaque touche soit le raccourci d'une commande `:`. Seule
`:size` est servie pour l'instant — elle a remplacé l'ancien `b<n>`.

| Commande | Action | |
|---|---|---|
| `:size <n>` | Taille des branches, 1 à 9 (sans argument : l'affiche) | ✅ |
| `:comfort <0-5>` | Zone de confort ([0001](decisions/0001-confort-familiarite.md)) | 📋 |
| `:sync` `:push` `:pull` | Synchroniser usage et fiches entre machines | 📋 |
| `:fork` | Forker le catalogue ([0008](decisions/0008-le-fork-est-la-surcouche.md)) | 📋 |

## Touches multimédia (MPRIS / D-Bus)

Actives dès que MPRIS s'enregistre, comme pour `playerctl`.

| Touche | Action | |
|---|---|---|
| ⏭ | Suivant (= `l`) | ✅ |
| ⏮ | Précédent (= `h`) | ✅ |
| ⏯ | Pause / lecture (= `p`) | ✅ |
| ⏹ | Arrêt | ✅ |

## Mode file d'attente

Un mode à part, avec sa propre table, encore à concevoir : préparer les
branches à l'avance, retirer un morceau, retirer une branche (**seule, ou
toute la profondeur qui en découle**), intercaler.

Point structurant repéré : `rounds` est aujourd'hui une **liste plate**,
alors que « retirer toute la profondeur » suppose un arbre manipulable.

## Comment la saisie fonctionne

Depuis le 05/09/2026, `ecouter` lit le clavier en **mode brut** : chaque
touche agit sans Entrée (`src/keys.rs`, termios via `libc`, garde RAII qui
rend le terminal même sur panique). `/` et `:` quittent le mode brut pour
une ligne éditable, où une requête a sa place.

La grammaire est **sans préfixe** : aucune commande complète n'est le début
d'une plus longue. C'est ce qui permet de déclencher **sans délai ni
timeout**, là où vim s'en remet à `timeoutlen`. Un test exhaustif sur
toutes les séquences de trois touches vérifie la propriété
(`grammar_is_prefix_free`), pour qu'un ajout futur ne la casse pas en
silence.

Cette contrainte a décidé deux choses :

- **Le modificateur précède le compte** (`fn3`, `f!3`), et non l'inverse :
  `f3n` rendrait `f3` à la fois complet et préfixe.
- **Le compte suit le namespace** (`f3`, `e3`), et non `3e` comme dans vim.
  Une frappe de `3` serait sinon à la fois « branche 3 » et « début d'un
  compte », indécidable sans attendre la touche suivante — ce qui
  ralentirait le geste le plus fréquent. Avantage collatéral : `f` et `e`
  deviennent symétriques.

## Le prix à payer

**Les gestes fréquents coûtent deux frappes** (`tl` pour aimer, `ts` pour
passer), là où vim garde une touche pour ce qu'on fait le plus. C'est le
coût de la régularité, et il ne se jugera qu'à l'usage.

Deux garde-fous : `1`…`9` restent le raccourci de `f1`…`f9` — l'exception
assumée, choisir une branche étant *le* geste du produit ; et le clavier nu
est assez vide pour qu'on y promeuve plus tard un geste qui se révélerait
constant.

## Ce qui reste à trancher

1. **`ts` (skip track) vs `l` (suivant).** Deux gestes pour passer un
   morceau, avec une différence invisible : `l` avance sans rien noter,
   `ts` avance **et** le note dans `learned/`. Nuance juste sur le papier,
   peut-être insensible dans les doigts.
2. **`aL` ou `ac`** pour lier deux artistes. `aL` est le seul geste dont la
   casse ne dit pas la même chose qu'ailleurs : `tt`/`tT` sont un verbe et
   son inverse, `al`/`aL` sont deux verbes différents. La justification
   tient (minuscule = mesure, majuscule = édition) mais elle est plus
   faible. `ac` (*connect*) l'éviterait, au prix du mot « link », qui est
   celui du format sur disque.

## Lettres libres

Le clavier nu ne garde que `h`, `l`, `p`, `e`, `f`, `t`, `a`, `u`, `q`, `Q`.
Restent libres : `b`, `c`, `d`, `g`, `i`, `j`, `k`, `m`, `n`, `o`, `r`,
`s`, `v`, `w`, `x`, `y`, `z`, et toutes les majuscules hors `Q`. Le mode
file d'attente peut s'installer sans rien déplacer.

## D'où vient cette grammaire

Le 05/09/2026, l'inventaire des touches a révélé **huit collisions**, dont
six invisibles tant que la table vivait en trois exemplaires. Les
namespaces les font toutes tomber, et par construction : deux gestes ne
peuvent se croiser que dans un même namespace, où l'on maîtrise les lettres.

| Collision d'alors | Résolution |
|---|---|
| `u` : « annuler » (0013) vs « branche précédente » (le code) | `u` annule un geste, `fu` remonte d'une branche |
| `n` : sauter, non, et le modificateur « maintenant » | `n` = **now** ; `y`/`n` ne vit que dans une invite modale |
| `d` à la fois action (door) et préfixe (`da`/`dt`) | `d` quitte le clavier nu : c'est `td` |
| `dt`/`da` recouvrent `X` et `-`, déjà décidés | Absorbés par `tb`, `as` et `ab` |
| `.` recouvre `e` (deux touches pour encore) | `.` reprend son sens vim (répéter) |
| `h`/`l` (rassurant/aventureux) recouvrent `1 2 3` | `h`/`l` deviennent la navigation |
| `p` à la fois action (prévoir) et préfixe (`p1`) | `p` quitte les branches : c'est `fp`, et `p` devient pause |
| `pr` en collision avec `p<n>` | `fr`, dans le namespace |

**« Fork » a deux sens, et c'est assumé** (arbitrage de Joel, 05/09/2026) :
forker le **catalogue** ([0008](decisions/0008-le-fork-est-la-surcouche.md))
est un geste rare, une fois par machine, qui reste `:fork` ; forker le
**parcours** est le geste constant de l'écoute, et c'est la touche `f`. Ils
ne se croisent jamais. La nuance est au vocabulaire de
[`vision.md`](vision.md).
