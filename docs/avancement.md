# Avancement

Mis à jour le **03/09/2026**. Ce fichier est le point d'entrée pour reprendre
le travail : ce qui est fait, ce qui attend Joel, ce qui vient ensuite.

## Fait

- **Conception** : vision et philosophie (« Reprendre la main sur
  l'algorithme »), vocabulaire, 13 décisions (`docs/decisions/`), 5 notes
  vivantes (`docs/conception/`). Rust, TOML, MBID, deux dépôts, format de
  fiche v1 (links typés anglais + proximité en cascade, doors en critère
  additionnel, tout optionnel sauf `format`/`name`/`mbid`). Dernières
  actées le 01/09/2026 : rotation des morceaux (0012) et affinage au
  clavier — mesure ou édition (0013).
- **Catalogue amorcé** (`~/Work/forkstify-catalog`, GitHub privé) :
  `catalogue.toml` (grille type → proximité), **30 fiches** écrites
  (`generated = true`, le haut du classement de Joel), **outillage/** —
  7 scripts Python d'amorçage (lecture bibliothèque/playlists Spotify via la
  session Omarchy-Spotify, récolte amis Spotify/Deezer, résolution MBID,
  classement), **usage/** — 741 artistes scorés (titres aimés, albums,
  #fipway, road trip BDX//ATX, suivis), 728 MBID résolus.
- **Analyse d'Omarchy-Spotify** (code lu) : deux OAuth PKCE navigateur sans
  dashboard (client id ncspot pour l'API Web, client id desktop Spotify pour
  librespot), backend Rust ~800 lignes autour de librespot. Voir
  `docs/conception/spotify.md`.
- **Démarrage à froid conçu** : pipeline de génération de fiches (faits
  MusicBrainz, tops Deezer `/artist/top`, similaires Deezer
  `/artist/related` — vérifiés sans clé), trois chantiers de la base. Voir
  `docs/conception/catalogue.md`.
- **Générateur de fiches prototypé** (01/09/2026,
  `outillage/generer-fiches.py`) et **lot 2 généré : 58 fiches** — les
  slugs appelés par les links du lot 1 (56 réels, pas 61) plus 2 membres de
  Destiny's Child appelés en cascade. Faits, dates, origine et relations
  typées MusicBrainz ; tops et similaires Deezer ; tags genres + pays +
  décennie (groupes seulement — le begin d'une personne est sa naissance) ;
  liens scene par recoupement. Descriptions absentes des fiches générées :
  la relecture enrichit.
- **Lot 3 élargi généré** (02/09/2026) : **126 fiches** — les 89 slugs
  appelés par les links du lot 2, plus les artistes du classement à
  score ≥ 5 sans fiche. Correction en route : « Experience » (bibliothèque
  de Joel) résolu à tort en The Jimi Hendrix Experience — c'est
  **Expérience** (Michel Cloup, Toulouse), MBID corrigé dans
  `usage/mbid.json`. Le catalogue compte **214 fiches** ; le lot 3 appelle
  à son tour **100 slugs** (lot 4, non généré — la traîne du classement à
  score 1–4 est aussi laissée de côté : ces artistes entreront quand un
  link les appellera).
- **Vecteurs prototypés** (02/09/2026) : `outillage/vectoriser.py`
  compose le texte de chaque fiche depuis sa structure (tags, dates,
  origine, liens sortants et entrants, description si présente) et
  calcule les vecteurs dans un conteneur — modèle
  `paraphrase-multilingual-MiniLM-L12-v2` (fastembed, 384 dimensions,
  mean pooling, dispo en Python et en Rust). Index dérivé commité :
  `vecteurs/vecteurs.jsonl` (214 fiches) + `meta.toml`.
  `outillage/voisins.py` (stdlib) = prototype de `forkstify check` :
  voisins cohérents (The Cure → Joy Division/Siouxsie ; IAM → le rap
  français ; Nina Simone → Ella/Nat King Cole) ; les fiches maigres ont
  des voisins flous à scores bas, ce que `check` doit justement révéler.
- **Le son câblé sur la navigation** (04/09/2026) : `forkstify ecouter
  <graine>` — même moteur et mêmes menus que `parcours`, mais **ça joue**.
  Modules `sound.rs` (lecteur librespot embarqué), `spotify.rs` (Web API
  ncspot : résolution titre → `spotify:track:`, cache disque, backoff 429)
  et `listen.rs` (boucle async : lecture en fond, menu par-dessus, segment
  fini → auto-avance pour ne jamais s'arrêter ; `1-3` saute vers une
  branche, `j`/`k` morceau suivant/précédent (player classique, timeline
  passé/courant/file, événements filtrés par `play_request_id`), `e`/`<n>e`
  intercale, `b<n>` la taille, `u` branche précédente, `q` quitte). Le moteur reste
  intact — il produit des morceaux, `sound`/`spotify` les jouent. `parcours`
  reste le mode à sec (rapide, sans Premium, pour itérer sur le moteur).
  Construction via l'image **`forkstify-build`** (`Dockerfile` : rust +
  pkg-config + libasound2-dev).
- **Navigation à sec prototypée** (03/09/2026) — le premier code Rust,
  dans ce dépôt : `forkstify parcours <graine>` propose 3 branches
  lisibles avec leurs raisons (graphe d'abord — liens typés dans les deux
  sens, proximité en cascade —, vecteurs pour combler et pour la branche
  aventureuse), choix `1-3`, entrée = auto pondéré, `u` retour, `q` quitter ;
  segments = 3 tops tirés sans remise (esprit 0012). `forkstify check
  <artiste>` = les voisins dans l'espace, liens du graphe marqués. Se
  construit dans un conteneur (`docker run --rm -v "$PWD":/app -w /app -v
  forkstify-cargo:/usr/local/cargo/registry rust:1-slim cargo build
  --release`), s'exécute sur l'hôte. Parcours constatés cohérents :
  IAM → Zebda → Fabulous Trobadors → La Rue Kétanou → Camille ;
  The Cure → Siouxsie → Cult Hero → The Fall → Joy Division. **Code et
  commentaires en anglais** (visée open source — règle dans AGENTS.md).
  Retouches au test de Joel : **une branche est un segment** — poncer
  l'artiste courant, ou une marche de n morceaux sur plusieurs artistes
  (un par artiste traversé), taille réglable par `b<n>` — les
  directions se proposent depuis **la branche entière** (voisins de
  graphe cumulés, centroïde des vecteurs), pas depuis le seul dernier
  artiste — **rien n'est déterministe** (têtes et sauts tirés au sort
  pondéré, 0012 appliquée aux branches) — une branche **« rester dans
  l'univers du parcours »** fait tourner dans le cluster, revisites
  permises tant qu'il reste des morceaux non joués — la branche
  aventureuse a un **plancher** (cosinus ≥ 0.72 + un tag de genre commun,
  constantes à piloter par le confort) — et **poncer n'est plus une
  branche mais la touche `e` / `<n>e`** (commande `:encore`) qui
  intercale n morceaux de l'artiste en cours (voir
  `docs/conception/forme-de-l-application.md`).
- **Spike Spotify Connect fait** (03/09/2026, `src/bin/spike-connect.rs`,
  lancé par Joel sur son compte Premium). **Découverte zeroconf entrante :
  ✓** — l'appareil apparaît sur le téléphone, les identifiants arrivent, la
  session librespot s'ouvre (le son est donc validé de bout en bout, sans
  rien installer sur l'hôte, sans Omarchy). **Jeton de session pour l'API
  Web : ✗** — keymaster répond 403, login5 sort un jeton refusé par l'API
  en 429 persistant (client id desktop en quota restreint). Verdict : le
  son passe par librespot embarqué, l'API Web passera par l'OAuth
  navigateur + client id de ncspot (voie de tout l'écosystème). Détail
  dans `docs/conception/spotify.md`.
- **API Web validée** (03/09/2026, `src/bin/spike-webapi.rs`) : OAuth PKCE
  navigateur avec le client id de ncspot (`librespot-oauth`), refresh token
  en cache. `/v1/me`, `/v1/search` (titre → `spotify:track:`) et
  `/v1/me/albums` (247 albums) répondent. Leçon : les 429 rencontrés
  étaient un throttle **compte/IP** temporaire (Retry-After décroissant,
  se résorbe au repos), pas un blocage de client id — le client réel doit
  respecter `Retry-After` (le spike le fait).
- **Lecture validée** (03/09/2026, `src/bin/spike-play.rs`) : lecteur
  librespot embarqué (`librespot-playback`, backend rodio → alsa), charge
  un `spotify:track:` et **le son sort du binaire** — testé par Joel, « ça
  marche très bien ». **Les quatre briques du chantier son sont validées**
  (zeroconf, session, API Web, lecture) ; forkstify est lui-même
  l'appareil, on ne pilote aucun autre appareil par l'API.
- **Dépôts** : `kbyjoel/forkstify` et `kbyjoel/forkstify-catalog`, privés,
  branche `main`. Plan de reprise chorizo à jour.

## En attente de Joel

- **Pas de relecture fiche à fiche** (décision de Joel, 02/09/2026) : il a
  regardé l'ensemble, l'affinage se fera **à l'utilisation** (keybinds,
  décision 0013). Les points connus restent notés pour mémoire : MBID
  incertains (28 du lot 2, 14 du lot 3 — rapports du générateur), tops
  vides (cabadzi, le-motel, la-ruda-salska), doublons de versions chez
  J.P. Nataf, un top russe parasite chez Expérience.
- Identifiants Deezer/Spotify d'**amis consentants** pour élargir la base
  (`outillage/amis-*.py`).

## Prochaines étapes, dans l'ordre

1. **Éprouver `ecouter`** : le tester en vrai (Joel), ajuster la résolution
   titre → id (versions live/remaster, homonymes) et le rythme des menus.
2. **Étoffer la navigation** : zone de confort (0001) dans le choix auto et
   les seuils de l'aventureuse, lecture d'`usage/` (cooldowns 0012),
   premiers keybinds d'affinage (0013) — au fil de l'usage.
3. **Trousseau GNOME** pour les jetons (refresh OAuth, identifiants
   librespot) au lieu des caches `target/` (spike-cache, spike-webapi-refresh).
4. **TUI** (étape 4), à la neomd : touche unique sans Entrée, auto sur
   silence, affichage du morceau en cours.
5. **Traduire en anglais** les scripts d'`outillage/` écrits avant la
   règle de langue du code (à l'occasion).

## Corrections en attente (petites)

- MBID de **Les Thugs** introuvable (homonymie probable) et une entrée au
  nom vide dans `usage/mbid.json` ; 110 MBID résolus « par nom » à relire.
- `resoudre-mbid.py` ne lit que les fichiers Spotify — à adapter aux
  récoltes Deezer (`usage/amis/*-deezer.json`).

## Règles de session

- **Commits et push au fil de l'eau** sur `forkstify` et
  `forkstify-catalog` (demandé par Joel le 31/08/2026). Chorizo : toujours
  demander avant de pousser.
- Les scripts qui lisent le trousseau GNOME sont lancés **par Joel** avec le
  préfixe `!`.
