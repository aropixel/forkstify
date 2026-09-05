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

# Mapping mnémonique proposé (05/09/2026)

Demandé par Joel : **chaque touche doit venir d'un mot anglais**, à la vim
(`y` yank, `c` change, `d` delete), en partant de son idée — **`f` pour
fork**, puisque le produit s'appelle forkstify et que tout y est branche.

Statut : **proposition**, rien n'est tranché. Elle remplacerait les tables
ci-dessus et résoudrait les huit collisions.

## Les trois règles

1. **Une touche = un verbe anglais.** Sa majuscule est le geste lourd ou
   l'inverse (`t` top / `T` untop, `l` like / `L` link).
2. **Verbe + cible.** Sans cible, le geste porte sur le **morceau en
   cours** ; le suffixe `a` (*artist*) l'élargit à l'artiste : `b` bannit
   le morceau, `ba` bannit l'artiste. C'est la généralisation du `da`/`dt`
   de Joel, avec le cas fréquent à une seule touche.
3. **Un geste fréquent a une touche ; un réglage a une commande `:`.**
   `b<n>` (taille) devient `:size 5`, `z` (confort) devient `:comfort 2`.
   C'est ce qui libère les lettres et garde la surface petite.

## Fork — les branches

| Touche | Mot | Action |
|---|---|---|
| `f<n>` | **fork** | Prendre la branche n, **après la branche en cours** |
| `f<n>n` | fork **now** | …après le morceau en cours, le reste conservé |
| `f<n>n!` | fork now, **force** | …après le morceau en cours, le reste retiré |
| `1` `2` `3` | | Raccourci de `f1` `f2` `f3` |
| entrée | | Auto : tire au sort parmi les branches affichées |
| `p` | **peek** | Prévoir les branches sans attendre |
| `r` | **reroll** | Reproposer trois autres branches |
| `w` | **wander** | Partir loin, hors de l'univers courant (retour n° 6) |
| `u` | **undo** | Annuler la dernière action — y compris le dernier fork |

Le `!` est le *force* de vim (`:w!`) : « et tant pis pour ce qui suivait ».
Le `n` est *now*. Les deux se lisent à voix haute : « fork 1, now, force ».

## Encore — même grammaire

| Touche | Mot | Action |
|---|---|---|
| `<n>e` | **encore** | n morceaux de plus de l'artiste, **en fin de branche** |
| `<n>en` | encore **now** | …après le morceau en cours |
| `<n>en!` | encore now, **force** | …après le morceau en cours, le reste retiré |

« Encore » est un mot anglais qui veut dire exactement ça — on le garde.

## Mesures — ce que le moteur apprend

| Touche | Mot | Action | Cible |
|---|---|---|---|
| `l` / `la` | **like** | Aimer (reflété en titre aimé Spotify) | morceau / artiste |
| `s` / `sa` | **skip** | Passer — « pas celui-là, pas maintenant » | morceau / artiste |
| `b` / `ba` | **ban** | « Plus jamais » (liste noire de `learned/`) | morceau / artiste |
| `m` | **mark** | Mettre dans une récolte à trier plus tard | morceau |
| `+` / `-` | | Cet artiste, plus / moins souvent | artiste |

`s` et `b` remplacent `x`/`X` ; `l` remplace `a` (aimer) ; `b`/`ba`
remplacent le `dt`/`da` demandé, sans doublonner avec l'existant.

## Éditions — ce qui produit un commit

| Touche | Mot | Action |
|---|---|---|
| `t` / `T` | **top** | Promouvoir / retirer des tops |
| `d` | **door** | En faire une door vers la dernière direction prise |
| `L` | **link** | Lier l'artiste en cours à un autre (retour n° 8, format 0010) |
| `E` | **edit** | Ouvrir la fiche dans `$EDITOR`, recommit, vecteur recalculé |

`link` est déjà le mot du format sur disque (0010) — la touche reprend le
vocabulaire du projet plutôt que d'en inventer un.

## Lecture et session

| Touche | Mot | Action |
|---|---|---|
| `j` / `k` | | Morceau suivant / précédent (convention vim, pas mnémonique) |
| espace | | **Pause / lecture** — comble le manque relevé |
| `q` | **quit** | Quitter |
| `Q` | **queue** | Entrer en mode file d'attente (retour n° 11) |
| `.` | | Répéter la dernière action (son vrai sens vim, libéré de « poncer ») |
| `?` | **why** | La phrase qui explique le morceau ou la branche |
| `/texte` | | Chercher |
| `:commande` | | L'équivalent de chaque touche, plus les réglages |

## Commandes `:` sans touche

Les gestes rares n'encombrent pas le clavier : `:size <n>` (taille des
branches), `:comfort <0-5>` (zone de confort, 0001), `:sync` / `:push` /
`:pull` (retour n° 10), `:fork` (forker le catalogue, 0008).

## Ce que ça résout

Les huit collisions tombent :

| Collision | Résolution |
|---|---|
| `u` undo vs branche précédente | `u` = **undo**, et annuler un fork *est* revenir en arrière — même geste |
| `n` à trois sens | `n` = **now**, un seul sens ; `y`/`n` ne vit que dans une invite modale, comme les chiffres après `/` |
| `d` action et préfixe | `d` = **door** seul ; le dislike devient `b`/`ba` |
| `dt`/`da` recouvrent `X`/`-` | Fusionnés dans `b`/`ba`, plus de doublon |
| `.` recouvre `e` | `.` reprend son sens vim (répéter) |
| `h`/`l` recouvrent `1 2 3` | Abandonnés ; `l` devient **like** |
| `p` action et préfixe | `p` = **peek** seul ; le préfixe des branches est `f` |
| `pr` vs `p<n>` | `r` = **reroll**, sans préfixe |

## La tension à trancher

**« Fork » a déjà un sens dans le projet.** 0002 et 0008 l'emploient pour
le **catalogue** — « le fork est la surcouche », « catalogue forkable ».
Mettre `f` sur « prendre une branche » lui donne un second sens.

Trois sorties possibles :

- **(a)** garder `f` = prendre une branche (geste très fréquent, dans le
  flux) et laisser le fork du catalogue en `:fork` (geste rare, une fois
  par machine). Le mot a deux sens, mais jamais dans le même contexte.
- **(b)** `b` = **branch** pour les branches, et le bannissement passe
  ailleurs (`x` = *exclude* ?). Fidèle au vocabulaire des docs, qui disent
  « branche » partout, mais perd l'idée de Joel.
- **(c)** renommer le concept : les branches **sont** des forks, dans les
  docs comme dans le code. Cohérent avec le nom du produit, mais c'est une
  décision de vocabulaire (`docs/vision.md`), pas un choix de touche.

Ma recommandation : **(a)**, avec la nuance notée dans `vision.md`.

## Lettres libres après ce mapping

`c`, `g`, `h`, `i`, `o`, `v`, `x`, `y`, `z`, et les majuscules hors
`T`/`L`/`E`/`Q`/`B`. De quoi absorber le mode file d'attente et la suite
sans réouvrir la table.
