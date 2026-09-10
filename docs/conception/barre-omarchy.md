# La barre Omarchy — l'icône qui bouge, et la popover du morceau

Note de travail, ouverte le 10/09/2026. **Décidé** = acté ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Ce que Joel veut

Reprendre ce qu'il avait sous waybar : une animation de petites barres
(`▂▄▆`, cinq images, 100 ms) dans la barre quand ça joue, rien quand c'est
en pause. Et au clic, une popover **sous l'icône, comme tous les plugins
Omarchy** : titre, artiste, progression, prochain morceau.

L'ancien script : `dotfiles/.config/waybar/custom_modules/media/media-animation.sh`
du dépôt `kbyjoel/arch-linux` — une boucle `playerctl status`.

## Ce qui existe déjà, des deux côtés

- **Omarchy a un widget média MPRIS** (`omarchy.media`, service +
  bar-widget, désactivé chez Joel) : caché tant qu'aucun lecteur n'a de
  métadonnées, puis une icône ▶/⏸ et « titre · artiste » dans la barre, et
  une `PopupCard` au clic avec pochette, titre, artiste, album,
  précédent / pause / suivant. **Pas de progression, pas de prochain
  morceau, pas d'animation.** Quickshell expose d'un lecteur `position`,
  `length`, `isPlaying`, `trackTitle`, `trackArtist` et la **map brute
  `metadata`** — une clé maison y passe.
- **forkstify est déjà un lecteur MPRIS** (`src/mediakeys.rs`) — mais ne
  publie que « joue » à l'ouverture : **ni titre, ni artiste, ni position,
  ni changement d'état** en pause. Le widget d'Omarchy le verrait vide.
- Les plugins tiers s'installent par `omarchy plugin add <git-url>`, qui
  attend un `manifest.json` **à la racine du dépôt**, et vivent dans
  `~/.config/omarchy/plugins/<id>/` (rechargés à chaque sauvegarde). Le
  plugin `io.github.sspaeti.neomd` de Joel en est un exemple : un
  `BarWidget.qml`, une `PopupCard`, un `Model.js`.

## Orientation : deux étages, le premier portable

1. **forkstify publie tout ce qu'un bureau attend** (portable, hors
   Omarchy) : titre, artiste, durée (`xesam:` / `mpris:length`), position
   et `Seeked`, l'état à chaque pause / reprise, et une clé maison dans les
   métadonnées, **`forkstify:next`** = « titre — artiste » du prochain
   morceau de la file. GNOME, KDE, waybar + playerctl en profitent
   d'emblée ; le widget média d'Omarchy s'allume tel quel. C'est
   `mpris-server` : `set_metadata`, `set_playback_status`, `set_position`.
   La pochette (`mpris:artUrl`) demanderait l'image d'album du morceau
   résolu — la recherche Spotify la donne, à garder pour plus tard.
2. **Un plugin Omarchy `forkstify`**, bar-widget, calqué sur `omarchy.media`
   mais **ne regardant que le lecteur `forkstify`** : dans la barre, les
   cinq images de l'animation sur un `Timer` de 100 ms quand `isPlaying`,
   une image fixe en pause, rien si forkstify ne tourne pas. Au clic, la
   `PopupCard` : titre, artiste, une barre de progression `position /
   length`, « à suivre : … » lu dans `metadata["forkstify:next"]`, et les
   trois boutons. Les autres lecteurs restent au widget d'Omarchy, s'il
   veut l'activer.

Pourquoi deux étages : l'étage 1 sert tout le monde et ne dépend de rien ;
l'étage 2 est le seul morceau lié à Omarchy, et il reste petit.

## À trancher

1. **Le plugin regarde-t-il seulement forkstify**, ou tout lecteur comme
   l'ancien script waybar ? Recommandation : seulement forkstify — le
   « prochain morceau » n'a de sens que chez lui, et `omarchy.media`
   existe pour le reste.
2. **Où vit le plugin ?** `omarchy plugin add` veut le manifeste à la
   racine d'un dépôt git : (a) un dépôt `forkstify-omarchy` à part,
   installable en une commande, ou (b) un dossier `omarchy/` dans ce dépôt,
   copié ou lié à la main dans `~/.config/omarchy/plugins/`. (b) pour
   développer, (a) pour distribuer — les deux se cumulent (un dépôt qui ne
   contient que ce dossier). Recommandation : commencer en (b).
3. **La pochette** dans la popover : tout de suite (une image d'album de
   plus à résoudre par morceau) ou plus tard ? Recommandation : plus tard,
   l'étage 1 d'abord.
4. **Un clic milieu / molette** sur l'icône ? L'ancien script n'en avait
   pas ; les widgets Omarchy n'en font pas un usage régulier. Rien pour
   commencer.
