# 0021 — Le dépôt est le plugin Omarchy, et le lecteur parle MPRIS pour de vrai

- **Date** : 2026-09-10
- **Statut** : accepté

## Contexte

Joel veut dans la barre Omarchy ce qu'il avait sous waybar : une animation
`▂▄▆` quand ça joue, et au clic une popover titre / artiste / progression /
prochain morceau, comme tous les plugins Omarchy. Et il veut que
**`omarchy plugin add <dépôt>` installe forkstify et fasse apparaître
l'icône** — le projet n'est pas distribué autrement.

Ce que la mécanique impose : `omarchy plugin add` clone le dépôt, exige
`manifest.json` **à sa racine**, valide, déplace dans
`~/.config/omarchy/plugins/<id>/`, active. Pas de crochet d'installation.
Et forkstify, déjà lecteur MPRIS, ne publiait que « joue » à l'ouverture.

## Décision

1. **Le dépôt forkstify est lui-même le plugin Omarchy.** `manifest.json`
   à la racine, le code du widget dans `omarchy/`. Cloné par
   `omarchy plugin add`, c'est le dépôt entier qui vit dans
   `~/.config/omarchy/plugins/<id>/` — avec `bin/build` dedans.
2. **C'est le plugin qui installe et lance le binaire.** Tant que
   `forkstify` n'est pas sur le `PATH`, la popover propose « Installer »
   (construit dans le conteneur, lie `target/release/forkstify` dans
   `~/.local/bin`) ; tant qu'il ne tourne pas, elle propose « Lancer »
   (`omarchy-launch-or-focus-tui forkstify`). Prérequis : les jetons et
   caches quittent `target/` pour un dossier XDG — lancé depuis la barre,
   forkstify n'a plus de dossier courant.
3. **forkstify publie de vraies métadonnées MPRIS**, pour tout bureau :
   titre, artiste, durée, position, l'état à chaque pause et reprise, et
   la clé maison **`forkstify:next`** (« titre — artiste » du prochain
   morceau). Poussées depuis `paint`, comme l'écran : dérivées de l'état,
   seulement quand elles changent.
4. **Le widget ne regarde que le lecteur `forkstify`.** Les autres lecteurs
   restent au widget média d'Omarchy. Pas de pochette pour commencer, rien
   sur le clic milieu ni la molette.

## Conséquences

- Trois étages, dans l'ordre : les métadonnées MPRIS (fait le jour même),
  les chemins XDG, puis le plugin dans `omarchy/`.
- Le manifeste à la racine est une interface publique du dépôt : en
  anglais, comme le reste.
- La note vivante : [`docs/conception/barre-omarchy.md`](../conception/barre-omarchy.md).
