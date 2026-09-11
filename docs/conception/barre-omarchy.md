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

## Décidé le 10/09/2026 — [0021](../decisions/0021-le-depot-est-le-plugin-omarchy.md)

Joel a tranché les quatre questions : le widget ne regarde que forkstify ;
**le dépôt est lui-même le plugin** (`manifest.json` à la racine,
`omarchy/` pour le widget), et c'est le plugin qui **installe et lance** le
binaire, puisque `omarchy plugin add` ne fait que cloner, valider et
activer ; pochette plus tard ; rien sur le clic milieu ni la molette.
Trois étages, **tous faits le 10/09/2026** : les métadonnées MPRIS ; les
jetons et caches sous `~/.local/state/forkstify` (repris depuis `target/`
au premier lancement, sans ré-autoriser) ; le plugin — `manifest.json` à
la racine, `omarchy/BarWidget.qml`, `omarchy/install.sh`. Un écart assumé
avec l'ancien script : **l'icône reste visible** en pause et quand forkstify ne
tourne pas — le même escalier `▂▄▆`, fixe et atténué (Joel : des barres à
plat « donnent l'impression d'un bug ») — sinon rien ne permettrait de
cliquer pour lancer ou installer. Chez Joel, le dépôt est
**lié** dans `~/.config/omarchy/plugins/io.github.aropixel.forkstify` ;
ailleurs, `omarchy plugin add git@github.com:aropixel/forkstify.git`.

**Recharger le widget après une modification du QML** : le shell surveille
`~/.config/omarchy/plugins/` avec `inotifywait -r`, qui ne descend pas dans
un dossier lié, et son cache de composants survit à `rescanPlugins` et
même au « Local plugin changed » que provoque la recréation du lien.
Seul **`omarchy restart shell`** fait prendre un nouveau QML (vérifié le
10/09/2026 par capture de la barre). Une seconde de clignotement.

**La position dans Quickshell** (11/09/2026) : `MprisPlayer.position` se
calcule à chaque lecture (dernier échantillon + temps écoulé), mais le
signal `positionChanged` n'est émis qu'à un `Seeked` ou un changement
d'état — jamais pendant la lecture. Une liaison QML (`root.position:
player.position`) reste donc figée sur la dernière valeur signalée, 0 au
début du morceau, et la carte affichait 0:00. Le widget demande le signal
lui-même : un `Timer` d'une seconde, actif carte ouverte et morceau en
lecture, appelle `player.positionChanged()`. Le service média d'Omarchy
n'affiche pas la position et n'a pas ce problème.

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

## Tranché (voir ci-dessus) — gardé pour l'historique

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
