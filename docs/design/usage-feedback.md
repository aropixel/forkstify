# Retours d'usage et chantier clavier

Note vivante, ouverte le **05/09/2026** après les premières sessions
longues d'`ecouter` (Joel, plusieurs sessions le matin du 05/09). Elle
tient la liste des retours, l'inventaire de ce qui existe vraiment, et ce
qui reste à trancher avant d'ajouter quoi que ce soit.

Elle se traite **au fur et à mesure** : chaque entrée porte son statut, et
ce qui est fait descend dans `docs/avancement.md`.

**Point après `learned/` (05/09/2026) — 7 faits, 1 à moitié, 3 non
commencés, et toujours rien de vérifié en écoute réelle.** Les statuts
ci-dessous sont relus contre le code, pas contre le souvenir.

## L'inventaire d'abord (retour n° 4)

Joel : « je commence à me mélanger, je ne veux pas rajouter et que ça
devienne inutilisable ». Voici donc l'état réel, relevé dans
`src/listen.rs` (`on_input`, `on_control`) le 05/09/2026 — à distinguer de
la table *projetée* de `forme-de-l-application.md`, qui reste une
orientation largement non implémentée.

### Ce qui marche aujourd'hui dans `ecouter`

Onze gestes câblés, listés avec tout le reste dans
**[`docs/keybindings.md`](../keybindings.md)** — la référence unique
depuis le 05/09/2026.

### Ce qui manque et qu'on croit parfois avoir

- **Aucune touche pause/lecture au clavier.** ⏯ ne passe que par les
  touches multimédia (MPRIS). Dans le terminal, rien.
- **Aucune des touches d'affinage de 0013** : `t`/`T` (tops), `x`/`X`
  (sauter/écarter), `a` (aimer), `d` (door), `m` (marquer), `E` (éditer la
  fiche), `-` (moins souvent), `z` (confort), `?` (pourquoi), `y`/`n`. La
  table est écrite, rien n'est câblé. C'est exactement le retour n° 7.
- **Aucune commande `:`**, alors que 0013 en fait le socle (« chaque touche
  n'est que le raccourci d'une commande `:` »).
- **Aucune écriture dans `learned/`** : la boucle d'apprentissage (0014)
  n'est pas fermée, donc aucune mesure n'est encore enregistrable.

### Deux conflits à trancher avant d'ajouter

**1. `u` a deux sens.** La décision **0013** (acceptée) dit : « `u` annule
la dernière action, quelle qu'elle soit ». L'implémentation actuelle en a
fait « revenir à la branche précédente ». Tant qu'il n'y avait ni mesure ni
édition, l'ambiguïté ne coûtait rien ; dès que `da`/`dt` et les tops
arrivent (retours n° 8 et 9), il faut un vrai undo. **À trancher** : `u` =
undo (0013) et la navigation arrière passe sur autre chose (`h` ? déjà
évoqué dans la table projetée), ou 0013 est révisée par une nouvelle
décision.

**2. Les touches proposées recouvrent des touches déjà réservées.** Le
`d` de 0013 est **door** (une édition qui produit un commit) ; le `da`/`dt`
du retour n° 9 en ferait un préfixe *dislike*. Et surtout : `X` (« plus
jamais celui-là ») **est déjà** le dislike d'un morceau, `-` (« cet
artiste, moins souvent ») est déjà le tiède sur l'artiste. **À trancher** :
`da`/`dt` sont-ils de nouveaux gestes, ou les noms `:` des gestes `X` et
`-` déjà prévus ? Le risque, sinon, est d'avoir deux touches pour la même
chose — précisément ce que le retour n° 4 veut éviter.

## Les retours, un par un

### 1. Valider sans « Entrée », à la neovim

**Statut** : **fait** (05/09/2026, `src/keys.rs`) — non vérifié en écoute
réelle.

Aujourd'hui l'entrée est ligne par ligne (`std::io::stdin().lines()`,
choix assumé du 04/09 : « le temps réel appartient à l'étape TUI »).

**Point dur repéré** : ce retour et les retours n° 2, 3, 5 et 9 se
contredisent en apparence — on ne peut pas taper `2en!`, `p1n!`, `da` ou
`pr` avec une lecture « une touche = une action ». **Neovim résout
exactement ça** et c'est la référence citée : lecture en mode brut avec un
**tampon d'attente**, les chiffres sont des *counts*, les lettres des
opérateurs, `!` un modificateur, et la séquence s'exécute dès qu'elle est
non ambiguë. `/` et `:` basculent en mode ligne (avec Entrée), ce dont la
recherche a de toute façon besoin.

C'est ce qui a été fait : termios via `libc`, garde RAII qui rend le
terminal même sur panique, flèches ← → reconnues, `/` et `:` qui ouvrent
une ligne éditable. La grammaire est **sans préfixe**, donc tout se
déclenche sans délai ni `timeoutlen` — propriété vérifiée par un test
exhaustif sur toutes les séquences de trois touches.

### 2. Encore, en trois nuances

**Statut** : **fait** (05/09/2026) — non vérifié en écoute réelle.

Demandé : ajouter n morceaux en fin de branche · après le morceau en
cours · après le morceau en cours en retirant ce qui était prévu.

Câblé sous la forme `e<n>` / `en<n>` / `e!<n>` : le modificateur précède le
compte, contrainte de la grammaire sans préfixe (voir 0015). L'ancien
comportement était déjà la variante « now » ; les deux autres sont
nouvelles.

### 3. Choisir une branche : même grammaire

**Statut** : **fait** (05/09/2026), **bug compris** — non vérifié en
écoute réelle.

Joel : « quand on choisit une branche à l'avance, elle se joue après le
morceau en cours, pas après la branche en cours ».

Vérifié : `choose()` met la branche en attente pour la fin du **morceau**,
et `start_segment()` fait `self.queue = stops.into()` — le reste du segment
est **jeté**. Le comportement actuel est donc la variante la plus
destructrice des trois, et c'est le défaut.

Câblé sous la forme `f<n>` / `fn<n>` / `f!<n>` (le préfixe est `f`, pas
`p` : voir 0015). Le défaut est désormais « après la branche en cours » —
**le segment n'est plus jeté**, ce qui était le bug.

**Observation qui sert le retour n° 4** : les retours 2 et 3 décrivent
**la même grammaire** — un geste, puis les modificateurs `n` (« maintenant »)
et `n!` (« maintenant, et tant pis pour la suite »). Une seule règle à
apprendre pour deux commandes, et elle se généralisera aux suivantes. C'est
la piste à tenir pour que la surface reste petite.

**À trancher** : si `p1` existe, faut-il garder `1` tout court ? Deux
façons de faire la même chose, c'est ce qu'on veut éviter. Proposition :
`1` reste le geste rapide (= `p1`), `p` devient le préfixe qui accepte les
modificateurs.

### 4. Faire le point sur les raccourcis

**Statut** : **fait**, et allé plus loin que demandé. L'inventaire a
produit [`docs/keybindings.md`](../keybindings.md), la table unique, puis
la refonte complète de la grammaire (décision
[0015](../decisions/0015-grammaire-clavier-namespaces.md)) : quatre
namespaces, huit collisions résolues, chaque touche adossée à un mot
anglais.

### 5. Reproposer des branches

**Statut** : **fait** (05/09/2026) — `fr`, dans le namespace des
branches ; `pr` est abandonné, il entrait en collision avec `p<n>`.

Une touche qui retire trois nouvelles branches quand aucune ne convient.
Joel propose `pr` ou `r` (refresh/reload).

**Remarque** : c'est exactement ce que faisait `auto_advance()` par accident
avant la correction du 05/09 — le tirage existe déjà (`recompute()`), il
suffit de l'exposer.

**À trancher** : `pr` entre en collision avec le schéma `p<n>` du retour
n° 3 (`p1`, `p1n`). `r` seul est plus net et laisse `p` cohérent.

### 6. « Partir sur complètement autre chose »

**Statut** : **tranché et câblé le 11/09/2026** — lecture **(a)**, avec une
cible optionnelle : `fw` seul saute loin (une tête sous le plancher du
confort, hors du parcours et de ses voisins de graphe, la plus lointaine
pesant le plus, le curseur penchant comme pour toute tête), `fw <artiste>`
saute chez cet artiste du catalogue. La touche ouvre la ligne `:wander `
déjà remplie, entrée part. La branche va en fin de ce qui est décidé.

Deux lectures possibles, et elles ne mènent pas au même travail :

- **(a)** une branche/touche qui **sort de l'univers courant** — ignorer le
  plancher de la branche aventureuse (cosinus ≥ 0.72 + un tag commun) pour
  sauter loin volontairement ;
- **(b)** repartir d'une **nouvelle graine** en cours de session, sans
  quitter l'application (ce que `/texte` fait déjà à moitié).

Ma lecture penche pour **(a)**, vu la place de la note dans une liste de
gestes d'écoute — mais je ne tranche pas à ta place.

### 7. Câbler les raccourcis manquants (tops, édition de fiche…)

**Statut** : **à moitié fait.** Le blocage est levé, il s'est déplacé.

**Les mesures marchent** (05/09/2026, `src/learned.rs`) : `tl` aimer, `ts`
passer, `tb` bannir le morceau, `tm` récolter, `al`/`as` le poids de
l'artiste, `ab` bannir l'artiste. Elles écrivent dans
`learned/artists/<slug>.toml` à chaque geste, et le moteur les relit
(exclusion des bannis, poids sur les branches).

**Les cinq éditions restent à faire** — `tt`/`tT` (tops), `td` (door),
`ae` (ouvrir la fiche), `aL` (lier). Elles ne touchent pas l'appris mais la
**fiche**, et doivent produire un commit lisible (0013). C'est une couche
d'écriture du catalogue qui n'existe pas encore : c'est elle qui bloque
désormais, plus `learned/`.

Note : l'ordre proposé le matin — les éditions d'abord — a été inversé, et
c'était le bon choix. Les mesures partagent toutes le même stockage, donc
les câbler ensemble a coûté un module ; les éditions, elles, demandent une
mécanique de commit qu'aucune autre ne réutilise encore.

### 8. Lier l'artiste en cours à un autre

**Statut** : **non commencé**, et c'est désormais une **édition** parmi
cinq (voir retour n° 7) : la touche `aL` est décidée, il manque la couche
qui écrit et commite une fiche, plus le mode de désignation de la cible.

Une touche qui crée un **link typé** depuis l'artiste du morceau en cours
vers un autre artiste — donc une **édition** au sens de 0013 (commit dans
le catalogue, format des links de 0010).

**À spécifier** : comment on désigne la cible (recherche `/` réutilisée ?),
quel type de link par défaut, et si la proximité se saisit ou se déduit.

### 9. Dire qu'on n'aime pas (`da` / `dt`)

**Statut** : **fait** (05/09/2026) — non vérifié en écoute réelle.

`tb` (ban track) et `ab` (ban artist) absorbent à la fois le `dt`/`da`
demandé et les `X` et `-` déjà décidés : plus de doublon. Sur l'artiste,
les trois verbes forment une échelle — `al` plus souvent, `as` moins
souvent, `ab` plus jamais.

Les deux bans agissent **tout de suite** sur ce qui est prévu : `tb`
retire le morceau de la file, `ab` en retire tous les morceaux de
l'artiste, et le moteur cesse de les proposer.

### 10. Synchroniser par git (`gh`) entre machines

**Statut** : **non commencé.** Seules les commandes sont réservées
(`:sync`, `:push`, `:pull`). Axe différent des autres — c'est de
l'infrastructure, pas du clavier.

Idée de Joel : commits et push réguliers de l'usage et des fiches, `pull` au
démarrage pour retrouver son usage d'une machine à l'autre.

Cohérent avec 0008 (« le fork est la surcouche ») et 0002 (catalogue
versionné). Points à traiter : que faire des **conflits** sur `learned/`
(compteurs décrus, demi-vie 6 mois — une fusion additive a du sens, un
`git merge` textuel non), à quelle **fréquence** pousser sans transformer
l'écoute en machine à commits, et le comportement **hors ligne**.

`gh` est installé et authentifié sur cette machine (compte `kbyjoel`).

### 11. Mode file d'attente

**Statut** : **largement caduc** (constat de Joel, 06/09/2026) — et c'est
la meilleure nouvelle de la journée.

Le mode file d'attente devait exister parce que la file était *subie* : une
branche en remplaçait une autre, on ne voyait qu'un pas devant soi, et il
fallait une seconde vue pour préparer. Depuis que **choisir une branche
l'ajoute à la file** (06/09), la file principale fait déjà l'essentiel de ce
que le mode devait apporter :

| Ce que le mode promettait | Où c'en est |
|---|---|
| préparer les branches à l'avance | **fait** — on enchaîne les choix, la file s'allonge |
| voir toute la profondeur préparée | **fait** — tout est dans l'axe, branches nommées et filetées |
| se déplacer dans la file | **fait** — ↑↓ déplacent une sélection |
| retirer un morceau | **fait** — `tx`, sans bannir |
| revenir à l'écoute | **sans objet** — on ne l'a jamais quittée |
| retirer une branche entière (seule ou avec sa profondeur) | **reste** |
| déplacer un morceau dans la file | **reste** |
| intercaler un morceau choisi | **reste** — `/` en fait déjà une partie |
| annuler un retrait (`u`) | **reste** — c'est le `u` global de 0013 |

Il ne reste donc pas un **mode** à écrire, mais **quatre gestes** à ajouter
là où l'on est déjà. La touche `Q` n'a plus d'objet évident ; la question
« liste plate ou arbre » ne se pose plus non plus, puisque la chaîne est une
liste et qu'elle suffit.

**Ce qui reste vraiment de la maquette**, et qui n'a pas d'équivalent : le
constat de la planche 3b — **une chaîne préparée a déjà consommé la zone de
confort**, qui ne gouverne donc plus rien au-delà du premier maillon. C'est
vrai de notre file aujourd'hui, et ce n'est dit nulle part à l'écran.

Un mode à part entière : préparer les branches à l'avance, retirer des
morceaux, retirer une branche entière (**seulement elle, ou toute la
profondeur qui en découle**), intercaler un morceau.

C'est une **deuxième vue** sur l'état du parcours, avec sa propre table de
touches — à ne pas confondre avec la vue d'écoute. La distinction
« retirer cette branche » / « retirer toute la profondeur » suppose que
l'arbre du parcours soit manipulable, alors que `rounds` est aujourd'hui
une **liste plate**. C'est le point structurant à regarder en premier.

Lié à la « vraie prévisualisation + choix à l'avance » déjà notée comme
appartenant à l'interface (04/09/2026).

### 12. Explorer la discographie d'un artiste

**Statut** : **fait** le 07/09/2026 — `ad` ouvre la modale (forme 1a de la
maquette), note dédiée :
[`exploration-d-un-artiste.md`](exploration-d-un-artiste.md).

Joel, après une graine « Cat Power » : « je n'aime quasiment que des
morceaux de l'album *What Would the Community Think* ; j'aurais aimé une
commande (`:explore` ?) pour avoir la liste visuelle des morceaux classés
par albums, et pouvoir faire des `tt` sur ceux que j'aime et `tT` sur les
tops que je veux enlever. »

Le manque n'est pas l'édition — `tt`/`tT` écrivent et commitent depuis le
06/09 — mais le fait de ne pouvoir les exercer que sur **le morceau qui
sonne** : redresser un artiste demanderait de le poncer en entier. La
discographie est déjà en cache (`:warm`), les glyphes et l'appris aussi :
ce qui manque est un **écran**. Quatre points attendent l'arbitrage — la
cible (le morceau en cours ou la sélection, question qui vaut pour tout le
namespace `t`/`a`), le nom de la commande, la granularité des commits, et
ce que fait entrée.

### 13. Le petit cercle d'artistes au confort 3–4 (14/09/2026)

**Statut** : **câblé le 14/09/2026** (pistes A + la traîne par familiarité) — retour de Joel après quelques jours.

> Après quelques jours, j'ai alterné entre confort 3 et 4, et les mêmes
> artistes reviennent trop souvent : l'impression de tourner en rond, de
> n'avoir qu'un petit cercle, alors que mon catalogue et mes artistes likés
> sont gros. Ça dépend de la graine, mais quand même.

**Diagnostic (code lu).** Plusieurs forces se cumulent, et la principale
est une **boucle de renforcement** :

1. `familiarity01` **croît avec les écoutes** (compteur décru de `learned/`).
   Au confort 3–4, `Comfort::favours` **penche vers le familier** (pull 0,2
   à c3, 0,6 à c4) : plus on écoute un artiste, plus il est familier, plus
   il est tiré comme tête de branche — donc réécouté. Le cercle **se
   resserre tout seul**, et les likés, étant les plus familiers, dominent.
2. **Aucune rotation des artistes dans le temps.** La fraîcheur 0012 §2 ne
   pénalise que les **morceaux** récemment joués (`freshness`, par titre) et
   `visited` n'exclut que le parcours **courant**. Rien ne dit « tu as
   beaucoup entendu cet artiste ces jours-ci, lève le pied ». Une nouvelle
   graine rentre dans le même noyau.
3. **Le tirage des têtes est très pointu** : graphe en `poids² × favours`,
   vecteur en `(score−0,5)³ × favours`, et seulement les **6 premiers**
   voisins. Les plus proches d'un artiste familier gagnent presque toujours.
4. **La branche `stay`** puise dans le voisinage de l'univers du parcours —
   elle renforce le cluster courant.

Le gros catalogue n'aide pas tant que le moteur penche vers ce noyau à
forte familiarité.

**Pistes (réversibles, à trancher).**

- **(A) Une fraîcheur au niveau de l'artiste**, transposée de 0012 §2 : un
  artiste entendu récemment est atténué comme **tête de branche**, la
  pénalité décroît sur quelques jours. `learned/` porte déjà `plays` et
  `last` par artiste — la donnée existe. C'est le levier le plus direct, il
  ne change pas le sens du confort, et il s'explique en une phrase (règle du
  projet). **Recommandé.**
- **(B) Adoucir la pointe du tirage** : réduire les exposants (² et ³) et
  élargir la fenêtre (`take(6)` → 12–20), pour que plus de voisins aient une
  vraie chance. Simple, élargit le cercle sans le casser.
- **(C) Casser la boucle** : plafonner l'effet de la familiarité **acquise**
  sur le penchant des têtes (baser le lean sur la familiarité de la graine
  plutôt que sur les écoutes cumulées), pour que jouer un artiste ne le
  fasse pas revenir davantage.
- **(D) Un budget de nouveauté par session** : garantir à chaque session
  quelques têtes pas entendues récemment, même au confort haut.

**Câblé le 14/09/2026.** (A) `Learned::artist_freshness` : un artiste
entendu récemment recule comme tête de branche (demi-vie 4 jours, plancher
0,3), appliqué au graphe, à l'aventureuse et à `stay`. Et la traîne est
**modulée par la familiarité de l'artiste** — `share = tail_share() ×
familiarité` : un artiste nouveau (familiarité 0) est mené par ses tops,
un artiste connu ouvre ses fonds de tiroir. Baisser le confort élargit
donc les artistes sans noyer dans la traîne. (B), (C), (D) restent en
réserve si le cercle se resserre encore. Reste à éprouver dans la durée.

**Où vit la fraîcheur, et la synchro entre postes (14/09/2026).** Joel :
« ces notions de fraîcheur sont liées à un poste ; on pourrait envisager
un fichier embarqué dans le fork du catalogue ? » — c'est **déjà le cas**.
Les compteurs et dates (`plays`, `last`, par artiste et par titre) vivent
dans `learned/artists/*.toml`, **versionné dans le fork du catalogue et
synchronisé par 0017** (commit toutes les dix minutes, pull au démarrage,
push à la sortie, pilote de fusion `merge-learned` compteur par compteur
quand deux postes ont appris en même temps). La fraîcheur d'artiste comme
celle des morceaux voyage donc entre le poste du travail et le laptop, sans
fichier nouveau. Le seul réglage encore **local** est la zone de confort
(`~/.local/state/forkstify/comfort`) ; à décider si on la synchronise aussi.

**La suite du retour (14/09/2026).** Joel : « j'étais tenté de baisser le
confort pour avoir plus de nouvelles propositions, mais baisser le confort
amène beaucoup de longue traîne — donc des morceaux plus ou moins bons,
puisqu'on ne tape ni dans les tops ni dans les likes, mais dans le reste,
un peu au hasard. »

C'est le nœud : **le confort mêle deux axes que Joel veut régler
séparément.**

- **La largeur (nouveauté d'artistes)** — jusqu'où les branches vont vers
  des artistes nouveaux. Portée par `favours(familiarité)`.
- **La profondeur (longue traîne)** — jusqu'où on descend dans la
  discographie d'un artiste, sous les tops et les likes. Portée par
  `tail_share() = openness()`.

Un seul curseur tient les deux (0001 pour la familiarité, 0012 §4 pour la
traîne), donc **on ne peut pas avoir l'un sans l'autre** : baisser le
confort pour élargir les artistes fait entrer la traîne, et inversement.

**Piste de fond (réversible), très dans l'esprit du projet et sans second
curseur** : **lier la profondeur de la traîne à la familiarité de
l'artiste**, pas (ou pas seulement) au confort global.

- Un artiste **familier** (beaucoup écouté) → sa traîne pèse plus : on creuse
  ses fonds de tiroir, ce qu'on veut d'un artiste qu'on aime.
- Un artiste **nouveau** → mené par ses **tops** : on le présente sous son
  meilleur jour, pas par un morceau au hasard.

Alors baisser le confort **élargit les artistes**, et chaque nouvel artiste
arrive **par ses tops** — exactement ce que Joel cherchait. La traîne
profonde reste pour les artistes qu'on connaît. 0012 §4 (« pas de second
réglage ») est préservée : c'est la familiarité **par artiste**, déjà dans
`learned/`, qui module la traîne, pas un nouveau bouton. Se cumule avec (A)
la fraîcheur d'artiste : ensemble, largeur sans noyade dans la traîne.

## À trancher — récapitulatif

Les cinq points du matin ont tous été tranchés le 05/09 (voir
[0015](../decisions/0015-grammaire-clavier-namespaces.md) et
[`keybindings.md`](../keybindings.md)) : `u` annule un geste et `fu`
remonte d'une branche · `da`/`dt` sont absorbés par `tb`/`ab` · `p1` et
`1` sont devenus `f<n>` et le raccourci `1`…`9` · reproposer est `fr`.

Restent ouverts :

1. **Retour n° 6** : « partir sur complètement autre chose » — sortir de
   l'univers courant (a), ou repartir d'une nouvelle graine (b) ? La touche
   `fw` attend la réponse.
0. **Sauvegarder la playlist** — tout est là (le passé, la file, les noms de
   branches), mais où l'écrire n'est pas tranché : une playlist Spotify, un
   fichier du catalogue, un `.m3u` ? Cela mérite une décision.
2. **`ts` (skip track) vs `l` (suivant)** : deux gestes pour passer un
   morceau, la différence — noter ou non dans `learned/` — étant invisible
   dans les doigts.
3. **`aL` ou `ac`** pour lier deux artistes.
6. **Retirer un artiste des aimés** depuis l'accueil, écrit dans `learned/`
   — voir [ecran-d-accueil.md](ecran-d-accueil.md) § À trancher, point 5.
7. **Un setup fluide** : connexion, import de la bibliothèque, playlists à
   cocher — voir [premiere-installation.md](premiere-installation.md)
   § À trancher, point 5.
4. ~~**La cible de `t`/`a`** (retour n° 12)~~ — tranché le 09/09/2026
   ([0020](../decisions/0020-la-cible-d-un-geste.md)) : la ligne surlignée
   prime, sinon le morceau en cours, pour `t`, `a` et `e`. Le déclencheur :
   un `en3` qui servait l'artiste du bout de la chaîne des branches, pas
   celui qui sonnait.

## Ce qui n'a pas été vérifié

**Rien de ce qui a été câblé le 05/09 n'a tourné dans une session
d'écoute** — ni la saisie en mode brut, ni les trois variantes, ni les
sept mesures. Tout est vérifié à la compilation et par dix tests
unitaires, rien sous les doigts.

Cela pèse davantage depuis `learned/` : **les mesures écrivent dans le
catalogue**. Le premier `ts` créera `learned/artists/` pour de vrai, et
c'est du contenu versionné. Une session de test avant d'empiler la couche
des éditions serait prudente.

Trois paris ne se jugeront qu'à ce moment-là :

- le **coût des deux frappes** sur les gestes fréquents (`tl`, `ts`) ;
- le **sens de `h`/`l`**, la navigation étant passée à l'horizontale alors
  que la file s'affiche verticalement ;
- l'utilité réelle de **`ts` face à `l`**, dont la différence — noter ou
  non — reste invisible dans les doigts.
