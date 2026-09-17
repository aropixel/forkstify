# 0023 — Les indices du moteur se règlent dans la configuration

- **Date** : 2026-09-17
- **Statut** : accepté

## Contexte

La règle du projet tient en une phrase : toute décision automatique doit
être explicable en une phrase et modifiable en un commit. Les décisions
l'étaient ; leurs **coefficients** ne l'étaient que pour qui recompile.
Le cooldown d'un morceau (`0.1`, sept jours), celui d'un artiste (`0.3`,
quatre jours), ce que vaut « moins souvent » (`×0.7`) et « plus souvent »
(son miroir), le poids d'un aimé (`×10` au cocon, `×2` ouvert), d'une
door, de la traîne, les seuils du saut aventureux : tout vivait en
constantes Rust, dans `engine.rs` et `learned.rs`.

Joel, 17/09/2026 : « pousser la logique du *reprendre la main sur
l'algorithme* jusqu'au bout et donner la possibilité à la personne qui
aura installé forkstify de changer les valeurs de tous les indices via le
fichier de configuration ».

## Décision

1. **Tous les nombres du moteur et de l'appris sont des réglages**, dans
   une section `[tuning]` de `~/.config/forkstify/config.toml`, un nom en
   anglais par nombre (`track_cooldown_floor`, `less_often`,
   `liked_weight_cocoon`, `leap_floor_open`…). Les valeurs par défaut
   sont celles que le code livrait : ne rien écrire, c'est garder le
   comportement d'aujourd'hui. Le gabarit écrit au premier lancement
   liste chaque réglage, commenté, à sa valeur par défaut.
2. **Un réglage est lu au lancement et ne bouge pas pendant un
   parcours** : `config::tuning()` donne les valeurs en vigueur, celles
   du fichier une fois chargé, les défauts avant (et dans les tests).
   Modifier un indice, c'est éditer le fichier et relancer.
3. **Un nombre absurde est dit et remis à son défaut, seul** : une part
   hors de 0–1, une demi-vie négative, un poids nul. Les autres restent
   tels qu'écrits. Le moteur ne doit jamais tourner avec un poids
   négatif, mais une faute de frappe ne doit pas effacer les autres choix.
4. **Ce qui n'est pas un indice du moteur ne s'y règle pas** : les délais
   de l'interface (durée d'un toast, redémarrage d'un morceau, offre de
   graine), les tailles d'écran, le nombre de tops d'un album restent
   des constantes. Ils ne décident rien de ce qui se joue.
5. **Ce qui reste structurel reste dans le code** : le nombre de
   candidats tirés (`take(6)`, `truncate(12)`), les puissances de la
   pondération (`poids²`, `(score − 0,5)³`), le poids `4.0` de la branche
   `stay`. Ce sont des formes, pas des indices ; s'il faut en ouvrir un,
   ce sera une ligne de plus dans `[tuning]`, pas une autre mécanique.

## Conséquences

- `config.rs` porte la structure `Tuning`, ses défauts et ses garde-fous ;
  `engine.rs` et `learned.rs` n'ont plus de constante numérique. Un
  réglage nouveau se fait en trois lignes : le champ, son défaut, sa ligne
  de gabarit.
- Un utilisateur qui partage son fork de catalogue ne partage pas ses
  réglages : ils sont dans sa configuration, pas dans le dépôt. C'est
  voulu — la fiche dit ce qu'est un artiste, le réglage dit comment *on*
  écoute.
- [`docs/reglages.md`](../reglages.md) est la référence de chaque
  réglage : à quoi il sert, son défaut, l'effet de le monter ou le
  baisser. Le gabarit de configuration en donne la version courte ;
  `zone-de-confort.md` et `moteur-de-branches.md` disent la mécanique.
