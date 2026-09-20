# Avancement

Mis à jour le **20/09/2026**. Ce fichier est le point d'entrée pour reprendre
le travail : ce qui est fait, ce qui attend Joel, ce qui vient ensuite.

## Fait

- **Conception** : vision et philosophie (« Reprendre la main sur
  l'algorithme »), vocabulaire, 13 décisions (`docs/decisions/`), 5 notes
  vivantes (`docs/conception/`, 12 aujourd'hui). Rust, TOML, MBID, deux dépôts, format de
  fiche v1 (links typés anglais + proximité en cascade, doors en critère
  additionnel, tout optionnel sauf `format`/`name`/`mbid`). 14 décisions ;
  dernière le 04/09/2026 : **forme de l'appris** — dossier `learned/`, un
  fichier par artiste, compteurs décrus (demi-vie 6 mois) (0014). Le
  vocabulaire sur disque (chemins, champs) est en anglais comme le code.
- **Catalogue amorcé** (`~/Work/forkstify-catalog`, GitHub privé) :
  `catalog.toml` (grille type → proximité), **30 fiches** écrites
  (`generated = true`, le haut du classement de Joel), **tools/** —
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
  `tools/generate-cards.py`) et **lot 2 généré : 58 fiches** — les
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
- **Vecteurs prototypés** (02/09/2026) : `tools/vectoriser.py`
  compose le texte de chaque fiche depuis sa structure (tags, dates,
  origine, liens sortants et entrants, description si présente) et
  calcule les vecteurs dans un conteneur — modèle
  `paraphrase-multilingual-MiniLM-L12-v2` (fastembed, 384 dimensions,
  mean pooling, dispo en Python et en Rust). Index dérivé commité :
  `vectors/vectors.jsonl` (214 fiches) + `meta.toml`.
  `tools/voisins.py` (stdlib) = prototype de `forkstify check` :
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
- **Dépôts** (déplacés le 08/09/2026) : `aropixel/forkstify` et
  `aropixel/forkstify-catalog`, la référence, privés, branche `main` ;
  `kbyjoel/forkstify-catalog` est le **fork** de Joel, celui que
  l'application lit (`origin`), l'amont en `upstream`. Plan de reprise
  chorizo à jour.

## En attente de Joel

- **Pas de relecture fiche à fiche** (décision de Joel, 02/09/2026) : il a
  regardé l'ensemble, l'affinage se fera **à l'utilisation** (keybinds,
  décision 0013). Les points connus restent notés pour mémoire : MBID
  incertains (28 du lot 2, 14 du lot 3 — rapports du générateur), tops
  vides (cabadzi, le-motel, la-ruda-salska), doublons de versions chez
  J.P. Nataf, un top russe parasite chez Expérience.
- Identifiants Deezer/Spotify d'**amis consentants** pour élargir la base
  (`tools/amis-*.py`).

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

## La longue traîne, quatrième source (05/09/2026)

`src/discography.rs` + `WebApi::discography()`. Le réservoir de 0012 §1 est
complet : tops, aimés, doors, **et le reste de la discographie**.

Source **Spotify**, par le champ `spotify` des fiches — présent depuis
toujours, jamais lu jusqu'ici. Il évite le `search` par artiste qu'imposerait
Deezer (aucune fiche ne porte d'identifiant Deezer) et rend des
`spotify:track:` directement, donc un morceau de traîne ne peut jamais
devenir un « introuvable sur Spotify ».

Cache dans `~/.cache/forkstify/discography/<slug>.json` — hors du dépôt,
comme 0012 et `catalogue.md` le demandent : régénérable, jamais commité, non
synchronisé. Récolté **au moment du besoin** (quand `e<n>` demande plus que
la fiche n'a) ou par `:warm`. Pas de péremption.

**Le curseur de confort a enfin son troisième levier** : la part de la
traîne *est* l'ouverture du confort (0012 §4) — zéro au cocon, pleine à
l'exploration.

Déduplication par titre normalisé : Spotify livre la même chanson sous dix
habillages, et un titre déjà top n'entre pas dans la traîne. Marque `·`.

**Non vérifié en réseau** : la récolte demande une session Spotify, donc
`:warm` n'a jamais tourné en vrai. Le reste est couvert par 14 tests.

## La zone de confort branchée (05/09/2026)

`engine::Comfort` implémente [0001](decisions/0001-confort-familiarite.md) :
0 = cocon, 5 = exploration, lu dans `[journey] comfort` du fichier de
config et réglable en écoute par `:comfort <n>` (avec un mot à côté du
chiffre : cocon, prudent, équilibré, curieux, aventureux, exploration).

Deux leviers, ceux qu'`avancement.md` désignait déjà comme « constantes à
piloter par le confort » : le **plancher de la branche aventureuse**
(cosinus ≥ 0.80 au cocon, ≥ 0.60 ouvert — **le confort 2 reproduit
exactement l'ancien 0.72 / 0.80**) et la **familiarité qui penche le tirage
des têtes**, bornée à [0.25, 2.0] : on décourage, on n'interdit pas.

**Un piège de polarité consigné** dans
[`zone-de-confort.md`](conception/zone-de-confort.md) et figé par un test :
le « confort haut » de 0012 §4 désigne le *sentiment* de confort, donc la
valeur **0**, pas 5. Lu à la lettre, tout le curseur s'inverse.

Question ouverte de 0001 **tranchée de fait** : ce que l'application sait
de ce qu'on connaît, c'est `learned/` — nos écoutes décrues, saturantes,
et à défaut `classement.json` ramené sur la même échelle par son maximum.

**Il manque au curseur son troisième levier** : la profondeur du tirage
dans le réservoir (0012 §4), qui n'aura rien à régler tant que la longue
traîne n'existe pas — voir [`longue-traine.md`](conception/longue-traine.md).

## Le réservoir ouvert, les doors réveillées (05/09/2026)

Question de Joel — « est-ce qu'on a prévu que des morceaux soient joués
sans être top ? » — qui a mis au jour un écart : **[0012](decisions/0012-rotation-des-morceaux.md) §1
prévoyait quatre sources, le moteur n'en tirait qu'une**. Pire, le champ
`doors` était écrit dans **13 fiches** depuis le 02/09 et le mot n'apparaissait
nulle part dans `src/` : [0011](decisions/0011-doors-critere-additionnel.md)
dormait.

`engine::reservoir()` cumule désormais trois des quatre sources, chacune
avec son poids : les **tops** (1.0), les **titres aimés** de `learned/`
(0.8), les **doors** (0.4, ×2.5 quand la direction de la branche recoupe
leurs tags — le bonus de 0011). Un morceau souvent passé recule
(`poids ÷ (1 + skipped)`), un banni sort. Le tirage est pondéré, sans
remise dans un parcours.

**Chaque morceau affiché porte sa provenance** : `♪` top · `♥` aimé ·
`↳` door · `+` hors tops · `~` hors catalogue. Vérifié à sec — depuis
Joy Division ou The Fall, `↳ A Forest — The Cure` apparaît.

**La quatrième source manque** : la longue traîne de la discographie
(cache API Deezer/Spotify), qui demande une couche de cache inexistante.
Le cooldown daté de 0012 §2 n'est pas appliqué non plus, ni la zone de
confort de 0001 qui doit régler la profondeur du tirage.

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

## README, LICENSE, et un historique sans note personnelle (20/09/2026)

Joel veut passer l'application en public. Vérifié avant : aucun jeton,
aucun mot de passe, aucune adresse dans les 188 commits ; l'historique
est gardé tel quel — les décisions datées et les commits qui y renvoient
sont la mémoire du projet, un squash dirait le contraire de ce que le
dépôt est. Fait :

- **`README.md`** en anglais (0022) : ce que c'est, ce qu'il faut
  (Linux, Premium, git, docker), l'installation (plugin Omarchy ou
  `bin/build`), le premier lancement (les sept étapes), la fiche, la
  configuration, où sont les docs. **`LICENSE`** : MIT, comme le
  manifeste l'annonçait.
- **Une note de travail personnelle retirée de l'historique** (Joel,
  20/09/2026), par `git filter-branch`, et `main` poussé de force. Elle
  vit désormais hors de tout dépôt, dans `~/Work/forkstify-private/`.
  **L'autre poste doit refaire son clone**, ou `git fetch && git reset
  --hard origin/main` — son historique local ne correspond plus.
- Non fait, jugé non nécessaire : les vieux scripts `amis-*.py` restent
  dans l'historique des deux dépôts du catalogue (Joel : « pas grave »),
  comme les huit fichiers d'appris de la référence ; à reconsidérer
  ensemble le jour où la référence passe en public.

## L'action de la référence et son CONTRIBUTING (20/09/2026)

Joel : « mets en place l'action GitHub et le CONTRIBUTING.md sur la
référence ». Les trois pièces de « La relecture côté référence » sont là :

- **`forkstify validate [catalog]`** (`src/validate.rs`) — chaque fiche
  lue en TOML brut et en `Card` : `format = 1`, un `name`, un `mbid`
  **unique dans tout le catalogue** (une même personne sous deux fichiers
  est refusée), un nom de fichier en slug, des `links` dont la cible est
  un slug (une cible sans fiche est une proposition, 0016) et le type dans
  la liste fermée de 0010 ou déclaré dans `catalog.toml`. Un nom qui ne
  correspond plus à son fichier (`Ye` à `kanye-west`) est un
  avertissement, pas une erreur. Sortie 1 sur une erreur. Les deux
  catalogues passent : 0 erreur, 18 avertissements de noms.
- **`.github/workflows/catalog.yml`** sur `aropixel/forkstify-catalog` :
  `check` sur chaque PR — seules les fiches peuvent changer, puis
  `forkstify validate .` ; `index` sur chaque push de `main` qui touche
  les fiches — `forkstify vectors .` et l'index commité par le bot.
  Une action composite `.github/actions/forkstify` construit le binaire
  depuis `aropixel/forkstify` (cache cargo, alsa) ; le modèle
  d'embedding est mis en cache entre les runs.
- **`CONTRIBUTING.md`** (anglais, 0022) : comment une proposition se fait
  (`Cp`), ce que l'action vérifie, comment elle est lue — générées : un
  coup d'œil ; retouches : les faits se prennent, un `similar` avec sa
  note, un top s'il corrige une erreur. Le README pointe dessus.

~~**À faire par Joel** : le secret `FORKSTIFY_TOKEN`~~ — sans objet :
**`aropixel/forkstify` est public depuis le 20/09/2026** (Joel), le jeton
du workflow suffit à l'action pour cloner l'application. Le README
installe par `https://` désormais.

## Le catalogue sans outillage, la référence sans appris (20/09/2026)

Joel : « retire les scripts Python de `tools/` du catalogue et nettoie
l'appris de la référence ». Fait sur les deux dépôts :

- **`kbyjoel/forkstify-catalog`** (le fork, celui que l'application lit) :
  `tools/` retiré — les dix scripts ont chacun leur remplaçant dans
  forkstify (setup et `:library`, génération à la volée, `forkstify
  vectors`, `forkstify check`) ; `amis-*.py`, sans équivalent, partent
  avec. `learned/` **reste** : c'est l'appris de Joel, et
  `classement.json` est lu tant que `library.toml` n'existe pas.
- **`aropixel/forkstify-catalog`** (la référence) : `tools/` et
  **`learned/` entier** retirés — un fork neuf n'hérite plus de la
  bibliothèque de Joel. Il ne reste que `cards/`, `vectors/`,
  `catalog.toml`, `.gitattributes` (le pilote de fusion) et le README.
- Le premier `Cu` du fork verra `learned/` modifié d'un côté, supprimé de
  l'autre : la fusion garde les siens sans rien demander (fork.rs).

## Le namespace `C` — diff, propose, update (20/09/2026)

Le chantier B de [`conception/sortie.md`](conception/sortie.md), codé
d'après `Catalogue.dc.html` (Claude Design, sept écrans). Tout ce qui
avait été tranché les 19 et 20/09 est câblé, dans `src/fork.rs` :

- **`:catalog`** — l'état en une ligne (origin, avance/retard, dernière
  mise à jour, fiches au-delà de la référence), en mode local le dit.
- **`Cd`** — l'overlay de `:mine`, renommé : le compte et la date, les
  fiches nouvelles (générées / écrites, tags, liens) puis les retouchées
  (+n −m, sections touchées lues dans le diff — tops, tags, description,
  types de liens —, note de provenance).
- **`Cp`** — fetch, worktree `~/.local/state/forkstify/proposal` sur une
  branche `proposal` depuis `upstream/main`, l'état de `cards/` copié
  depuis `main`, un commit `Propose N cards (a generated, b edited)` dont
  le corps est en deux listes pour le relecteur, `push --force`. Avec `gh`
  connecté : l'overlay montre la PR et **`y`** l'ouvre (`gh pr create`),
  toute autre touche n'envoie rien ; une PR déjà ouverte est mise à jour
  par le push. Sans `gh` : la page de comparaison GitHub, titre et corps
  dans l'URL.
- **`Cu`** — l'appris commité, fetch, **merge** `upstream/main` ; conflits
  réglés seuls sur `vectors/` (amont, régénéré), `learned/` (à soi),
  `tools/` (amont) ; une fiche modifiée des deux côtés **arrête** : la
  fusion reste en cours, l'overlay nomme les fiches et ce que chaque côté
  a changé, `o` ouvre la première, `:catalog` reprend après `git add`
  (et `git commit`, ou pas). Puis l'index est régénéré si des fiches ont
  changé, la date mémorisée, et **la session recharge son catalogue**.
- **`:catalog fork <url>`** — sortie du mode local.
- Les gestes tournent en `spawn_blocking`, un à la fois, la lecture
  continue. Écarts : `:catalog` en overlay, pas dans le flux ; `Cd` ne
  déroule pas ; `o` passe par `xdg-open`.
- 86 tests verts, dont un **test d'intégration sur trois dépôts git
  temporaires** (référence, fork, clone) qui enchaîne diff, propose deux
  fois, l'update qui fusionne, l'update qui s'arrête sur une fiche et la
  reprise — `git` est entré dans l'image `forkstify-build` pour cela
  (`Dockerfile`, image à reconstruire : `docker build -t forkstify-build .`).
  **Non éprouvé en vrai** : `Cp` jusqu'à `gh pr create`, `Cu` sur le fork
  de Joel.

## Le setup après l'installation, d'après la maquette (20/09/2026)

Le chantier A de [`conception/sortie.md`](conception/sortie.md), codé le
jour même d'après `Installation.dc.html` (Claude Design, projet « Accueil
Forkstify » — neuf écrans, importés par l'agent). La maquette a tranché
les quatre points ouverts : mode local gardé, un seul `library.toml`,
seuil ≥ 5 et 30 fiches au plus, les deux scopes acceptés.

- **Le premier lancement** : `forkstify` sans catalogue lisible ouvre le
  setup au lieu d'échouer (`main::accueil`). L'écran 0 liste les sept
  étapes, `⏎` commence.
- **`src/setup.rs`** — les sept étapes et les deux écrans de sortie.
  1 le catalogue : l'URL d'un fork collée, `gh repo fork` + `gh repo
  clone` si `gh` est là et connecté, ou la référence clonée en **mode
  local** (`git config forkstify.local`, la synchro commite sans
  pousser, l'accueil dit `⇅ local`) ; le clone va dans
  `~/.local/share/forkstify/catalog` (XDG), `upstream` ajouté,
  `[catalog] path` écrit dans `config.toml` (`config::set_catalog_path`,
  textuel, les commentaires restent). 2 l'identité git, seulement si la
  config globale n'en a pas, écrite `--local`. 3 la connexion : le
  téléphone (zeroconf en fond, échap saute) et le navigateur (`o`) ; **sept
  scopes** désormais (`user-follow-read`, `playlist-read-private`), et un
  jeton accordé avec les cinq d'avant **repasse une fois par le
  navigateur** — `spotify::needs_reauthorization`, les scopes mémorisés
  dans l'état, l'écran dit que ce n'est pas une panne. 4 la bibliothèque :
  `/me/tracks`, `/me/albums`, `/me/following`, une barre par source,
  l'artiste principal seul. 5 les playlists : les siennes d'abord, espace
  coche, `/` filtre, `⏎` récolte les cochées, mémorisées. 6 le confort à
  la jauge. 7 la couverture : le classement croisé au catalogue par
  tranches (≥ 20, 10–19, 5–9), `o` génère les fiches manquantes de score
  ≥ 5 (30 au plus) — une par une en fond, vecteurs à la fin, **un seul
  commit** `library: N cards generated`. L'écran 8 récapitule en toasts,
  l'écran 9 (`:setup`) liste les étapes cochées et rejoue l'une ;
  `:library` rejoue 4, 5, 7.
- **`src/library.rs`** — `learned/library.toml`, un fichier en anglais
  (`name`, `spotify`, `liked_tracks`, `liked_albums`, `followed`,
  `playlist_tracks`, `score`, `sources`, la date et les playlists
  cochées), la formule de `classement.py` inchangée ; `learned.rs` le lit
  d'abord, `classement.json` tant qu'il n'y a pas de `library.toml`.
- **`keys::parse_setup`** (chiffres, `j`/`k`, `o`, espace, `⏎`, échap) et
  **`tui::render_setup`** (le pas et sa jauge à droite de l'en-tête, les
  lignes : choix, champ, case à cocher, barre, étape, clé/valeur, noms).
  Un champ ouvert après un autre ne garde plus la ligne du précédent
  (génération du mode texte — bug vu au test de fumée).
- **Écarts assumés** : la génération (7) et la récolte (4) se regardent,
  échap arrête ; `:setup` et `:library` ferment la session (le son
  s'arrête) et l'accueil revient sur le catalogue rejoué. Les scripts de
  `tools/` restent dans le catalogue jusqu'à ce que `:library` ait tourné
  sur le fork de Joel.
- 81 tests verts, aucun avertissement ; test de fumée du fil complet
  (clone par URL, identité, confort, récapitulatif) sur des dossiers XDG
  temporaires. **Non éprouvé en session réelle** : la récolte, les
  playlists, la génération de couverture, `gh`.

**Au prochain lancement de Joel** : le navigateur s'ouvre une fois pour
les deux scopes de plus, puis `:library` depuis l'accueil récolte sa
bibliothèque et remplace `classement.json`.

## `fg<n>` — générer un creux sans le prendre, et la branche d'un creux marche (20/09/2026)

Le chantier C de [`conception/sortie.md`](conception/sortie.md), codé le
jour de l'arbitrage. Joel : les artistes sans fiche proposés en creux
obligeaient à les mettre à la file pour obtenir la fiche ; il veut « les
générer, et que cela propose de nouvelles branches en fonction », et « une
branche régénérée comme les autres, avec un morceau de l'artiste généré et
d'autres morceaux d'autres artistes ».

- **`fg<n>`** (`Cmd::ForkGenerate`, `After::Gap`) : la fiche du creux n
  naît — composée, vectorisée, commitée, adoptée par la session comme
  aujourd'hui — et **rien n'est mis à la file**. À l'arrivée, la ligne
  « ○ no card yet » devient une branche jouable à la suite des branches
  affichées, les autres ne bougent pas (la raison pour laquelle `⏎` tire
  parmi ce qui est affiché), et le toast dit son numéro. Sur un numéro de
  branche, `fg<n>` répond « f<n> takes it ».
- **Les creux se rafraîchissent** autour du contexte *et* de la fiche
  fraîche (`refresh_gaps`) : ses propres liens vers le vide apparaissent en
  gris à leur tour — le catalogue grandit le long de ses liens, un cran
  plus loin. Le prochain recalcul repart de la liste telle qu'elle est,
  comme `fr`.
- **La branche d'un creux est une marche** (`engine::branch_from`, le
  `walk` de `propose` et de `wander`, la fiche fraîche en tête, la raison
  et la proximité du lien pour raison et poids). Correction au passage :
  `branch_to` — le chemin du `f<n>` sur un creux — construisait la branche
  avec `encore`, donc n morceaux du seul artiste généré. `f<n>` et `fg<n>`
  passent tous deux par la marche.
- Tests : `fg2` se lit, la grammaire reste sans préfixe ; la branche d'une
  tête fraîche traverse plus d'un artiste, une tête inconnue ne donne rien.
  72 tests verts, aucun avertissement. **Non éprouvé en session réelle.**

## Le cahier des trois derniers chantiers avant la sortie (19/09/2026)

Joel : « affiner les dernières choses avant de pouvoir sortir le projet » —
le setup après l'installation, un namespace « catalogue » (`:mine` renommé
en diff, une PR des nouvelles fiches vers la référence, une mise à jour du
fork depuis la référence), et générer un creux sans le prendre (`fg<n>`),
les branches se proposant ensuite avec la fiche fraîche. Le cahier est
[`conception/sortie.md`](conception/sortie.md) : pour chaque chantier, ce
qui est demandé, ce qui est proposé, ce qui reste à trancher ; l'ordre
proposé (C, puis B, puis A) ; et ce que la sortie demande en plus.

Propositions à valider, en bref : le setup en **sept étapes** (catalogue
forké ou mode local, identité git, connexion, bibliothèque, playlists à
cocher, confort, couverture par génération), `learned/library.toml` en
anglais à la place de `classement.json`, deux scopes OAuth de plus ; le
namespace **`C`** pour le catalogue (`Cd` diff, `Cp` propose, `Cu`
update — la lettre tranchée par Joel le jour même, après avoir écarté un
`c` partagé avec le confort), une PR qui porte **l'état des
fiches** sur une branche `proposal` depuis `upstream/main` (jamais
l'appris, jamais les vecteurs) et s'ouvre dans le navigateur, une
**fusion** plutôt qu'un rebase parce que `main` est partagé par deux
postes ; `fg<n>` qui fait du creux **une branche à son numéro** sans
rejouer les autres, puis rafraîchit les creux. Tranché le jour même : `C`, la fusion pour
`Cu`, et pour `Cp` la PR par `gh` avec confirmation quand il est là et
connecté, le navigateur sinon. Le 20/09/2026, contre la crainte de
validations laborieuses : sur le fork de Joel, 46 fiches nouvelles toutes
générées pour 3 retouchées — d'où une **action GitHub** sur la référence
(vérifications, index régénéré à la fusion), une PR **composée pour le
relecteur** en deux listes, et une **règle de fusion** écrite dans un
`CONTRIBUTING.md` (générées : un coup d'œil ; retouches : les faits se
prennent, un `similar` avec sa note, un top s'il corrige une erreur).
Puis une seule branche `proposal` réécrite, et le nom `Cp` gardé : **le
chantier B n'a plus rien à trancher**. Chantier C, le 20/09/2026 : la branche née d'un creux est une **marche**
comme les autres (`engine::walk`, la fiche fraîche en tête, d'autres
artistes ensuite), pas un encore de l'artiste généré — ce qui corrige au
passage le `f<n>` actuel sur un creux. Relevé au passage : la
référence porte encore l'appris de Joel, à retirer avant qu'elle soit
publique.

## Les indices du moteur se règlent dans `config.toml` (17/09/2026)

Joel : « pousser la logique du *reprendre la main sur l'algorithme*
jusqu'au bout et donner la possibilité à la personne qui aura installé
forkstify de changer les valeurs de tous les indices (comme
`ARTIST_COOLDOWN_HALF_LIFE`, `LESS_OFTEN`, `WEIGHT_FLOOR`…) via le fichier
de configuration ». **Décision
[0023](decisions/0023-les-indices-du-moteur-se-reglent.md).**

- **Section `[tuning]`** dans `~/.config/forkstify/config.toml`, vingt
  réglages nommés, tous à leur ancienne valeur par défaut : les cooldowns
  (`track_cooldown_floor` / `_half_life`, `artist_cooldown_floor` /
  `_half_life`), le goût (`less_often`, `more_often` — `0` = le miroir de
  `less_often` —, `weight_floor`, `weight_ceiling`), la familiarité
  (`plays_reference`, `plays_half_life`), le réservoir (`top_weight`,
  `liked_weight_cocoon` / `_open`, `door_weight`, `door_bonus`,
  `tail_weight`) et le saut aventureux (`leap_floor_cocoon` / `_open`,
  `leap_trust_cocoon` / `_open`). Le gabarit du premier lancement les
  liste tous, commentés.
- `config::Tuning` + `config::tuning()` (chargé une fois, au lancement) ;
  `engine.rs` et `learned.rs` n'ont plus de constante numérique. Une
  valeur absurde est dite sur stderr et remise à son défaut, seule.
- Hors champ, à dessein : les délais de l'interface et les formes du
  tirage (`take(6)`, puissances). Voir la décision.
- Deux tests de config (parsing, garde-fous). 71 tests.

**Sur un poste existant** : le fichier de configuration n'est pas réécrit ;
sans section `[tuning]`, tout est au défaut. Copier la section depuis le
gabarit (`src/config.rs`, `TEMPLATE`) pour l'avoir sous la main.

## `fr` repart de la liste telle qu'elle est (17/09/2026)

Joel : « Quand je fais un `fr`, les chansons ajoutées dans la playlist via
un `ti` ou via un `e` depuis une discographie ne sont pas prises en compte.
Il faudrait qu'il prenne en compte l'état actuel complet de la playlist.
Pareil pour les titres ajoutés avec `fw`. »

Cause : l'état du moteur (`state()`) ne lisait que le registre des rounds
— branches prises, encores `e<n>`, `fw`. Un `ti`, un `e` dans la
discographie ou un `J`/`K` touchent la file sans écrire de round : `fr`
repartait du dernier round, et pouvait reproposer les titres déjà en file.

- **La liste fait foi.** `state()` fonde maintenant l'état sur l'axe (passé,
  en cours, à venir) : le **contexte** est le dernier segment de la liste
  (`engine::segment_of` — depuis la dernière tête de segment, branche ou
  « inséré (ti) », jusqu'au bout ; toute la liste s'il n'y a pas de tête,
  cas de la graine), ses artistes rejoignent l'**univers** et les visités,
  et **tout titre de l'axe compte comme joué**. Les rounds restent le repli
  quand l'axe n'a rien de connu (morceau hors catalogue), et le registre
  que `fu` dépile.
- Conséquence assumée : après `fn<n>` (branche insérée après le morceau en
  cours, le reste gardé), les directions partent de ce qui **termine** la
  file, pas de la branche insérée — c'est ce que dit la liste.
- `fu` vide la file **avant** de relire l'état, et prend l'artiste d'avant
  dans le registre, pas dans la liste.
- Test `the_last_segment_of_the_playlist` (69 tests).

## Fraîcheur d'artiste et traîne modulée par la familiarité (14/09/2026)

Le retour du petit cercle (retours-usage n° 13), câblé.

- **Fraîcheur au niveau de l'artiste** (`Learned::artist_freshness`,
  demi-vie 4 jours, plancher 0,3) : un artiste entendu récemment recule
  comme tête de branche, et récupère sur quelques jours. Appliquée aux têtes
  du graphe, à l'aventureuse et à la branche `stay`. Casse la boucle de
  renforcement qui resserrait le cercle.
- **Traîne modulée par la familiarité de l'artiste** : dans `reservoir`,
  `share = tail_share() × familiarité`. Un artiste nouveau est mené par ses
  tops, un artiste connu ouvre sa longue traîne. Baisser le confort élargit
  les artistes sans faire entrer des fonds de tiroir au hasard.
- Le test de traîne est réécrit (familier ⇒ traîne, nouveau ⇒ tops), un
  test de `artist_freshness` ajouté. 68 tests.
- **La donnée est déjà synchronisée** : `plays`/`last` vivent dans
  `learned/`, versionné dans le fork et synchronisé par 0017. La fraîcheur
  voyage entre postes sans fichier nouveau (question de Joel, 14/09/2026).

## `:warm` sur une traîne vide, et une fiche à l'identifiant faux (14/09/2026)

Joel : impossible de récupérer la discographie de Jarvis Cocker. Deux
causes.

- **La fiche pointait vers le mauvais Spotify.** `2kTHIUipN0SYKBbmcTCLfQ`
  est « Jarvis Branson Cocker », un homonyme quasi vide (2 176 auditeurs,
  sans discographie propre) ; le vrai est `13W7XLRXdWeLmIu9vacE1w` (profil
  vérifié). Vérifié sur open.spotify.com, corrigé dans le catalogue.
  L'API répondait donc, mais avec zéro album.
- **Une traîne vide se mettait en cache et bloquait la reprise.** `:warm`
  voyait `[]` en cache et disait « déjà en cache, 0 morceau » sans jamais
  réessayer. `:warm` **oublie maintenant le cache avant de récolter**, donc
  il refait toujours l'appel ; et une récolte à zéro morceau le dit
  clairement (« son identifiant Spotify est peut-être faux ») au lieu de
  passer pour un succès. Le cache vide sur disque a été supprimé.

## `:warm` récolte l'artiste sous l'aiguille (14/09/2026)
## `:warm` récolte l'artiste sous l'aiguille (14/09/2026)

Joel : `:warm` prenait le dernier artiste du contexte (`state().1`), pas ce
qui joue ni la ligne surlignée. Il utilise maintenant la même cible que
`e`/`t`/`a` (0020) : `target()` — la sélection, sinon le morceau en cours —
avec repli sur le contexte si la cible est hors catalogue.

## Un peu de couleur sur l'encart d'infos du morceau (14/09/2026)

Joel : « mets un peu de couleur sur la fenêtre d'information du morceau ».
`info_line` dans `tui.rs` teinte les libellés de l'encart `ta` (featuring,
album, tags, from here, off-catalog) en cyan gras, éclaircit leur valeur, et
passe la ligne de métriques (familiarité, poids, liens) en jaune avec des
séparateurs discrets. Les lignes qui ne sont pas « libellé : valeur » — les
rangs du menu d'aide, qui partagent le même encart — restent grises, comme
avant. La pochette dans `ta` attend toujours le protocole graphique du
terminal.

## Reprendre reprend tout le parcours, et plus rien sur stdout (14/09/2026)
## Reprendre reprend tout le parcours, et plus rien sur stdout (14/09/2026)

Joel, deux points.

- **Le « recommencer » (`r`) reprend tout le parcours en cours** :
  historique, morceau en cours et morceaux à venir, au lieu de repartir du
  seul dernier morceau. `LastSession` porte maintenant `past`, `current`,
  `queue` et `rounds` (Stop/Source/Head/Round dérivent serde) ; `remember`
  les enregistre à la sortie, `restore` les replace et rejoue le morceau qui
  sonnait. Un `last.json` antérieur, sans ces champs, retombe sur l'ancien
  comportement (partir du dernier morceau).
- **Plus de parcours écrit sur stdout à la sortie.** `run` ne renvoie plus
  la liste des artistes et les deux appelants n'impriment plus « Journey:
  … » : l'écran alterné est rendu, rien ne s'affiche derrière.

## Le confort est retenu d'un lancement à l'autre (14/09/2026)

Joel : « je voudrais que forkstify se souvienne de la dernière zone de
confort choisie ». Un fichier `~/.local/state/forkstify/comfort`, écrit à
chaque changement (`config::remember_comfort` sur `c<n>`, `:comfort`, `cc`
validé, accueil et écoute) et relu au démarrage (`config::comfort_at_start`,
appelé partout où l'ancien code lisait `config.journey.comfort`). L'état
prime sur `config.toml`, qui devient la graine du premier lancement.

## Quatre retours d'usage : goût, infos, jauge, album (14/09/2026)

Joel, après quelques jours.

- **`tl` bascule aimé / non-aimé.** Sur un morceau déjà aimé, `tl` retire
  l'aimé **sans pénalité** — `ts` restait « moins souvent + retire l'aimé »,
  il manquait le simple retrait. `learned.track_liked` / `unlike_track` ;
  vaut à l'écoute et dans la discographie.
- **`ta` — track about**, nouvelle touche (mot anglais, pas « fiche ») :
  album, featuring, année quand la discographie les a, plus tags,
  familiarité, poids et la première branche. **Remplace `?`/why**, dont la
  raison est repliée dedans. Featuring lu du titre (`feat.`/`ft.`/`with`),
  album et année du cache de discographie, sans appel réseau.
- **La jauge de confort** est désormais **en haut à droite sur les deux
  écrans, apparence d'édition permanente** (`comfort_spans`), `cc` ajoutant
  « ↑↓ ». Le **statut de l'accueil ne s'affiche que dégradé** (Joel doutait
  de son utilité en continu) : rouge, deuxième ligne, sinon rien.
- **⏎ sur une ligne d'album** de la discographie **écoute l'album entier** :
  ses morceaux ouvrent une nouvelle graine dans l'ordre, puis les branches
  partent de l'artiste (`start_album`). Sur un morceau, ⏎ garde le sens du
  11/09 (partir de lui).

La pochette dans `ta` reste pour plus tard : elle demande le protocole
graphique du terminal (sixel/kitty), un chantier à part.

## `aL` lie à un artiste choisi par la recherche (14/09/2026)

Joel : « je ne comprends pas le geste à faire pour lier un artiste à un
autre… j'écoutais King Hannah et je voulais le lier à Peter Kernel ».
`aL` liait à l'artiste **d'où l'on venait**, une cible implicite qu'aucun
écran n'annonçait, et qui ne pouvait pas atteindre un artiste hors du
parcours.

`aL` ouvre désormais la modale de recherche, en-tête « link <artiste> » ;
entrée sur une ligne écrit un lien `similar` dans la fiche courante vers
l'artiste choisi. Cible catalogue (`Hit::Artist` ou un titre avec fiche) ou
hors catalogue (nom slugifié — un lien vers une fiche absente est une
proposition, 0016). Un `link_from` porté par le `Finder`, résolu en tête de
`take_found` ; l'édition se commite comme avant (compte au prochain
lancement). Marche à l'accueil comme en écoute. Un artiste sans fiche
courant ne peut pas être lié : le toast renvoie vers `:generate`.

## `:generate <nom> <mbid>` propose au lieu d'écraser (11/09/2026)

Joel : avec un mbid, `:generate` lançait une nouvelle liste et écrasait la
lecture en cours. Désormais, s'il y a une liste en cours (un morceau joue
ou la file n'est pas vide) et qu'un mbid est donné, la fiche est faite mais
**le parcours ne démarre pas** : un toast de succès qui dure plus longtemps
(`SEED_OFFER_SECONDS`, 12 s) propose de partir de l'artiste — **⏎** accepte
et remplace la liste, toute autre touche garde la liste et fait son office.
Nouvelle branche `After::Offer`, une graine en attente `pending_seed`
consommée par entrée dans `on_cmd`. Sans mbid, à l'accueil ou sans rien qui
joue, le comportement d'avant ne bouge pas. Le toast porte une durée propre
maintenant (`linger`).

## Les branches ne défilent plus dans le vide, et la card rouvre (11/09/2026)

Deux retours de Joel après une vraie session.

- **Les branches se choisissaient à l'infini sans jouer.** Depuis que la
  traîne est récoltée pour les artistes proposés, une branche peut tirer un
  morceau de traîne (au confort 3, la traîne pèse). Un morceau indisponible
  dans la région finit à l'instant où il commence : la branche s'épuise,
  `auto_advance` en tire une autre, sans un son — et rien ne l'arrêtait.
  Garde-fou : `MAX_DRY_ADVANCES` (4) compte les branches enchaînées sans
  qu'un morceau atteigne les enceintes ; passé ce seuil, la lecture
  s'arrête et rend la main (« tracks may be unavailable in your region »).
  Le compteur retombe à zéro dès qu'un `Playing` arrive.
- **La card Omarchy ne rouvrait pas forkstify éteint.** Le bouton « Show
  forkstify » pointait sur le mot nu `forkstify`, qui attrapait aussi un
  terminal resté dans `~/Work/forkstify` : le focus partait sur ce shell au
  lieu de lancer. Retour à `omarchy-launch-or-focus-tui forkstify`, sur
  l'app-id `org.omarchy.forkstify` — précis, comme avant l'ajout du bouton.

## Entrée dans la discographie part du morceau (11/09/2026)

Joel : « je veux pouvoir démarrer une nouvelle graine depuis une chanson
de l'écran de discographie (avec entrée ?) ». Entrée gardait le sens du
08/09, écrire la fournée. Les deux se cumulent : entrée écrit la fournée
s'il y en a une (un commit), puis, **sur un morceau, part de lui** —
`start_journey` sur `Choice::Track`, comme la recherche. Sur un album,
entrée écrit seulement ; un banni ne part pas. Le handler de la modale est
synchrone, la graine part par `start_requested` après lui, comme `:search`
et `:wander`. Détail dans
[exploration-d-un-artiste.md](conception/exploration-d-un-artiste.md).

## `fw` — partir loin, ou chez quelqu'un (11/09/2026)

Joel : « implémentons fw. Je veux aussi pouvoir faire `fw <nom de
l'artiste>` pour cibler un univers particulier ». Le retour n° 6 est
tranché dans sa lecture (a), sortir de l'univers, avec une cible en
option.

- **`engine::wander`** : sans cible, la tête est tirée parmi les artistes
  les plus loin du centre du parcours — hors du parcours et de tout ce qui
  en est à un lien, sous le plancher du confort (ce que la branche
  aventureuse refuse), avec des tops non joués ; le plus loin pèse le
  plus, le curseur penche comme pour toute tête. Avec une cible, la tête
  est cet artiste, quelle que soit la distance. Puis la même marche qu'une
  branche. Un test fige les deux cas.
- **La touche** : `fw` ouvre la ligne `:wander ` déjà remplie
  (`read_line` prend un début de ligne) ; entrée seule part loin, un nom
  part chez lui. `:wander [artiste]` est la commande épelée. Le nom se
  résout par `search_names` sur le catalogue ; absent, le toast renvoie
  vers `:generate`.
- **Où** : la branche va en fin de ce qui est décidé, comme `f<n>`. À
  trancher à l'usage : si « partir sur complètement autre chose » veut dire
  maintenant, `fn`/`f!` ont le geste, `fw` pourrait le prendre.


Joel, avant de relancer : « la page d'accueil évoque un random avec enter,
c'est faux » et « cc + flèches ne marche pas ».

- **Entrée à l'accueil** prenait la première porte de la page, alors que
  l'écran et [ecran-d-accueil.md](conception/ecran-d-accueil.md) promettent
  « au hasard — tirage pondéré, la porte qui ne demande pas de choisir ».
  C'est maintenant vrai : un tirage sur toutes les portes de la page,
  pondéré par ce que le curseur fait de la familiarité de l'artiste
  (`Comfort::favours`, le même qu'au moteur) — le familier au cocon,
  l'inconnu grand ouvert, jamais un poids nul. Une ligne surlignée garde
  la priorité.
- **`cc` à l'accueil** ne faisait rien : l'accueil prenait la touche avant
  que le cadran ne soit consulté, et ne connaissait pas `ComfortMode`. Le
  cadran passe devant l'accueil dans `on_cmd`, `cc` l'ouvre depuis
  l'accueil, la jauge s'allume comme à l'écoute, et la validation ne
  recalcule les branches que s'il y a une session.
- **La jauge « bizarre » pendant le réglage** (Joel, dans la foulée) : le
  mode cadran passait toute la jauge en noir sur cyan, donc les blocs
  pleins devenaient noirs et les vides colorés — l'inverse. Les blocs
  gardent leur couleur, seuls les mots s'allument, aux deux écrans.
- **L'ordre des blocs à l'envers** (Joel : « quand je passe en cocoon,
  les never played passent en premier… c'est contre-intuitif non ? ») :
  [ecran-d-accueil.md](conception/ecran-d-accueil.md) dit « au cocon les
  habitués devant, à l'exploration les délaissés », et le code testait
  `confort ≥ 4` pour mettre les délaissés devant — un reste de l'ancien
  sens du curseur (5 = cocon depuis le 06/09/2026). Inversé, et un test
  fige le sens aux quatre coins (5, 3, 1, 0). Premier test de `home.rs`.

## La traîne suit les branches (11/09/2026)

Joel, « très content » après quelques jours d'écoute (noté dans
[atouts.md](atouts.md)), mais au confort 3 « jamais de longue traîne ».
Vérifié dans le code : une branche ne récoltait jamais la discographie —
seuls `e<n>`, `:warm` et `ad` le faisaient, 16 artistes sur 314 en
avaient une en cache. Le poids, lui, était juste.

Option 1 retenue par Joel : après chaque `recompute`, les artistes des
branches proposées sans traîne sont **récoltés en fond**, sans un mot, dès
que le confort ouvre la traîne. `harvesting` devient une carte slug →
silencieux, pour que le toast « discography — loading » ne serve que les
récoltes demandées. Détail dans [longue-traine.md](conception/longue-traine.md).

Un premier câblage retirait au sort les branches encore proposées à
l'arrivée de la traîne ; à l'usage, « les chansons des branches changent
immédiatement », et Joel n'en veut pas. Entre attendre la récolte avant
d'afficher et afficher les tops puis laisser le cache servir les tirages
suivants, **la seconde** est câblée (la plus simple, réversible) : une
proposition affichée ne bouge plus. `engine::redraw` et son test sont
retirés. **À éprouver : la marque `·` doit apparaître dès les tirages
suivants, pas au premier d'un artiste inconnu.**

## La carte de la barre suit enfin l'aiguille (11/09/2026)

Joel : la barre de progression de la carte Omarchy « reste à 0:00 alors
qu'un morceau joue bien » — le retour du bug du 10/09. Cette fois le bus
est hors de cause : `busctl` donne la position, la durée, « Playing », et
`Seeked` part bien à chaque morceau. Le défaut est dans Quickshell :
`MprisPlayer.position` se **calcule à chaque lecture** (dernier échantillon
+ temps écoulé) mais le signal `positionChanged` n'est jamais émis pendant
la lecture — une liaison QML garde donc la valeur lue à la découverte du
lecteur, ou celle du dernier `Seeked` : 0 au début du morceau. Vérifié dans
une instance Quickshell à part : la propriété liée reste à 152,91 s tandis
qu'une lecture directe avance ; un appel à `player.positionChanged()`
resynchronise la liaison. Le widget gagne un `Timer` d'une seconde, actif
seulement **carte ouverte et morceau en lecture**, qui demande ce signal.
Le `Seeked` du 10/09 reste utile pour les sauts (`h`, une correction).
Consigné dans [barre-omarchy.md](conception/barre-omarchy.md).

## Ye et Kanye West ne font qu'un (10/09/2026)

Après le correctif précédent, `entrée` sur « Kanye West » répondait « Ye a
déjà une fiche » et s'arrêtait. Deux causes :

- **La collection doublait l'artiste.** La fiche `kanye-west` porte le nom
  que MusicBrainz lui donne aujourd'hui, « Ye » ; la bibliothèque Spotify
  dit encore « Kanye West ». La collection rapprochait le classement des
  fiches **par nom** : deux lignes, « Ye » avec fiche mais à familiarité
  nulle, « Kanye West » sans fiche. Le seed de `learned/` est désormais
  **indexé par slug**, et la familiarité comme les aimés se cherchent par
  le slug de la fiche ou par celui du nom affiché : une ligne, la bonne
  familiarité. Test `le_seed_se_retrouve_par_le_slug_quand_le_nom_a_change`.
- **Une fiche déjà là bloquait le geste.** `:generate` sur un artiste qui
  a sa fiche disait « a déjà une fiche » et ne faisait rien de ce qu'on
  voulait de lui. Désormais l'intention (`After`) s'exécute quand même,
  par le même chemin qu'une fiche fraîche (`Job::Existing`, `after_card`) :
  entrée démarre chez lui, `ad` ouvre sa discographie.

Reste que la fiche s'appelle « Ye » : c'est le nom MusicBrainz du moment,
la fiche est à Joel — un `ae` la renomme s'il préfère « Kanye West ».

## Entrée sur un artiste sans fiche génère, et tout se dit en toast (10/09/2026)

Joel, sur Kanye West surligné dans la collection : « cela me met "Kanye
West n'a pas de fiche : rien d'où brancher" ». Or arriver chez un artiste,
c'est lui faire une fiche (0016), comme `ad` depuis le matin et comme la
modale de recherche : `entrée` sur une ligne sans fiche passe par
`:generate` — la fiche naît, le parcours part de chez lui.

Et « la notification apparaît en bas et n'est pas bien visible. Note-le en
règle : toutes les notifications doivent apparaître en toast. » La règle du
08/09 disait déjà « plus de ligne de statut », mais l'accueil gardait sa
ligne du bas pour ce qu'il disait (`(inconnu : …)`, « rien de surligné »,
ce que `tell` lui remontait). Désormais **tout passe par le toast**, sous
l'accueil comme en écoute : l'accueil dépose ce qu'il a à dire, la session
le relève après chaque touche et le pose en cartouche ; `tell` ne distingue
plus les écrans. Règle consignée dans
[`forme-de-l-application.md`](conception/forme-de-l-application.md) § « Tout
se dit en toast », test `the_home_says_everything_in_a_toast`.

## Le morceau suivant remonte dans la ligne d'écoute (10/09/2026)

Maquettes **4a** et **4a′** de `Lecture.dc.html` (Claude Design, projet
« Accueil Forkstify ») : la ligne « à suivre » sous la barre coûtait une
ligne de pied pour redire ce que la liste montre deux lignes plus haut, et
tant qu'une ligne suivait la barre, l'œil la prenait pour un séparateur.
Câblé tel quel :

- **Le pied passe de trois lignes à deux** : la ligne d'écoute porte tout
  l'axe du temps — `▶ ce qui sonne — artiste  (2 / 10) │ then ♪ suivant —
  artiste` à gauche, `♪ top │ 1:48 / 3:49 -2:00` à droite —, puis la barre,
  qui ferme le pied. Même chose sous l'accueil.
- **`→ fork in 8`** remonte dans la barre de titre avec les compteurs de
  segment, à la place de « n ahead » : c'est un état du parcours, pas de
  la lecture. `→ fork next` quand la file est vide.
- **L'ordre de sacrifice de 4a′** quand la fenêtre rétrécit, le bloc de
  droite intouchable : 1. l'artiste du suivant · 2. `then` et le compteur,
  le `│` suffit · 3. la provenance et le restant `-20:44` · 4. le titre en
  cours coupé à l'ellipse, jamais sous 16 caractères · 5. le suivant
  quitte la ligne et la liste le marque **`▸` dans la gouttière** (le repli
  de 4b : ` 2 ▸↻ ♪ Israel`). Un seul `…` par ligne ; le titre du suivant
  n'est jamais tronqué. Pas de seuils fixes : chaque étape se prend dès
  que la précédente ne tient pas.
- **Tranché en attendant mieux** : si même le titre seul ne tient pas,
  l'artiste en cours s'efface entièrement plutôt que de subir une seconde
  ellipse (la maquette montrait les deux coupés, sa note interdit deux
  coupes). Réversible.

Tests : la ligne aux largeurs 170 / 145 / 138 / 120 / 96 / 72 ; la
gouttière `▸` à 60 colonnes. `Bar` perd `ahead`.

## L'interface passe en anglais (10/09/2026)

Joel veut publier une première version sous peu, utilisable par le plus
grand nombre : **tout ce qui s'affiche est désormais en anglais**
([0022](decisions/0022-interface-en-anglais.md)) — TUI, toasts, tables des
touches, sorties de la ligne de commande, gabarit de `config.toml`, widget
Omarchy et son `install.sh`, libellés du moteur (`liked` · `tail` ·
`non-top` · `off-catalog`, `shared members`, `close to the branch's
center`…). Les sous-commandes suivent : **`journey`** (ex-`parcours`) et
**`listen`** (ex-`ecouter`). La section `[catalogue]` de la configuration
devient `[catalog]`, l'ancien nom reste lu. Les notes écrites dans les
fiches par `td`/`aL` sont en anglais (`set while listening, <date>`). Ne
bougent pas : le texte vectorisé d'`embed.rs` (l'index en dépend), les
spikes de `src/bin/`, et la doc, qui reste en français. Dans la foulée,
**tous les commentaires du code sont traduits** (~1 000 lignes, seuls des
commentaires ont bougé, tests verts), puis les **51 noms de tests** et
leurs messages d'assertion ; ne restent français que les sorties des spikes.

## L'agent : une IA qui pilote forkstify de l'extérieur (10/09/2026)

Idée de Joel : une commande `:agent` qui transmet un prompt à une IA
connectée (le Claude Code de son poste) — « crée-moi une playlist de 20
titres dans l'ambiance Kanye West, Drake, Kendrick Lamar » —, l'IA se
servant des outils de forkstify, du catalogue et de la session d'écoute,
après un échange éventuel pour éclaircir. Discutée, rien de codé ;
orientation consignée dans [`agent.md`](conception/agent.md).

En bref : **l'agent pilote, il ne choisit pas dans sa tête** — il cherche,
génère des fiches (`:generate`), demande des branches, lit leurs raisons et
met en file ; la playlist est le produit dérivé, le catalogue a grandi. Et
**l'agent est dehors** : pas de client LLM ni de clé dans le binaire.
Étage 1, sans `:agent` : un socket de contrôle et `forkstify cmd ':…'` plus
quelques lectures JSON — l'API de l'agent, ce sont les commandes `:` de
0013, la conversation se tient dans Claude Code. Étage 2, `:agent` dans la
TUI avec une commande externe configurée, seulement si l'étage 1 se révèle
trop lourd à l'usage. Six questions à trancher dans la note (plafond de
génération, trace dans la file, morceaux « de sa tête », nom, trailer,
nécessité de l'étage 2).

## Une journée d'écoute : la file, la cible, la génération, les aimés (09/09/2026)

La première vraie session d'écoute longue de Joel, et ses retours traités
au fil de l'eau — l'étape 1 des prochaines étapes est **entamée**. Dans
l'ordre :

- **La liste de lecture montrait une file tronquée.** La file est un
  `VecDeque` ; après un `push_front` (branche prise « maintenant », retour
  arrière, `ti`) le tampon s'enroule et `as_slices().0` n'en rendait que la
  première moitié : une branche choisie manquait, ou revenait quelques
  morceaux plus tard. `make_contiguous()` avant de dessiner.
- **`tx` existait** mais manquait dans l'aide de `t` ; ajouté, et son index
  suit désormais le même axe que le déplacement.
- **L'encore visait le bout de la chaîne.** Depuis que choisir une branche
  s'ajoute à la file, le « courant » tiré des rounds est le dernier artiste
  empilé, plus ce qui sonne : un `en3` servait le mauvais artiste. D'où la
  **décision [0020](decisions/0020-la-cible-d-un-geste.md)** : un geste vise
  la **ligne surlignée, sinon ce qui sonne**, pour `t`, `a` et `e` ; `ts` et
  `tb` ne font avancer la musique que s'ils visent ce qui sonne ; `en<n>`
  et `e!<n>` se posent **derrière la ligne surlignée** quand elle est à
  venir. Le retour n° 12 (cible de `t`/`a`) est clos par là.
- **La Ruda** : cinq tops posés à la main, et la fiche renommée « La Ruda
  Salska » — le nom MusicBrainz n'est pas celui de Spotify, et la
  résolution d'un morceau cherche par nom. Question notée dans
  [generation-a-la-volee.md](conception/generation-a-la-volee.md).
- **`:generate <nom> <mbid>`** : l'identifiant trouvé à la main remplace la
  recherche par le nom ; si MusicBrainz se tait, fiche minimale marquée à
  relire ; un artiste proposé en creux prend sa branche. Lu de l'accueil
  comme de l'écoute.
- **« Lojo est introuvable »**, deux causes : MusicBrainz répond 503 par
  rafales (on patiente six essais sur une demi-minute, et on dit
  « occupé » plutôt qu'« introuvable ») ; et un nom venu d'un slug a perdu
  ses apostrophes (« Lojo » pour Lo’Jo) — **Deezer prête l'orthographe**,
  vérifiée à la clé des fiches. Test réseau ignoré par défaut.
- **`docs/atouts.md`** : les impressions positives de Joel, datées et
  citées, pour lister les atouts le moment venu. Première entrée : la
  redécouverte de ce que Spotify ne proposait pas, la cohérence maîtrisée,
  l'inattendu quand même.
- **L'accueil montre les aimés par défaut**, `v` bascule sur tout le
  catalogue. Aimé = un « plus souvent » ou un ♥ ici, un titre, un album ou
  un suivi sur Spotify (`classement.json`).
- **Les invités d'un titre aimé ne sont pas des aimés** (Bosh, Bossikan,
  Bow Wow — le fils de Joel) : les scripts de récolte ne comptent plus que
  l'artiste principal ; récolte relancée par Joel, classement recalculé :
  605 artistes classés au lieu de 741.
- **`al` / `as` / `ab` à l'accueil**, sur la ligne surlignée. `as` pose un
  drapeau `unliked` dans `learned/artists/<slug>.toml` qui prime sur
  Spotify et survit aux récoltes ; `al` l'efface ; la fusion 0017 le
  traite comme un ban. Un artiste sans fiche s'écrit et se relit sous le
  slug de son nom.

Deux questions de conception notées, tranchées en partie : **retirer un
artiste des aimés** (fait, ci-dessus) et **un setup fluide** — connexion,
import de la bibliothèque, playlists à cocher — tranché « premier
lancement *et* rejouable », maquette Claude Design à venir de Joel avant
de coder ([premiere-installation.md](conception/premiere-installation.md)).

## Le vecteur naît avec la fiche (09/09/2026)

Joel a tranché (b) : **l'application vectorise elle-même**
([0019](decisions/0019-vectorisation-par-l-application.md)).

- **`embed.rs`** : la composition du texte portée mot pour mot depuis
  `vectoriser.py` (vérifiée identique sur les 316 textes de la référence,
  `forkstify vectors --texts` contre `vectoriser.py --textes`), le modèle
  par `fastembed` (features rustls, variante quantifiée, `max_length =
  128`, cache dans `$XDG_CACHE_HOME/forkstify/fastembed`), l'écriture d'une
  ligne dans `vectors.jsonl` en ordre de slug, la régénération complète.
  `Card` porte désormais `begin`, `end`, `origin`, `description`.
- **En session** : `Job::Generated` compose le texte et envoie le modèle en
  fond (`spawn_blocking`), `Job::Vectorized` adopte la fiche **et** son
  vecteur dans le même commit (`Edit.also`). Si le modèle manque, la fiche
  entre quand même et le toast dit « sans vecteur ». Premier calcul : le
  toast prévient que le modèle se télécharge.
- **`forkstify vectors [catalogue] [--texts]`** régénère l'index et
  `meta.toml` (clés anglaises, `max_length`, `normalized`). L'import le
  fait dans son commit, plus de docker.
- **Image de build** : `g++` ajouté au Dockerfile. Binaire : 23 → 58 Mo.
- **Index de référence régénéré** par l'application et poussé sur le fork
  (`dd82e8d` côté catalogue) : cosinus ≥ 0,999999 avec l'ancien, normes
  à 1. `tools/vectoriser.py` marqué remplacé, gardé pour mémoire.

**Reste** : une édition de lien en session ne recalcule pas les vecteurs
(elle ne compte qu'au prochain lancement) — noté dans la note de
conception. L'index régénéré est sur le fork, pas encore sur la référence
`aropixel` : à pousser.

## L'essai fastembed en Rust (09/09/2026)

Pour trancher la vectorisation d'une fiche générée, un spike jetable dans
`~/Work/tries/fastembed-spike` (hors dépôt) a vectorisé les 316 textes de
la référence avec le crate `fastembed` 6 et les a comparés à
`vectors/vectors.jsonl`. Résultat : **cosinus ≥ 0,999999 partout à
`max_length = 128`** (à 512, défaut du crate, les fiches riches divergent
jusqu'à 0,82), build à froid 26 s, binaire +35 Mo, `g++` requis dans
l'image de build, modèle 241 Mo téléchargé au premier usage, 14 ms par
texte. Les features par défaut tirent OpenSSL : prendre les variantes
`rustls`. Découvert au passage : la référence Python **n'est pas
normalisée** (normes 2,5–3,6), ce que le centroïde du moteur subit.
Chiffres, conditions et orientation (b, avec repli en feature cargo) dans
[conception/generation-a-la-volee.md](conception/generation-a-la-volee.md).
**Joel tranche.**

## Les versions d'un titre se distinguent dans la recherche (09/09/2026)

Retour de Joel : « quand la chanson apparaît plusieurs fois (*Quand on n'a
que l'amour*), je ne sais pas laquelle est laquelle ». Les lignes `[spotify]`
de la modale ne portaient que titre, artiste et une mention générique :
cinq versions de Brel (studio, Olympia, best-of…) faisaient cinq lignes
identiques.

- `search_tracks` ramène désormais **album, année et durée** (`SearchHit`
  dans `spotify.rs`, à la place du triplet titre/artiste/uri).
- La note d'une ligne Spotify les dit **d'abord** — `Olympia 64 · 1964 ·
  3:07 · branche ensuite` — puis ce que fait entrée : `branche ensuite`
  quand l'artiste a une fiche, `⏎ génère la fiche` sinon. Les anciennes
  parenthèses `(branche ensuite) la fiche existe` / `(hors catalogue — ⏎
  génère la fiche)` disparaissent ; l'en-tête du groupe dit déjà « hors
  catalogue sauf mention ».
- Le catalogue n'est pas touché : ses titres sont déjà dédoublonnés par
  (titre, fiche), et l'artiste suffit à les distinguer.

Sur un terminal étroit, c'est la fin de la note qui se coupe — donc la
mention, jamais l'album. Vérifié à sec (tests) ; à constater à l'usage sur
un titre à plusieurs versions.

## La génération de fiche à la volée (09/09/2026)

Joel, le matin : « comment faire pour ajouter un artiste ? J'ai envie
d'écouter Jacques Brel mais il n'est pas dans le catalogue. J'ai pu le
trouver via la recherche Spotify, mais je ne peux pas jouer le morceau et
cela ne crée pas la fiche artiste. » Puis : « oui je veux pouvoir faire de la
génération à la volée. »

C'était la moitié non écrite de
[0016](decisions/0016-base-large-et-generation-a-la-volee.md). Le point que
la décision avait laissé ouvert — **quand** la génération se déclenche — est
tranché : **les deux**, la recherche et l'arrivée. Le sujet a sa note,
[conception/generation-a-la-volee.md](conception/generation-a-la-volee.md).

- **`src/generate.rs`** — le pipeline de `tools/generate-cards.py` porté en
  Rust : MusicBrainz (identité, dates, origine, genres, relations typées)
  puis Deezer sans clé (cinq tops, quatre similaires). Quatre appels, environ
  trois secondes, en `spawn_blocking` comme la récolte de discographie. La
  limite d'une requête par seconde de MusicBrainz est tenue, et le 503 —
  fréquent — se réessaie au lieu de passer pour une absence.
- **Une différence avec le script, voulue** : un lien `similar` vers un
  artiste **sans fiche** est conservé au lieu d'être abandonné. C'est
  précisément ce qui fait grandir le catalogue le long de ses liens.
- **La recherche fait entrer quelqu'un de neuf** : `entrée` sur un résultat
  hors catalogue génère la fiche, la commite, et démarre chez lui — de
  l'accueil comme de l'écoute. `:generate <nom>` fait la même chose sans
  passer par un morceau. Fini le « rien d'où brancher ».
- **L'arrivée fait grandir le catalogue** : `graph_neighbors` jetait les
  liens dont la fiche manque (Brel en avait trois). `engine::missing_neighbors`
  les rend, la colonne les affiche en gris — « ○ fiche à générer » — et ils
  se prennent au chiffre suivant les branches.
- **La session possède désormais son catalogue** (`Live { catalog: Catalog }`
  au lieu de `&'a Catalog`) et y insère la fiche fraîche : une génération
  qu'il faudrait relancer pour voir n'en serait pas une. `listen::run` charge
  le catalogue lui-même ; l'appelant ne le prête plus.
- **Une fiche générée est une édition** (0013) : `edit::create_card` écrit,
  refuse d'écraser, et commite avec le trailer `Forkstify: edit`.

**Reste ouvert, et Joel décidera** : la **vectorisation** d'une fiche
générée. Elle naît sans vecteur — elle navigue par ses liens et ses tags,
l'écran le dit — et le rattrapage passe encore par la commande docker de
`tools/vectoriser.py`. Les deux sorties (commande `:vectors`, ou `fastembed`
en Rust) sont chiffrées dans la note ; aucune n'a de code à défaire.

**Non éprouvé en session réelle** : le pipeline l'est (test réseau
`le_pipeline_compose_une_vraie_fiche`, ignoré par défaut), les deux
déclencheurs ne le sont pas.

## `:search` s'ouvre aussi de l'accueil (09/09/2026)

Joel : « quand on fait `:search` depuis l'accueil, la fenêtre de recherche
ne s'ouvre pas » ; « sur l'accueil, il y a encore un ancien texte qui dit
que `/` fait une recherche, alors que `/` filtre maintenant ».

- **La modale n'existait que sur l'écran d'écoute.** À l'accueil,
  `:search <texte>` faisait encore l'ancien geste — résoudre un nom dans le
  catalogue et démarrer — et `:search` seul ne faisait rien du tout. C'est
  désormais **la même modale sur les deux écrans** : le clavier lui est
  donné **avant** l'aiguillage d'écran, et elle se dessine par-dessus le
  corps de l'accueil, collection comprise ; le pied de lecture et l'invite
  restent.
- **Entrée démarre un parcours**, depuis l'accueil : sur l'artiste, ou sur
  le morceau — le titre d'abord, puis les branches de son artiste. C'est la
  règle de l'accueil, un chiffre y fait déjà la même chose. Un titre
  Spotify **sans fiche** n'a rien d'où brancher : l'accueil le dit et
  renvoie à l'écoute, où `:search` le joue quand même.
- **Le texte de l'accueil parlait encore de l'ancien `/`** : le bloc
  « chercher » annonçait `/` pour « catalogue et spotify ». Il annonce
  `:search`, et une seconde ligne dit ce que `/` fait vraiment — filtrer la
  collection, échap efface. L'écran non connecté aussi.
- **`:search bowie` ouvre la modale remplie, et la frappe continue le
  mot** : le mode texte du lecteur de touches vidait sa ligne au passage,
  si bien que la première touche effaçait « bowie ». La ligne de départ est
  posée avec le mode (`keys::set_text(true, "bowie")`).

## Lot 4 : 100 fiches, la référence passe à 314 (08/09/2026)

Joel : « est-ce qu'il reste des fiches à créer ? on les importe sur
aropixel ? » Relevé : 100 slugs appelés par des liens sans fiche (le lot
que le lot 3 appelait depuis le 02/09), 22 appelés par au moins deux fiches ;
561 classés sans fiche, tous sous le score 5, laissés à la traîne comme
prévu. Une fiche générée est de la connaissance, pas du goût : elle va à la
**référence** `aropixel` (0016), le fork la reçoit par `upstream`.

**Les scripts d'outillage étaient cassés depuis le renommage du 06/09** :
`generate-cards.py`, `vectoriser.py` et `voisins.py` lisaient encore
`fiches/` et `vecteurs/vecteurs.jsonl`. Corrigés sur la branche `lot-4`
partie de `upstream/main`. Le générateur réessaie désormais sur un délai
dépassé — le premier lancement était mort au premier artiste sur un
`TimeoutError` de MusicBrainz.

**Généré dans un conteneur `python:3.12-slim`** (bibliothèque standard
seule, MusicBrainz à une requête par seconde) : **100 fiches écrites**, le
catalogue compte **314 fiches**. Rapport : à relire, MBID incertain —
blundetto, dalle-beton, lej, ozuna, palatine ; sans Deezer, donc sans tops —
mahmoud-ahmed. Vecteurs recalculés dans le conteneur `fastembed` (modèle
téléchargé, ~220 Mo, cache dans `tools/cache/`, désormais ignoré par git).

Poussé sur `aropixel/main`, puis le fork `kbyjoel` rebasé dessus : les
commits d'appris de Joel ne partent pas vers la référence.

## Échap passe du premier coup (08/09/2026)

Joel : « quand je veux fermer avec échap, je dois souvent appuyer plusieurs
fois. » Le lecteur de touches, après un `ESC`, lisait **deux octets de
plus** pour reconnaître une flèche (`ESC [ A`), en bloquant — or la touche
échap seule n'en envoie qu'un : il fallait deux frappes de plus pour qu'elle
passe. Corrigé : après un `ESC`, le lecteur **interroge le descripteur**
(`poll`, vingt millisecondes) — une séquence arrive d'un bloc, un échap
seul n'a pas de suite. Le lecteur lit désormais l'entrée **sans tampon**
(`libc::read`), parce que le tampon de `std::io::stdin` aurait caché la
suite d'une séquence au `poll`. Même chose en mode texte et dans les lignes
`/` et `:`, où une flèche ne retombe plus dans la grammaire. `ESC O A`
(mode application) est reconnu aussi.

## Plus de ligne de statut : tout en toast (08/09/2026)

Joel : « je ne veux plus aucune notification en dessous de à suivre, elles
doivent toutes apparaître en toasts ; à suivre doit toujours être suivi des
indications de raccourcis. » La ligne « dernière chose dite » disparaît du
pied, en session comme sous l'accueil ; les touches suivent « à suivre »
directement. Tout ce qui se dit passe en toast, parenthèses comprises (en
gris, quatre secondes). Le journal reste en mémoire pour les blocs (`?`).

## `J` / `K` déplacent un morceau de la liste (08/09/2026)

Joel : « comment sélectionner un morceau de la liste de lecture et le
déplacer ? » Trois voies proposées — `J`/`K` d'un cran tout de suite, `tg`
saisir-poser aux flèches, `tm<n>` en position n — et la première retenue :
un cran couvre l'usage réel, remonter un morceau qu'on veut entendre plus
tôt, sans mode ni validation, la touche contraire annule. Seul ce qui est à
venir bouge ; le nom de branche voyage avec son morceau ; le déplacement se
voit dans la numérotation et ne se dit pas. `tg` viendra si les longs
déplacements se révèlent fréquents.

## La modale de recherche, et `ti` (08/09/2026)

Joel : « :search se lance sans argument, une modale s'ouvre, avec une ligne
séparant la zone de texte de la zone de résultats ; ti — track insert —
insère une track où on est dans la liste, en ouvrant la même modale. »
Maquette `Recherche.dc.html` (Claude Design) : 1a `:search`, 1b `ti` ancré,
1c les deux bords.

- **La modale est modale, et c'est nouveau** : tant qu'elle est ouverte, la
  grammaire sans préfixe ne s'applique plus — sinon taper « cros »
  déclencherait c, r, o, s. Le lecteur de touches a un **mode texte**
  (`keys::set_text`) : tout est frappe, sauf ↑↓, entrée, tab et échap.
  D'où l'invite `⟩`, pour dire « ici, on écrit ».
- **Une seule règle coupe la saisie des résultats**, et porte le décompte :
  `── catalogue 3 · spotify 5 ──`. Pas de champ encadré.
- **Le catalogue avant Spotify, toujours**, en deux groupes jamais mêlés
  (bleu écrit par un humain, cyan deviné). Le catalogue répond à chaque
  caractère — artistes par nom, titres connus des fiches et de l'appris,
  avec la provenance `♪ ♥` et les écoutes ; Spotify répond **derrière**
  (`Job::Searched`), le groupe cyan dit « … interrogation » jusque-là, et
  une réponse à une frappe plus ancienne est jetée : la liste ne saute
  jamais sous le curseur.
- **Entrée fait selon la porte** : par `:search`, un artiste branche là
  (un segment chez lui), un titre sonne maintenant et les branches
  repartent de son artiste s'il a une fiche ; par `ti`, le titre **entre
  dans la file à l'ancre** — avant la ligne surlignée si elle est à venir,
  sinon juste après ce qui sonne — marqué « inséré (ti) » en gris à côté ;
  un artiste choisi insère son meilleur morceau non joué. Le toast dit
  « → inséré en 4 : titre — artiste » ou « → … — via :search ».
- **`ti` lit son ancre en haut** (« l'insertion tombe en 4 — entre X et
  Y »), comme la maquette le voulait : insérer à l'aveugle dans une file
  qu'on ne voit plus est le geste le plus facile à rater.
- **Tab** masque Spotify. Vide, la modale dit quoi taper ; sans résultat,
  elle le dit aussi.
- L'ancienne recherche par journal (résultats numérotés, un chiffre choisit)
  et son état `pending` disparaissent.

**Écarté de la maquette** : `e` (mettre à la file), `tb`, `A`/`N` dans la
modale — ce sont des lettres, elles se tapent ; la portée « fiches du
parcours » de `ti` (tout le catalogue répond, avec Spotify derrière) ; les
durées ; les cinq dernières recherches. Un test de rendu. Non vérifié en
session réelle.

## `ag` — l'artiste dans le navigateur (08/09/2026)

Joel : « ajoute une commande, ag ?, pour googler l'artiste en cours dans le
navigateur par défaut ». `ag` ouvre `https://www.google.com/search?q=…` par
`xdg-open`, détaché (entrées et sorties fermées, pour ne rien laisser
s'imprimer sous la TUI). Il vise comme `ad` : la ligne surlignée s'il y en a
une, sinon ce qui sonne. Le toast dit « → artiste — dans le navigateur ».

## Quatre retouches du clavier (08/09/2026)

Joel : retirer le mode file d'attente de l'aide ; « je ne vois plus le
raccourci pour intercaler un morceau, c'est moi ? » ; un raccourci `c<n>`
pour le confort ; `/` devient filtrer, la recherche devient `:search`.

- **`Q` disparaît** — de l'aide, de la grammaire, de la table. Le mode file
  d'attente est tombé le 06/09, la file s'enchaîne dans l'écran d'écoute.
- **Intercaler** : ce n'est pas lui. Ce qui existe : `e<n>` / `en<n>`
  (encore, même artiste, en fin de branche ou tout de suite), `fn<n>` (une
  branche après ce morceau), et `e` dans la discographie (à la file, à la
  fin). **Intercaler un morceau précis à un endroit précis n'existe pas** —
  c'était le `i` du mode file d'attente, jamais câblé. À concevoir avec
  `:search` : un résultat pourrait s'intercaler après le morceau en cours
  plutôt que jouer tout de suite.
- **`c` devient un namespace** : `c<n>` règle le confort d'un coup (0 à 5,
  à l'accueil aussi), `cc` ouvre la jauge aux flèches. `c` seul ne pouvait
  plus être complet sans casser la grammaire sans préfixe.
- **`/texte` filtre, `:search <texte>` cherche.** À l'accueil, `/` filtre la
  collection (échap efface, ↑↓ entrée démarrent) ; dans la discographie il
  filtrait déjà ; en écoute, il n'y a pas de liste à filtrer et il le dit.
  `:search` fait ce que `/` faisait : catalogue + Spotify en écoute, un
  chiffre choisit ; catalogue seul à l'accueil, l'artiste trouvé démarre.

## Plus de `tt` dans la modale, et des toasts (08/09/2026)

Joel : « les raccourcis tt et tT sont toujours présents dans la modale de
discographie, je veux les retirer. Les messages de chargement doivent être
plus visibles — des toasts en bas d'une des deux colonnes, avec de la
couleur ? »

- **`tt` / `tT` quittent la modale** (grammaire `parse_modal`, touches,
  légende, `Explore::top` / `untop` et leurs tests). Reste `A`, qui promeut
  d'un coup les titres les plus écoutés d'un album ; un titre seul s'aime
  (`tl`), il ne se promeut plus. Le test d'écriture passe par `A`.
- **Les toasts** : un cartouche cadré de la couleur du message, texte en
  gras, en bas à droite du corps, au-dessus des touches de la colonne des
  branches — par-dessus la modale aussi. **Collant tant que ça charge**
  (« ⏳ en cours — chargement — titre — artiste », « discographie de X — en
  cours de chargement », en cyan, la couleur de l'attente réseau), sinon
  **quatre secondes** pour la dernière chose dite (`✓` vert, `⏹` rouge, `↻`
  jaune, `→` magenta — `tui::tone_of`, la même lecture que la ligne du
  pied). Les parenthèses restent sur la ligne du pied, sans toast. Le tic
  d'une seconde l'efface.

## « Précédent » recommence d'abord (08/09/2026)

Joel : « quand je fais flèche pour morceau précédent et qu'un morceau est en
cours, je veux que cela recommence le morceau au début ; on appuie une
deuxième fois pour revenir au morceau d'avant. » Comme tout lecteur : passé
trois secondes (`RESTART_AFTER_MS`), ← et ⏮ remettent l'aiguille à zéro
(`Sound::restart`, un `seek(0)` de librespot — la progression suit son
`Seeked`) ; dans les trois premières secondes, ou en pressant deux fois, on
remonte au morceau d'avant comme avant.

## L'accueil devient un écran de la session (08/09/2026)

Joel : « les deux écrans accueil et lecture sont indépendants : quand je
quitte l'écran lecture, ma session s'arrête et elle est perdue. Je voudrais
lancer une session, revenir à l'accueil, conserver l'écoute et la barre en
bas, et revenir sur mon écran de session sans jamais perdre ma session. »

Avant, `main.rs` bouclait : l'accueil rendait un choix, la session naissait
(son, API, MPRIS), vivait, mourait, et l'accueil revenait. Désormais **la
session est l'application**, et l'accueil l'un de ses deux écrans
(`Screen::Home` / `Screen::Session`) :

- **`q` en écoute rend l'accueil**, l'écoute continue en dessous : le son,
  la file, les branches, l'appris, tout reste. **`r`** (ou échap sans
  curseur) **ramène à l'écran de session**. `q` à l'accueil quitte pour de
  bon, avec le commit et le push de 0017.
- **Le pied de lecture s'affiche sous l'accueil** — ce qui sonne, sa barre,
  ce qui suit, la dernière chose dite — sur les quatre lignes au-dessus de
  l'invite (`tui::Bar`, le même pied qu'en session, `render_bar`). `p` y
  tient la pause ; les touches multimédia marchent partout.
- **Choisir une graine à l'accueil pendant qu'une session joue démarre un
  nouveau parcours** qui remplace l'ancien (`Live::start_journey`) — sans
  reconnecter quoi que ce soit, donc sans l'écran d'attente.
- `home::run` devient `home::Home` (un état : ce qui est tapé, le tri, le
  curseur) avec `draw` et `on_cmd` → `Outcome::{Stay, Start, Back, Quit}` ;
  la boucle unique de `listen.rs` route les touches selon l'écran. Le son
  et l'API web ne se connectent qu'une fois par lancement.

Un test de rendu de l'accueil avec le pied. Non vérifié en session réelle.

## L'écran d'abord, Spotify derrière (08/09/2026)

Joel : « au lancement d'une session d'écoute, ou à l'ouverture et à la
fermeture de la modale de discographie, il y a souvent de gros temps de
latence. J'aimerais mieux gérer cela : via du cache quand c'est possible, et
en affichant d'abord puis en chargeant après, en indiquant qu'un chargement
est en cours. »

**La cause** : chaque appel à l'API Spotify — la résolution d'un titre en
adresse `spotify:track:`, la discographie d'un artiste — était **attendu
dans la boucle de commandes**, qui ne redessine qu'une fois le geste fini.
Le repli sur un 429 (quota compte/IP, fréquent juste après la rafale d'une
discographie) dort jusqu'à soixante secondes dans cette même attente : c'est
le gel à la fermeture de la modale, quand `prefetch_next` résolvait le
morceau suivant. Le cache existait déjà (résolutions dans
`target/resolve-cache.json`, discographies dans `~/.cache/forkstify/`), il
n'évitait que la seconde fois.

**Ce qui change** : l'API web est partagée derrière un verrou
(`Arc<Mutex<WebApi>>`) et les appels partent en **tâches de fond**
(`spawn_local`), qui rendent leur résultat à la boucle par un canal
(`Job::Resolved`, `Job::Harvested`), comme le push de 0017.

- **Jouer un morceau** : si le cache connaît son adresse, il sonne tout de
  suite ; sinon il **s'affiche tout de suite** avec « · chargement… » dans
  le pied, et sonne quand la réponse arrive. Un morceau sauté pendant
  l'attente n'entre pas dans le passé ; un introuvable passe au suivant ;
  une panne le remet en tête de file, comme avant. Le préchargement du
  suivant ne bloque plus rien.
- **La discographie** s'ouvre à l'instant sur ce qu'on a — les tops et
  l'appris — avec « … discographie en cours de chargement » et le mot
  « chargement… » dans son titre ; les albums arrivent derrière et l'écran
  se reconstruit sans perdre tri, filtre ni éditions en attente
  (`Explore::reload`). Une récolte ne se lance jamais deux fois.
- **`e<n>` et `:warm`** lancent la récolte derrière et le disent : « sa
  traîne arrive — refais e<n> dans un instant ».

**Reste attendu dans la boucle** : la recherche `/texte`, dont on attend le
résultat par nature, et la connexion au démarrage (librespot, jeton), que
l'écran d'attente montre étape par étape. Non vérifié en session réelle :
c'est à toi de dire si les gels ont disparu.

## Le catalogue devient un fork, la référence part chez aropixel (08/09/2026)

Joel : « pour régler le problème du catalogue qui n'est pas un fork parce que
je suis le concepteur, je voudrais déplacer forkstify et forkstify-catalog
dans mon GitHub aropixel. Comme ça, je le publierai en tant qu'aropixel, ce
qui me permet de faire un fork avec mon compte kbyjoel. »

Fait par l'API GitHub : les deux dépôts sont **transférés** à l'organisation
`aropixel` (ils restent privés), et `aropixel/forkstify-catalog` est **forké**
en `kbyjoel/forkstify-catalog`, qui reprend le nom libéré. En local,
`~/Work/forkstify` pointe sur `aropixel/forkstify` ; `~/Work/forkstify-catalog`
pointe sur le fork en `origin` — c'est lui que le pull du démarrage tire et
que l'appris rejoint — et sur la référence en `upstream`, que `:mine` compare
déjà en premier. Joel est enfin un utilisateur comme les autres (0016) : ses
éditions vivent sur son fork, et remontent à la référence par une PR.

Les deux commits Cat Power sont restés sur la référence ; à reporter sur le
fork ou à y laisser, au choix de Joel. Le plan de reprise de chorizo clone
désormais les bons dépôts.

## Le cooldown daté (08/09/2026)

Joel : « applique le cooldown daté de 0012 alors ». §2 disait : chaque
lecture est datée, un morceau joué récemment est pénalisé, la pénalité
décroît avec le temps. Les dates étaient là depuis 0014 (`last` par top),
le réservoir ne les lisait pas.

`Learned::freshness` : **un dixième de son poids le jour où il a sonné**,
retrouvé avec une **demi-vie d'une semaine** — 55 % à sept jours, 78 % à
quinze, 94 % à un mois ; entier s'il n'a jamais sonné ici. Le réservoir
multiplie par ce facteur, après les « moins souvent ». Un aimé écouté hier
(6,8 × 0,13 ≈ 0,9) pèse donc comme un top jamais joué, et reprend sa place
au fil de la semaine. Deux constantes (`COOLDOWN_FLOOR`,
`COOLDOWN_HALF_LIFE`), « à régler au fil du PoC » comme 0012 le prévoyait.
Deux tests.

## Un seul geste pour le goût (08/09/2026)

Joel, après une journée d'écoute : « j'avais du mal à savoir s'il valait
mieux que je like ou que je mette en top : pour moi c'était la même
chose. » Puis : « je ne mettrais même pas de raccourci tt. On laisse la
possibilité de modifier les tops sur son fork, mais on ne laisse qu'un seul
geste simple pour dire je veux voir ce morceau plus souvent. En
contrepartie, il faut aussi pouvoir dire ce morceau ne m'intéresse pas, et
les likes doivent être entièrement prioritaires sur les tops, qui ne
deviennent plus que des portes d'entrée sur un fork vierge. »

Décision [0018](decisions/0018-un-seul-geste-pour-le-gout.md), câblée :

- **`tl` plus souvent, `ts` moins souvent**, et l'un défait l'autre (aimer
  remet les sauts à zéro, sauter retire l'aimé). `tb` reste le plus jamais.
- **Un aimé prime sur les tops** dans le réservoir : ×10 au cocon, ×2 grand
  ouvert, le curseur entre les deux (`liked_weight`). Un top aimé prend ce
  poids et porte `♥`. Avant, un aimé pesait 0,8 et un top aimé restait un
  top ordinaire — c'est ce qui rendait `tl` inaudible.
- **Plus de `tt` / `tT` à l'écoute** : la session les refuse et renvoie à
  la discographie ; la grammaire les lit encore, parce que c'est là, dans
  `ad`, qu'ils servent (`edit::set_tops`) — une première version les avait
  retirés de l'analyseur et cassait la modale, corrigé dans la foulée. `add_top` / `remove_top`, qui
  ne servaient qu'à `tt` / `tT`, sont retirés.

Trois tests (réservoir, appris, grammaire). Le sujet du dépôt de référence
contre le fork personnel (une organisation GitHub pour l'amont, ton clone
devenant ton fork) reste à faire de ta main ; l'application n'a besoin de
rien pour ça, sauf plus tard un `:upstream`.

## Compiler soi-même, et compter l'usage (08/09/2026)

Deux wrappers dans `bin/` — la convention de la machine, mise les met dans
le `PATH` dès qu'on entre dans le dossier : **`build`** (cargo build
--release dans le conteneur `forkstify-build`, construit au premier appel,
registre cargo dans le volume `forkstify-cargo`) et **`test`**. Les trois
commits rapatriés de l'autre poste (l'exploration d'un artiste, « ad »)
compilent sans avertissement, 41 tests verts.

Les commandes qui comptent les commits produits par forkstify à travers les
forks publics (trailer `Forkstify:`) sont notées dans
[`premiere-installation.md`](conception/premiere-installation.md), section
« Mesurer l'usage à travers les forks ».

## L'appris se synchronise tout seul (07/09/2026)

Joel, en changeant de poste : « learned s'est enrichi, mais sans aucun
commit ; je n'ai plus mes enregistrements. On met en place des commits
automatiques et un pull automatique à l'ouverture ? Une meilleure
solution ? » Puis : « je valide, mais je veux que le message soit en
anglais. Comment identifier le nombre de commits des dépôts publics des
différents utilisateurs ? »

Décision [0017](decisions/0017-synchronisation-de-l-appris.md), câblée le
jour même (`src/sync.rs`, `learned::merge_artist`, sous-commande
`merge-learned`) :

- **Pull au démarrage** (accueil et `ecouter`), après avoir commité ce qui a
  été appris ici ; l'en-tête de l'accueil dit `⇅ à jour` / `⇅ appris
  commité, catalogue mis à jour` / `⇅ hors ligne`. **Commit toutes les dix
  minutes** si `learned/` a bougé, push en fond, résultat dans le pied.
  **Commit et push à la sortie**, affiché une seconde. **`:sync`** à la
  demande. Délais réseau courts : hors ligne, rien ne se suspend.
- **Fusion par compteur** : le pilote git `merge=learned` appelle
  `forkstify merge-learned`, qui additionne ce que chaque côté a compté
  depuis l'ancêtre (décru au jour), garde le ban ou l'aimé posé d'un côté,
  suit le poids qui a bougé, laisse entrer les tops nouveaux. Trois tests
  unitaires, **et un scénario git réel** : deux clones, deux écoutes
  concurrentes du même artiste, `pull --rebase` sans conflit, `plays`
  passé de 3 à 6,01 (5 + 4 − 3 décru d'un jour).
- **Les tops sont écrits triés** (`BTreeMap`) : l'ordre de hachage faisait
  de chaque écriture un faux diff.
- **Messages en anglais, trailer `Forkstify: <kind> <version>`** sur les
  commits de l'appris, des éditions et des imports. Pour compter l'usage :
  `gh api search/commits -f q='"Forkstify:"' --jq .total_count` (dépôts
  publics, branche par défaut) et `forks_count` du dépôt de référence.

**Reste à faire de la main de Joel** : lancer forkstify sur l'autre poste —
il commitera son appris, rebasera sur ce que ce poste a poussé, et le
pilote fusionnera. Ce poste a poussé le sien à l'occasion de ce commit.

## La discographie s'ouvre en modale (07/09/2026)

Joel, après une graine Cat Power : « je n'aime quasiment que des morceaux
de l'album *What Would the Community Think* ; j'aurais aimé une commande
pour avoir la liste visuelle des morceaux classés par albums, et pouvoir
faire des `tt` sur ceux que j'aime et `tT` sur les tops que je veux
enlever. » Puis, sur la maquette : « partons sur `ad` et une modale », forme
**1a** de `Discographie.dc.html`.

Conçu dans
[`conception/exploration-d-un-artiste.md`](conception/exploration-d-un-artiste.md),
câblé le jour même :

- **`ad` (ou `:discography`) pose une modale sur l'écoute** — elle ne la
  remplace pas, le son ne cesse pas, l'en-tête et le pied restent. Elle vise
  l'artiste de la ligne **surlignée**, celui du morceau en cours à défaut.
- **Les albums sont pliés** : cent quatre-vingt-sept titres deviennent douze
  lignes, celui du curseur s'ouvre seul, avec sa part des écoutes en jauge.
  L'en-tête répond à la question qu'on vient poser — « 2 albums portent
  79 % des 118 écoutes — 6 albums jamais ouverts ».
- **La modale a sa propre table** (`keys::parse_modal`, la première du
  produit) : `j`/`k` descendent, `h`/`l` plient et déplient, `tt`/`tT`
  corrigent les tops, `A` promeut les quatre titres les plus écoutés de
  l'album, `tl`/`tb` mesurent, `e` met à la file, `s` change l'ordre, `v` la
  vue (tout, ♪, ♥, ⊘), `/` filtre, `u` défait, ⏎ écrit, échap ferme.
- **Une fournée, un commit** (`edit::set_tops`) : les éditions s'accumulent
  en bas avec le sujet du commit à venir, et partent en une écriture. Le
  premier échap prévient s'il en reste. Cela ne contredit pas 0017, qui
  porte sur l'appris : mesures et éditions n'ont jamais eu la même règle.
- **Les tops que la discographie ne rend pas** tombent en fin de liste
  (« tops hors discographie ») et restent retirables : c'est là qu'une
  fiche générée se relit.
- **Le cache de la traîne** gagne la date, le rang, la durée et le type
  (album/single) ; une récolte d'avant est refaite en silence — le cache est
  régénérable et hors dépôt.

Dix tests de plus (dédoublonnage des rééditions, appariement des tops par
titre normalisé, gestes contraires qui s'annulent, curseur qui survit au
pliage, et un rendu complet de la modale). **Non vérifié en session
réelle.**

## Le pied ne grandit jamais, le geste se voit dans la liste (07/09/2026)

Joel, après un `e3` : « il ne m'a ajouté qu'un morceau ; je voudrais
enlever la notification de l'action : le pied ne grandit jamais, et le geste
est vérifiable — on voit ce qui a été ajouté par une icône spéciale devant
le morceau, ↻ à la place de →. Une action sans effet visible dans la liste
(un ban, une édition de fiche, une erreur de lecture) doit quand même
apparaître ; il faut mettre en forme un peu plus cette notification. »

- **Ce qui se voit ne se dit plus** : prendre une branche (elle apparaît
  avec sa raison), un encore (ses morceaux portent **`↻`** en jaune à la
  place de `→` — `Stop::encore`), retirer de la file, pause et reprise (le
  glyphe du pied). Neuf `say!` tombent.
- **Ce qui ne se voit pas s'affiche mieux** : la ligne « dernière chose
  dite » est mise en forme par nature, au glyphe qui l'ouvre, comme le
  composant Notice du design system — `✓` vert, `⏹` `⊘` rouge, `↻` `⚑`
  jaune, `→` magenta, une parenthèse en gris, « pas encore câblé » en
  italique estompé ; le détail après « — » ou entre parenthèses finales
  s'estompe. Testé.
- **L'encore qui n'ajoute qu'un morceau.** Deux causes possibles, toutes
  deux traitées. La récolte de la traîne se décidait sur le nombre de tops
  de la fiche moins un plafond global, pas sur les tops **non joués de cet
  artiste** — corrigé. Et quand la traîne manquait (pas d'identifiant
  Spotify, API injoignable, traîne fermée au cocon ou épuisée), l'encore
  servait ce qu'il pouvait **sans le dire** — la ligne unique du pied étant
  aussitôt écrasée par « ↻ encore… ». Désormais `harvest` rend son résultat
  au lieu de parler, et l'encore ne parle **que** s'il ne sert pas la
  demande : « (1 seulement chez X — sa traîne est épuisée) ». `:warm` dit
  `✓ discographie de X — n titres en cache` ou `⏹ …`.

## L'aide à la saisie (07/09/2026)

Joel : « je veux changer le fonctionnement de la fenêtre des raccourcis :
on l'ouvre avec espace et on la ferme avec échap ; si je l'ouvre et que
j'appuie sur e, cela affiche la fenêtre des raccourcis de e, je dois pouvoir
revenir en arrière ; si j'appuie ensuite sur 3, cela déclenche l'action
voulue — ce n'est plus une simple fenêtre de raccourcis, mais une aide à la
saisie. »

C'est which-key pour de vrai. Le leader ne **vidait** plus rien : il
effaçait la séquence en cours pour montrer un menu, et il fallait tout
retaper. Désormais :

- **espace** ouvre l'aide sur le niveau en cours (tout, ou le namespace à
  moitié tapé) **et laisse la séquence en cours** ; espace au niveau
  d'entrée la referme, **échap** la ferme de partout.
- **Chaque touche tapée dans l'aide passe par la grammaire** : `e` fait
  descendre l'aide au niveau de e (la session suit `Cmd::Pending`), `3`
  complète `e3` — la commande part et l'aide se ferme.
- **⌫ remonte d'un niveau** : le lecteur de touches efface la dernière
  touche de la séquence et le dit ; hors de l'aide, c'est simplement
  effacer.

Le pied de chaque niveau le rappelle : « une touche = l'action · ⌫ retour ·
échap fermer ». Les entrées du niveau d'entrée disent « tape f pour ses
touches » au lieu de « espace pour le détail ».

Côté code, `Live::help_open` est le seul état ajouté ; un bloc posé sur
l'écran tombait au geste suivant, l'aide fait exception tant que la séquence
n'a pas abouti. Non vérifié en session réelle.

## L'écran de lecture d'après la maquette 2b (07/09/2026)

Joel, `Lecture.dc.html` enrichie de deux variantes d'en-tête et de pied :
« retravaille l'écran de lecture d'après la maquette **2b**. Finalement on
enlève le filet vertical. Concernant la colonne des branches, la
présentation que nous avons me convient, ne la modifie pas. Passe juste les
raisons en gris. »

2b : « la graine en bloc — et une progression pleine largeur ». Le parcours
d'artistes disparaît du haut (la liste jouée le dit déjà) ; à sa place, une
ligne d'en-tête et le bloc de la graine ; en bas, ce qui sonne et ce qui
suit, puis l'invite. L'écran tient désormais ainsi :

    forkstify ecouter the-cure     segment 2 · 6 morceaux · 3 à venir · confort 3 ███░░ équilibré

    ── graine ──────────────
    The Cure  [catalogue]  fiche écrite · 41 liens · 12 tops  dernière écoute -3s
    1 embranchement depuis — 2 artistes traversés
          ♪ A Forest — The Cure  graine : the-cure        1 écoute · hier │ ── branches 2
     1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees        jamais joué │   1  The Creatures
     2 →  ♪ Israel — Siouxsie and the Banshees                jamais joué │      membres en commun — …
                                                                          │      ♪ Right Now — The Creatures
     3    horizon  rien de tiré au-delà — 1-3 pour ajouter une branche    │      █████ membres en commun
    ▶ Cities in Dust — Siouxsie and the Banshees  (2 / 3)                 ♪ top │ 2:34 / 3:47 -1:13
    ████████████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
    à suivre  Israel — Siouxsie and the Banshees                  → embranchement dans 1 morceau
    (la dernière chose dite)
    [1-3 branche · h/l · p · espace = les touches · q] █

- **L'en-tête sur une ligne** : la commande à gauche, l'état à droite —
  segment, morceaux, à venir, et la jauge de confort, qui quitte l'invite
  (elle s'inverse là quand `c` la règle).
- **La graine en bloc**, sous sa règle : le nom, `[catalogue]`, ce que dit
  sa fiche (écrite ou générée, liens, tops — `Card` lit désormais
  `generated`), sa dernière écoute d'après `learned/`, et le compte des
  embranchements pris et des artistes traversés.
- **La liste sans filet** : le demi-trait du matin tombe, les morceaux se
  suivent ; le numéro passe en gris, seule la flèche garde sa couleur ; un
  blanc avant l'horizon.
- **Le pied** : ce qui sonne avec sa position dans la liste et sa
  provenance en toutes lettres (`♪ top`, `♥ aimé`, `↳ door`, `· traîne`…
  — `Source::word()`), puis ce qui suit et « embranchement dans n
  morceaux ». La dernière chose dite garde sa ligne, l'invite reste la
  dernière avec un curseur bloc.
- **La colonne des branches ne change pas**, sauf le mot sous la jauge
  (`membres en commun`, `0.78`) qui passe en gris comme la raison : seules
  les cellules de la jauge disent encore la nature du lien.

**La progression, finalement** (Joel : « on ne peut vraiment pas avoir une
barre de progression ? »). Si : librespot dit la position à chaque
démarrage, pause et saut (`Playing`, `Paused`, `Seeked`,
`PositionCorrection`) et la durée à chaque changement de piste
(`TrackChanged`). `Live::follow_needle` les suit — pour la requête en cours
seulement, un morceau sauté parle encore — et **un tic par seconde**
redessine tant que ça joue, la position étant extrapolée depuis le dernier
échantillon. Le pied gagne `♪ top │ 2:34 / 3:47 -1:13` et une **barre
pleine largeur** en cyan, comme le module media de waybar. Sans donnée
encore (avant le premier `Playing`), la barre est vide et les temps
absents.

**Pas fait, faute de donnée** : « 1 door écartée par le confort » — le
moteur ne compte pas ce qu'il écarte.

## La liste de lecture sur la grille de la chaîne (07/09/2026)

Joel, maquette `File d'attente.dc.html` : « est-ce qu'on peut présenter la
liste de lecture un peu comme la maquette **3a** ? C'était une ancienne
maquette du projet de file d'attente avorté, mais j'aime bien le graphisme
et je veux le récupérer pour la liste de lecture. » Puis, sur un premier
essai qui faisait un maillon par branche : « je voudrais reprendre l'idée de
numérotation devant les morceaux en cours et à venir, et la séparation avec
le tiret vertical. Si on peut trouver des petites stats ou infos
intéressantes à mettre sur le côté en gris comme sur la maquette, ce serait
pas mal. »

3a dessinait « la chaîne » sur une grille à quatre colonnes : le numéro et
la flèche, le nom, la raison en gris, une info à droite ; un filet `│` entre
les lignes ; l'horizon au bout. Le mode file d'attente est tombé le 06/09,
mais la grille convient à ce que l'axe est devenu. **Chaque morceau y est
une ligne** :

          ♪ A Forest — The Cure  graine : the-cure          1 écoute · hier
     ╵
          ♪ Push — The Cure                                     jamais joué
     ╵
     1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees   liens…    3 écoutes · -2s
     ╵
     2 →  ♪ Israel — Siouxsie and the Banshees                  jamais joué
     ╵
     3 →  ♪ Right Now — The Creatures  membres en commun        jamais joué
     ╵
     4    horizon  rien de tiré au-delà — 1-3 pour ajouter une branche

- **Ce qui sonne est le 1, ce qui vient compte à partir de lui** — `▶` (ou
  `⏸`) en vert sur le courant, `→` en magenta sur la suite. Ce qui a sonné
  n'est pas numéroté et s'estompe.
- ~~Un filet entre chaque morceau~~ — essayé en `│` puis en demi-trait `╵`,
  retiré le soir même avec la maquette 2b.
- **La raison de la branche se lit en gris à côté du morceau qui l'ouvre**
  (en cyan quand elle vient des vecteurs), la graine à côté du premier. La
  tête d'une branche porte donc désormais sa raison avec son nom
  (`engine::Head { label, reason }`) ; `tx` sur une tête les passe au
  morceau suivant.
- **À droite, ce que l'écoute sait du morceau** : `3 écoutes · -2s`,
  `jamais joué`, `passé 2×`, `hors catalogue` — lus dans `learned/`
  (`track_stats`, compteurs décrus à aujourd'hui), avec l'âge écrit comme
  dans la collection de l'accueil. Ce sont les seules stats par morceau que
  l'application possède : pas de durée (l'API ne nous la rend pas), pas de
  branches écartées (le moteur ne les compte pas).
- **L'horizon** remplace « plus rien à suivre ». L'en-tête dit
  « n morceaux · k à venir ».
- **Rien ne change aux gestes** : la sélection ↑↓, entrée et `tx` portent
  sur les mêmes lignes qu'avant.

Les colonnes se serrent d'elles-mêmes : la raison prend ce qui reste entre
le morceau et la note, et s'efface sous huit cellules. À 100 colonnes
(59 pour l'axe), la note passe, la raison rarement ; à 160, tout se lit.
Vérifié par un test de rendu à 160 colonnes ; pas encore tourné en session
réelle.

## Les branches se déplient dans leur colonne (07/09/2026)

Joel, maquette `Lecture.dc.html` (Claude Design) à l'appui : « passe plutôt
l'affichage des forks en colonne à droite comme sur la maquette **1a**. Par
contre, je veux que les morceaux de chaque branche soient affichés pour
pouvoir faire son choix en connaissance de cause. »

Le volet du 06/09 avait la permanence de 1a mais le **placement de 1b** — un
bloc encadré, posé en bas de sa colonne. Il devient la colonne elle-même :
**toute la hauteur**, un filet `│` à gauche pour la séparer de l'axe (la
seule règle qu'elle trace — le système interdit les cadres), `── branches 3`
en tête, et les touches au pied (`1-3 prendre · fr reproposer`, `fn1 sans
attendre la fin`), qui ne bougent pas.

**Chaque branche est dépliée** comme le composant `Branch` du design system
l'écrit : le numéro et le nom, la raison repliée à cinq cellules, puis **ses
morceaux** — ils sont déjà tirés au moment de la proposition, il n'y avait
aucune raison de les cacher — avec leur provenance (`♪` `♥` `↳` `·`), le
titre devant, l'artiste derrière en bleu. Dessous, la **jauge de proximité**
de la maquette : `█████ membres en commun` pour un lien du graphe (bleu,
catalogue), `████░ 0.78` pour la branche aventureuse (cyan, vecteurs) — le
poids que le moteur porte déjà sur l'échelle 1–5. « Rester dans l'univers du
parcours » n'est dit qu'une fois, par la raison ; la jauge se tait.

Partage **60/40, le même que l'accueil** (Joel, 07/09/2026) — la constante
`LEFT_SHARE` sert aux deux écrans, et sous 60 colonnes la droite s'efface
sur les deux. La maquette donnait 38 colonnes fixes à la droite ; en
proportion, un terminal de 100 colonnes lui en donne 40, un de 160 en donne
64 et les morceaux longs ne se coupent plus. Une liste se coupe, une raison
se replie — le repli est fait à la main, ratatui n'y touche pas.

**Écarté, faute de donnée** : la ligne « ↳ 1 door écartée par le confort »
de la maquette. Le moteur ne compte pas ce qu'il écarte ; l'inventer serait
mentir.

**Vérifié par trois tests de rendu** (`TestBackend`, 100×30 et 70×30) — la
colonne, ses morceaux, ses jauges, ses touches au pied ; et un terminal
étroit qui garde l'axe. Pas encore tourné en session réelle.

## Le catalogue parle anglais, et s'importe (06/09/2026)

**Renommage.** `AGENTS.md` impose l'anglais pour « tout ce qui est interface
publique du dépôt — chemins, sous-dossiers », et
[0010](decisions/0010-format-revise-links-sans-portes.md) le redit du format.
Les chemins étaient pourtant restés en français ; seuls `learned/` (0014) et
les champs des fiches suivaient la règle. Corrigé : `fiches/` → **`cards/`**
(le code appelle déjà ça une `Card`), `outillage/` → **`tools/`**,
`vecteurs/` → **`vectors/`**, `catalogue.toml` → **`catalog.toml`**.
Le vocabulaire français du projet ne bouge pas — on dit toujours « une
fiche », c'est le chemin sur disque qui parle anglais.

Les **décisions restent intactes** : elles mentionnent les anciens noms et
sont immuables. Une note en tête de
[`keybindings.md`](keybindings.md) dit d'y lire les nouveaux, comme 0014
l'avait fait pour `usage/` → `learned/`.

**`forkstify import <url>`.** Reprendre les fiches d'un autre catalogue :
ajoute le remote, récupère, prend **les fiches qu'on n'a pas** — jamais
celles qu'on a, ses corrections sur nos artistes relevant d'une PR où l'on
discute — commite le tout en une fois, puis régénère les vecteurs.

C'est une **sous-commande, pas un geste d'écoute** : la vectorisation
demande un conteneur et plusieurs minutes. Si docker manque, la commande
exacte s'affiche plutôt que d'échouer en silence — sans vecteurs à jour, les
fiches reprises n'existeraient que pour le graphe.

## La première installation, question ouverte (06/09/2026)

Joel : « que se passe-t-il lorsqu'un nouvel utilisateur installera forkstify
pour la première fois ? » et surtout « comment conjuguer catalogue de base
commun et modifications de l'utilisateur ? ».

La seconde est **déjà tranchée**, mais éparpillée entre 0002, 0004, 0008 et
0014 : on ne conjugue pas, on **superpose dans le même dépôt** et git fait le
travail — la base vient de l'amont, le mien est mes commits par-dessus,
l'appris vit dans `learned/` et n'est jamais reversé. Rassemblé dans
[`premiere-installation.md`](conception/premiere-installation.md).

Ce qui manque est l'**amorce** : rien ne clone le catalogue, rien ne scanne
la bibliothèque de l'utilisateur, rien ne génère une fiche à la volée. Et un
point dur qui n'était écrit nulle part : **la base actuelle n'est pas neutre,
c'est l'univers de Joel** — 214 fiches nées de son classement. Un nouvel
utilisateur au goût éloigné ne pourrait presque rien démarrer, puisqu'une
graine sans fiche ne démarre pas.

## La collection entière, à droite de l'accueil (06/09/2026)

Demande de Joel, d'après la colonne ajoutée à `Accueil.dc.html` : à gauche
ce que forkstify **propose**, à droite ce qu'il **possède**. « La colonne ne
propose rien : elle liste. »

Le partage est **proportionnel, 60/40** (Joel, 06/09/2026) : la gauche porte
des raisons et des morceaux, la droite une liste. Une seule constante à
changer pour passer à moitié-moitié (`LEFT_SHARE`). Sous 60 colonnes la
liste s'efface — mieux vaut une colonne lisible que deux illisibles.

`gg` et `G` sautent aux deux bouts, comme dans vim — dans la collection à
l'accueil, et dans l'axe en écoute, puisque c'est le même geste sur le même
genre de liste. `g` seul n'est rien : il attend son second, et le test
exhaustif de `keys.rs` vérifie que la grammaire reste sans préfixe.

Elle réunit le catalogue **et** le classement — 780 noms, dont 214 avec
fiche — avec pour chacun une jauge de familiarité, son nom, et depuis quand
il n'a pas sonné (`auj.`, `hier`, `-3s`, `-7m`, `jamais`, en orange au-delà
de six mois). `s` fait tourner l'ordre : familiarité, a-z, dernière écoute.

**Deux écarts avec la maquette, tous deux assumés.**

La maquette entre dans la colonne par `c` — or `c` est le réglage du confort
depuis la veille. Plutôt que de déplacer l'un des deux, **il n'y a pas de
touche pour entrer** : les flèches ↑↓ y déplacent un curseur, et `entrée`
démarre l'artiste sous lui. C'est exactement le geste de la session
(sélection puis entrée), et sans curseur `entrée` garde son sens de
toujours — « choisis pour moi ». Une touche de moins à apprendre.

La maquette dit qu'« un artiste sans fiche démarre quand même : la première
branche vient des vecteurs ». **Ce n'est pas vrai chez nous** : le moteur
part de la fiche (liens, tags, vecteur), et un artiste du seul classement
n'en a aucun. La colonne les liste — c'est bien la collection entière — mais
les choisir répond que le catalogue grandit avec l'usage, plutôt que
d'échouer sans le dire.

## Les éditions écrivent enfin dans les fiches (06/09/2026)

`src/edit.rs` ferme le dernier tiers de
[0013](decisions/0013-affinage-clavier-mesure-ou-edition.md) : quatre des
cinq éditions modifient une fiche **et produisent un commit lisible**
(`tt`, `tT`, `td`, `aL`). Le message de commit et ce qui s'affiche à l'écran
sont **la même phrase** — ce que l'utilisateur lit est ce que git retiendra.

**Les fiches sont retouchées textuellement, jamais réécrites.** Une
relecture par serde perdrait tout ce que le code ne modélise pas —
`format`, `generated`, `mbid`, `spotify`, `begin`, `origin`, `description`,
l'ordre des clés, les guillemets choisis à la main. Une fiche est une
**interface publique** ([0002](decisions/0002-catalogue-partage-forkable.md))
qu'un humain lit et corrige : on y insère une ligne, on n'en régénère pas le
tout. Six tests couvrent ce qui se corromprait en silence, dont deux qui
vérifient qu'après chaque insertion **la fiche se relit encore comme une
fiche** — titre à guillemets compris.

Choix faits faute d'un moyen de saisie : `td` prend pour direction les tags
de l'**artiste suivant** dans la file (0011 : une door pointe vers des tags,
là où l'on va), et `aL` lie à l'artiste **d'où l'on vient**. Les deux se
relisent et se corrigent dans la fiche.

**`ae` reste la cinquième**, pour une raison d'architecture : ouvrir
`$EDITOR` demande de rendre l'entrée au terminal, or le lecteur de touches
tient `stdin` en permanence et lui volerait ses frappes. Il affiche le
chemin de la fiche en attendant une saisie interrogée plutôt qu'un fil
bloqué.

**Limite dite à l'écran** : le catalogue en mémoire ne bouge pas, donc une
édition ne compte pour le moteur qu'au prochain lancement.

## L'accueil rapproché de sa maquette (06/09/2026)

Retouches graphiques demandées par Joel, toutes tirées de
`Accueil.dc.html` :

- **Le mot-marque et l'état sur une seule ligne** — `forkstify` à gauche,
  `✓ librespot · ✓ api web` à droite. Faute de justification en cellules,
  l'écart est calculé depuis la largeur de la zone.
- **Les liserés courent jusqu'au bout de la mesure** (66 colonnes au plus).
  Ce sont eux qui séparent les blocs, puisque le système interdit les cartes.
- **Le titre devant, l'artiste derrière**, partout : sous une entrée
  (`♪ Crystal Frontier — Calexico`), sur la graine qui est un morceau, et
  sur la ligne « reprendre » — qui les avait encore dans l'autre sens.
- **Les couleurs sont des rôles** : le numéro et les touches en magenta
  (branche), l'artiste en bleu (catalogue, écrit par un humain), le `♪` en
  vert, les raisons en gris, la ponctuation estompée.

## La file s'enchaîne : le parcours devient une playlist (06/09/2026)

Joel, après avoir vécu la TUI : « j'aimerais que le choix d'une nouvelle
branche s'ajoute à ce qui a été décidé auparavant. Actuellement, cela le
remplace. Ainsi on peut construire une playlist rapidement en quelques choix
de branches. »

C'est un changement de modèle, et il est plus juste : **choisir une branche
l'ajoute à la file** au lieu de la remplacer, et les branches suivantes se
proposent depuis le **bout** de la file, pas depuis ce qui sonne. C'est la
« chaîne » des maquettes de file d'attente, obtenue sans mode séparé.

Conséquences :

- **Il n'y a plus de branche « en attente ».** Tout ce qui est décidé est
  dans la file — `pending_branch` disparaît, et `next()` se réduit à
  « avancer, sinon tirer ». `fn<n>` et `f!<n>` gardent leur sens : insérer
  après le morceau en cours, ou en retirant le reste.
- **Chaque branche s'ouvre par son nom dans la file** : le premier de ses
  morceaux porte l'étiquette (`Stop::head`), les suivants un filet magenta.
  On voit donc plusieurs branches empilées, chacune délimitée.
- **Tout le passé reste à l'écran** — c'est la playlist en train de se
  faire, pas un historique à oublier. L'axe défile pour suivre ce qui joue.
- **`tx` retire de la file** le morceau sélectionné. Il reste proposable :
  ce n'est pas un ban, c'est un « pas dans cette soirée ». Si c'était la tête
  d'une branche, le suivant en reprend le nom.
- **Plus de repli de ligne dans l'axe** : une liste se coupe, elle ne se
  replie pas. C'était le défaut signalé — la colonne s'étant rétrécie avec le
  volet des branches, les longs libellés passaient à la ligne.

**Reste ouvert** : sauvegarder la playlist. Tout ce qu'il faut est là — le
passé, la file, les noms de branches — mais où l'écrire et sous quel format
n'est pas tranché (une playlist Spotify ? un fichier du catalogue ?).

## La branche retenue se déplie dans « à suivre » (06/09/2026)

Joel : « quand je choisis une branche, le parcours s'affiche en dessous des
chansons à suivre mais sur une ligne sans détail ». Elle tenait en effet sur
un `→ label`, alors que ses morceaux sont **déjà tirés** au moment du choix
— il n'y avait aucune raison de les cacher.

Ils se rangent maintenant à la suite de la file, séparés d'elle par un
**filet magenta** (`│`) qui remplace l'indentation : on voit d'un coup où la
branche commence et jusqu'où elle va. Ils sont estompés, parce qu'ils n'ont
pas encore sonné, et la ligne de tête dit **quand** la branche prendra la
main — à la fin de la branche, à la fin du morceau, ou en retirant le reste.

**Limite assumée** : la sélection (↑↓) s'arrête au bout de la file. Les
morceaux de la branche s'affichent mais ne se choisissent pas encore — ils
ne sont pas dans la file tant que la branche n'a pas pris la main, et
prétendre le contraire ferait un surlignage qui ment.

## Les branches sont toujours à l'écran (06/09/2026)

Demande de Joel : « je voudrais que le panel des prochaines branches soit
tout le temps affiché ». C'est la variante **1a** des maquettes — le volet
permanent — avec le placement de 1b, en bas à droite.

Une nuance a été ajoutée à la demande : **le volet a désormais sa place
réservée** au lieu d'être posé sur l'axe. Un volet flottant en permanence
masquerait le bas de la file pour toujours ; une colonne réservée ne cache
rien. L'axe prend ce qui reste à gauche, le volet 54 colonnes à droite — et
sous 48 colonnes de large, il s'efface plutôt que d'écraser l'axe.

Conséquences : `fp` (*peek*) perd son emploi et le dit au lieu de ne rien
faire ; un cul-de-sac s'affiche dans le volet au lieu de le laisser vide.

## Plus rien ne s'imprime sous la TUI (06/09/2026)

Joel, au test suivant : « quand j'appuie sur 2, le bas bouge toujours », avec
des textes qui se chevauchent (« ▶ ♪ Vilaine — Odezenne forkstify (touches
multimédia actives…) »).

La cause était plus large que le démarrage. **Trois familles d'impressions
écrivaient dans l'écran alterné à l'insu de ratatui**, qui ne redessine que
ce qu'il croit avoir changé — d'où les restes :

1. les quatre messages de démarrage d'une session (appris, connexion, MPRIS) ;
2. les deux messages d'autorisation de `spotify.rs` ;
3. et surtout **le lecteur de touches lui-même** : l'écho des séquences à
   moitié tapées, la ligne éditée après `/` ou `:`, les effacements.

Le lecteur ne dit plus rien à l'écran : il **remonte son état** —
`Cmd::Pending` pour une séquence en cours, `Cmd::Typing` pour une ligne,
`Cmd::Unknown` pour une séquence sans emploi — et c'est la TUI qui l'affiche,
sur la ligne d'invite. Le bas de l'écran ne bouge donc plus que pour montrer
**la commande en cours**, ce qui était la demande exacte.

Le démarrage d'une session a désormais son écran d'attente, avec ses étapes
cochées à mesure (`Tui::splash`), et `Tui::clear()` repart d'un écran vide à
chaque changement de vue.

## Premiers retours d'usage sur la TUI (06/09/2026)

Cinq retours de Joel après la première vraie prise en main, tous appliqués.

- **L'échelle de confort est retournée** : **5 = cocon, 0 = exploration**.
  « Le confort, c'est ce qu'on connaît bien. » Le dépôt était l'intrus :
  [0012](decisions/0012-rotation-des-morceaux.md) §4 écrivait déjà « confort
  haut : tirage serré sur les tops », ce qui se lit maintenant au pied de la
  lettre. Seule `zone-de-confort.md` disait l'inverse, et une note de
  conception cède devant l'usage. Défaut par défaut : 3.
- **La jauge se règle au clavier** : `c` ouvre le réglage, ↑↓ bougent,
  entrée valide, échap rend la valeur d'avant. Rien n'est appliqué avant la
  validation.
- **Le bas de l'écran ne bouge plus.** Le journal s'allongeait vers le bas et
  devenait brouillon ; il tient désormais sur **une ligne fixe** (la dernière
  chose dite), et ce qui est long — le menu du leader, `?` — se **pose sur
  l'écran** en bloc au lieu de descendre.
- **La navigation devient verticale et différée.** L'axe s'affiche
  verticalement, donc ↑↓ y déplacent une **sélection** surlignée en jaune ;
  **entrée** joue ce qui est sélectionné, échap annule. Sans sélection,
  entrée garde son sens de toujours — tirer une branche. ←→ et `h`/`l`
  restent le geste de transport immédiat, comme ⏮ ⏭.

**Et un défaut trouvé en lisant l'écran** : une seule écoute réelle
*remplaçait* le classement, si bien que jouer une fois son artiste préféré
le faisait tomber de 100 % à 13 % de familiarité. Le classement et l'écoute
se combinent désormais par le maximum — écouter ne peut qu'ajouter.

## La TUI, première version (06/09/2026)

`ratatui` entre dans le projet (décision [0006](decisions/0006-rust.md), qui
le nommait déjà). `src/tui.rs` dessine la session ; **la saisie reste celle
de `keys.rs`** — termios brut, grammaire sans préfixe — parce qu'elle est
éprouvée et que ratatui n'a pas besoin de posséder l'entrée.

**Variante 1b des maquettes** : une colonne pleine largeur pour l'axe de
lecture, et un volet qui se pose dessus à l'embranchement puis s'en va
(`Clear` + `Block::bordered`). Passer à 1a — le volet permanent — ne
changera qu'un `Layout` ; c'est resté ouvert exprès.

L'écran tient en quatre zones : l'en-tête (le parcours, la graine, le
segment, le confort), **l'axe** (ce qui a sonné, ce qui sonne en inversion,
ce qui suit), le **journal** de ce que forkstify vient de dire, et l'invite
en dernière ligne avec la jauge de confort — la place que
[`ecran-d-accueil.md`](conception/ecran-d-accueil.md) lui donnait.

Conséquence sur le code : les **69 impressions** de la session sont devenues
des lignes de journal (`say!`), vidées à chaque commande pour qu'un bloc —
le menu du leader, `?` — s'affiche seul et en entier. Le journal est un
`RefCell` : dire quelque chose ne demande pas d'emprunt exclusif, ce qui
évitait de rendre mutables trente méthodes qui ne le sont pas.

**La couture est levée le jour même** (Joel l'a vue au premier lancement :
« quand je le lance, j'ai du terminal basique »). L'accueil passe lui aussi
par la TUI : **un seul écran alterné pour toute l'application**, ouvert par
`accueil()` et prêté à la session. L'accueil décide *quoi* dire — une liste
de `Row` — et la TUI *comment* : la même séparation qu'entre le moteur et le
son. Le parcours final remonte à l'accueil au lieu de s'imprimer sur un
écran qui disparaît.

**Non vérifié** : rien de tout cela n'a tourné en session réelle. La TUI est
vérifiée à la compilation ; l'accueil, lui, tourne.

## Retours d'usage (05/09/2026)

Les premières sessions longues d'`ecouter` ont produit **11 retours** de
Joel, consignés et instruits dans
[`docs/conception/retours-usage.md`](conception/retours-usage.md) — avec
l'**inventaire réel des raccourcis** (ce qui marche vs la table projetée,
largement non implémentée) et **5 points à trancher** avant d'ajouter quoi
que ce soit. Ils se traitent au fur et à mesure.

## Prochaines étapes, dans l'ordre

Relu le 05/09/2026 au soir, contre le code. Les étapes 2 et 4 de la liste
précédente sont largement faites ; ce qui suit est ce qui reste.

1. **Éprouver `ecouter` en vrai.** Entamé le 09/09/2026 : une longue
   session de Joel, dix retours traités le jour même (section ci-dessus),
   et les premiers atouts dans `docs/atouts.md`. Reste à éprouver : les
   mesures qui écrivent dans les fiches, le réservoir, `:warm`.
2. **Les cinq éditions** — `tt`/`tT` (tops), `td` (door), `ae` ($EDITOR),
   `aL` (lier deux artistes). Elles touchent une **fiche**, pas l'appris.
   La couche qui écrit et commite le catalogue **existe depuis le
   06/09/2026** (`src/edit.rs`, et `create_card` depuis le 09/09) : `tt`,
   `tT`, `td` et `aL` sont câblées ; il reste `ae` ($EDITOR), que le lecteur
   de touches empêche. C'est le dernier tiers de [0013](decisions/0013-affinage-clavier-mesure-ou-edition.md)
   et le retour n° 8 de Joel.
3. ~~**Le cooldown daté**~~ — fait le 08/09/2026 (un dixième le jour même,
   demi-vie d'une semaine).
4. **`u` — annuler le dernier geste** ([0013](decisions/0013-affinage-clavier-mesure-ou-edition.md)).
   Sans lui, un `tb` de travers ne se reprend qu'à la main dans le TOML.
   Il devient nécessaire dès que les éditions arrivent (revert d'un commit).
5. **Le mode file d'attente** (`Q`, retour n° 11). Le plus gros morceau :
   `rounds` est une **liste plate**, alors que « retirer toute la profondeur
   d'une branche » suppose un arbre manipulable.
6. ~~**La synchronisation git**~~ — faite le 07/09/2026 (0017).
7. ~~**Trancher la vectorisation d'une fiche générée**~~ — tranché le
   09/09/2026 ([0019](decisions/0019-vectorisation-par-l-application.md)) :
   l'application vectorise elle-même, fait le jour même.
8. ~~**`fw` — partir hors de l'univers**~~ (retour n° 6) — tranché et fait
   le 11/09/2026 : sortir du cluster, avec `fw <artiste>` pour viser un
   univers.
13. ~~**Le setup fluide**~~ (Joel, 09/09/2026) — fait le 20/09/2026
    d'après la maquette `Installation.dc.html` (section ci-dessus) ; les
    scripts Python se retirent après le premier `:library` en vrai.
14. **La barre Omarchy** (Joel, 10/09/2026) : l'animation `▂▄▆` dans la
    barre, et au clic une popover titre / artiste / progression / prochain
    morceau. Deux étages proposés dans
    [barre-omarchy.md](conception/barre-omarchy.md) : forkstify publie
    d'abord de vraies métadonnées MPRIS (portable), puis un petit plugin
    Omarchy les montre. **Fait le 10/09/2026** ([0021](decisions/0021-le-depot-est-le-plugin-omarchy.md)) :
    le dépôt est le plugin, l'état vit sous `~/.local/state/forkstify`, le
    widget installe et lance le binaire. Éprouvé : la progression de
    la carte, figée à 0:00, se rafraîchit depuis le 11/09/2026.
9. **Trousseau GNOME** pour les jetons, au lieu des caches `target/` — un
   `cargo clean` efface aujourd'hui l'authentification.
10. **La vraie TUI** : l'écran ne se redessine pas, tout défile. La saisie
   touche par touche est faite, l'affichage reste celui d'un terminal qui
   déroule.
11. ~~**Traduire en anglais** les scripts de `tools/`~~ — retirés le
    20/09/2026 : forkstify récolte, classe, génère et vectorise lui-même.
15. **Les trois chantiers de la sortie** (Joel, 19/09/2026) — cahier dans
    [conception/sortie.md](conception/sortie.md). ~~Les trois~~ faits le
    20/09/2026. Le même jour, `tools/` retiré des deux dépôts du catalogue
    et l'appris de Joel retiré de la référence (Joel, 20/09/2026). Reste à
    éprouver en vrai : `:library`, `Cp`, `Cu` — le premier `Cu` du fork
    réglera seul les conflits `learned/` (les siens gardés). L'action
    GitHub et le `CONTRIBUTING.md` sont en place le même jour, et
    l'application est **publique** depuis (README, LICENSE).
12. ~~**Explorer la discographie d'un artiste**~~ — faite le 07/09/2026
    (`ad`, modale 1a). La cible de `t`/`a`/`e` est unifiée depuis le
    09/09/2026 ([0020](decisions/0020-la-cible-d-un-geste.md)).

## Corrections en attente (petites)

- MBID de **Les Thugs** introuvable (homonymie probable) et une entrée au
  nom vide dans `learned/mbid.json` ; 110 MBID résolus « par nom » à relire.
- **Les commits d'édition parlent français** (« Cat Power — tops : +2 −0 »,
  « Jacques Brel — fiche générée ») alors que `AGENTS.md` veut l'anglais pour
  les messages que l'application produit — `learned:` et `import:` le sont.
  `edit::Edit.summary` sert à la fois de phrase à l'écran et de sujet de
  commit : les séparer, ou trancher la règle.
- `resoudre-mbid.py` ne lit que les fichiers Spotify — à adapter aux
  récoltes Deezer (`learned/amis/*-deezer.json`).
- **Le nom MusicBrainz n'est pas toujours celui de Spotify** (La Ruda /
  La Ruda Salska) et la résolution d'un morceau cherche par nom : alias,
  vérification de l'identifiant Spotify des résultats, ou nom Spotify à la
  génération — à trancher.

## Règles de session

- **Commits et push au fil de l'eau** sur `forkstify` et
  `forkstify-catalog` (demandé par Joel le 31/08/2026). Chorizo : toujours
  demander avant de pousser.
- Les scripts qui lisent le trousseau GNOME sont lancés **par Joel** avec le
  préfixe `!`.
