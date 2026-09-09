# L'écran d'accueil

Note ouverte le **05/09/2026** : Joel veut que les sous-commandes
(`parcours`, `check`, et à terme `ecouter`) disparaissent au profit d'un
simple `forkstify` qui ouvre l'application. Il faut donc un écran d'accueil,
qu'il maquettera avec Claude Design.

Rien n'est codé, rien n'est décidé. Cette note rassemble ce que le dépôt
contraint déjà, ce que les données permettent vraiment, et les questions à
trancher.

## Ce qui est déjà écrit

`forme-de-l-application.md` a tranché **deux entrées** pour la graine :

- **`/` puis du texte** — recherche fusionnée catalogue + Spotify.
  **Implémentée** le 04/09/2026.
- **Une liste** — « la bibliothèque de l'utilisateur, artistes et albums
  aimés sur Spotify, parcourue au clavier, filtrée par `/` ». Jamais écrite.

L'écran d'accueil est donc le lieu où cette liste existe enfin. Il ne part
pas de zéro.

## Ce que les données permettent — les chiffres

Relevés le 05/09/2026 sur le catalogue réel.

| | |
|---|---|
| Artistes dans `classement.json` (la bibliothèque de Joel) | **741** |
| Fiches du catalogue | **214** |
| Artistes classés **qui ont une fiche** | **175** |
| Fiches d'artistes **absents** du classement | **39** |

Et la couverture par tranche du classement :

| Tranche | Ont une fiche |
|---|---|
| top 20 | 20/20 |
| top 50 | 48/50 |
| top 100 | 83/100 |
| score ≥ 20 | 25/25 (100 %) |
| score ≥ 5 | 82/89 (92 %) |
| score ≥ 2 | 122/289 (42 %) |
| tout (score ≥ 1) | 175/741 (23 %) |

**Trois conclusions qui commandent la maquette :**

1. **Le haut de l'écoute est intégralement couvert.** Une liste « vos
   habitués » trouvera toujours une fiche. Aucun risque de proposer un
   artiste depuis lequel on ne peut pas brancher.
2. **Le bas ne l'est pas.** Sous le score 2, moins d'un artiste sur deux a
   une fiche — et **une graine sans fiche ne peut pas démarrer un parcours**
   (`resolve()` exige une carte, et les branches viennent des liens et des
   vecteurs de la fiche). L'axe « pousser ce qu'on écoute peu » bute donc
   sur le bord du catalogue.
3. **Le score médian du classement est 1.** La traîne du classement est du
   bruit (un artiste croisé une fois). Le vivier utile, c'est le score ≥ 5 :
   89 artistes, presque tous avec fiche.

## Le point aveugle : il n'y a pas encore d'usage

`learned/artists/` **n'existe pas** — aucune session réelle n'a encore été
jouée. Au premier lancement, l'écran d'accueil n'aura donc que
`classement.json`, c'est-à-dire **une photo de la bibliothèque Spotify**, pas
un usage de forkstify.

C'est la même contrainte que la zone de confort : l'écran doit être **bon le
premier jour avec zéro historique**, et meilleur ensuite. Une maquette qui
suppose des données d'écoute riches décrira un écran qu'on ne verra pas
avant des semaines.

## Sur l'idée de lister le compte Spotify

**Ne pas le faire, et c'est une bonne nouvelle.** `classement.json` *est*
déjà la bibliothèque de Joel, récoltée par `tools/` : titres aimés,
albums aimés, playlists, artistes suivis, #fipway, road trip. C'est plus
riche que ce que l'API rendrait, et c'est **local, instantané, hors ligne**.

Repasser par l'API coûterait en plus une **réautorisation** : `/v1/me/top/artists`
demande le scope `user-top-read`, absent de nos cinq scopes actuels
(`spotify.rs`). Ce serait payer un OAuth pour une donnée qu'on a déjà en
moins bien.

## L'idée forte : « délaissé » vaut mieux que « peu écouté »

Joel propose d'opposer souvent écouté / peu écouté pour ne pas toujours
tourner sur les mêmes. La couche `learned/` permet mieux que ça.

Les compteurs de [0014](../decisions/0014-forme-de-l-appris.md) **décroissent
d'eux-mêmes** (demi-vie six mois) et chaque artiste porte un `last`. On peut
donc distinguer :

- **peu écouté** — familiarité basse, on ne l'a jamais vraiment fréquenté ;
- **délaissé** — familiarité qui *fut* haute et qui a décru, `last` ancien :
  « vous les aimiez, vous ne les écoutez plus ».

Le second est infiniment plus juste comme relance : c'est un rappel, pas une
découverte. Et il est calculable dès qu'il y a de l'usage, sans donnée
nouvelle. Le premier jour, seul « peu écouté » existera (via le classement).

## La proposition : chaque bloc est une raison

La règle de marque du projet — *toute décision automatique s'explique en une
phrase* — s'applique à l'accueil. Un écran qui montre six artistes doit dire
**pourquoi** chacun est là. Donc : pas de grille indifférenciée, mais quelques
**portes d'entrée**, chacune portant sa raison en une ligne.

Pistes, de la plus sûre à la plus discutable :

1. **Chercher** (`/`) — la première ligne, toujours. Déjà implémentée, c'est
   l'échappatoire qui rend tout le reste facultatif.
2. **Reprendre** — la dernière graine et où on s'est arrêté. Demande une
   donnée nouvelle (le dernier parcours), minuscule.
3. **Vos habitués** — familiarité haute. Marche le premier jour.
4. **Délaissés** — familiarité décrue, `last` ancien. Ne marche qu'après
   usage ; le premier jour, remplacer par « du catalogue, jamais écoutés »
   (les 39 fiches absentes du classement sont exactement ça).
5. **Au hasard** — tirage pondéré, la porte qui ne demande pas de choisir.

**Et le mélange devrait être gouverné par la zone de confort**, pas par un
nouveau réglage. Le curseur 0–5 dit déjà « je reste chez ce que je connais »
ou « je vais vers ce que je ne connais pas » ; c'est exactement l'axe entre
« habitués » et « délaissés ». Au cocon l'accueil met les habitués devant, à
l'exploration il met les délaissés. **Un seul axe dans le produit** —
[0012](../decisions/0012-rotation-des-morceaux.md) §4 dit déjà « pas de
deuxième réglage ».

## Une porte de plus, très en phase avec le projet

**Les artistes qu'on écoute beaucoup et qui n'ont pas de fiche** — 17 dans
le top 100. Les montrer, c'est proposer de faire grandir le catalogue là où
l'usage le réclame, ce qui est la promesse de
[0002](../decisions/0002-catalogue-partage-forkable.md) et la *promotion* du
vocabulaire.

Mais c'est une **édition** (créer une fiche), et aucune édition n'est câblée.
À garder pour plus tard, et à ne pas maquetter comme si ça marchait.

## Tranché par Joel le 05/09/2026

- **La graine peut être les deux** : un artiste (on démarre un segment sur
  lui, branches natives) ou un morceau (on le joue, puis on branche depuis
  son artiste s'il a une fiche). C'est déjà ce que fait la recherche `/`
  depuis le 04/09 ; l'accueil applique la même règle, et la contradiction de
  vocabulaire tombe : `vision.md` dit « le morceau de départ », le code seed
  sur un artiste — les deux sont vrais.
- **Deux écrans, selon l'état de connexion.** Un écran spécifique quand on
  n'est pas connecté, le vrai accueil quand on l'est. L'accueil ne se
  dégrade donc pas : il n'existe qu'une fois les autorisations en place.

Conséquence à ne pas manquer : **il y a deux autorisations distinctes**, et
l'écran non connecté doit les traiter séparément —

1. **librespot** : les identifiants viennent du téléphone par zeroconf (on
   tape le nom de l'appareil dans Spotify). Sans eux, aucun son.
2. **l'API Web** : OAuth navigateur, client id ncspot. Sans elle, on ne
   résout aucun titre.

Et deux situations très différentes se cachent derrière « non connecté » :
**jamais autorisé** (accueil de premier lancement, il faut expliquer les deux
gestes) et **autorisation perdue** (le jeton a expiré — cas réel, corrigé le
05/09 : l'application redemande désormais l'autorisation d'elle-même). La
seconde ne doit pas ressembler à la première.

Enfin : **le catalogue est local**. Même sans aucune connexion, les fiches,
les vecteurs et `learned/` sont lisibles — la navigation à sec reste
possible. L'écran non connecté n'est donc pas un cul-de-sac.

**La vue de la collection** (Joel, 09/09/2026) : par défaut elle ne montre
que **les aimés** — un artiste ou un titre aimé ici, un titre, un album ou
un suivi sur Spotify (`classement.json`) — et `v` bascule sur **tous** les
artistes du catalogue. Le tri `s` s'applique à la vue.

## Les maquettes (Claude Design, 05/09/2026)

Projet **« Accueil Forkstify »**, deux planches :

- **`Raccourcis.dc.html`** — le menu du leader, un namespace à moitié tapé,
  les touches multimédia. **Ce n'est pas une proposition** : c'est le rendu
  fidèle de `help()`, vérifié ligne à ligne contre `src/listen.rs`. Il vaut
  comme spécification visuelle de l'existant.
- **`Accueil.dc.html`** — trois écrans : **A1** non connecté premier
  lancement, **A2** autorisation perdue, **B** l'accueil connecté. Props
  réglables : le confort (0–5, qui réordonne réellement les blocs), l'écran
  montré, le thème Omarchy.

Ce que la maquette adopte, et qui tient : chaque bloc porte sa raison en une
ligne · la numérotation `1-5` court **à travers** les blocs, donc choisir une
graine est le même geste que choisir une branche · la graine mélange artistes
et morceaux dans la même liste (l'entrée 3 est un morceau, avec sa raison :
« il se joue, puis les branches partent de New Order ») · le confort fait
passer les délaissés devant à partir de 4 · A2 dit que librespot tient
toujours, seuls les titres ne se résolvent plus — ce qui est exactement le
comportement réel.

### Quatre frottements — tranchés et corrigés le 05/09/2026

1. **`p` reste la pause.** « Parcourir à sec » passe sur **`b`** (*browse*),
   lettre libre et mot anglais comme 0015 l'exige.
2. **« Au hasard » n'a pas de touche neuve** : c'est **entrée**, qui veut dire
   « choisis pour moi » partout ailleurs. Le `*` de la maquette disparaît.
3. **`:comfort`**, en anglais, comme le code l'écrit déjà.
4. **Le nom zeroconf est passé à « forkstify (omarchy) »** dans le code
   (`src/sound.rs`) : ici c'était la maquette qui avait raison.

Ajouté au clavier : **`r`** (*resume*) pour reprendre. Les deux nouvelles
touches sont sans préfixe, le test exhaustif de `keys.rs` le vérifie.

### Les frottements, tels qu'ils ont été relevés

1. **`p` est déjà la pause.** La maquette lui donne « parcourir à sec » sur
   les écrans non connectés. Il n'y a pas de lecture à ce moment-là, donc pas
   de collision *technique* — mais l'utilisateur apprend une touche, pas une
   touche par écran. Et « parcourir » n'est pas un mot anglais, alors que
   0015 l'exige. `b` (*browse*) ou `d` (*dry*) sont libres.
2. **`*` pour « au hasard »** n'est pas un mot anglais non plus. Et le geste
   existe déjà : **entrée** veut dire « choisis pour moi » partout ailleurs.
   La réutiliser ici ne coûte aucune touche neuve.
3. **`:confort`** apparaît sur la ligne d'invite de l'écran B — la commande
   réelle est `:comfort`, en anglais comme toute interface publique du dépôt
   (l'autre planche l'écrit correctement).
4. **Le nom de l'appareil zeroconf** affiché est « forkstify (omarchy) » ; le
   binaire annonce aujourd'hui **« forkstify (spike) »**
   (`src/bin/spike-connect.rs`). Le nom de la maquette est meilleur — c'est le
   code qu'il faudra changer, pas la maquette.

### Deux comportements que la maquette suppose et qui n'existent pas

Ils sont justes tous les deux, mais ce sont des chantiers, pas de l'affichage.

- **forkstify s'annonce lui-même en zeroconf pendant que l'écran A1
  s'affiche** (« en attente — aucun appareil ne s'est encore annoncé »).
  Aujourd'hui la découverte vit dans le binaire séparé `spike-connect` ;
  `Sound::connect()` se contente de relire le cache et échoue s'il est vide.
  Il faut déplacer la boucle de découverte dans l'application.
- **L'autorisation OAuth est différée** : la maquette attend qu'on appuie sur
  une touche pour ouvrir le navigateur. Aujourd'hui `WebApi::new()` l'ouvre
  **tout seul** au lancement. Le choix de la maquette est meilleur — on ne
  veut pas d'un navigateur qui surgit à chaque démarrage — mais c'est un
  changement de flux.

## À trancher

1. ~~**Graine = un artiste ou un morceau ?**~~ — tranché : les deux. `vision.md` dit « le morceau de
   départ », le code seed sur un **artiste** (`resolve()` rend un slug de
   fiche). L'écran d'accueil force à choisir — ou à assumer les deux, comme
   la recherche le fait déjà : un artiste démarre un segment, un morceau se
   joue puis branche depuis son artiste.
2. **Que deviennent `parcours` et `check` ?** `check` (les voisins d'une
   fiche) est utile et pourrait devenir un geste sur un artiste sélectionné.
   `parcours` est un outil de développement — un drapeau, ou rien.
3. **L'accueil se rend-il avant la connexion Spotify ?** Aujourd'hui
   `ecouter` ouvre librespot **et** l'OAuth avant d'afficher quoi que ce
   soit. Un accueil devrait s'afficher **instantanément** depuis le catalogue
   et `learned/` — tous deux locaux — et ne connecter qu'au moment de jouer.
   C'est un changement d'ordre d'initialisation, pas un détail d'affichage.
4. **Combien d'entrées par bloc ?** Trois branches à un embranchement ; la
   même contrainte de lisibilité vaut sans doute ici.
