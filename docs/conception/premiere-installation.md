# La première installation, et la vie du catalogue

Note ouverte le **06/09/2026**, sur une série de questions de Joel :
que se passe-t-il quand quelqu'un d'autre installe forkstify ? faut-il
scanner son Spotify ? créer les fiches qui manquent ? et surtout —
« comment conjuguer catalogue de base commun à tous et ajouts de
l'utilisateur ? ».

La dernière est déjà tranchée dans le dépôt, mais éparpillée entre quatre
décisions. Cette note la rassemble, puis dit ce qui manque vraiment.

## D'où viennent les 780 noms de la colonne

Question de départ : « il y a plus d'artistes que ce que j'avais sur
Spotify ». Oui, et c'est exact :

| | |
|---|---|
| Le classement, tiré de la bibliothèque Spotify | **741** |
| Fiches du catalogue | **214** |
| Fiches d'artistes **absents** du classement | **39** |
| **La colonne = l'union des deux** | **780** |

Ces 39 ne viennent pas de Spotify : ce sont des artistes que le **catalogue
a appelés lui-même**. Le générateur suit les `links` d'une fiche vers les
artistes qu'elle cite, si bien que Cult Hero entre parce que The Cure le
nomme. Le catalogue n'est donc pas une copie de la bibliothèque : c'est un
**voisinage** autour d'elle, et il déborde exprès.

## Le modèle est déjà décidé — trois couches, un seul dépôt

Quatre décisions le disent, chacune une pièce :

- **[0002](../decisions/0002-catalogue-partage-forkable.md)** — le catalogue
  est un dépôt de fichiers texte, partagé et **forkable**.
- **[0004](../decisions/0004-deux-depots-catalogue-ciblable.md)** —
  **importer = cloner**, puis déclarer « c'est celui-là que j'utilise ». On
  peut en avoir plusieurs et basculer.
- **[0008](../decisions/0008-le-fork-est-la-surcouche.md)** — « il n'y a pas
  de surcouche à part. Le catalogue actif est un clone git, et les
  modifications personnelles sont des **commits dedans** ».
- **[0014](../decisions/0014-forme-de-l-appris.md)** — l'usage vit à part,
  dans `learned/`, versionné mais **jamais reversé**.

D'où la réponse à « comment conjuguer la base et mes ajouts » : **on ne les
conjugue pas, on les superpose dans le même dépôt, et git fait le travail.**

| Couche | Où | Partageable ? |
|---|---|---|
| **La base** | les fiches telles qu'elles viennent de l'amont | c'est l'amont |
| **Le mien** | mes commits sur ces fiches — `tt`, `td`, `aL`, `ae` | **oui**, c'est ce qu'on propose en PR |
| **L'appris** | `learned/` — compteurs, dates, poids, bans | **jamais** |

Mettre à jour la base, c'est `git pull` sur l'amont : mes commits sont
au-dessus, les siens en dessous, et un conflit n'arrive que si nous avons
touché la même ligne de la même fiche — ce qui est rare, une fiche par
artiste étant justement faite pour ça. Contribuer, c'est une **PR** qui ne
contient que des fiches. `learned/` n'y entre jamais, et l'application le
sait.

La **promotion** est le pont entre les deux dernières couches : « tu as
choisi six fois la branche Cocteau Twins depuis The Cure, j'ajoute la
connexion ? ». Un signal d'usage devient une connaissance lisible, et c'est
un commit qu'on peut relire et annuler.

## Ce qui manque vraiment

Le modèle tient ; c'est son **amorce** qui n'existe pas.

1. **Rien ne clone le catalogue.** L'application lit
   `~/Work/forkstify-catalog` s'il est là et échoue sinon. Un nouvel
   utilisateur doit cloner à la main. `forkstify` devrait proposer d'importer
   le catalogue de référence au premier lancement — c'est 0004, jamais écrit.
2. **Rien ne scanne le Spotify de l'utilisateur.** `classement.json` a été
   produit par sept scripts Python de `tools/`, lancés à la main par Joel,
   avec sa session. Pour quelqu'un d'autre, ce fichier n'existe pas : il
   n'aurait **aucune familiarité de départ**, donc un accueil sans habitués
   ni délaissés, et une zone de confort qui ne pencherait vers rien.
3. **Rien ne génère de fiche à la volée.** `tools/generate-cards.py` sait
   le faire (MusicBrainz pour les faits, Deezer pour les tops et les
   similaires), mais c'est un script hors de l'application. Or
   [catalogue.md](catalogue.md) en fait un mécanisme central : « arrivée chez
   un artiste sans fiche → génération d'une fiche, marquée générée jusqu'à
   relecture ».

## Le point dur, qui n'est écrit nulle part

**La base actuelle n'est pas neutre : c'est l'univers de Joel.** Les 214
fiches ont été générées depuis son classement et de proche en proche. Un
utilisateur qui aime le jazz ou le rap US trouverait un catalogue qui ne
parle pas de lui — et comme **une graine sans fiche ne peut pas démarrer**,
il ne pourrait presque rien lancer.

Trois sorties, à trancher :

- **(a) Une base neutre et large**, générée en amont sur quelques milliers
  d'artistes courants. Coûteux à produire, mais l'installation marche pour
  tout le monde tout de suite. C'est ce que « catalogue de référence »
  suppose implicitement.
- **(b) Une base mince, et la génération à la volée devient obligatoire.**
  Le catalogue de chacun grandit vers son univers dès la première écoute.
  Fidèle à « le catalogue couvre l'univers de l'utilisateur et grandit avec
  ses écoutes », mais rend l'application dépendante des API au démarrage.
- **(c) Plusieurs bases, par famille de goût**, qu'on importe selon soi
  (0004 le permet déjà : « celui de quelqu'un d'autre parce qu'on le trouve
  cool »). Le plus fidèle à l'esprit du projet, le plus lourd à amorcer.

Rien n'oblige à choisir maintenant — **tant que forkstify n'a qu'un
utilisateur, la question ne se pose pas**. Mais elle décide de ce que
signifie « catalogue de référence », et donc de ce qu'on met dans le premier
dépôt public.

## Tranché : (a) **et** (b) — décision [0016](../decisions/0016-base-large-et-generation-a-la-volee.md)

Arbitrage de Joel, 06/09/2026. La base de référence vise la **largeur**, et
l'application **génère une fiche à la volée** quand on arrive chez un artiste
qui n'en a pas. Les deux se complètent : la largeur fait que l'installation
marche tout de suite, la génération fait qu'elle ne reste jamais étrangère.
(c) n'est pas écartée — 0004 permet déjà d'importer le catalogue de
quelqu'un d'autre, aucune décision n'est nécessaire pour ça.

### Un fork, pas un dépôt de différences

Précision demandée par Joel : « chaque utilisateur a son repo de
modifications ? ». **Non — il a un *fork*.**

    aropixel/forkstify-catalog         la référence, l'amont
        └── kbyjoel/forkstify-catalog      son fork : TOUT le catalogue, plus ses commits
                └── ~/…/forkstify-catalog      son clone local, celui que l'application lit

Son dépôt contient **tout le catalogue**, pas seulement ses changements.
C'est ce qui permet les deux mouvements : `git pull` depuis l'amont pour
recevoir les fiches des autres, et une **PR** vers l'amont pour proposer les
siennes. Un dépôt qui ne contiendrait que les différences ne saurait faire ni
l'un ni l'autre — et contredirait
[0008](../decisions/0008-le-fork-est-la-surcouche.md), « il n'y a pas de
surcouche à part ».

`learned/` vit dans ce même fork, versionné pour être portable d'une machine
à l'autre, mais **n'entre jamais dans une PR** (0014).

Le chemin du catalogue actif est désormais un réglage, `[catalogue] path`,
l'argument de ligne de commande le surchargeant.

Depuis le 08/09/2026, c'est exactement la situation de Joel : la référence
est chez l'organisation `aropixel`, son catalogue est un fork dans son compte
`kbyjoel`, avec l'amont en `upstream`.

## À trancher

1. ~~**(a), (b) ou (c)**~~ — tranché : (a) et (b), voir 0016.
2. **Le scan de la bibliothèque doit-il entrer dans l'application** — ou
   rester un outillage lancé à part ? Il demande un scope OAuth de plus
   (`user-library-read` est déjà là ; `user-follow-read` et `user-top-read`
   ne le sont pas) et plusieurs centaines d'appels.
3. ~~**La génération de fiche à la volée**~~ — tranché le 09/09/2026 :
   **les deux**, la recherche pour faire entrer quelqu'un de neuf et
   l'arrivée pour grandir le long des liens. Le sujet a désormais sa note,
   [generation-a-la-volee.md](generation-a-la-volee.md) ; la
   **vectorisation** de la fiche générée est tranchée par
   [0019](../decisions/0019-vectorisation-par-l-application.md).
4. **L'import au premier lancement** : forkstify clone-t-il lui-même, ou
   demande-t-il une URL ?

## Mesurer l'usage à travers les forks

Depuis [0017](../decisions/0017-synchronisation-de-l-appris.md), chaque
commit que l'application produit — appris, édition, import — porte un
trailer git `Forkstify: <kind> <version>`. La recherche de commits de GitHub
indexe les messages des dépôts publics, ce qui permet de compter ces commits
à travers tous les forks sans rien demander à personne :

```sh
# tous les commits produits par forkstify, dépôts publics confondus
gh api search/commits -f q='"Forkstify:"' --jq .total_count

# par nature : l'appris, les éditions de fiches, les imports
gh api search/commits -f q='"Forkstify: learned"' --jq .total_count
gh api search/commits -f q='"Forkstify: edit"'    --jq .total_count
gh api search/commits -f q='"Forkstify: import"'  --jq .total_count

# les dépôts concernés, un par ligne
gh api search/commits -f q='"Forkstify:"' --paginate \
  --jq '.items[].repository.full_name' | sort | uniq -c | sort -rn

# et le nombre de forks du dépôt de référence, qui compte les utilisateurs
gh api repos/aropixel/forkstify-catalog --jq .forks_count
```

Limites : seules les **branches par défaut** des dépôts **publics** sont
indexées ; un fork privé n'est pas compté ; le trailer dit que forkstify a
écrit le commit, pas qui. Tant que le dépôt de référence est privé, ces
commandes ne comptent que ce qu'on y pousse soi-même.
