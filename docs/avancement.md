# Avancement

Mis à jour le **01/09/2026**. Ce fichier est le point d'entrée pour reprendre
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
  liens scene par recoupement. Le catalogue compte **88 fiches** et le lot
  2 appelle à son tour **89 slugs** (lot 3). Descriptions absentes des
  fiches générées : la relecture enrichit.
- **Dépôts** : `kbyjoel/forkstify` et `kbyjoel/forkstify-catalog`, privés,
  branche `main`. Plan de reprise chorizo à jour.

## En attente de Joel

- **Relecture du lot 1** (30 fiches) : ses tops (The Cure…), tops vides de
  Cabadzi et Le Motel, retirer `generated` des fiches relues.
- **Relecture du lot 2** : 28 fiches à MBID incertain (résolu par nom,
  liste dans le rapport du générateur), tops en doublon de versions chez
  J.P. Nataf, 3 fiches sans links (caballero-jeanjass, rendez-vous, sza).
- Identifiants Deezer/Spotify d'**amis consentants** pour élargir la base
  (`outillage/amis-*.py`).

## Prochaines étapes, dans l'ordre

1. **Vecteurs** : texte composé depuis la structure (jamais demandé aux
   humains), embedding (`fastembed` côté Rust, ou prototype Python), et le
   contrôle par les voisins (`forkstify check`).
2. **Navigation à sec** (étape 2 du PoC, premier code Rust) : graine →
   3 branches lisibles avec leurs raisons → choix clavier → segments
   affichés. Critère : des parcours cohérents en les lisant.
3. **Spike Spotify Connect** (étape 3) : librespot embarqué, découverte
   depuis le téléphone, jeton d'API — voir `docs/conception/spotify.md`.
4. **TUI** (étape 4), à la neomd.

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
