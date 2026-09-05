# Avancement

Mis à jour le **04/09/2026**. Ce fichier est le point d'entrée pour reprendre
le travail : ce qui est fait, ce qui attend Joel, ce qui vient ensuite.

## Fait

- **Conception** : vision et philosophie (« Reprendre la main sur
  l'algorithme »), vocabulaire, 13 décisions (`docs/decisions/`), 5 notes
  vivantes (`docs/conception/`). Rust, TOML, MBID, deux dépôts, format de
  fiche v1 (links typés anglais + proximité en cascade, doors en critère
  additionnel, tout optionnel sauf `format`/`name`/`mbid`). 14 décisions ;
  dernière le 04/09/2026 : **forme de l'appris** — dossier `learned/`, un
  fichier par artiste, compteurs décrus (demi-vie 6 mois) (0014). Le
  vocabulaire sur disque (chemins, champs) est en anglais comme le code.
- **Catalogue amorcé** (`~/Work/forkstify-catalog`, GitHub privé) :
  `catalogue.toml` (grille type → proximité), **30 fiches** écrites
  (`generated = true`, le haut du classement de Joel), **outillage/** —
  7 scripts Python d'amorçage (lecture bibliothèque/playlists Spotify via la
  session Omarchy-Spotify, récolte amis Spotify/Deezer, résolution MBID,
  classement), **learned/** — 741 artistes scorés (titres aimés, albums,
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
  `learned/mbid.json`. Le catalogue compte **214 fiches** ; le lot 3 appelle
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
  intercale, `b<n>` la taille, `u` branche précédente, `q` quitte). **Touches
  multimédia** ⏮ ⏭ ⏯ prises en charge via **MPRIS** (D-Bus, module
  `mediakeys.rs`, crate `mpris-server`) — comme `playerctl` ; boucle passée
  en runtime current-thread + LocalSet pour héberger le serveur MPRIS.
  Affichage (04/09/2026) : la **file des morceaux à venir** s'affiche (le
  courant sur la ligne `▶`), les **branches ne s'affichent qu'au dernier
  morceau** du segment, et `p` les prévoit à la demande (la vraie
  « prévisualisation + choix à l'avance » viendra avec l'interface).
  Recherche `/texte` (04/09/2026) : cherche dans le catalogue **et** sur
  l'API Spotify, liste fusionnée `[catalogue]`/`[spotify]` — un artiste du
  catalogue démarre un segment, une piste Spotify se joue et se raccroche à
  la fiche de son artiste s'il en a une (sinon hors catalogue). Choisir une
  branche **ne coupe pas le morceau en cours** : elle est mise en attente et
  démarre à la fin de la piste (`j`/⏭ force tout de suite). **Préchargement**
  (04/09/2026) : la résolution titre → `spotify:track:` du morceau suivant
  (branche en attente ou tête de file) est faite d'avance, en cache, pour
  une transition sans attente API (le préchargement audio librespot reste en
  réserve si besoin). **Configuration** (04/09/2026, `src/config.rs`) :
  `~/.config/forkstify/config.toml` (créé au premier lancement), option
  `[playback] prefer_studio` (défaut vrai) — à la résolution, on récupère
  plusieurs résultats et on écarte les versions live (titre ou album marqués
  live/unplugged/concert), sauf si le titre demandé est lui-même live. Le moteur reste
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
- **Panne d'authentification silencieuse corrigée** (05/09/2026, premier
  test long d'`ecouter` par Joel) : au bout de ~2 h, tous les morceaux
  devenaient « introuvable sur Spotify » et le parcours s'arrêtait. Cause :
  le flux PKCE de Spotify **fait tourner les refresh tokens**, or
  `refresh_if_needed` ne gardait que l'access token — ni en mémoire ni sur
  disque — et `WebApi::new` prenait le refresh de la *réponse*, vide quand
  elle n'en porte pas. Le jeton stocké devenait donc périmé, et
  `resolve()` noyait l'erreur dans un `None` indiscernable d'un morceau
  absent. Trois corrections : (a) `resolve()` rend un **`Resolved`**
  (`Track` / `Absent` / `Failed`) — un échec d'appel n'est plus une absence,
  `search_tracks` de même ; (b) le refresh renouvelé est **conservé et
  réécrit** à chaque rotation (`keep_refresh`), et un refresh mort
  **redemande l'autorisation navigateur** au lieu d'échouer ; (c) le cache
  disque ne mémorise plus que les **vraies** absences — un appel qui n'a pas
  abouti n'y entre pas (une entrée déjà empoisonnée purgée :
  The Limiñanas — « Au début c'était le début »). Côté navigation, une panne
  **arrête le parcours** au lieu de brûler la file : le morceau reste en
  tête, `j` réessaie. **Et « entrée/auto » tire désormais parmi les branches
  affichées** — `auto_advance` recalculait trois nouvelles branches et jouait
  donc ce que Joel n'avait pas vu (arbitrage Joel, 05/09/2026).
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

## Grammaire clavier câblée (05/09/2026)

**Décision [0015](decisions/0015-grammaire-clavier-namespaces.md)** :
quatre namespaces — `f` la branche, `e` encore, `t` le morceau, `a`
l'artiste — la cible se préfixe, un geste fréquent a une touche et un
réglage une commande `:`. Table unique dans
[`docs/keybindings.md`](keybindings.md).

Câblé le jour même (`src/keys.rs`, `src/listen.rs`) :

- **Saisie en mode brut, sans Entrée** (retour n° 1) : termios via `libc`,
  garde RAII qui rend le terminal même sur panique, flèches ← → reconnues,
  `/` et `:` ouvrent une ligne éditable.
- **Grammaire sans préfixe** : aucune commande complète n'est le début
  d'une plus longue, donc tout se déclenche sans délai. C'est ce qui a
  déplacé le modificateur **avant** le compte (`fn3`, `f!3`, `en2`, `e!2`).
  Un test exhaustif sur toutes les séquences de trois touches le vérifie.
- **Les trois variantes** (retours n° 2 et 3) pour les branches et pour
  encore : fin de branche, `n` maintenant, `!` maintenant en retirant ce
  qui suivait. **Corrige au passage** le défaut signalé : choisir une
  branche ne jette plus le reste du segment — c'était la variante `!` qui
  servait de défaut.
- **`fp` peek, `fr` reroll** (retour n° 5), **`fu`** (branche précédente),
  **espace** (la pause au clavier qui manquait), **`h`/`l`** et les flèches.

Pas encore câblé, et le disant à l'écran : `t` et `a` (l'affinage, bloqué
par `learned/` — étape 2 ci-dessous), `fw`, `u`, `.`, `?`, `Q`, `:`.

## La boucle d'apprentissage ouverte (05/09/2026)

`src/learned.rs` implémente [0014](decisions/0014-forme-de-l-appris.md) :
`learned/artists/<slug>.toml` dans le catalogue, un fichier par artiste,
**compteurs à décroissance intégrée** (`plays = plays × ½^((now−last)/6 mois) + 1`),
`learned/marks/inbox.toml` pour les récoltes. Écrit à chaque geste,
silencieux, jamais reversé. `classement.json` (741 artistes) sert de
familiarité de départ.

**Sept mesures câblées** : `tl` aimer, `ts` passer (note et avance), `tb`
bannir le morceau, `tm` récolter, `al`/`as` le poids de l'artiste, `ab`
bannir l'artiste. Plus le **comptage automatique** d'une écoute complète —
seul `EndOfTrack` compte, un saut n'est pas une écoute.

**Trois lectures par le moteur** : exclusion des bannis (artistes et
morceaux), poids de l'artiste appliqué aux branches qui partent de lui, et
`?` qui affiche familiarité et poids. Les bans passent par les canaux
d'exclusion que le moteur a déjà (`visited` par slug, `played` par titre),
donc sans toucher à sa signature.

**Ce qui manque encore côté lecture** : la familiarité ne nourrit pas
encore la zone de confort ([0001](decisions/0001-confort-familiarite.md)),
et le cooldown de [0012](decisions/0012-rotation-des-morceaux.md) n'est pas
appliqué. Côté écriture, les **éditions** (`tt`, `tT`, `td`, `ae`, `aL`)
touchent les fiches et demandent la couche qui écrit et commite le
catalogue.

## Retours d'usage (05/09/2026)

Les premières sessions longues d'`ecouter` ont produit **11 retours** de
Joel, consignés et instruits dans
[`docs/conception/retours-usage.md`](conception/retours-usage.md) — avec
l'**inventaire réel des raccourcis** (ce qui marche vs la table projetée,
largement non implémentée) et **5 points à trancher** avant d'ajouter quoi
que ce soit. Ils se traitent au fur et à mesure.

## Prochaines étapes, dans l'ordre

1. **Éprouver `ecouter`** : le tester en vrai (Joel), ajuster la résolution
   titre → id (versions live/remaster, homonymes) et le rythme des menus.
2. **Fermer la boucle d'apprentissage** (format acté, 0014) : le moteur lit
   `learned/artists/` (familiarité → zone de confort 0001 et seuils de
   l'aventureuse, cooldowns 0012, poids) et les touches d'affinage (0013)
   écrivent dedans (mesures) ou dans les fiches (éditions, commit). C'est le
   cœur — « reprendre la main sur l'algorithme » rendu réel.
3. **Trousseau GNOME** pour les jetons (refresh OAuth, identifiants
   librespot) au lieu des caches `target/` (spike-cache, spike-webapi-refresh).
4. **TUI** (étape 4), à la neomd : touche unique sans Entrée, auto sur
   silence, affichage du morceau en cours.
5. **Traduire en anglais** les scripts d'`outillage/` écrits avant la
   règle de langue du code (à l'occasion).

## Corrections en attente (petites)

- MBID de **Les Thugs** introuvable (homonymie probable) et une entrée au
  nom vide dans `learned/mbid.json` ; 110 MBID résolus « par nom » à relire.
- `resoudre-mbid.py` ne lit que les fichiers Spotify — à adapter aux
  récoltes Deezer (`learned/amis/*-deezer.json`).

## Règles de session

- **Commits et push au fil de l'eau** sur `forkstify` et
  `forkstify-catalog` (demandé par Joel le 31/08/2026). Chorizo : toujours
  demander avant de pousser.
- Les scripts qui lisent le trousseau GNOME sont lancés **par Joel** avec le
  préfixe `!`.
