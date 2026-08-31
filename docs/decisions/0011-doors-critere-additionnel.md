# 0011 — Les portes reviennent (`doors`), en critère additionnel

**Date** : 2026-08-31 · **Statut** : acceptée · **Amende** [0010](0010-format-revise-links-sans-portes.md)
et rétablit partiellement la partie « portes » de [0003](0003-titres-tops-et-portes.md)

## Contexte

[0010](0010-format-revise-links-sans-portes.md) avait retiré les portes :
le choix du morceau revenait entièrement au moteur (tops + usage + zone de
confort). À l'usage de la réflexion, Joel est revenu dessus :
« occasionnellement, sur certains morceaux ciblés, ça peut être utile ;
mais ce ne sera qu'un critère additionnel, pas le principal ».

## Décision

- Le champ **`doors`** revient dans la fiche, optionnel, au format en ligne
  des `links` : `{ track = "A Forest", to = ["post-punk", "atmospherique"], note = "…" }`.
  `to` pointe vers des **tags** (une direction), pas vers des artistes.
- **Critère additionnel, jamais principal** : quand le moteur quitte un
  artiste vers une direction qui recoupe les tags d'une door, ce morceau
  reçoit un **bonus** — il ne devient ni obligatoire ni exclusif. Tops,
  usage et zone de confort restent les critères premiers. Une fiche sans
  door fonctionne exactement comme avant.

## Conséquences

- Les doors écrites au premier lot sont restaurées dans les fiches
  concernées.
- Le moteur traite `doors` comme une pondération de choix de morceau, pas
  comme une règle de routage.
