# Forme de l'application et PoC

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Références

Points de départ de la réflexion de Joel, et ce qu'on en retient :

- [stappmus/Omarchy-Spotify](https://github.com/stappmus/Omarchy-Spotify) —
  plugin Omarchy (QML dans la barre, JS pour l'API et l'OAuth), son via un
  backend Rust local ou le paquet `spotifyd` d'Omarchy, pilotage par l'API
  Web. ~60 Mo au lieu des ~950 Mo du client officiel. **Retenu** : le modèle
  d'intégration à Omarchy, et le fait que le son est un appareil Spotify
  Connect local piloté par l'API Web.
- [crmne/fastpotify](https://github.com/crmne/fastpotify) — client complet
  en Rust (egui + librespot), OAuth PKCE, MPRIS, commandes en ligne
  (`fastpotify next`). **Retenu** : librespot est viable ; un client complet
  est beaucoup trop gros pour ce qu'on veut.
- [ssp-data/neomd](https://github.com/ssp-data/neomd) — client mail TUI en
  Go (Bubble Tea, Glamour, Lipgloss), raccourcis vim, configuration en
  fichiers texte, rien de stocké en local. **Retenu** : la forme. Clavier
  d'abord, un écran, du texte, minimaliste. Joel a déjà écrit
  [`omarchy-neomd`](https://github.com/kbyjoel/omarchy-neomd), le widget de
  barre qui l'accompagne — le couple TUI + widget de barre est un chemin
  connu.

## Décidé

- Pas de maquette pour l'instant ; **le concept et le PoC passent avant
  l'interface**.
- **Rust** : `ratatui`, `rspotify`, `tokio`, `fastembed` pour les vecteurs
  ([0006](../decisions/0006-rust.md)).
- **Affinage au clavier : chaque touche est une mesure (`usage/`) ou une
  édition (un commit)**, `u` annule la dernière, chaque touche n'est que le
  raccourci d'une commande `:`
  ([0013](../decisions/0013-affinage-clavier-mesure-ou-edition.md)). La
  table des touches ci-dessous reste une orientation.

## Orientations

### Séparer le cerveau du son — dans le code, pas dans le binaire

Le moteur de branches ignore comment le son sort : il produit des morceaux
à jouer, un autre composant les joue. Première idée : pousser dans la file
d'un appareil Spotify Connect existant (`spotifyd`, client officiel). Après
vérification ([spotify.md](spotify.md)), l'orientation est plutôt que
**forkstify embarque librespot et est lui-même l'appareil Connect** :
l'utilisateur n'installe rien d'autre, et se connecte en choisissant
« forkstify » dans la liste des appareils de son téléphone. Pousser vers un
autre appareil Connect reste possible par l'API Web, en plus.

### Une TUI clavier d'abord, à la neomd

Quand il y aura une interface : un terminal, un écran, raccourcis vim. Un
embranchement, c'est trois lignes ; on choisit avec `1` `2` `3` ou
`h` / `l` (rassurant / aventureux), on ponce avec `.`, on saute avec `n`.
Lancée par `omarchy-launch-or-focus-tui forkstify`, complétée plus tard par
un widget de barre sur le modèle d'`omarchy-neomd` (morceau en cours,
prochain embranchement, un clic pour ouvrir).

### Affiner l'algorithme au clavier

Principe acté le 01/09/2026
([0013](../decisions/0013-affinage-clavier-mesure-ou-edition.md)) ; la
table des touches, elle, s'ajuste au fil du PoC. **Chaque touche est
soit une mesure, soit une édition.**

- Une **mesure** modifie `usage/` en silence (sauter, aimer) — c'est
  l'*appris* de [catalogue.md](catalogue.md).
- Une **édition** modifie une fiche et produit **un commit lisible**
  (« top : + A Forest ») — c'est le *mien*, immédiatement partageable.
- `u` **annule la dernière**, quelle qu'elle soit — revert pour une
  édition, effacement pour une mesure. L'historique d'affinage *est* le
  log git.

Sur le **morceau en cours** :

| Touche | Action | Nature |
|---|---|---|
| `t` | Promouvoir en top | commit |
| `T` | Retirer des tops | commit |
| `e` / `<n>e` | **Encore** (« poncer », dans l'argot du projet) : intercaler n morceaux de plus de l'artiste du morceau en cours juste après lui, le segment reprend ensuite. n = taille des branches par défaut (`:encore n`) | lecture |
| `x` | Sauter — « pas celui-là, pas maintenant » | mesure |
| `X` | Écarter — « plus jamais celui-là » (liste noire dans `usage/` : un dégoût est personnel) | mesure |
| `a` | Aimer (et le refléter en titre aimé Spotify) | mesure |
| `d` | En faire une **door** vers la direction du dernier embranchement pris — pré-remplie avec les tags de la branche, on valide | commit |
| `m` | Marquer : le mettre dans une **récolte** à trier plus tard, sans interrompre l'écoute | mesure |

Le `d` se fait au moment exact où une door a du sens (« c'est depuis ce
morceau que je suis parti vers le post-punk ») — personne ne l'écrirait à
froid dans un fichier.

Sur l'**artiste** :

| Touche | Action | Nature |
|---|---|---|
| `E` | Ouvrir la fiche dans `$EDITOR`, recommit au retour, vecteur recalculé | commit |
| `-` | Cet artiste, moins souvent (poids d'usage, sans le bannir) | mesure |

Le `E` est le geste ultime à la vim : quand les raccourcis ne suffisent
plus, on édite le texte — le fork est la surcouche, littéralement. (Il
était sur `e`, cédé à encore le 03/09/2026 : minuscule = geste léger et
fréquent, majuscule = geste lourd, comme `t`/`T` et `x`/`X`.)

Sur le **parcours** (en plus de `1 2 3`, `h`/`l`, `.`, `n`) :

| Touche | Action |
|---|---|
| `j` / `k` | Morceau **suivant / précédent**, comme un player classique (implémenté dans `ecouter` le 04/09/2026) — aussi pilotable par les **touches multimédia** ⏮ ⏭ ⏯ via MPRIS (D-Bus), au même titre que `playerctl` |
| `u` | Annuler : le dernier choix de branche, ou la dernière édition |
| `z` puis `0`–`5` | Régler la zone de confort en cours de route |
| `?` | **Pourquoi** : afficher la phrase qui explique le morceau ou la branche |
| `y` / `n` | Répondre à une **promotion** proposée (« j'ajoute la connexion ? ») |

Enfin, très neovim : chaque touche n'est que le raccourci d'une **commande
`:`** (`:top`, `:door post-punk`, `:fiche`, `:confort 2`). Les commandes
rendent tout découvrable et scriptable ; les keybinds deviennent une table
de correspondance, remappable dans un fichier de config.

### Une branche est un segment

Demandé par Joel au premier test de la navigation à sec (03/09/2026) :
une branche ne propose pas *un artiste* mais **un segment de quelques
morceaux, potentiellement sur plusieurs artistes**. Depuis The Cure :

1. Siouxsie and the Banshees → Cult Hero → Joy Division ;
2. New Order → Depeche Mode → Nouvelle Vague ;
3. (et, plus tard dans le parcours, « rester dans l'univers »).

**Poncer n'est pas une branche** (Joel, 03/09/2026) : les branches
proposent des futurs, poncer réagit au moment — « encore de *ça* ». C'est
donc une touche, `e` / `<n>e` (commande `:encore`), qui intercale n
morceaux de plus de l'artiste du morceau en cours juste après lui ; le
segment choisi reprend ensuite. Conséquence assumée : le mode auto ne
ponce plus jamais — s'attarder est un désir d'auditeur, pas une décision
de moteur (la zone de confort pourra redonner ce penchant à l'auto).
Nuance PoC : à sec il n'y a pas de « morceau en cours », `e` vise le
dernier artiste du segment ; la vraie sémantique arrive avec le son.

Affichage pendant l'écoute (Joel, 04/09/2026) : on montre la **file des
morceaux à venir** (le morceau courant sur la ligne `▶`), et les
**branches n'apparaissent qu'au dernier morceau** du segment — au moment
de choisir. `p` permet de les voir à l'avance à tout instant (et d'en
choisir une avec `1`-`3`) ; la vraie « prévisualisation puis choix
anticipé » se fera dans l'interface graphique.

Une direction est une petite **marche** dans le graphe : on part d'un
voisin, on enchaîne vers son voisin le plus proche, un morceau par
artiste traversé (un artiste sans tops est traversé sans morceau). La
**taille des branches** se règle en cours de route — raccourci provisoire
`b<n>`, future commande `:taille`. En condition réelle, la graine sera un
morceau précis trouvé par la recherche, pas seulement un artiste.

Et les directions suivantes se proposent depuis **la branche entière**,
pas depuis son seul dernier artiste (Joel, 03/09/2026) : côté graphe, les
voisins des n artistes cumulés — un candidat lié à plusieurs d'entre eux
monte (« lié à 2 artistes de la branche ») ; côté vecteurs, le voisin du
**centroïde** de la branche — son centre de gravité. Le ponçage et la
marche interne d'une branche restent sur le dernier artiste.

Deux retours du deuxième test (Joel, 03/09/2026) :

- **Rien n'est déterministe.** Deux parcours depuis la même graine
  diffèrent : têtes de branches et sauts de marche sont **tirés au sort
  pondéré** dans un réservoir de bons candidats — la décision
  [0012](../decisions/0012-rotation-des-morceaux.md) appliquée aux
  branches elles-mêmes, pas seulement aux morceaux.
- **Une branche « rester dans l'univers »**, quatrième au menu quand elle
  a du sens : un segment tiré du voisinage de **tout le parcours** (ses
  artistes et leurs voisins de graphe, classés par proximité au centroïde
  du parcours), où les artistes déjà visités **reviennent** tant qu'ils
  ont des morceaux non joués. C'est la branche qui permet de tourner dans
  un cluster aussi longtemps qu'on veut.

Et un garde-fou du troisième test (The Cure proposait la chanson
française à 0.67 de cosinus) : **la branche aventureuse a un plancher**.
Un candidat vecteur doit être assez proche en absolu (≥ 0.72) **et**
partager au moins un tag de genre avec la branche — pays et décennie ne
justifient pas un pont — sauf s'il est très proche (≥ 0.80). En dessous,
la branche aventureuse disparaît et le graphe reprend la place. Ces deux
seuils sont des constantes en attendant d'être pilotés par la **zone de
confort** (0001) : confort bas = plancher haut.

### Choisir la graine

Deux entrées, dans l'esprit neovim :

- **`/` puis du texte** : recherche, dans le catalogue actif **et** dans
  Spotify, présentée en une liste fusionnée. Implémentée dans `ecouter` le
  04/09/2026 : un résultat `[catalogue]` démarre un segment sur cet artiste
  (branches natives) ; un résultat `[spotify]` joue la piste — et si son
  artiste a une fiche, le parcours s'y raccroche pour continuer à brancher,
  sinon c'est hors catalogue (pas de branche depuis là jusqu'à la génération
  de fiche à la volée). Les deux recherches ne servent pas la même chose :
  les fiches disent *d'où brancher*, l'API permet de jouer *n'importe quoi*.
- **Une liste** : la bibliothèque de l'utilisateur — artistes et albums
  aimés sur Spotify — parcourue au clavier (`j` / `k`), filtrée par `/`.

Dans le PoC sans interface, la graine de départ est l'argument de la
commande ; `/` sert ensuite à sauter n'importe où en cours de parcours.

### Le PoC : jouer l'application avant de jouer le son

Le critère est de Joel : « on doit même pouvoir jouer l'application avant de
pouvoir jouer le son — choisir une chanson de départ, tester le parcours par
branches sur la base locale, et constater que la navigation est cohérente.
À ce moment-là on sait qu'on a un PoC qui fonctionne. » Le son, Spotify et
l'interface viennent après ; aucun lecteur ne sauvera des branches
incohérentes.

1. **La base initiale** — le catalogue de Joel, fiches générées (faits
   depuis MusicBrainz / Wikidata / Last.fm, sens écrit par l'agent en
   session), relues, vecteurs calculés ([catalogue.md](catalogue.md)).
2. **La navigation à sec** — une commande : on donne une graine, elle
   propose trois branches lisibles avec leurs raisons, on choisit au
   clavier, elle affiche le segment (titres, sans les jouer), et ainsi de
   suite. D'abord sur les seules connexions et tags ; les vecteurs ensuite,
   pour combler les trous. **Critère de réussite : des parcours cohérents,
   constatés en les lisant.**
3. **Le son** — le spike Spotify ([spotify.md](spotify.md)) puis la lecture
   des segments. Peut démarrer en parallèle de 2, il n'en dépend pas.
4. **La TUI** — quand on sait ce qu'il y a à afficher.

## À trancher

- **Où tourne le PoC** : tout est dockerisé sur la machine de Joel ; le
  binaire se construit dans un conteneur (`cargo build`) et s'exécute sur
  l'hôte sans rien y installer.
- **Sans interface, comment on choisit** : un chiffre dans le terminal
  suffit pour le PoC ; la zone de confort choisit seule sur un délai.
