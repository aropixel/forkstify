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
**Espace est le leader** : hors grammaire, il ouvre l'**aide à la saisie**
— tout ce qu'on peut taper, ou seulement le namespace en cours de frappe,
comme which-key dans LazyVim. Ce n'est pas une affiche : la séquence reste
en cours, et la touche tapée dans l'aide fait l'action (`espace`, `e`, `3`
= `e3`). `⌫` remonte d'un niveau, `échap` ferme (Joel, 07/09/2026).

Chaque touche vient d'un **mot anglais**, à la vim (`y` yank, `c` change) :
`f` fork, `e` encore, `t` track, `a` artist, puis `l` like, `s` skip,
`b` ban, `m` mark, `t` top, `d` door, `e` edit, `L` link, `p` peek/pause,
`r` reroll, `w` wander, `u` undo.

> **Chemins renommés le 06/09/2026.** Le catalogue parle anglais sur disque,
> comme `AGENTS.md` et [0010](decisions/0010-format-revise-links-sans-portes.md)
> l'exigent : `fiches/` → `cards/`, `outillage/` → `tools/`, `vecteurs/` →
> `vectors/`, `catalogue.toml` → `catalog.toml`. Les décisions antérieures
> sont **immuables** et mentionnent les anciens noms : y lire les nouveaux.
> Le vocabulaire français ne change pas — on dit toujours « une fiche ».

## Légende

| Marque | Sens |
|---|---|
| ✅ | Câblé, utilisable dans `ecouter` |
| 📋 | Décidé (0015), **pas encore câblé** — la touche répond « pas encore câblé » au lieu de ne rien faire |

## `f` — la branche

| Touche | Mot | Action | |
|---|---|---|---|
| `f<n>` | **fork** | Branche n, **ajoutée à la suite de ce qui est déjà décidé** — on enchaîne les choix et la soirée se construit | ✅ |
| `fn<n>` | fork **now** | …après le morceau en cours, le reste conservé | ✅ |
| `f!<n>` | fork now, **force** | …après le morceau en cours, le reste retiré | ✅ |
| `fp` | fork **peek** | Sans emploi depuis le 06/09 : les branches sont **affichées en permanence**, à droite. La touche le dit plutôt que de ne rien faire | ✅ |
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
| `tl` | track **like** | Aimer le morceau | mesure | ✅ |
| `ts` | track **skip** | « Pas celui-là, pas maintenant » — note et passe | mesure | ✅ |
| `tb` | track **ban** | « Plus jamais celui-là » — le retire aussi de la file | mesure | ✅ |
| `tm` | track **mark** | Mettre dans `learned/marks/inbox.toml` | mesure | ✅ |
| `tx` | track **remove** | Retirer de la file le morceau sélectionné — il reste proposable, ce n'est pas un ban | file | ✅ |
| `tt` | track **top** | Promouvoir en top | édition | ✅ |
| `tT` | track **untop** | Retirer des tops | édition | ✅ |
| `td` | track **door** | En faire une door vers la direction où l'on va (les tags de l'artiste suivant) | édition | ✅ |

## `a` — l'artiste en cours

| Touche | Mot | Action | Nature | |
|---|---|---|---|---|
| `al` | artist **like** | Cet artiste, plus souvent (poids ×1.43, plafond 3) | mesure | ✅ |
| `as` | artist **skip** | Cet artiste, moins souvent (poids ×0.7, plancher 0.1) | mesure | ✅ |
| `ab` | artist **ban** | Plus jamais cet artiste — vide aussi la file | mesure | ✅ |
| `ae` | artist **edit** | Affiche le chemin de la fiche. L'ouvrir sur place attend une saisie interrogée : le lecteur de touches tient `stdin` en permanence et volerait ses frappes à `$EDITOR` | 📋 |
| `aL` | artist **link** | Lier à l'artiste **d'où l'on vient**, type `similar` ([0010](decisions/0010-format-revise-links-sans-portes.md)) | édition | ✅ |

Les trois verbes forment sur l'artiste une **échelle lisible** : `al` plus
souvent, `as` moins souvent, `ab` plus jamais.

**Les mesures des deux namespaces écrivent dans `learned/`** depuis le
05/09/2026 ([0014](decisions/0014-forme-de-l-appris.md)) : un fichier TOML
par artiste dans le catalogue, compteurs à décroissance intégrée
(demi-vie six mois), écrit à chaque geste, silencieux et jamais reversé.

**Les éditions écrivent dans les fiches depuis le 06/09/2026** : `tt`,
`tT`, `td` et `aL` modifient une fiche **et produisent un commit lisible**
(`src/edit.rs`). La fiche est retouchée textuellement, jamais réécrite —
c'est une interface publique, et une relecture par serde perdrait tout ce
que le code ne modélise pas. Une édition ne compte pour le moteur qu'au
**prochain lancement**, et le produit le dit.

## Navigation et session

| Touche | Mot | Action | |
|---|---|---|---|
| `h` / `l` | | Morceau **précédent / suivant** — vim, axe horizontal | ✅ |
| ← / → | | Idem, pour les doigts hors de la rangée d'accueil | ✅ |
| `p` | **pause** | Pause / lecture | ✅ |
| ↑ / ↓ | | Déplacer la **sélection** dans l'axe — elle surligne, elle ne joue pas | ✅ |
| entrée | | Jouer la sélection ; sans sélection, tirer une branche | ✅ |
| `c` | **comfort** | Régler la zone de confort : ↑↓ bougent, entrée valide, échap annule | ✅ |
| échap | | Annuler la sélection, fermer un bloc — l'aide comprise | ✅ |
| espace | | **Le leader** : ouvre l'aide à la saisie — tout, ou le namespace en cours de frappe ; la séquence continue dedans, une touche fait l'action. Espace au niveau d'entrée la referme | ✅ |
| ⌫ | | Effacer la dernière touche de la séquence — dans l'aide, remonter d'un niveau | ✅ |
| `/texte` | | Chercher — catalogue + Spotify, un chiffre choisit | ✅ |
| `q` | **quit** | Quitter (affiche le parcours) | ✅ |
| `r` | **resume** | Reprendre le dernier parcours — **accueil seulement** | ✅ |
| `s` | **sort** | Changer l'ordre de la collection : familiarité → a-z → dernière écoute — **accueil seulement** | ✅ |
| `gg` / `G` | | Les deux bouts d'une liste, comme dans vim — la collection à l'accueil, l'axe en écoute. `g` seul attend son second | ✅ |
| `b` | **browse** | Parcourir à sec — **écrans non connectés seulement** | 📋 |
| `u` | **undo** | Annuler la dernière action : mesure ou édition (0013) | 📋 |
| `.` | | Répéter la dernière action (son sens vim) | 📋 |
| `?` | **why** | Tags, familiarité, poids, liens, et la première branche d'ici | ✅ |
| `Q` | **queue** | ~~Mode file d'attente~~ — **largement caduc** depuis que la file s'enchaîne (06/09/2026) : préparer, voir, parcourir et retirer se font dans l'écran d'écoute. Restent quatre gestes à ajouter là où l'on est : retirer une branche, déplacer, intercaler, annuler | 📋 |

**`u` et `fu` ne sont pas la même chose** : `u` annule le dernier *geste*
(un top posé de travers, un ban), `fu` remonte d'un cran dans le *parcours*.

## D'où vient chaque morceau

Toute chanson affichée — dans la file, dans les branches, sur la ligne `▶`
— porte la marque de sa **provenance**. Le réservoir d'un artiste cumule
plusieurs sources ([0012](decisions/0012-rotation-des-morceaux.md) §1 :
« le top est un poids, pas une liste fermée »), et la marque dit laquelle
a gagné le tirage.

| Marque | Provenance |
|---|---|
| `♪` | Un **top** de la fiche |
| `♥` | Un titre **aimé** ici (`tl`), qui n'est pas un top |
| `↳` | Une **door** ([0011](decisions/0011-doors-critere-additionnel.md)) dont la direction recoupe celle de la branche. Le glyphe lui est réservé : la recherche dit `(branche ensuite)` en toutes lettres, un glyphe ne portant qu'un sens (05/09/2026) |
| `+` | Artiste connu, morceau **hors tops** — c'est celui que `tt` promouvrait |
| `·` | La **longue traîne** : le reste de la discographie, qui ne pèse qu'à mesure que le confort s'ouvre |
| `~` | **Hors catalogue** : joué depuis Spotify, sans fiche |

Une door ne prend sa marque que **quand elle s'ouvre** : hors de sa
direction, elle reste un morceau comme un autre. C'est ce que 0011 appelle
« critère additionnel, jamais principal ».

## Commandes `:`

0013 veut que chaque touche soit le raccourci d'une commande `:`. Seule
`:size` est servie pour l'instant — elle a remplacé l'ancien `b<n>`.

| Commande | Action | |
|---|---|---|
| `:size <n>` | Taille des branches, 1 à 9 (sans argument : l'affiche) | ✅ |
| `:comfort <n>` | Zone de confort, **5 = cocon → 0 = exploration** ([0001](decisions/0001-confort-familiarite.md)) ; sans argument, l'affiche | ✅ |
| `:warm` | Récolter la discographie de l'artiste en cours (la longue traîne) | ✅ |
| `:sync` / `:push` | | Commiter et pousser l'appris maintenant — sinon toutes les dix minutes, à la sortie, et pull au démarrage ([0017](decisions/0017-synchronisation-de-l-appris.md)) | ✅ |
| `:mine` | Ce que ce catalogue a de plus que l'amont — la surcouche personnelle, calculée par `git diff` plutôt que stockée ([0008](decisions/0008-le-fork-est-la-surcouche.md)) | ✅ |
| `:sync` `:push` `:pull` | Synchroniser usage et fiches entre machines | 📋 |
| `:fork` | Forker le catalogue ([0008](decisions/0008-le-fork-est-la-surcouche.md)) | 📋 |

Et en sous-commande, parce qu'elles n'ont pas leur place au milieu d'une
écoute : `forkstify import <url>` reprend les fiches d'un autre catalogue —
celles qu'on n'a pas, jamais celles qu'on a — puis régénère les vecteurs.

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
