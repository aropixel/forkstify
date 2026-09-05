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
