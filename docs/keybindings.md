# Raccourcis

Référence unique des touches de forkstify : ce qui marche, ce qui est
décidé mais pas câblé, ce qui est proposé. Ouvert le **05/09/2026** parce
que la table vivait en trois exemplaires (le code, `forme-de-l-application.md`,
`retours-usage.md`) et qu'on commençait à s'y perdre.

**C'est ici que la table vit désormais.** Les autres notes y renvoient.

## Légende

| Marque | Sens |
|---|---|
| **✅** | Implémenté et utilisable aujourd'hui dans `ecouter` |
| **📋** | Décidé (0013) ou orienté (`forme-de-l-application.md`), **pas câblé** |
| **💡** | Proposé le 05/09/2026 par les retours d'usage, non tranché |
| **⚠️** | En collision avec une autre touche — voir « Collisions » |

Rappel de 0013 : chaque touche n'est que le raccourci d'une **commande `:`**,
et chaque geste est soit une **mesure** (écrit dans `learned/`, silencieux),
soit une **édition** (modifie une fiche, produit un commit).

## Navigation et lecture

| Touche | Action | Statut |
|---|---|---|
| `j` | Morceau suivant (force la branche en attente) | ✅ |
| `k` | Morceau précédent | ✅ |
| `q` | Quitter (affiche le parcours) | ✅ |
| `/texte` | Chercher — catalogue + Spotify, un chiffre choisit | ✅ |
| — | **Pause / lecture au clavier** | 💡 **manque** : seules les touches multimédia le font |
| `?` | Pourquoi : la phrase qui explique le morceau ou la branche | 📋 |

## Branches — la grammaire

Le cœur du chantier du 05/09. Un geste, puis deux modificateurs :
`n` = « maintenant », `n!` = « maintenant, et tant pis pour ce qui suivait ».

| Touche | Action | Statut |
|---|---|---|
| `1` `2` `3` | Choisir une branche | ✅ mais **jette le reste du segment** (= la variante `!`) |
| `p1` | Branche 1, **après la branche en cours** | 💡 défaut souhaité |
| `p1n` | Branche 1, après le **morceau** en cours, le reste conservé | 💡 |
| `p1n!` | Branche 1, après le morceau en cours, le reste **retiré** | 💡 (= comportement actuel de `1`) |
| entrée | Auto : tire une branche au sort pondéré | ✅ parmi celles affichées (corrigé le 05/09) |
| `p` | Prévoir les branches sans attendre le dernier morceau | ✅ ⚠️ préfixe de `p1` |
| `r` ou `pr` | **Reproposer** trois autres branches | 💡 touche à trancher |
| `u` | Branche précédente | ✅ ⚠️ 0013 dit « annuler » |
| `h` / `l` | Choisir rassurant / aventureux | 📋 ⚠️ fait doublon avec `1 2 3` |
| `b<n>` | Taille des branches (1 à 9) | ✅ |
| `z` puis `0`–`5` | Zone de confort en cours de route (0001) | 📋 |

## Encore — même grammaire

| Touche | Action | Statut |
|---|---|---|
| `<n>e` | Encore : n morceaux de plus de l'artiste en cours, **en fin de branche** | 💡 défaut souhaité |
| `<n>en` | …après le **morceau** en cours, le reste conservé | 💡 (= comportement actuel de `<n>e`) |
| `<n>en!` | …après le morceau en cours, le reste **retiré** | 💡 |
| `e` | Encore avec n = taille des branches | ✅ |
| `.` | Poncer | 📋 ⚠️ fait doublon avec `e` |

## Sur le morceau en cours

| Touche | Action | Nature | Statut |
|---|---|---|---|
| `t` | Promouvoir en top | édition | 📋 |
| `T` | Retirer des tops | édition | 📋 |
| `a` | Aimer (reflété en titre aimé Spotify) | mesure | 📋 |
| `x` | Sauter — « pas celui-là, pas maintenant » | mesure | 📋 ⚠️ doublon avec `n` |
| `X` | Écarter — « plus jamais celui-là » | mesure | 📋 ⚠️ c'est déjà `dt` |
| `dt` | Dislike track | mesure | 💡 ⚠️ recouvre `X` |
| `d` | En faire une **door** vers la dernière direction prise | édition | 📋 ⚠️ préfixe de `da`/`dt` |
| `m` | Marquer : mettre dans une récolte à trier plus tard | mesure | 📋 |
| `n` | Sauter | 📋 | ⚠️ **trois sens** — voir Collisions |

## Sur l'artiste

| Touche | Action | Nature | Statut |
|---|---|---|---|
| `E` | Ouvrir la fiche dans `$EDITOR`, recommit, vecteur recalculé | édition | 📋 |
| `-` | Cet artiste, moins souvent (poids, sans bannir) | mesure | 📋 ⚠️ c'est déjà `da` |
| `da` | Dislike artist | mesure | 💡 ⚠️ recouvre `-` |
| — | **Lier** l'artiste en cours à un autre (link typé, 0010) | édition | 💡 touche à définir |

## Session

| Touche | Action | Statut |
|---|---|---|
| `u` | **Annuler la dernière action** (revert d'une édition, effacement d'une mesure) | 📋 — 0013, ⚠️ le code en a fait « branche précédente » |
| `y` / `n` | Répondre à une promotion proposée | 📋 ⚠️ `n` surchargé |
| `:commande` | Toute touche a son équivalent `:` (`:top`, `:encore 3`, `:confort 2`) | 📋 — socle de 0013, rien de câblé |

## Touches multimédia (MPRIS / D-Bus)

| Touche | Action | Statut |
|---|---|---|
| ⏭ | Suivant (= `j`) | ✅ |
| ⏮ | Précédent (= `k`) | ✅ |
| ⏯ | Pause / lecture | ✅ — **le seul moyen de mettre en pause** |
| ⏹ | Arrêt | ✅ |

## Mode file d'attente

Un mode à part, avec sa propre table, encore à concevoir (retour n° 11) :
préparer les branches à l'avance, retirer un morceau, retirer une branche
(**seule, ou toute la profondeur qui en découle**), intercaler.

Point structurant repéré : `rounds` est aujourd'hui une **liste plate**,
alors que « retirer toute la profondeur » suppose un arbre manipulable.

## Collisions

Relevées le 05/09/2026 en rassemblant les trois tables. Aucune n'est
bloquante aujourd'hui — toutes le deviennent dès qu'on câble.

1. **`u` a deux sens.** 0013 (acceptée) : « annuler la dernière action ».
   Le code : « branche précédente ». À trancher avant les premières mesures
   et éditions, sinon il n'y a pas d'undo.
2. **`n` a trois sens.** « sauter » (esquisse TUI), « non » (réponse à une
   promotion), et le modificateur « maintenant » des retours 2 et 3. Le
   modificateur est le plus récent et le plus structurant ; les deux autres
   ont des remplaçants naturels (`x` saute déjà, `y`/`n` peut devenir
   `y`/`N` ou entrée/échap).
3. **`d` est un préfixe et une action.** `d` = door (édition), mais
   `da`/`dt` = dislike. Un automate à la neovim peut vivre avec (`d` seul
   après un délai ou une validation), mais c'est fragile pour un geste
   fréquent.
4. **`dt` recouvre `X`, `da` recouvre `-`.** Le dislike d'un morceau et le
   tiède sur un artiste **existent déjà** dans la table décidée. À trancher :
   `da`/`dt` sont-ils de nouveaux gestes, ou juste les noms `:` de `X` et
   `-` ?
5. **`.` recouvre `e`.** Deux touches pour encore, héritage de l'esquisse
   TUI d'avant `e`.
6. **`h`/`l` recouvrent `1 2 3`.** Choisir « le rassurant » ou
   « l'aventureux » plutôt que par numéro — utile, mais à confirmer, sinon
   c'est une deuxième façon de faire la même chose.
7. **`p` est une action et un préfixe.** `p` = prévoir, `p1` = choisir. Même
   remarque que pour `d`.
8. **`pr` entre en collision avec `p<n>`.** `r` seul évite le problème.

## Touches encore libres

`c`, `f`, `g`, `i`, `o`, `s`, `v`, `w`, et les majuscules hors `T`/`X`/`E`.
`g` et `f` sont des préfixes naturels à la vim si on a besoin de familles.

## Ce qu'il faut retenir

Sur **une trentaine de gestes** décrits dans le projet, **onze** sont
câblés. Le reste est décidé ou proposé, et **huit collisions** attendent
un arbitrage. C'est le moment de les trancher : elles ne coûtent rien
aujourd'hui, elles coûteront cher une fois les touches dans les doigts.

---

# La grammaire — décidée le 05/09/2026 ([0015](decisions/0015-grammaire-clavier-namespaces.md))

Demandé par Joel : **chaque touche vient d'un mot anglais**, à la vim
(`y` yank, `c` change, `d` delete). Trois révisions le 05/09 ont mené à une
grammaire à **quatre namespaces**, où le premier caractère dit *sur quoi*
on agit et le second *ce qu'on fait*.

Statut : **acceptée** (Joel, 05/09/2026), formalisée par la décision
[0015](decisions/0015-grammaire-clavier-namespaces.md).

**Câblé le 05/09/2026** : la saisie en mode brut (sans Entrée), le
namespace `f` en entier sauf `fw`, le namespace `e` avec ses trois
variantes, la navigation `h`/`l` et les flèches, l'espace, `/` et `q`.
**Pas encore câblé** : `t` et `a` (l'affinage — il attend `learned/`,
étape 2 de l'avancement), `fw`, `u`, `.`, `?`, `Q` et les commandes `:`.
Ces touches répondent « décidé, pas encore câblé » plutôt que de ne rien
faire.

Le modificateur se place **avant** le compte — `fn3`, `f!3`, `en2`, `e!2`
— et non après comme d'abord écrit. C'est imposé par l'analyse : la
grammaire doit être **sans préfixe** (aucune commande complète n'est le
début d'une plus longue) pour se déclencher sans délai ni retour arrière.
`f3n` rendrait `f3` à la fois complet et préfixe. Un test exhaustif sur
toutes les séquences de trois touches vérifie la propriété
(`src/keys.rs`, `grammar_is_prefix_free`).

## La règle, en une ligne

> **`f` la branche · `e` encore · `t` le morceau · `a` l'artiste** —
> le reste du clavier ne sert qu'à naviguer et à piloter la session.

Trois conséquences :

1. **Aucune collision possible.** Deux gestes ne peuvent se marcher dessus
   que dans le même namespace, où l'on maîtrise les lettres.
2. **Le compte suit le namespace** : `f3` la branche 3, `e3` trois encores.
   La règle vim (`3dd`) voudrait l'inverse, mais elle est incompatible avec
   le raccourci `1 2 3` — une frappe de `3` serait à la fois « branche 3 »
   et « début d'un compte », et l'analyseur ne peut pas trancher sans
   attendre la touche suivante, ce qui ralentirait le geste le plus
   fréquent. Avantage collatéral : `f` et `e` deviennent symétriques.
3. **Le clavier nu est presque vide**, donc extensible sans rien casser.

## `f` — la branche

| Touche | Mot | Action |
|---|---|---|
| `f<n>` | **fork** | Prendre la branche n, **après la branche en cours** |
| `fn<n>` | fork **now** | …après le morceau en cours, le reste conservé |
| `f!<n>` | fork now, **force** | …après le morceau en cours, le reste retiré |
| `fp` | fork **peek** | Prévoir les branches sans attendre le dernier morceau |
| `fr` | fork **reroll** | Reproposer trois autres branches |
| `fw` | fork **wander** | Partir loin, hors de l'univers courant (retour n° 6) |
| `fu` | fork **undo** | Revenir à la branche précédente |
| `1` `2` `3` | | Raccourci de `f1` `f2` `f3` |
| entrée | | Auto : tire au sort parmi les branches affichées |

Le `!` est le *force* de vim (`:w!`) : « et tant pis pour ce qui suivait ».
Le `n` est *now*. Ça se lit à voix haute : « fork now 3 », « fork force 3 ».

Après `f`, un **chiffre** désigne une branche, une **lettre** une opération.

## `e` — encore

| Touche | Mot | Action |
|---|---|---|
| `e<n>` | **encore** | n morceaux de plus de l'artiste, **en fin de branche** |
| `en<n>` | encore **now** | …après le morceau en cours, le reste conservé |
| `e!<n>` | encore now, **force** | …après le morceau en cours, le reste retiré |

`e` seul n'est pas une commande : le compte est obligatoire, pour la même
raison qu'il suit le namespace (règle 2).

`f` et `e` sont les deux namespaces de **lecture** : ils décident de ce qui
va sonner. `t` et `a` sont ceux de l'**affinage** : ils décident de ce que
le moteur retient.

## `t` — le morceau en cours

| Touche | Mot | Action | Nature |
|---|---|---|---|
| `tl` | track **like** | Aimer (reflété en titre aimé Spotify) | mesure |
| `ts` | track **skip** | « Pas celui-là, pas maintenant » | mesure |
| `tb` | track **ban** | « Plus jamais celui-là » | mesure |
| `tm` | track **mark** | Mettre dans une récolte à trier plus tard | mesure |
| `tt` | track **top** | Promouvoir en top | édition |
| `tT` | track **untop** | Retirer des tops | édition |
| `td` | track **door** | En faire une door vers la dernière direction prise | édition |

## `a` — l'artiste en cours

| Touche | Mot | Action | Nature |
|---|---|---|---|
| `al` | artist **like** | Cet artiste, plus souvent | mesure |
| `as` | artist **skip** | Cet artiste, moins souvent | mesure |
| `ab` | artist **ban** | Plus jamais cet artiste | mesure |
| `ae` | artist **edit** | Ouvrir la fiche dans `$EDITOR`, recommit, vecteur recalculé | édition |
| `aL` | artist **link** | Lier à un autre artiste (retour n° 8, format 0010) | édition |

Les trois verbes forment sur l'artiste une **échelle lisible** : `al` plus
souvent, `as` moins souvent, `ab` plus jamais. Les anciens `+`/`-`
disparaissent — ils étaient les deux seuls gestes sans mot derrière.

## Navigation et session

| Touche | Mot | Action |
|---|---|---|
| `h` / `l` | | Morceau **précédent / suivant** — vim, axe horizontal |
| ← / → | | Idem, pour les doigts qui ne sont pas sur la rangée d'accueil |
| espace | | **Pause / lecture** — comble le manque relevé |
| `u` | **undo** | Annuler la dernière action : mesure ou édition (0013) |
| `q` | **quit** | Quitter |
| `Q` | **queue** | Entrer en mode file d'attente (retour n° 11) |
| `.` | | Répéter la dernière action (son sens vim) |
| `?` | **why** | La phrase qui explique le morceau ou la branche |
| `/texte` | | Chercher |
| `:commande` | | L'équivalent de chaque geste, plus les réglages |

**`u` et `fu` ne sont pas la même chose** : `u` annule le dernier *geste*
(un top posé de travers, un ban), `fu` remonte d'un cran dans le *parcours*.

`j` et `k` ne servent plus : la navigation passe à l'horizontale, ce qui
correspond à la façon dont Joel lit le temps d'un parcours (05/09/2026).

## Commandes `:` sans touche

Les gestes rares n'encombrent pas le clavier : `:size <n>` (taille des
branches), `:comfort <0-5>` (zone de confort, 0001), `:sync` / `:push` /
`:pull` (retour n° 10), `:fork` (forker le catalogue, 0008).

## Ce que ça résout

Les huit collisions relevées le 05/09 tombent toutes :

| Collision | Résolution |
|---|---|
| `u` undo vs branche précédente | `u` annule un geste, `fu` remonte d'une branche |
| `n` à trois sens | `n` = **now** ; `y`/`n` ne vit que dans une invite modale |
| `d` action et préfixe | `d` disparaît du clavier nu : c'est `td` |
| `dt`/`da` recouvrent `X`/`-` | Absorbés par `tb` et `as`/`ab` |
| `.` recouvre `e` | `.` reprend son sens vim (répéter) |
| `h`/`l` recouvrent `1 2 3` | `h`/`l` deviennent la navigation, `1 2 3` restent les branches |
| `p` action et préfixe | `p` disparaît du clavier nu : c'est `fp` |
| `pr` vs `p<n>` | `fr`, dans le namespace |

## Le prix à payer

**Les gestes fréquents coûtent deux frappes** (`tl` pour aimer, `ts` pour
passer), là où vim garde une touche pour ce qu'on fait le plus. C'est le
vrai coût de la régularité, et il ne se jugera qu'à l'usage.

Deux garde-fous existent si ça pèse : `1` `2` `3` restent le raccourci de
`f1` `f2` `f3` — l'exception assumée, parce que choisir une branche est
**le** geste du produit ; et le clavier nu est assez vide pour qu'on y
promeuve plus tard un ou deux gestes qui se révéleraient constants.

## Le point faible

`aL` (link) est le seul geste dont la lettre ne suit pas la règle de
casse du reste : dans son namespace, `al` est *like* et `aL` est *link* —
deux verbes différents, pas un verbe et son inverse comme `tt`/`tT`. La
justification tient (minuscule = mesure, majuscule = édition), mais elle
est plus faible qu'ailleurs. Alternative si ça gratte : `ac` pour *connect*,
au prix de perdre le mot « link », qui est celui du format sur disque
([0010](decisions/0010-format-revise-links-sans-portes.md)).

## « Fork » a deux sens — tranché

**Arbitrage de Joel, 05/09/2026 : `f` = prendre une branche.**

0002 et 0008 emploient déjà « fork » pour le **catalogue** (« le fork est
la surcouche »). Le mot garde deux sens, mais ils ne se croisent jamais :
forker le catalogue est un geste rare, une fois par machine, qui reste la
commande `:fork` ; forker le parcours est le geste constant de l'écoute, et
c'est la touche `f`.

La nuance est consignée dans le vocabulaire de
[`docs/vision.md`](vision.md).

## Ce qui reste à trancher

1. **`ts` (skip track) vs `l` (suivant).** Deux gestes pour passer un
   morceau, avec une différence invisible : `l` avance sans rien noter,
   `ts` avance **et** le note dans `learned/`. Nuance juste sur le papier,
   peut-être insensible dans les doigts.
2. **`aL` ou `ac`** pour lier deux artistes — voir « le point faible ».

## Lettres libres après ce mapping

Le clavier nu ne garde que `h`, `l`, `e`, `f`, `t`, `a`, `u`, `q`, `Q`.
Restent donc libres : `b`, `c`, `d`, `g`, `i`, `j`, `k`, `m`, `n`, `o`,
`p`, `r`, `s`, `v`, `w`, `x`, `y`, `z`, et toutes les majuscules hors `Q`.
Le mode file d'attente peut s'installer sans rien déplacer.
