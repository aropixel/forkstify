# 0014 — Forme de l'appris : `learned/`, un fichier par artiste, compteurs décrus

- **Date** : 2026-09-04
- **Statut** : accepté

## Contexte

Le modèle base / mien / appris ([catalogue.md](../conception/catalogue.md))
place l'*appris* — ce que l'usage produit — dans un dossier à part, versionné,
jamais reversé à l'amont. Les décisions [0012](0012-rotation-des-morceaux.md)
(cooldown daté) et [0013](0013-affinage-clavier-mesure-ou-edition.md) (chaque
mesure écrit dans cette couche) en dépendent, mais sa **forme** restait à
trancher : un fichier ou un par artiste, quels compteurs, quelle décroissance.

Ces notes antérieures nommaient la couche `usage/` ; cette décision fixe le
nom définitif et supersède cet usage incident.

## Décision

- **Le dossier de l'appris est `learned/`** (et non `usage/`). Le vocabulaire
  sur disque — chemins, sous-dossiers, champs — est en **anglais**, comme le
  format de fiche : le dépôt vise l'open source, ces noms sont une interface
  publique.
- **Un fichier par artiste**, `learned/artists/<slug>.toml`, en miroir des
  fiches : diffs propres, aucun conflit avec l'amont (chacun le sien), passage
  à l'échelle. Les **récoltes** (touche `m`) vivent à part et transverses :
  `learned/marks/<name>.toml`.
- **Compteurs à décroissance temporelle intégrée.** On ne garde pas
  l'historique des écoutes mais un compte décru : à chaque écoute,
  `plays = plays × ½^((now − last)/demi-vie) + 1`, `last = now`. Un flottant
  et une date par artiste et par top suffisent ; une écoute ancienne ne pèse
  presque plus. **Demi-vie : 6 mois.**
- **Silencieux, jamais reversé.** L'appris se modifie sans confirmation
  (mesure) et n'entre jamais dans une PR.
- Chaque champ, ce que chaque touche de 0013 écrit, et ce que le moteur lit
  (exclusion des `blacklisted`, familiarité → confort, cooldown, poids) sont
  détaillés dans l'orientation « Forme de l'appris » de
  [catalogue.md](../conception/catalogue.md).
- `learned/classement.json` (l'amorce, 741 artistes scorés depuis la
  bibliothèque) devient la **familiarité de départ**, prolongée par
  `learned/artists/`.

Restent réglables au fil du PoC, sans nouvelle décision : la fenêtre de
cooldown (0012) et la formule familiarité → zone de confort 0–5 (0001).

## Conséquences

- Le dossier `usage/` du catalogue est renommé `learned/` ; les scripts
  d'`outillage/` suivent. Les fichiers d'amorce encore nommés en français
  (`amis/`, `artistes-*.json`) et leurs clés seront traduits avec les scripts
  (tâche d'`outillage/` à part).
- Les décisions 0012 et 0013, immuables, mentionnent `usage/` : lire
  `learned/`.
