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

1. **Éprouver `ecouter` en vrai.** Rien de ce qui a été livré le 05/09 n'a
   tourné dans une session : le mode brut, les sept mesures qui **écrivent
   dans le catalogue**, le réservoir, le confort, la traîne. C'est le
   premier geste, et il passe avant tout ajout. `:warm` est le test le plus
   court de la récolte.
2. **Les cinq éditions** — `tt`/`tT` (tops), `td` (door), `ae` ($EDITOR),
   `aL` (lier deux artistes). Elles touchent une **fiche**, pas l'appris, et
   demandent la couche qui écrit et commite le catalogue, qui n'existe pas.
   C'est le dernier tiers de [0013](decisions/0013-affinage-clavier-mesure-ou-edition.md)
   et le retour n° 8 de Joel.
3. **Le cooldown daté** ([0012](decisions/0012-rotation-des-morceaux.md) §2) :
   `learned/` date chaque écoute par morceau, le réservoir ne s'en sert pas
   encore. Un morceau joué hier devrait reculer, la pénalité décroissant
   avec le temps. Les données sont là, la formule reste à régler.
4. **`u` — annuler le dernier geste** ([0013](decisions/0013-affinage-clavier-mesure-ou-edition.md)).
   Sans lui, un `tb` de travers ne se reprend qu'à la main dans le TOML.
   Il devient nécessaire dès que les éditions arrivent (revert d'un commit).
5. **Le mode file d'attente** (`Q`, retour n° 11). Le plus gros morceau :
   `rounds` est une **liste plate**, alors que « retirer toute la profondeur
   d'une branche » suppose un arbre manipulable.
6. **La synchronisation git** (`:sync`/`:push`/`:pull`, retour n° 10) : à
   concevoir — fusion des compteurs de `learned/` (une fusion textuelle n'a
   pas de sens sur des flottants décrus), fréquence, comportement hors ligne.
7. **`fw` — partir hors de l'univers** (retour n° 6), en attente de la
   clarification de Joel : sortir du cluster, ou repartir d'une graine ?
8. **Trousseau GNOME** pour les jetons, au lieu des caches `target/` — un
   `cargo clean` efface aujourd'hui l'authentification.
9. **La vraie TUI** : l'écran ne se redessine pas, tout défile. La saisie
   touche par touche est faite, l'affichage reste celui d'un terminal qui
   déroule.
10. **Traduire en anglais** les scripts d'`outillage/` écrits avant la règle
    de langue du code (à l'occasion).

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
