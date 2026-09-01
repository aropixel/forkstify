# 0013 — Affinage au clavier : chaque touche est une mesure ou une édition

**Date** : 2026-09-01 · **Statut** : acceptée

## Contexte

Pendant l'écoute, l'utilisateur doit pouvoir affiner son algorithme sans
quitter le flux : promouvoir un morceau en top, l'écarter, en faire une
porte… L'interface est clavier d'abord, à la neovim.

## Décision

- **Chaque touche est soit une mesure, soit une édition.** Une **mesure**
  modifie `usage/` en silence (sauter, aimer) — c'est l'*appris*. Une
  **édition** modifie une fiche et produit **un commit lisible**
  (« top : + A Forest ») — c'est le *mien*, immédiatement partageable.
- **`u` annule la dernière action**, quelle qu'elle soit : revert pour une
  édition, effacement pour une mesure. L'historique d'affinage *est* le
  log git.
- **Chaque touche n'est que le raccourci d'une commande `:`** (`:top`,
  `:door post-punk`, `:confort 2`) : tout est découvrable et scriptable,
  les keybinds sont une table de correspondance remappable en config.

## Conséquences

- La table des touches (`t`/`T`, `x`/`X`, `a`, `d`, `m`, `e`, `-`, `z`,
  `?`, `y`/`n`) est une orientation de
  `docs/conception/forme-de-l-application.md` — elle s'ajuste au fil du
  PoC sans revenir sur cette décision.
- Rien de ce que le moteur apprend n'est une boîte noire : toute édition
  est un diff relisible et annulable, cohérent avec « le fork est la
  surcouche » ([0008](0008-le-fork-est-la-surcouche.md)).
