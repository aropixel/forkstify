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
  `catalogue.toml`** ([0010](../decisions/0010-format-revise-links-sans-portes.md)).
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
# fiches/the-cure.toml
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

Et à la racine du catalogue, `catalogue.toml` porte l'identité et les
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
  (défaut par type dans `catalogue.toml`, correctif local) — pas de
  sémantique invisible, pas de fragilité au merge.
- **Résolution de `proximity`, en cascade** : la valeur sur le lien s'il y
  en a une ; sinon la grille `[proximity]` du `catalogue.toml` du catalogue
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
  l'amont régénère tout à chaque changement de modèle.
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
   (`usage/`), versionné pour être portable, que l'application sait ne
   jamais inclure dans une PR.

Entre les deux derniers, la **promotion** : transformer un signal d'usage en
connaissance. « Tu as choisi 6 fois la branche Cocteau Twins depuis The Cure,
j'ajoute la connexion `voisinage` à la fiche ? » Une promotion = un commit
lisible. C'est ce qui empêche le catalogue de devenir une boîte noire : tout
ce qu'il a appris seul est un diff qu'on peut relire et annuler.

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

### La base initiale

Deux cercles :

1. **Le catalogue de Joel** : une fiche par artiste aimé, générée puis
   relue. Pour l'amorçage, le modèle de langage peut être **l'agent en
   session** : Joel donne les artistes, l'agent écrit les fiches, Joel
   corrige. Aucune intégration à coder pour commencer.
2. **Les artistes connus et communs** qui n'y sont pas, pour que le
   catalogue de référence serve à d'autres que Joel dès le départ. Comment
   les choisir, à trancher : un saut de voisinage depuis le cercle 1
   (similaires Last.fm des artistes de Joel), les artistes les plus écoutés
   (charts Last.fm / statistiques ListenBrainz), ou les deux. Ces fiches
   restent marquées générées jusqu'à relecture — par Joel ou par les futurs
   contributeurs.

Plus tard, l'application appelle un modèle elle-même pour la fiche d'un
artiste inconnu (fournisseur et clé à trancher). Une centaine de fiches
relues valent plus qu'un million générées.

## À trancher

- **Suivre l'amont.** Le scénario de Joel : « j'ai téléchargé le catalogue
  initial, je l'ai fait évoluer selon mes goûts ; six mois plus tard, le
  catalogue initial a doublé en volume et en qualité — comment en
  profiter ? » C'est une fusion git de l'amont dans le fork, et la structure
  aide : une fiche par artiste (les nouvelles fiches arrivent sans aucun
  conflit), l'appris dans `usage/` (jamais en conflit avec l'amont), le mien
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
- **Forme de l'appris** : un fichier par artiste dans `usage/`, ou un seul
  fichier ? Quels compteurs exactement, avec quelle décroissance dans le
  temps (une écoute d'il y a trois ans compte-t-elle encore) ?
- **Marquage des fiches générées** : un champ (`generee = true`), un
  dossier à part, ou les deux ?
- **Licence** du catalogue partagé : une licence de données (ODbL comme
  OpenStreetMap, ou CC BY-SA), et vérifier la compatibilité des sources
  versées — MusicBrainz et Wikidata oui, Last.fm plus flou.
- **Le modèle de langage à l'exécution** (fiche d'un artiste inconnu) :
  quel fournisseur, quelle clé, quel repli hors-ligne ?
