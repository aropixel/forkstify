# 0002 — Le catalogue est un ensemble de fichiers texte, partagé et forkable

**Date** : 2026-08-30 · **Statut** : acceptée — la partie « deux couches »
est remplacée par [0008](0008-le-fork-est-la-surcouche.md)

## Contexte

Les branches ont besoin de savoir quels sont les morceaux d'un artiste et
quels artistes sont proches. Cette connaissance pouvait venir d'une API
(Spotify, dont les points d'API de recommandation sont fermés aux nouvelles
applications depuis fin 2024), d'une base locale opaque, ou de fichiers texte.

Le projet s'adresse à l'écosystème Linux : on y a l'habitude de récupérer des
fichiers de configuration, de les forker, de les adapter.

## Décision

Le catalogue est **un ensemble de fichiers texte versionnés**, un par artiste,
destiné à être **partagé et forké**. Chacun récupère le catalogue, se
l'approprie — ses tops de The Cure sont *A Forest* et *10:15 Saturday Night*,
pas *Boys Don't Cry* — et peut reverser ses fiches.

## Conséquences

- **Le format des fiches est une interface publique.** Il doit être stable,
  documenté, lisible et modifiable à la main. Voir
  [conception/catalogue.md](../conception/catalogue.md).
- Deux couches : une **base** partagée et une **surcouche** personnelle qui
  gagne toujours.
- Le catalogue est portable d'une installation à l'autre : le cloner suffit.
- Tout ce qui est dérivé des fiches (vecteurs, index) est un cache
  régénérable, jamais une source de vérité.
- Le catalogue et l'application ont des cycles de vie différents ; ils
  pourraient vivre dans deux dépôts distincts (à trancher).
