# Zone de confort

Note de travail. **Décidé** = acté dans `docs/decisions/` ; **orientation** =
proposé, non contredit, pas encore acté ; **à trancher** = question ouverte.

## Décidé

- Réglage de **0 à 5**, défini **à l'ouverture**.
- Mesure la **familiarité** ([0001](../decisions/0001-confort-familiarite.md)) :
  **5 = cocon, 0 = exploration** — *retourné le 06/09/2026*. La note disait
  l'inverse ; au premier usage réel, Joel : « si je veux le cocon, je devrais
  mettre le confort à 5 — le confort, c'est ce qu'on connaît bien ». Il a
  raison, et le dépôt était l'intrus : [0012](../decisions/0012-rotation-des-morceaux.md) §4
  écrit « confort haut : tirage serré sur les tops », ce qui se lit désormais
  au pied de la lettre. Une note de conception cède devant l'usage.
- Sert à **choisir seul** la branche à un embranchement quand l'utilisateur ne
  choisit pas activement. L'application ne bloque jamais.

## Câblé le 05/09/2026

`engine::Comfort` (0 à 5), lu dans `~/.config/forkstify/config.toml`
(`[journey] comfort`) et réglable en écoute par `:comfort <n>`. Il agit sur
deux leviers, ceux que `avancement.md` désignait déjà comme « constantes à
piloter par le confort » :

- **Le plancher de la branche aventureuse** s'abaisse quand on ouvre :
  cosinus ≥ 0.80 au cocon, ≥ 0.60 à l'exploration. **Le confort 2 reproduit
  exactement le réglage fixe d'avant** (0.72 / 0.80) — l'ancien accord
  devient le milieu du curseur, pas une valeur perdue.
- **La familiarité penche le tirage des têtes de branche** (0001 : le
  confort *est* la familiarité). Au cocon, un artiste familier passe devant
  un inconnu ; ouvert, c'est l'inverse. Le facteur est borné à [0.25, 2.0] :
  **on décourage, on n'interdit jamais** — l'application ne décide pas à la
  place de l'oreille.

**Le piège de polarité, à ne jamais rouvrir sans lire ceci.**
[0012](../decisions/0012-rotation-des-morceaux.md) §4 écrit « confort haut :
tirage serré sur les tops ; confort bas : la longue traîne pèse davantage ».
Ce « confort haut » désigne le **sentiment** de confort — le cocon — c'est-à-dire
la valeur **0** de cette échelle, pas 5. Lu à la lettre avec « 5 =
exploration », le curseur s'inverse entièrement. Un test le fige
(`le_cocon_penche_vers_le_connu_et_l_exploration_vers_l_inconnu`).

Détail de forme : avec six valeurs entières, **il n'y a pas de milieu
exact**. 2 penche encore un peu vers le connu, 3 déjà un peu vers l'inconnu ;
la bascule tombe entre les deux.

## Orientations

- **Le réglage structure l'éventail proposé**, pas seulement le choix par
  défaut : à chaque embranchement, les branches sont étalées sur l'axe — une
  plus rassurante que le réglage, une au niveau, une plus aventureuse.
  L'utilisateur acquiert un modèle mental stable (« à gauche je me rassure,
  à droite je m'aventure ») et choisit sans lire.
- **Le réglage est ajustable en cours de parcours.** La valeur à l'ouverture
  n'est qu'un point de départ.
- **Techniquement, le confort est une distance** entre un artiste candidat et
  le centre de gravité de ce que l'utilisateur connaît, dans l'espace
  vectoriel décrit dans [moteur-de-branches.md](moteur-de-branches.md).

## À trancher

- ~~Comment l'application sait ce que l'utilisateur **connaît**~~ —
  **tranché de fait par [0014](../decisions/0014-forme-de-l-appris.md)** :
  c'est `learned/`. Nos écoutes décrues d'abord (saturantes — la dixième
  écoute dit beaucoup moins que la première), et à défaut `classement.json`,
  ramené sur la même échelle 0–1 par son propre maximum, puisque l'un est un
  compte et l'autre un score composite. Restent hors du calcul : les fiches
  modifiées dans le fork, qui pourraient peser un jour.
- Combien de temps l'application attend à un embranchement avant de choisir
  seule : jusqu'à la fin du segment, un délai fixe, ou pas d'attente du tout
  (elle enchaîne et l'utilisateur peut dévier à tout moment) ?
- Le confort est-il un curseur réglé par l'utilisateur, un indicateur affiché
  par l'application, ou les deux ?

## Sa place à l'écran (14/09/2026)

Joel : la jauge doit être **toujours au même endroit sur les deux écrans,
en haut à droite, et toujours avec l'apparence qu'elle a en édition**. Fait.
Un seul rendu (`comfort_spans` dans `tui.rs`) sert l'accueil et l'écoute :
les blocs gardent leur couleur, le libellé reste allumé en permanence (le
noir sur cyan qui ne servait qu'en mode `cc`), et `cc` ajoute « ↑↓ » pour
dire que la jauge est vive. À l'accueil, la jauge prend la place qu'avaient
les indicateurs de statut.

**Le statut de l'accueil ne s'affiche plus que dégradé** (question de Joel :
« est-ce que ces infos sont vraiment utiles ? »). librespot, l'API web et la
sync ne s'affichent que lorsqu'un d'eux cloche (en rouge, sur la deuxième
ligne devant le census) : tout vert, on ne montre rien, la place va au
confort. C'est là qu'ils servent — perte d'auth, API injoignable, sync en
échec.
