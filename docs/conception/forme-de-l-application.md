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

### Choisir la graine

Deux entrées, dans l'esprit neovim :

- **`/` puis du texte** : recherche incrémentale, d'abord dans le catalogue
  actif (artistes, tops, portes), puis dans Spotify si rien ne correspond.
- **Une liste** : la bibliothèque de l'utilisateur — artistes et albums
  aimés sur Spotify — parcourue au clavier (`j` / `k`), filtrée par `/`.

Dans le PoC sans interface, la graine est simplement l'argument de la
commande.

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
