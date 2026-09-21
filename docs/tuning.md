# Les réglages du moteur (`[tuning]`)

Tout nombre avec lequel le moteur décide de ce qui se joue est un réglage,
dans la section `[tuning]` de `~/.config/forkstify/config.toml`
([0023](decisions/0023-les-indices-du-moteur-se-reglent.md)). Cette page
dit, pour chacun, **à quoi il sert, sa valeur par défaut, et ce que fait
le monter ou le baisser**. Le fichier est lu au lancement : on édite, on
relance.

Trois règles pour tous :

- **Ne rien écrire, c'est garder le défaut.** Une ligne retirée du
  fichier revient à sa valeur d'origine ; pour remettre un réglage, il
  suffit de le remettre à la valeur de la colonne « défaut » ou d'effacer
  la ligne.
- **Un réglage discourage ou favorise, il n'interdit jamais.** Aucun
  nombre ne ferme une branche ni ne retire un morceau : les seules
  interdictions sont les bannissements (`tb`, `ab`), qui ne sont pas des
  réglages.
- **Une valeur absurde est remise à son défaut, seule**, et dite sur
  stderr au lancement (part hors de 0–1, demi-vie négative, poids nul).
  Les autres lignes restent telles qu'écrites.

Le gabarit complet, commenté, est celui que forkstify écrit au premier
lancement (`src/config.rs`, `TEMPLATE`) ; sur un poste installé avant le
17/09/2026, le copier à la main dans le fichier.

## Les cooldowns (0012 §2)

Ce qui vient de sonner recule dans le tirage, puis revient avec le temps.
Chaque cooldown a un **plancher** (ce qui reste du poids le jour même) et
une **demi-vie** en jours (le temps pour récupérer la moitié du reste).

| Réglage | Défaut | À quoi il sert | Monter / baisser |
|---|---|---|---|
| `track_cooldown_floor` | `0.1` | Part du poids qu'un morceau garde le jour où il a sonné. | `1.0` = aucun cooldown, un morceau peut revenir le soir même. `0.0` = il ne revient pas le jour même (le lendemain, il a déjà récupéré un peu). |
| `track_cooldown_half_life` | `7.0` | En jours. Au défaut, un morceau joué aujourd'hui est à 55 % de son poids une semaine plus tard, 78 % à deux, 94 % au mois. | Plus long = les morceaux reviennent moins vite, la rotation s'élargit. Plus court = on réentend plus tôt. |
| `artist_cooldown_floor` | `0.3` | Part de son attrait qu'un **artiste** entendu aujourd'hui garde comme tête de branche. | `1.0` = les mêmes artistes peuvent mener branche sur branche. Plus bas = on tourne davantage entre artistes. |
| `artist_cooldown_half_life` | `4.0` | En jours : le temps pour l'artiste de redevenir une tête de branche à part entière. | Plus long = un artiste entendu cette semaine mène moins ; plus court = il revient vite. |

Le cooldown d'artiste ne touche que le choix des **têtes de branche** et
de la branche `stay` ; il ne retire pas les morceaux de l'artiste du
réservoir. Le cooldown de morceau multiplie le poids du morceau dans le
réservoir de son artiste.

## Le goût (0018) : `al` / `as`, `tl` / `ts`

Chaque artiste porte un **poids** dans `learned/`, à 1 au départ. « Plus
souvent » et « moins souvent » le multiplient ; ce poids multiplie
ensuite celui des branches qui commencent par cet artiste.

| Réglage | Défaut | À quoi il sert | Monter / baisser |
|---|---|---|---|
| `less_often` | `0.7` | Ce par quoi `as` / `ts` multiplie le poids de l'artiste. | Plus près de 1 = geste plus doux, il en faut plusieurs pour sentir la différence. Plus bas = un seul geste écarte franchement. |
| `more_often` | `0` | Ce par quoi `al` / `tl` multiplie le poids. `0` = le miroir de `less_often` (1 ÷ 0.7 ≈ 1.43), pour qu'un `al` défasse exactement un `as`. | Une valeur propre (≥ 1) casse la symétrie : `2.0` = « plus souvent » pèse plus lourd que « moins souvent ». |
| `weight_floor` | `0.1` | Le poids ne descend jamais sous cette valeur : un artiste « moins souvent » dix fois reste possible. | Plus bas = on peut presque effacer un artiste au clavier. Plus haut = le geste plafonne vite. |
| `weight_ceiling` | `3.0` | Le poids ne monte jamais au-dessus : une touche ne peut pas s'emballer. | Plus haut = un artiste aimé peut dominer les propositions. |

## La familiarité (0001, 0014)

La familiarité d'un artiste (0 à 1) vient de la bibliothèque importée
**ou** des écoutes dans forkstify, la plus forte des deux. C'est elle que
la zone de confort lit : au cocon, elle attire ; ouvert, elle repousse.

| Réglage | Défaut | À quoi il sert | Monter / baisser |
|---|---|---|---|
| `plays_reference` | `5.0` | Nombre d'écoutes (décrues) auquel la familiarité par l'écoute atteint la moitié. À 10 écoutes, 75 % ; à 20, 94 %. | Plus bas = un artiste devient « familier » vite ; plus haut = il faut l'avoir beaucoup écouté. |
| `plays_half_life` | `182.5` | En jours : la demi-vie des compteurs d'écoute (six mois, 0014). Une écoute d'il y a un an compte pour un quart. | Plus court = la familiarité suit l'actualité de l'écoute ; plus long = elle a de la mémoire. |

## Le réservoir (0012 §1)

Le réservoir d'un artiste est l'ensemble de ses morceaux tirables, chacun
avec un poids ; le moteur y tire au sort, pondéré. Un top vaut 1 : tout
le reste se lit par rapport à lui.

| Réglage | Défaut | À quoi il sert | Monter / baisser |
|---|---|---|---|
| `top_weight` | `1.0` | Le poids d'un top. La norme des autres ; le changer seul revient à changer tous les autres en sens inverse. | À laisser à 1 sauf raison précise. |
| `liked_weight_cocoon` | `10.0` | Le poids d'un morceau aimé (`tl`) au confort 5. Un aimé prime sur les tops : les tops ne sont que les portes d'entrée d'un fork vierge (0018). | Plus haut = au cocon, on n'entend presque que ses aimés. Plus bas = les tops reprennent leur place. |
| `liked_weight_open` | `2.0` | Le même, au confort 0 ; entre les deux, le curseur glisse de l'un à l'autre. | Plus bas = ouvert, l'aimé ne compte presque plus que comme un top : on cherche l'inconnu. |
| `door_weight` | `0.4` | Le poids d'une door (0011) qui n'est pas un top, quand la branche ne va pas dans sa direction. | Plus haut = les doors sortent souvent, même sans direction. |
| `door_bonus` | `2.5` | Ce par quoi une door est multipliée quand la branche **va** dans sa direction (un tag de la tête de branche correspond). | Plus haut = la door devient le passage obligé vers cette direction. |
| `tail_weight` | `0.25` | Le poids d'**un** morceau de la longue traîne (le reste de la discographie), avant que le curseur et la familiarité le réduisent. Une traîne a dix fois plus de morceaux qu'une fiche n'a de tops : bas par morceau, elle pèse lourd en cumul. | Plus haut = plus de fonds de tiroir, même à confort moyen. Plus bas = la traîne n'apparaît qu'ouvert, chez les artistes familiers. |

La part de la traîne vaut `tail_weight × ouverture × familiarité` : nulle
au confort 5 (rien à régler ici, c'est le curseur), et nulle chez un
artiste inconnu, qui est mené par ses tops.

## Le saut aventureux

Chaque tour propose une branche « par le graphe » (les liens des fiches)
et une branche « aventureuse » : un artiste hors du graphe, proche dans
l'espace des vecteurs. La proximité est un cosinus, de −1 à 1 ; en
pratique les voisins utiles sont entre 0.5 et 0.9. Chaque seuil a une
valeur au cocon (confort 5) et une ouvert (confort 0) ; entre les deux,
le curseur glisse.

| Réglage | Défaut | À quoi il sert | Monter / baisser |
|---|---|---|---|
| `leap_floor_cocoon` | `0.80` | Au confort 5, sous cette proximité, pas de saut hors du graphe. | Plus haut = presque jamais d'aventureuse au cocon. Plus bas = elle va plus loin. |
| `leap_floor_open` | `0.60` | Le même au confort 0. C'est aussi le plancher sous lequel `fw` va chercher ses « loin ». | Plus bas = ouvert, on saute très loin. |
| `leap_trust_cocoon` | `0.86` | Au confort 5, au-dessus de cette proximité, un saut n'a plus besoin d'un tag de genre commun avec la branche. | Plus bas = on fait confiance aux vecteurs seuls plus tôt. |
| `leap_trust_open` | `0.70` | Le même au confort 0. | Idem, ouvert. |

Entre plancher et confiance, un saut demande **un tag de genre partagé**
(pays et décennie ne comptent pas). Un plancher plus haut que la confiance
n'a pas de sens : la confiance ne servirait jamais.

## Ce qui n'est pas un réglage, et pourquoi

- **Les délais de l'interface** (durée d'un toast, seuil de redémarrage
  d'un morceau, offre de graine) : ils ne décident rien de ce qui se joue.
- **Les formes du tirage** : le nombre de candidats gardés avant le sort
  (`6` têtes, `12` pour `stay` et `fw`), les puissances qui creusent la
  pondération (`poids²` sur le graphe, `(proximité − 0,5)³` sur les
  vecteurs), le poids fixe `4.0` de la branche `stay`, la fourchette
  `0,25`–`2` de ce que le confort fait à une familiarité. Ce sont des
  mécaniques, pas des indices ; en ouvrir un, c'est une ligne de plus dans
  `[tuning]` et une ligne ici.
