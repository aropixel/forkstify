# Catalogue

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Décidé

- **Fichiers texte versionnés, un par artiste**, partagés et forkables
  ([0002](../decisions/0002-catalogue-partage-forkable.md)).
- **Pas de surcouche à part : le fork est la surcouche.** Le catalogue
  actif est un clone git, les modifications personnelles sont des commits
  dedans ([0008](../decisions/0008-le-fork-est-la-surcouche.md)).
- **TOML**, `format = 1` en tête de fiche
  ([0007](../decisions/0007-fiches-en-toml.md)).
- Chaque fiche porte des **tops** ([0003](../decisions/0003-titres-tops-et-portes.md)) ;
  et, occasionnellement, des **`doors`** — morceaux ciblés comme sortie vers
  une direction (tags), **critère additionnel jamais principal**
  ([0011](../decisions/0011-doors-critere-additionnel.md)).
- **Champs en anglais, liens typés en une ligne, proximités par défaut dans
  `catalog.toml`** ([0010](../decisions/0010-format-revise-links-sans-portes.md)).
- **Le format est une interface publique** : stable, documenté, éditable à la
  main.
- **Deux dépôts** : l'application d'un côté, le catalogue de l'autre.
  **Importer = cloner** un catalogue et le déclarer actif ; un seul actif à
  la fois, on bascule quand on veut
  ([0004](../decisions/0004-deux-depots-catalogue-ciblable.md)).
- **Chaque fiche porte la version de son format** (`format: 1`), pas de son
  contenu ([0005](../decisions/0005-version-dans-les-fiches.md)).

## Orientations

### Un dépôt git

Le catalogue est un dépôt git : portable (cloner suffit), diffable,
versionné, forkable au sens propre. S'approprier une fiche = un commit.
Reverser une amélioration = une PR.

### Forme d'une fiche

Le format 1, tel qu'appliqué au premier lot :

```toml
# cards/the-cure.toml
format = 1
generated = true          # disparaît à la relecture humaine
name = "The Cure"
mbid = "69ee3720-a7cb-4402-b48d-a02c366f2bcf"
spotify = "7bu3H8JO7d0UbMoVzbo70s"
begin = "1977"
origin = "Crawley"

tags = ["post-punk", "new-wave", "gothique", "uk", "80s"]

tops = [
  "Boys Don't Cry",
  "A Forest",
]

links = [
  { to = "siouxsie-and-the-banshees", type = "member", note = "Robert Smith y a joué de la guitare en 1983" },
  { to = "depeche-mode", type = "scene", note = "new wave, versant synthétique", proximity = 2 },
]

# optionnel : bonus de choix de morceau quand on part dans cette direction
doors = [
  { track = "A Forest", to = ["post-punk", "atmospherique"], note = "la porte vers le sombre" },
]

description = """
Post-punk puis pop sombre, Crawley, depuis 1977. ...
"""
```

Et à la racine du catalogue, `catalog.toml` porte l'identité et les
réglages — dont la grille type → proximité, que chacun ajuste dans son fork.

Sémantique du format, précisée à la relecture du premier lot :

- **Fiche minimale : `format`, `name`, `mbid`. Tout le reste est
  optionnel**, avec dégradation douce : sans `spotify`, résolution via
  MusicBrainz ; sans `tags` ni `links`, l'artiste flotte dans l'espace ;
  sans `tops`, le moteur pioche dans l'usage. La prose (description, notes)
  complète l'expérience, jamais requise.
- **`begin` / `end`** : chaînes à précision libre (`"1977"`,
  `"1960-03-27"`), comme MusicBrainz. La tranche d'activité, c'est le
  couple ; pas de `end` = toujours actif. Le moteur ne lit que l'année.
- **`origin` est informatif** (couche humaine) : la ville ne croise rien.
  Les croisements géographiques passent par les tags (`fr`, `uk`,
  `belgique`…), au bon grain.
- **L'ordre des `links` n'a aucun sens.** La priorité est `proximity`
  (défaut par type dans `catalog.toml`, correctif local) — pas de
  sémantique invisible, pas de fragilité au merge.
- **Résolution de `proximity`, en cascade** : la valeur sur le lien s'il y
  en a une ; sinon la grille `[proximity]` du `catalog.toml` du catalogue
  actif ; sinon les défauts embarqués dans l'application (identiques à la
  grille du catalogue de référence). Le cas normal est de ne rien écrire :
  le type suffit. Changer une valeur de la grille re-règle d'un coup tous
  les liens de ce type sans correctif local.

S'approprier une fiche = l'éditer et commiter : mes tops deviennent
`["A Forest", "10:15 Saturday Night"]`, et `generated` saute.

### On écrit pour les humains, la machine lit la structure

La description et les notes ne sont **pas** écrites « pour l'embedding » —
personne ne sait faire ça, et il ne faut pas le demander :

- **Le texte vectorisé est composé par l'application** à partir des champs
  structurés : tags, origine, dates, types de liens et voisins. La
  description, si elle est bonne, ajoute de la nuance ; vide ou plate, le
  plancher est garanti par la structure. Éditer des tags est à la portée de
  tous.
- **La prose garde son rôle humain** : la description dit qui est l'artiste,
  la note dit pourquoi le lien existe — c'est elle qui s'affiche quand une
  branche s'explique.
- **Le retour remplace l'effort** : `forkstify check` affichera, pour une
  fiche, ses voisins dans l'espace (« The Cure est proche de : Siouxsie,
  Joy Division… ça te va ? »). On ne juge pas son texte, on juge ses
  effets, et on ajuste un tag ou une proximité.

### Ce qu'on fournit : les fiches et leurs vecteurs

- **Les fiches TOML sont la source.** Ce qu'un humain écrit, lit, corrige,
  forke : description, tags, tops, portes, connexions, identifiant Spotify.
- **Les vecteurs sont un index dérivé des fiches, livré avec la base.**
  Dérivé, parce qu'on peut toujours le refaire depuis les fiches. Livré,
  parce que calculer un embedding demande un modèle (~100 Mo) et surtout
  parce que tout le monde doit avoir *les mêmes* vecteurs pour que « proche »
  veuille dire la même chose partout. Le dépôt contient donc les vecteurs et,
  dans ses métadonnées, le nom et la version du modèle qui les a produits.
  L'application recalcule localement le vecteur d'une fiche modifiée ;
  l'amont régénère tout à chaque changement de modèle. Prototypé le
  02/09/2026 (`tools/vectoriser.py` et `voisins.py`) : modèle
  `paraphrase-multilingual-MiniLM-L12-v2` (384 dimensions, mean pooling,
  supporté par fastembed en Python comme en Rust), index dans
  `vectors/vectors.jsonl` + `meta.toml`. Le texte composé cite les
  voisins des liens **sortants et entrants** (la relation vaut dans les
  deux sens, seul `influence` se retourne en « a influencé »).
- **Hors du dépôt** : caches de l'API Spotify (résolution titre → identifiant,
  pochettes), jetons.

### Ce que le catalogue apprend

Le catalogue **s'automodifie avec l'usage**. On ne pré-remplit pas Spotify
entier : le catalogue couvre l'univers de l'utilisateur et grandit avec ses
écoutes. Signaux et effets :

| Signal | Ce que ça dit | Ce que ça modifie |
|---|---|---|
| Choix d'une branche à un embranchement | cette direction me parle | poids de la connexion empruntée |
| Morceau sauté | pas celui-là, pas maintenant | poids du top, ou de l'artiste dans ce contexte |
| Morceau écouté en entier, souvent | il fait partie de mes tops | ordre des tops, proposition d'en faire un top |
| Branche née des vecteurs (sans connexion) qui plaît | le lien mérite d'exister | **écriture d'une connexion** dans la fiche |
| Arrivée chez un artiste sans fiche | il fait partie de mon univers | **génération d'une fiche** (Spotify, Last.fm, LLM), marquée générée jusqu'à relecture |

Les deux dernières lignes sont le mécanisme central : **l'espace implicite
alimente le graphe explicite**. Ce que les vecteurs devinent et que l'écoute
confirme devient une connexion lisible, dans un fichier texte, relisible et
reversable à l'amont.

### Base, mien, appris — et ce qui est partageable

Si le fork s'automodifie avec l'usage et qu'on le reverse à l'amont, on
pollue la base avec ses propres écoutes. Il faut séparer la **connaissance**
(partageable : une connexion, une description, un top consensuel) de
l'**usage** (personnel : poids, compteurs, dates). Trois états, tous dans le
fork, tous dans git :

1. **La base** — ce qui vient de l'amont.
2. **Le mien** — ce que j'ai écrit ou validé explicitement dans les fiches.
   C'est ce que je peux proposer à l'amont.
3. **L'appris** — ce que l'usage a produit, dans un dossier à part
   (`learned/`), versionné pour être portable, que l'application sait ne
   jamais inclure dans une PR.

### Tout « le mien » n'est pas également partageable (06/09/2026)

Question de Joel : « si je modifie les links d'artistes, cela ferait partie
de la PR ? Ce serait problématique ? ». Oui pour la première, **pas
forcément** pour la seconde — et la réponse tient dans la liste fermée des
types de [0010](../decisions/0010-format-revise-links-sans-portes.md), qui
se coupe en deux d'elle-même :

| Type | Ce que c'est | En amont |
|---|---|---|
| `member` `collab` `family` | des **faits** — qui a joué où, avec qui | précieux pour tout le monde |
| `scene` `influence` | une lecture критique, discutable mais argumentable | défendable, avec sa note |
| `similar` | **un rapprochement de goût** | c'est celui qui tire la base vers une oreille |

`aL` écrit `similar` : c'est honnête — on rapproche deux artistes parce
qu'ils vont bien ensemble *pour soi* — mais c'est donc le type le moins
partageable qui est le plus facile à produire.

**Ce qui protège déjà** : une PR est une **proposition**, relue en amont.
Rien n'est reversé automatiquement, et l'auteur choisit les commits qu'il
propose. Le modèle git suffit à empêcher la dérive ; ce n'est pas une
faille.

**Ce qui manque vraiment**, c'est de quoi *trier*. Rien ne disait pourquoi
une ligne existait, si bien que ni l'auteur ni le relecteur ne pouvaient
distinguer un fait vérifié d'un rapprochement d'un soir. Depuis le
06/09/2026, une ligne écrite pendant une écoute porte sa **provenance et sa
date** dans sa note — `note = "rapproché à l'écoute, 2026-09-06"` — ce que
0010 demandait déjà en substance : « le type nomme et explique la branche ».

### La surcouche par duplication : le besoin est juste, le mécanisme existe déjà

Proposition de Joel (06/09/2026) : plutôt qu'un fork où tout se mélange,
**dupliquer la fiche** — une fiche modifiée est copiée dans un espace
personnel, on y met ses doors et ses links, et l'application **additionne les
deux, la surcharge gagnant**. « On ne serait plus sur un fork pur, mais ce
serait plus clean, non ? »

Le besoin derrière est réel et mal servi aujourd'hui : **savoir ce qui est à
soi**. Rien dans forkstify ne le montre.

Mais la duplication le paierait cher :

- **Elle fige la fiche au jour de la copie.** L'amont corrige un MBID,
  ajoute un top, réécrit une description : la copie personnelle ne le voit
  jamais. Il faudrait alors fusionner champ par champ — c'est-à-dire
  réimplémenter git, en moins bien.
- **Elle demande un second format.** Ou bien la copie est une fiche entière,
  et modifier un lien fige tout le reste ; ou bien c'est un format de
  correctif, à concevoir, documenter, versionner. Or
  [0002](../decisions/0002-catalogue-partage-forkable.md) fait du format de
  fiche une **interface publique** : il y en aurait deux.
- **Elle éloigne le débat que Joel dit vouloir.** Proposer en amont
  demanderait d'extraire la ligne de la surcouche pour la reporter dans la
  fiche de base — de la friction exactement là où l'on veut de la fluidité.

**Et surtout : la surcouche existe déjà, elle est simplement invisible.**
Dans un fork, ce qui est à soi, ce sont **ses commits** — `git diff
upstream/main` les rend, fiche par fiche et ligne par ligne. C'est une
surcouche *calculée* plutôt que *stockée* : rien à fusionner, rien à figer,
rien à versionner en double. Ce que la duplication apporterait de « clean »,
git le donne déjà ; ce qui manque, c'est que forkstify le **montre**.

**Proposition, à trancher** : garder
[0008](../decisions/0008-le-fork-est-la-surcouche.md) et ajouter de quoi
voir sa propre couche — une commande `:mine` qui liste ce qui diffère de
l'amont, fiche par fiche, et d'où chaque ligne vient (la provenance étant
désormais dans les notes). Coût : quelques lignes autour de `git diff`.

Si la duplication reste préférée malgré tout, elle demande une **décision
qui remplace 0008**, et il faudra y trancher : granularité de la copie,
format du correctif, comportement quand l'amont change la fiche d'origine,
et chemin de contribution.

### Reprendre une partie du catalogue de quelqu'un d'autre

Question de Joel (06/09/2026) : « si quelqu'un a un gros catalogue jazz, je
serai intéressé pour le récupérer — pour autant, je ne serai peut-être pas
intéressé par tous ses commits ».

**C'est un cas que le format sert déjà**, et sans rien inventer :
[0010](../decisions/0010-format-revise-links-sans-portes.md) impose
`cards/` **plat, une fiche par artiste**. Reprendre « son jazz » n'est donc
pas reprendre des *commits* mais des **fichiers** — et git sait faire :

    git remote add untel git@github.com:untel/forkstify-catalog.git
    git fetch untel
    # voir ce qu'il a que je n'ai pas
    git diff --stat HEAD untel/main -- cards/
    # ne prendre que ce qu'on veut, par fichier
    git checkout untel/main -- cards/john-coltrane.toml cards/alice-coltrane.toml
    git commit -m "importe le jazz d'untel"

Un seul commit chez soi, choisi, qui dit ce qu'il fait. Aucun de ses commits
à lui n'entre — on prend l'**état** de ses fiches, pas son histoire.

Pour les trouver par famille plutôt qu'un par un, c'est le champ `tags` qui
sert : lister les fiches de son dépôt dont les `tags` contiennent `jazz`,
puis les passer à `git checkout`. Un script de `tools/` ferait ça en
quelques lignes.

**Deux propriétés du moteur rendent l'import partiel sûr**, et c'est
important puisqu'on prend un morceau d'un tout :

- **Un lien pendant est ignoré, pas fatal.** `graph_neighbors` cherche la
  cible d'un `link` avec `.get()` : un lien vers une fiche qu'on n'a pas
  passe simplement son tour. On peut donc prendre dix fiches d'un ensemble
  de cent sans rien casser.
- **Une fiche sans vecteur ne casse rien non plus** — elle est seulement
  invisible à la branche aventureuse, `vector_neighbors` ne travaillant que
  sur ce que `vecteurs.jsonl` contient. **Il faut donc régénérer les
  vecteurs après un import** (`tools/vectoriser.py`), sinon les fiches
  reprises ne seront atteignables que par le graphe.

Cette tolérance n'est pas un hasard : elle vient de « un fichier par
artiste » et du fait que le graphe et l'espace vectoriel sont deux chemins
indépendants vers le même artiste.

**Reste ouvert** : faut-il un lien **personnel**, qui ne parte jamais en PR ?
Deux façons, aucune tranchée — un champ dans la ligne (`personal = true`),
ou rien du tout, la relecture en amont faisant le tri. La seconde est plus
sobre et conforme à « pas d'abstraction avant le deuxième usage » ; la
première dit la chose plutôt que de compter sur la vigilance.

Entre les deux derniers, la **promotion** : transformer un signal d'usage en
connaissance. « Tu as choisi 6 fois la branche Cocteau Twins depuis The Cure,
j'ajoute la connexion `voisinage` à la fiche ? » Une promotion = un commit
lisible. C'est ce qui empêche le catalogue de devenir une boîte noire : tout
ce qu'il a appris seul est un diff qu'on peut relire et annuler.

### Forme de l'appris (acté le 04/09/2026, [0014](../decisions/0014-forme-de-l-appris.md))

Comment la couche *appris* (dossier `learned/`) stocke ce que l'écoute
apprend, et ce que le moteur en lit. Le vocabulaire sur disque est en
**anglais**, comme le format de fiche (le dépôt vise l'open source). Trois
principes qui découlent du reste :

- **Un fichier par artiste**, `learned/artists/<slug>.toml`, en miroir des
  fiches. Comme une fiche par artiste, ça diffe proprement, ça ne crée
  **jamais de conflit** avec l'amont (chacun le sien), et ça scale. Un seul
  gros fichier grossirait sans fin et casserait à chaque merge.
- **Des compteurs qui décroissent tout seuls dans le temps.** Plutôt que
  garder l'historique de chaque écoute, on garde **un compte décru** : à
  chaque écoute, `plays = plays × ½^((maintenant − last)/demi-vie) + 1`, et
  `last = maintenant`. Un seul flottant et une date par artiste (et par top),
  et une écoute d'il y a trois ans ne pèse presque plus — la question de la
  décroissance est réglée par construction. Demi-vie par défaut : **6 mois**,
  réglable.
- **Silencieux, jamais reversé.** L'appris se modifie sans rien demander
  (c'est de la mesure), et l'application ne l'inclut jamais dans une PR.

Forme d'un fichier :

```toml
# learned/artists/the-cure.toml
plays = 12.4          # écoutes, décrues dans le temps (familiarité)
last  = "2026-09-04"  # dernière écoute (récence + cooldown)
weight = 0.8          # correctif « - » (moins souvent) ; 1.0 = neutre
blacklisted = false   # « X » sur l'artiste entier

[tops."A Forest"]
plays = 5.0
last  = "2026-09-04"
liked = true          # « a »
skipped = 2           # « x » cumulés

[tops."Killing an Arab"]
blacklisted = true    # « X » sur ce morceau
```

Les **récoltes** (touche `m`, à trier plus tard) vivent à part, transverses
aux artistes : `learned/marks/<name>.toml` (liste de `{artist, title, at}`).

**Ce que chaque touche de [0013](../decisions/0013-affinage-clavier-mesure-ou-edition.md)
écrit** — mesures dans `learned/artists/`, éditions dans la fiche (commit) :

| Touche | Effet | Où |
|---|---|---|
| écoute complète (auto) | `plays += 1`, `last` | appris (mesure) |
| `x` sauter | `tops.<t>.skipped += 1` | appris (mesure) |
| `X` écarter | `blacklisted = true` (top ou artiste) | appris (mesure) |
| `a` aimer | `tops.<t>.liked = true` (+ titre aimé Spotify) | appris (mesure) |
| `-` moins souvent | `weight ×= 0.7` (plancher) | appris (mesure) |
| `m` marquer | ligne dans `learned/marks/` | appris (mesure) |
| `t`/`T` top | ajoute/retire des `tops` | **fiche (commit)** |
| `d` door | ajoute une `door` | **fiche (commit)** |
| `E` éditer | ouvre la fiche dans `$EDITOR` | **fiche (commit)** |

**Ce que le moteur lit** de l'appris, en plus de la fiche :

- **exclusion** des `blacklisted` (artiste et top) ;
- **familiarité** = `plays` décru (+ l'amorce `learned/classement.json` pour
  un artiste sans appris encore) → nourrit la **zone de confort** (0001) et
  les seuils de l'aventureuse ;
- **cooldown** (0012) : un `last` récent baisse le poids / suspend, pour que
  ce qu'on vient d'écouter tourne (le « sans remise » d'une session, lui,
  reste en mémoire) ;
- **poids** : `weight` multiplie le poids de branche de l'artiste ; un top
  souvent `skipped` recule dans le segment, un top `liked` avance.

`classement.json` (l'amorce, 741 artistes scorés depuis la bibliothèque)
devient donc **la familiarité de départ** ; `learned/artists/` la prolonge
et la corrige au fil de l'écoute.

> Vocabulaire : la couche « appris » est le dossier **`learned/`** (renommé
> depuis `learned/` le 04/09/2026 — pas de terme français dans les chemins ni
> le format). Les fichiers d'amorce encore nommés en français dans
> `learned/` (`amis/`, `artistes-*.json`) et leurs clés seront traduits avec
> les scripts de `tools/`.

Orientation : l'*appris* se modifie seul en silence (c'est de la mesure) ;
la *promotion* se propose par défaut et peut passer en automatique par
réglage — l'application n'exige jamais de décision, mais rend les siennes
visibles.

### Une base artiste libre

- **L'identité d'un artiste est son MBID**
  ([0009](../decisions/0009-identite-mbid.md)) ; Spotify est une
  implémentation parmi d'autres, Deezer ou d'autres viendront sans toucher
  au catalogue.
- **Le catalogue de référence est proposé au téléchargement par
  l'application** : au premier lancement, forkstify propose de le cloner,
  et chacun se crée sa propre version à partir de là
  ([0004](../decisions/0004-deux-depots-catalogue-ciblable.md),
  [0008](../decisions/0008-le-fork-est-la-surcouche.md)).
- **« Complète » se construit par l'usage, pas par un dump.** On ne descend
  pas MusicBrainz entier. Chaque fiche naît parce que quelqu'un est arrivé
  chez cet artiste — bibliothèque, parcours, PR des autres. Le catalogue est
  complet *au sens de ses usagers*.
- **Deux niveaux de qualité, visibles** : *générée* et *relue*. Une fiche
  relue par un humain vaut plus, et le moteur peut le savoir.
- **Les faits et le sens ne viennent pas du même endroit.** Les faits (nom,
  MBID, identifiants, pays, années, tags, artistes liés) viennent de sources
  libres et vérifiables — MusicBrainz, Wikidata, Last.fm pour les
  similaires — jamais d'un modèle de langage : pas de faits inventés dans une
  base libre. Le sens (description, connexions typées et commentées, portes)
  est là où un modèle aide, et où la relecture humaine compte.

### Le démarrage à froid et la base disponible

Un nouvel utilisateur ne doit rien avoir à écrire :

1. **Au premier lancement, l'application propose de cloner le catalogue de
   référence** (déjà acté). Personne ne part de zéro.
2. **Puis l'import personnel** (son Spotify ou son Deezer — les scripts
   de `tools/` en sont le prototype) : les artistes déjà dans la base ne
   coûtent rien (leurs signaux calibrent la zone de confort, dans
   `learned/`) ; les absents passent par le pipeline de génération.

**Le pipeline de génération** — mêmes sources vérifiées le 31/08/2026 :

| Champ | Source |
|---|---|
| identité, dates, origine, tags | MusicBrainz / Wikidata |
| tops | Deezer `/artist/top` (sans clé) + titres aimés de l'utilisateur |
| links `member` / `collab` / `family` | relations MusicBrainz (typées, factuelles) |
| links `similar` | Deezer `/artist/related` (sans clé), Last.fm en renfort |
| links `scene` | recoupement époque + pays + genres |
| notes, description | modèle de langage, ou absentes (tout est optionnel) |

Tout est marqué `generated`. « Tout optionnel » et « on écrit pour les
humains » rendent la génération automatique *suffisante* pour un produit
utilisable, et la relecture *améliorante* plutôt qu'obligatoire.

**Un seul pipeline, trois moments** : ensemencer la référence, importer au
premier lancement, générer en cours d'écoute (catalogue vivant).

**La base disponible se travaille sur trois chantiers :**

1. **Le noyau relu** — en cours : la bibliothèque de Joel (741 artistes
   scorés, 30 fiches écrites), ses amis, le lot 2 (les fiches appelées par
   les links).
2. **L'ensemencement** — le pipeline en batch sur les artistes les plus
   écoutés (charts Last.fm / ListenBrainz), pour couvrir la bibliothèque de
   n'importe quel nouveau venu au jour 1.
3. **La mutualisation** — quand l'application d'un utilisateur génère une
   fiche absente de la référence, elle propose de la reverser à l'amont
   (PR pré-mâchée). Chaque démarrage à froid enrichit le commun.

## À trancher

- **Suivre l'amont.** Le scénario de Joel : « j'ai téléchargé le catalogue
  initial, je l'ai fait évoluer selon mes goûts ; six mois plus tard, le
  catalogue initial a doublé en volume et en qualité — comment en
  profiter ? » C'est une fusion git de l'amont dans le fork, et la structure
  aide : une fiche par artiste (les nouvelles fiches arrivent sans aucun
  conflit), l'appris dans `learned/` (jamais en conflit avec l'amont), le mien
  concentré sur les fiches que j'ai touchées. Les conflits réels se limitent
  donc aux fiches modifiées des deux côtés — et là, une commande `forkstify
  catalogue sync` doit guider champ par champ (« l'amont a enrichi la
  description de The Cure, tu as changé les tops : je prends les deux ? »).
  À concevoir sérieusement le moment venu ; dans l'autre sens, faciliter la
  PR pour reverser une fiche.
- **Ergonomie de l'import** : `forkstify catalogue add <url>`, `forkstify
  catalogue use <nom>`, `forkstify catalogue list` ? Et où vivent les clones
  (`~/.local/share/forkstify/catalogues/<nom>` ?).
- **Identité des morceaux** : par titre (lisible, ambigu — versions live,
  remasters) ou par identifiant Spotify (précis, illisible) ? Probablement
  le titre dans la fiche, résolu en identifiant au moment de jouer, avec
  mise en cache.
- **Vocabulaire des tags** : libres, avec une liste recommandée à publier ?
  (Les types de liens sont fermés depuis [0010](../decisions/0010-format-revise-links-sans-portes.md).)
- **Nom du fichier** : le slug (`the-cure.toml`) est aussi la clé des
  connexions (`vers = "the-cure"`). Règle de slugification à fixer
  (accents, articles, homonymes).
- **Promotion automatique ou avec confirmation** : réglage par défaut, et
  granularité (par type de promotion ?).
- **Forme de l'appris** : proposée le 04/09/2026 (voir l'orientation
  « Forme de l'appris » plus haut — un fichier par artiste, compteurs
  décrus). Restent à régler au fil du PoC : la **demi-vie** de décroissance
  (6 mois par défaut), la **fenêtre de cooldown** (0012), et la formule
  familiarité → zone de confort 0–5 (0001).
- **Marquage des fiches générées** : un champ (`generee = true`), un
  dossier à part, ou les deux ?
- **Licence** du catalogue partagé : une licence de données (ODbL comme
  OpenStreetMap, ou CC BY-SA), et vérifier la compatibilité des sources
  versées — MusicBrainz et Wikidata oui, Last.fm plus flou.
- **Le modèle de langage à l'exécution** (fiche d'un artiste inconnu) :
  quel fournisseur, quelle clé, quel repli hors-ligne ?
- **L'ensemencement** : combien d'artistes (mille ? cinq mille ?), quelle
  source de charts, et Last.fm nécessite une clé d'API — Deezer non.
- **Prochain pas concret** : prototyper le générateur dans `tools/` et
  le lancer sur les 61 fiches appelées par les links du lot 1 — construit
  le lot 2 et valide le démarrage à froid.
