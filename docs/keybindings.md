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
| ✅ | Câblé, utilisable dans `listen` |
| 📋 | Décidé (0015), **pas encore câblé** — la touche répond « not wired yet » au lieu de ne rien faire |

## `f` — la branche

| Touche | Mot | Action | |
|---|---|---|---|
| `f<n>` | **fork** | Branche n, **ajoutée à la suite de ce qui est déjà décidé** — on enchaîne les choix et la soirée se construit | ✅ |
| `fn<n>` | fork **now** | …après le morceau en cours, le reste conservé | ✅ |
| `f!<n>` | fork now, **force** | …après le morceau en cours, le reste retiré | ✅ |
| `fp` | fork **peek** | Sans emploi depuis le 06/09 : les branches sont **affichées en permanence**, à droite. La touche le dit plutôt que de ne rien faire | ✅ |
| `fr` | fork **reroll** | Reproposer trois autres branches, **depuis la fin de la liste telle qu'elle est** : un titre inséré par `ti`, mis en file par le `e` de la discographie ou déplacé par `J`/`K` compte — comme contexte et comme déjà joué (Joel, 17/09/2026) | ✅ |
| `fu` | fork **undo** | Revenir à la branche précédente | ✅ |
| `fw` | fork **wander** | Partir loin, hors de l'univers courant : ouvre la ligne `:wander ` déjà remplie — **entrée** seule tire une tête parmi les artistes les plus loin du parcours (sous le plancher du confort, hors du parcours et de ses voisins de graphe), `fw <artiste>` part chez cet artiste du catalogue. La branche va en fin de ce qui est décidé, comme `f<n>` (Joel, 11/09/2026) | ✅ |
| `1`…`9` | | Raccourci de `f1`…`f9` | ✅ |
| entrée | | Auto : tire au sort parmi les branches affichées | ✅ |

**Les numéros continuent sur les creux** (09/09/2026,
[0016](decisions/0016-base-large-et-generation-a-la-volee.md)). Un lien de la
fiche vers un artiste **qui n'a pas encore de fiche** n'est plus jeté : il
s'affiche en gris au bas de la colonne, marqué « ○ fiche à générer », et se
prend au chiffre suivant. Le prendre **génère la fiche** — MusicBrainz puis
Deezer, quelques secondes — puis la branche part comme les autres. La
génération est une **édition** : elle commite, et la fiche porte
`generated = true`. Voir
[conception/generation-a-la-volee.md](conception/generation-a-la-volee.md).

`entrée` ne tire jamais un creux : elle choisit parmi ce qui peut sonner
tout de suite.

Le `!` est le *force* de vim (`:w!`) : « et tant pis pour ce qui suivait ».
Le `n` est *now*. Ça se lit à voix haute : « fork now 3 », « fork force 3 ».

Après `f`, un **chiffre** désigne une branche, une **lettre** une opération.

## `e` — encore

| Touche | Mot | Action | |
|---|---|---|---|
| `e<n>` | **encore** | n morceaux de plus de l'artiste **visé**, en fin de file | ✅ |
| `en<n>` | encore **now** | …derrière la ligne surlignée si elle est à venir, sinon après le morceau en cours ; le reste conservé | ✅ |
| `e!<n>` | encore now, **force** | …au même endroit, le reste retiré | ✅ |

L'artiste visé est celui de la ligne **surlignée**, sinon celui du morceau
qui sonne ([0020](decisions/0020-la-cible-d-un-geste.md)) — pas le bout
de la chaîne des branches, qui n'est plus ce qui sonne depuis que choisir
une branche s'ajoute à la file (Joel, 09/09/2026).

`e` seul n'est pas une commande : le compte est obligatoire.

`f` et `e` sont les namespaces de **lecture** — ils décident de ce qui va
sonner. `t` et `a` sont ceux de l'**affinage** — ils décident de ce que le
moteur retient.

## `t` — le morceau visé

**La cible d'un geste** ([0020](decisions/0020-la-cible-d-un-geste.md)) :
la ligne **surlignée** s'il y en a une (↑↓, `gg`, `G`), le morceau qui
sonne sinon. Vaut pour `t`, `a` et `e`. `ts` et `tb` ne font avancer la
musique que s'ils visent ce qui sonne ; sur une ligne à venir, `ts` la
sort de la file. Échap efface le surlignage.

| Touche | Mot | Action | Nature | |
|---|---|---|---|---|
| `tl` | track **like** | **Bascule aimé / non-aimé.** Aimé = plus souvent (prime sur les tops dans le tirage, ×10 au cocon, ×2 grand ouvert, et efface les « moins souvent ») ; sur un morceau déjà aimé, `tl` **retire l'aimé**, sans pénalité — au contraire de `ts` (Joel, 14/09/2026) | mesure | ✅ |
| `ts` | track **skip** | **Moins souvent** — il ne m'intéresse pas : note, retire l'aimé, et passe | mesure | ✅ |
| `tb` | track **ban** | « Plus jamais celui-là » — le retire aussi de la file | mesure | ✅ |
| `tm` | track **mark** | Mettre dans `learned/marks/inbox.toml` | mesure | ✅ |
| `tx` | track **remove** | Retirer de la file le morceau sélectionné — il reste proposable, ce n'est pas un ban | file | ✅ |
| `J` / `K` | | **Déplacer** la ligne surlignée d'un cran vers le bas / le haut, tout de suite — seul ce qui est à venir bouge, le nom de branche voyage avec son morceau, la touche contraire annule (Joel, 08/09/2026) | file | ✅ |
| `td` | track **door** | En faire une door vers la direction où l'on va (les tags de l'artiste suivant) | édition | ✅ |
| `ta` | track **about** | Album, featuring, année (quand la discographie les a), tags, familiarité, poids, et la première branche d'ici — remplace `?` (Joel, 14/09/2026) | ✅ |
| `ti` | track **insert** | **Insérer un titre** là où l'on est : la même modale, ancrée — avant la ligne surlignée si elle est à venir, sinon juste après ce qui sonne. L'ancre est écrite en haut ; le titre inséré porte « inséré (ti) » dans la liste. Un artiste choisi insère son meilleur morceau non joué. Un titre **hors catalogue** s'insère tout de suite et sa fiche se génère derrière ; à son arrivée, le morceau — même s'il joue déjà — est rattaché à la fiche, et `tl` a où écrire (Joel, 10/09/2026) | file | ✅ |

**Plus de `tt` / `tT` en écoute** (Joel, 08/09/2026,
[0018](decisions/0018-un-seul-geste-pour-le-gout.md)) : les tops ne sont que
les portes d'entrée d'un fork vierge, et se corrigent dans la discographie
(`ad`) ou à la main dans la fiche. À l'écoute, un seul geste simple dit
« je veux entendre ce morceau plus souvent », et son contraire.

## `a` — l'artiste en cours

**À l'accueil aussi** : tout le namespace `a` vise la ligne surlignée de la
collection (Joel, 10/09/2026) — `ad` ouvre la modale sur l'accueil, les
mesures écrivent dans l'appris, `ag` marche même sans fiche, `ae` et `aL`
demandent une fiche.

| Touche | Mot | Action | Nature | |
|---|---|---|---|---|
| `al` | artist **like** | Cet artiste, plus souvent (poids ×1.43, plafond 3) — et de retour parmi les aimés. **À l'accueil aussi**, sur la ligne surlignée (Joel, 09/09/2026) | mesure | ✅ |
| `as` | artist **skip** | Cet artiste, moins souvent (poids ×0.7, plancher 0.1) — et **hors des aimés** : un drapeau `unliked` dans `learned/`, qui prime sur les aimés Spotify et survit aux récoltes. À l'accueil aussi | mesure | ✅ |
| `ab` | artist **ban** | Plus jamais cet artiste — vide aussi la file. À l'accueil aussi | mesure | ✅ |
| `ae` | artist **edit** | Affiche le chemin de la fiche. L'ouvrir sur place attend une saisie interrogée : le lecteur de touches tient `stdin` en permanence et volerait ses frappes à `$EDITOR` | 📋 |
| `aL` | artist **link** | **Ouvre la recherche pour choisir l'artiste à lier** : entrée écrit un lien `similar` dans la fiche courante vers l'artiste choisi, catalogue ou hors catalogue (le lien vers une fiche absente est une proposition, [0016](decisions/0016-base-large-et-generation-a-la-volee.md)) ; un commit lisible ([0010](decisions/0010-format-revise-links-sans-portes.md)). Remplace l'ancien geste qui liait, sans le dire, à l'artiste d'où l'on venait (Joel, 14/09/2026) | édition | ✅ |
| `ad` | artist **discography** | Ouvrir la **modale de la discographie** : les albums pliés, ce que la fiche et l'appris savent de chaque morceau, `A` pour promouvoir un album. Elle a sa propre table, ci-dessous. **À l'accueil aussi**, sur la ligne surlignée de la collection, posée sur l'accueil ; un artiste **sans fiche** la reçoit d'abord ([0016](decisions/0016-base-large-et-generation-a-la-volee.md)), la discographie s'ouvre dès qu'elle est là (Joel, 10/09/2026) | édition | ✅ |
| `ag` | artist **google** | Chercher l'artiste visé (surligné, sinon en cours) dans le navigateur par défaut, via `xdg-open` (Joel, 08/09/2026) | session | ✅ |

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
**prochain lancement**, et le produit le dit — **sauf la génération d'une
fiche** (09/09/2026), qui entre dans le catalogue de la session sur-le-champ :
on la demande pour écouter maintenant.

## Navigation et session

| Touche | Mot | Action | |
|---|---|---|---|
| `h` / `l` | | Morceau **précédent / suivant** — vim, axe horizontal | ✅ |
| ← / `h` sur un morceau en cours | | **Recommence** le morceau ; une seconde fois — ou dans ses trois premières secondes — revient au précédent (Joel, 08/09/2026) | ✅ |
| ← / → | | Idem, pour les doigts hors de la rangée d'accueil | ✅ |
| `p` | **pause** | Pause / lecture | ✅ |
| ↑ / ↓ | | Déplacer la **sélection** dans l'axe — elle surligne, elle ne joue pas | ✅ |
| entrée | | Jouer la sélection ; sans sélection, tirer une branche | ✅ |
| `c<n>` | **comfort** | La zone de confort d'un coup, 5 cocon → 0 exploration — à l'accueil aussi (Joel, 08/09/2026) | ✅ |
| `cc` | **comfort** | Régler la zone de confort aux flèches : ↑↓ bougent, entrée valide, échap annule — à l'accueil aussi depuis le 11/09/2026. La jauge est **toujours en haut à droite sur les deux écrans, avec l'apparence d'édition** ; `cc` ajoute « ↑↓ » (Joel, 14/09/2026) | ✅ |
| échap | | Annuler la sélection, fermer un bloc — l'aide comprise | ✅ |
| espace | | **Le leader** : ouvre l'aide à la saisie — tout, ou le namespace en cours de frappe ; la séquence continue dedans, une touche fait l'action. Espace au niveau d'entrée la referme. **À l'accueil aussi**, avec sa propre table (Joel, 10/09/2026) | ✅ |
| ⌫ | | Effacer la dernière touche de la séquence — dans l'aide, remonter d'un niveau | ✅ |
| `/texte` | | **Filtrer** une liste — la collection à l'accueil, la discographie ; échap efface (Joel, 08/09/2026) | ✅ |
| `:search` | | **La modale de recherche** (maquette `Recherche.dc.html`, 08/09/2026) : une ligne de saisie `⟩`, les résultats se recalculent à chaque caractère — le catalogue d'abord (artistes et titres connus des fiches et de l'appris), Spotify derrière, jamais mêlés. ↑↓ choisissent, **entrée** branche sur un artiste ou joue un titre, **tab** masque Spotify, **échap** ferme. `:search <texte>` l'ouvre déjà remplie, et la frappe continue le mot. **Elle s'ouvre aussi de l'accueil** (Joel, 09/09/2026), où entrée démarre un parcours — sur l'artiste, ou sur le morceau puis les branches de son artiste ; un résultat **hors catalogue** génère la fiche de son artiste avant de partir (09/09/2026, [0016](decisions/0016-base-large-et-generation-a-la-volee.md)). Chaque ligne Spotify dit **album · année · durée**, pour distinguer les versions d'un même titre (09/09/2026) | ✅ |
| `q` | **quit** | En écoute : **revenir à l'accueil**, l'écoute continue en dessous avec son pied de lecture. À l'accueil : quitter (affiche le parcours) — Joel, 08/09/2026 | ✅ |
| `r` | **resume** | À l'accueil : **retour à l'écran d'écoute** si une session joue ; sinon **reprendre tout le dernier parcours** — historique, morceau en cours et morceaux à venir (Joel, 14/09/2026). Échap sans curseur fait de même | ✅ |
| `p` | **pause** | À l'accueil aussi : la session joue en dessous | ✅ |
| `s` | **sort** | Changer l'ordre de la collection : familiarité → a-z → dernière écoute — **accueil seulement** | ✅ |
| `v` | **vue** | La collection montre **les aimés** par défaut — artiste ou titre aimé ici, titre, album ou suivi sur Spotify — ou **tous** les artistes du catalogue (Joel, 09/09/2026). Accueil seulement ; le même mot que dans la discographie | ✅ |
| `gg` / `G` | | Les deux bouts d'une liste, comme dans vim — la collection à l'accueil, l'axe en écoute. `g` seul attend son second | ✅ |
| `b` | **browse** | Parcourir à sec — **écrans non connectés seulement** | 📋 |
| `u` | **undo** | Annuler la dernière action : mesure ou édition (0013) | 📋 |
| `.` | | Répéter la dernière action (son sens vim) | 📋 |

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
| `:comfort <n>` | Zone de confort, **5 = cocon → 0 = exploration** ([0001](decisions/0001-confort-familiarite.md)) ; sans argument, l'affiche. **Retenue d'un lancement à l'autre** (Joel, 14/09/2026) | ✅ |
| `:warm` | Récolter la discographie de l'artiste **sous l'aiguille** — la ligne surlignée, sinon ce qui joue (0020) — la longue traîne. Prenait à tort le dernier artiste de la chaîne (Joel, 14/09/2026) | ✅ |
| `:wander [artiste]` | Partir loin — raccourci `fw` ; avec un nom, chez cet artiste (11/09/2026) | ✅ |
| `:sync` / `:push` | | Commiter et pousser l'appris maintenant — sinon toutes les dix minutes, à la sortie, et pull au démarrage ([0017](decisions/0017-synchronisation-de-l-appris.md)) | ✅ |
| `:generate <nom> [mbid]` | **Faire entrer un artiste absent** du catalogue, puis partir de chez lui — ou, s'il était proposé **en creux**, prendre sa branche en fin de file. Un MBID en dernier mot remplace la recherche par le nom quand MusicBrainz ne trouve pas ; s'il ne répond pas du tout, la fiche naît minimale (nom, id, tops Deezer), marquée à relire ([0016](decisions/0016-base-large-et-generation-a-la-volee.md)) : fiche composée depuis MusicBrainz et Deezer, **son vecteur calculé** ([0019](decisions/0019-vectorisation-par-l-application.md)), un seul commit, et le catalogue de la session l'a tout de suite. Marche à l'accueil comme en écoute. **Avec un mbid, par-dessus une lecture en cours, ne démarre plus tout seul** (Joel, 11/09/2026) : le toast de succès dure et propose — **⏎** part de l'artiste et remplace la liste, toute autre touche la garde. La modale de recherche fait la même chose sur un résultat hors catalogue, par simple `entrée` — et **`entrée` sur une ligne sans fiche de la collection** aussi (Joel, 10/09/2026, sur Kanye West) | ✅ |
| `:mine` | Ce que ce catalogue a de plus que l'amont — la surcouche personnelle, calculée par `git diff` plutôt que stockée ([0008](decisions/0008-le-fork-est-la-surcouche.md)) | ✅ |
| `:discography` | La discographie de l'artiste, en modale — raccourci `ad` (Joel, 07/09/2026) | ✅ |
| `:fork` | Forker le catalogue ([0008](decisions/0008-le-fork-est-la-surcouche.md)) | 📋 |

Et en sous-commande, parce qu'elles n'ont pas leur place au milieu d'une
écoute : `forkstify import <url>` reprend les fiches d'un autre catalogue —
celles qu'on n'a pas, jamais celles qu'on a — puis régénère les vecteurs.

## La modale de la discographie (`ad`)

**Un mode à part, avec sa propre table** — la première, et le patron des
suivantes (`keys::parse_modal`). Elle est **sans préfixe** comme la
grammaire de l'écoute, et elle emprunte à vim ce que celle-ci laissait
libre : l'axe y est vertical, donc `j`/`k` descendent et montent, et
`h`/`l` plient et déplient. Câblée le 07/09/2026, d'après la maquette
**1a** de `Discographie.dc.html`.

Ce qui joue continue de jouer : la modale se pose sur l'écran d'écoute,
elle ne le remplace pas. `ad` vise l'artiste de la ligne **surlignée** s'il
y en a une, celui du morceau en cours sinon — et l'en-tête nomme l'artiste
ouvert, pour que le doute se lève à l'écran.

| Touche | Action | |
|---|---|---|
| `j` / `k`, ↑ / ↓ | Descendre, monter — l'album sous le curseur s'ouvre seul | ✅ |
| `h` / `l` | Tout plier (douze lignes), rouvrir l'album du curseur | ✅ |
| `gg` / `G` | Les deux bouts de la liste | ✅ |
| `A` | **Album** : promouvoir les quatre titres les plus écoutés de l'album, hors tops — mis **en attente**. Plus de `tt` / `tT` ici non plus (Joel, 08/09/2026) : un titre seul s'aime, il ne se promeut pas | ✅ |
| `tb` | Bannir la ligne — **mesure**, écrite tout de suite (aimer/retirer l'aimé : `tl`, bascule) | ✅ |
| `e` | Mettre le morceau **à la file**, sans fermer | ✅ |
| `tl` | Aimer / **retirer l'aimé** (bascule) la ligne — mesure écrite tout de suite (Joel, 14/09/2026) | ✅ |
| `s` | L'ordre : chronologique ⇄ mes écoutes d'abord | ✅ |
| `v` | La **vue** : tout → ♪ tops → ♥ aimés → ⊘ bannis | ✅ |
| `/texte` | Filtrer sur un titre ou un album | ✅ |
| `u` | Défaire la dernière édition en attente — gratuit, rien n'est écrit | ✅ |
| ⏎ | **Écrire la fournée** — une écriture, **un seul commit** — puis, **sur un morceau, partir de lui** ; **sur une ligne d'album, écouter l'album entier** : ses morceaux ouvrent une nouvelle graine dans l'ordre, puis les branches partent de l'artiste (Joel, 14/09/2026) | ✅ |
| échap | Fermer. Avec des éditions en attente, le premier échap prévient | ✅ |

**Pourquoi une fournée et un seul commit** : on corrige cinq tops d'une
même pensée, et cinq commits ne se relisent pas. Ce n'est pas contradictoire
avec [0017](decisions/0017-synchronisation-de-l-appris.md), qui commite
**l'appris** toutes les dix minutes et à la sortie : l'appris est mesuré et
silencieux, une édition est écrite et lisible. Les `tl`/`tb` de la modale
suivent la règle de 0017, ses `tt`/`tT` celle de 0013.

L'écran montre aussi les **tops que la discographie ne rend pas** — coquille,
live, titre de compilation — en fin de liste : `tT` y fonctionne, et c'est
là qu'une fiche générée se relit.

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

Depuis le 05/09/2026, `listen` (ex-`ecouter`) lit le clavier en **mode brut** : chaque
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
2. **`aL` ou `ac`** pour lier deux artistes. La cible n'est plus implicite
   depuis le 14/09/2026 : `aL` ouvre la recherche et on choisit l'artiste,
   ce qui a levé la confusion (Joel : « je ne comprends pas le geste »).
   Reste la casse : `aL` est le seul geste dont la majuscule ne dit pas la
   même chose qu'ailleurs — `tt`/`tT` sont un verbe et son inverse,
   `al`/`aL` deux verbes différents. La justification tient (minuscule =
   mesure, majuscule = édition) mais elle est plus faible. `ac` (*connect*)
   l'éviterait, au prix du mot « link », celui du format sur disque.

3. **Deux propositions du 19/09/2026**, dans
   [conception/sortie.md](conception/sortie.md) : `fg<n>` — générer la
   fiche d'un creux sans prendre la branche ; et `c` partagé entre le
   confort (chiffres, `cc`) et le catalogue (`cd` diff, `cp` propose,
   `cu` update), sur le modèle de `f`. Rien n'entre dans la table avant
   l'arbitrage.

## Lettres libres

Le clavier nu ne garde que `h`, `l`, `p`, `e`, `f`, `t`, `a`, `c`, `u`, `q`
— hors modale, où `j`, `k`, `s`, `v`, `e` et `A` servent (table ci-dessus).
`Q` est parti le 08/09/2026 avec le mode file d'attente. Restent libres :
`b`, `d`, `g`, `i`, `j`, `k`, `m`, `n`, `o`, `r`, `s`, `v`, `w`, `x`, `y`,
`z`, et toutes les majuscules. Dans les
namespaces, `d` a été pris chez `a` le 07/09/2026 (`ad`, discography). Le mode
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
