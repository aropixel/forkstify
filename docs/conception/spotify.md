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

## Ce qui n'est pas vérifié

- **Un jeton d'API Web obtenu par la session librespot**, sans client id du
  tout. librespot expose un fournisseur de jetons à scopes ; ni
  spotify-player ni Omarchy-Spotify ne s'en servent pour l'API Web, tous
  deux font un OAuth séparé avec le client id de ncspot. Confiance faible ;
  le spike reste le moyen de trancher, mais on ne parie plus dessus.
- **La découverte zeroconf entrante avec l'appli Spotify actuelle** :
  Omarchy-Spotify l'a désactivée sans dire pourquoi. Spotify a changé ses
  protocoles en 2024–2025. À tester ; si ça ne marche pas, le navigateur
  est le repli standard.

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
   Zone grise, révocable par Spotify du jour au lendemain.
2. **Jeton issu de la session librespot** — si le spike montre que ça
   marche, rien d'autre ; mais personne ne le fait, confiance faible.
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

- **Étape 0 du PoC** : un *spike* qui teste les deux points non vérifiés —
  forkstify découvert depuis le téléphone (zeroconf entrant), puis un appel
  à `api.spotify.com` avec le jeton de la session librespot. Deux jours au
  plus. Quel que soit le résultat, le repli est connu : OAuth navigateur +
  client id de ncspot, comme tout le monde.
- **Projet Linux ou projet Omarchy ?** Décide entre embarquer librespot et
  s'appuyer sur le backend d'Omarchy-Spotify.
- **Conditions d'utilisation** : librespot n'est pas supporté par Spotify.
  fastpotify affirme n'avoir connaissance d'aucun compte suspendu ; le risque
  existe et doit être dit à l'utilisateur.
