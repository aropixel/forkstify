# Spotify : lecture, authentification, API

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.
**Vérifié** = lu dans une source citée le 30/08/2026.

## Deux besoins distincts

1. **Faire sortir le son.** Spotify ne laisse jouer sa musique qu'à travers
   un client Spotify : l'appli officielle, le téléphone, une enceinte, ou un
   client compatible comme librespot. Un programme tiers ne lit jamais
   l'audio lui-même autrement.
2. **Savoir des choses et agir** : lire la bibliothèque de l'utilisateur
   (artistes et albums aimés, pour choisir la graine), chercher un morceau,
   résoudre un titre en identifiant, pousser dans la file. C'est l'**API
   Web**.

## Ce qui est vérifié

- **Modes de quota de l'API Web** ([doc officielle](https://developer.spotify.com/documentation/web-api/concepts/quota-modes)) :
  mode développement = « *Up to 5 authenticated Spotify users* », déclarés
  à la main, propriétaire Premium. Mode étendu, depuis mai 2025 : « *Spotify
  only accepts applications from organizations (not individuals)* »,
  entreprise établie, service lancé, « *at least 250k MAUs* ».
  **Conséquence** : forkstify ne peut pas obtenir de client id partageable.
- **Ce que fait l'écosystème libre** ([spotify-player](https://github.com/aome510/spotify-player),
  Rust, ratatui + rspotify + librespot) : « *By default, spotify-player uses
  ncspot's client ID* », un client id historique déjà en mode étendu, et
  déconseille d'en créer un : « *clients registered today start in the
  restricted default quota mode and commonly hit 429 / 403 errors* ».
  Zone grise tolérée, non garantie.
- **librespot** ([README](https://github.com/librespot-org/librespot)) :
  « *act as a Spotify Connect receiver* », découverte zeroconf incluse par
  défaut ; « *librespot only works with Spotify Premium. This will remain
  the case.* » Pas de dashboard développeur.
- **Le son sans rien installer** : spotify-player, via librespot, s'enregistre
  comme appareil Connect (« *registering a spotify-player device accessible
  via Spotify Connect* »). Forkstify peut faire pareil.

## Comment fait Omarchy-Spotify (lu dans le code, 30/08/2026)

[stappmus/Omarchy-Spotify](https://github.com/stappmus/Omarchy-Spotify),
MIT, ~23 000 lignes dont ~2 000 de Rust. Trois morceaux :

1. **Interface** : plugin QML dans le processus de la barre Omarchy
   (`Panel.qml` 6 600 lignes, `Service.qml` 4 000). Appelle l'API Web
   directement.
2. **Son** : `omarchy-spotify-backend`, processus Rust qui embarque un fork
   épinglé de librespot (`engine.rs`, ~800 lignes). C'est l'appareil Spotify
   Connect. Socket Unix privé en JSON par ligne (`load`, `add_to_queue`,
   `play`, `pause`, `seek`, `set_volume`…) + MPRIS. Lancé à la demande par
   une unité systemd utilisateur, arrêté après inactivité. `spotifyd` en
   repli, jamais en même temps.
3. **Données** : API Web depuis le QML.

**Authentification : deux OAuth PKCE dans le navigateur, aucun dashboard.**

- *API Web* : client id `d420a117a32841c2b3474932e49fb54b`, « *the public
  application identity also used by spotify-player and ncspot* » — celui de
  ncspot, historique, en mode étendu. Callback `127.0.0.1:8989/login`,
  refresh token dans GNOME Keyring, scopes limités aux fonctions visibles.
- *Son* : `backend authenticate` → `librespot_oauth` avec
  `SessionConfig::default().client_id`, c'est-à-dire **le client id du client
  officiel Spotify desktop** (`65b708073fc0480ea92a077233ca87bd`). Librespot
  se présente comme l'application officielle. Le jeton ouvre une session
  librespot, stockée comme identifiant réutilisable dans `~/.local/state`.
- **Pas de jeton d'API tiré de la session librespot** : deux grants, deux
  stockages. Indice fort que ce n'est pas simple — sinon un projet aussi
  soigné l'aurait fait.
- **Découverte zeroconf entrante désactivée** (`disable_discovery = true`) :
  on ne se connecte pas depuis le téléphone, on passe par le navigateur.
  Zeroconf est utilisé *vers l'extérieur* pour activer Sonos / JBL (helper
  Python de 700 lignes, échange de clés Diffie-Hellman à la main).

**Autres faits utiles :**

- Spotify a encore changé l'API en 2026 : endpoint *top tracks d'un artiste*
  supprimé, recherche limitée à 10 résultats, certaines playlists non
  possédées illisibles. Le tuyau s'appauvrit ; le catalogue, lui, nous
  appartient.
- Spotify refuse parfois un morceau à librespot (`audio_key_unavailable`) ;
  leur interface a un cas d'erreur dédié. À prévoir.
- 320 kbps maximum ; Spotify a demandé à librespot de ne pas contourner.
- Sécurité sérieuse : provenance GitHub du binaire vérifiée, pas de mot de
  passe ni de secret client, tokens redactés dans les erreurs. Modèle à
  suivre.

## Ce que le spike a tranché (03/09/2026)

Spike `src/bin/spike-connect.rs`, lancé par Joel avec un vrai compte
Premium et l'appli Spotify du téléphone (plus Omarchy-Spotify installé).

- **Découverte zeroconf entrante : ✓ ça marche avec l'appli actuelle.**
  « forkstify (spike) » apparaît dans la liste des appareils du téléphone,
  un toucher envoie les identifiants par le réseau local, la session
  librespot s'ouvre. Ce qu'Omarchy-Spotify avait désactivé n'était donc
  pas cassé. `librespot-discovery` 0.8, backend `libmdns` (Rust pur), TLS
  rustls — rien à installer sur l'hôte. Les identifiants sont réutilisables
  (cache librespot), le toucher du téléphone n'a lieu qu'une fois.
- **Jeton d'API Web tiré de la session librespot : ✗ inexploitable.**
  Deux voies testées, toutes deux avec le client id du client officiel
  desktop (celui de la session) :
  - *keymaster* (mercury, `get_token`) → **403 « Invalid request »**. La
    voie héritée, que librespot lui-même annonce en cours de remplacement.
  - *login5* (`login5().auth_token()`, la voie moderne) → le jeton **sort**
    (expire dans 3600 s) mais l'API Web le refuse **au premier appel** :
    `/v1/me` → **429 « API rate limit exceeded »**, `Retry-After: 47`, et
    **le 429 persiste après la fenêtre**. C'est le symptôme exact du client
    id en quota restreint décrit par le README de spotify-player. Le client
    id du desktop n'est pas habilité à porter nos appels d'API Web.

  **Conclusion : le son passe par librespot (validé de bout en bout), mais
  l'API Web ne peut pas s'appuyer sur le jeton de session.** La voie 2 de
  l'ordre de préférence ci-dessous est donc morte ; on part sur la voie 1
  (client id de ncspot, comme tout l'écosystème), voie 3 en repli.

- **Lecture par lecteur librespot embarqué : ✓ validée**
  (`src/bin/spike-play.rs`, `librespot-playback` 0.8, backend rodio → alsa,
  `libasound.so.2` présent sur l'hôte). forkstify **embarque le lecteur**
  (pas seulement découverte + jeton), charge un `spotify:track:` et **le
  son sort du binaire** — testé par Joel le 03/09/2026 (« ça marche très
  bien »). C'est l'archi cible : forkstify est lui-même l'appareil, on ne
  pilote aucun autre appareil par l'API. Construction : `rust:1-slim` +
  `pkg-config` + `libasound2-dev` (à figer dans un Dockerfile).
- **API Web par OAuth navigateur + client id de ncspot : ✓ validée**
  (`src/bin/spike-webapi.rs`, `librespot-oauth` 0.8, PKCE, callback
  `127.0.0.1:8989/login`). Le navigateur s'ouvre une fois, on autorise, le
  refresh token est mis en cache (le navigateur ne se rouvre plus). Testé :
  `/v1/me` (Joël, premium), `/v1/search` (« The Cure A Forest » →
  `spotify:track:4iVTSRiJAA18d3QglhyJ6Q`), `/v1/me/albums` (247 albums
  aimés). Le refresh avec ce client id rend **tous** les scopes de ncspot
  (playlist, user-top-read, library-modify…), plus large que demandé.
- **Leçon 429 : le throttle est au niveau compte/IP, pas par client id.**
  Nos premiers essais ont pris des 429 « rate limit exceeded » sur *les
  deux* client ids, avec un `Retry-After` **décroissant** (47 → 24 → 16 s)
  qui se résorbe au repos — c'est un throttle temporaire déclenché en
  martelant l'API, pas un blocage de quota (qui serait un 403 permanent).
  **À retenir pour le client réel : respecter `Retry-After` et réessayer**
  (le spike le fait), et ne pas enchaîner les appels inutiles.

## Orientations

### Forkstify est lui-même l'appareil Connect

Plutôt que de dépendre de `spotifyd` ou du client officiel, forkstify
**embarque librespot** ([0006](../decisions/0006-rust.md) le permet sans
réécriture) et apparaît comme un appareil dans la liste Connect. L'utilisateur
n'installe rien d'autre. Ça remplace l'orientation « séparer le cerveau du
son » de [forme-de-l-application.md](forme-de-l-application.md) — la
séparation reste vraie dans le code (le moteur ignore comment le son sort),
mais le son sort du même binaire.

### Se connecter depuis le téléphone, sans rien taper

Il n'existe pas de connexion par QR code pour les applications tierces. La
découverte Connect en tient lieu, et c'est plus court : lancer forkstify,
ouvrir Spotify sur le téléphone, toucher l'icône des appareils, choisir
« forkstify ». Les identifiants arrivent par le réseau local, rien à saisir,
pas de navigateur. Condition : téléphone et ordinateur sur le même réseau.
Repli : OAuth dans le navigateur, comme spotify-player.

### L'API Web, dans l'ordre de préférence

1. **Client id historique partagé** (celui de ncspot), comme spotify-player
   et Omarchy-Spotify — le standard de fait de l'écosystème libre Linux.
   Zone grise, révocable par Spotify du jour au lendemain. **Voie retenue**
   après le spike du 03/09/2026.
2. ~~**Jeton issu de la session librespot**~~ — **écartée par le spike** :
   keymaster répond 403, login5 sort un jeton que l'API Web refuse en 429
   persistant (client id desktop en quota restreint). Voir le spike ci-dessus.
3. **Chaque utilisateur crée sa propre application** sur le dashboard (cinq
   minutes, mode développement, cinq utilisateurs) et colle son client id
   dans la configuration. Laid mais solide ; à garder comme option de
   configuration de toute façon, pour le jour où le client id partagé tombe.

Pour Joel seul, la voie 3 fonctionne toujours.

### Deux façons de faire sortir le son

1. **Embarquer librespot**, comme Omarchy-Spotify (`engine.rs`, 800 lignes
   MIT à étudier). Forkstify est un appareil Connect, marche sur tout Linux.
2. **Parler au backend d'Omarchy-Spotify** par son socket (`load`,
   `add_to_queue`) : forkstify n'est que le cerveau. Beaucoup moins de code,
   mais ne marche que sur Omarchy avec ce plugin installé. Bon pour un PoC
   rapide, pas pour le produit — sauf à décider que forkstify est un projet
   Omarchy avant d'être un projet Linux.

## À trancher

- ~~**Étape 0 du PoC** : un *spike*~~ — **fait le 03/09/2026** (voir « Ce
  que le spike a tranché »). Découverte zeroconf ✓, jeton de session ✗ ;
  repli confirmé : OAuth navigateur + client id de ncspot.
- **Chantier son : les quatre briques validées** (03/09/2026) — découverte
  zeroconf, session librespot, API Web (client id ncspot), lecture par
  lecteur embarqué. La lecture directe est tranchée : on **ne pilote pas**
  l'API `/v1/me/player/*`, forkstify joue lui-même. Reste à **câbler** :
  brancher tout ça sur `forkstify parcours` — résoudre les titres d'un
  segment en `spotify:track:` (via `/v1/search`, fait dans spike-webapi),
  les enchaîner dans le lecteur, et gérer `e` / les branches en temps réel.
- **Projet Linux ou projet Omarchy ?** Décide entre embarquer librespot et
  s'appuyer sur le backend d'Omarchy-Spotify. Le spike montre qu'embarquer
  librespot marche sans Omarchy — Joel a d'ailleurs remplacé
  Omarchy-Spotify par le client officiel entre-temps.
- **Conditions d'utilisation** : librespot n'est pas supporté par Spotify.
  fastpotify affirme n'avoir connaissance d'aucun compte suspendu ; le risque
  existe et doit être dit à l'utilisateur.
