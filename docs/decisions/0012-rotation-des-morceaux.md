# 0012 — Rotation des morceaux : la répétition ne doit jamais être subie

**Date** : 2026-09-01 · **Statut** : acceptée

## Contexte

Les titres se choisissent par tops ([0003](0003-titres-tops-et-portes.md)).
Si le moteur prend toujours en tête des tops, on retombe constamment sur
les mêmes morceaux. Les tops existent pourtant *pour* être rejoués : la
répétition n'est pas un bug, elle ne doit juste jamais être **subie**.

## Décision

Quatre mécanismes cumulables, chacun explicable en une phrase :

1. **Le top est un poids, pas une liste fermée.** Le réservoir d'un artiste
   cumule les tops (poids fort), les titres aimés de l'utilisateur chez cet
   artiste (`usage/`), les doors et le reste de la discographie connue
   (cache API, hors catalogue). Le moteur **tire au sort pondéré** dans ce
   réservoir.
2. **La fraîcheur (cooldown).** Chaque lecture est datée dans `usage/` ; un
   morceau joué récemment est pénalisé, la pénalité décroît avec le temps.
3. **Sans remise dans le parcours.** Jamais deux fois le même morceau dans
   un parcours ; poncer tire sans remise — le deuxième ponçage descend vers
   le moins connu.
4. **La zone de confort règle la profondeur du tirage.** Confort haut :
   tirage serré sur les tops (répétition *choisie*) ; confort bas : la
   longue traîne pèse davantage. C'est le rôle que
   [0001](0001-confort-familiarite.md) donne déjà au curseur — pas de
   deuxième réglage.

## Conséquences

- `usage/` porte des dates de lecture, pas seulement des compteurs.
- La discographie élargie vit dans un cache API, jamais dans le catalogue.
- Restent à régler au fil du PoC : la vitesse de décroissance du cooldown
  et la forme exacte de l'appris (question ouverte de
  `docs/conception/catalogue.md`).
