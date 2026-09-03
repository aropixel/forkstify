# Vision

> **Reprendre la main sur l'algorithme.**

Forkstify est un lecteur de musique pour Linux, câblé à Spotify, qui
fonctionne **par branches** : on part d'un morceau, l'application en enchaîne
quelques-uns, puis propose plusieurs directions ; on en choisit une — ou on la
laisse choisir — et ainsi de suite. Comme on rechoisit régulièrement selon
l'évolution des morceaux et de son humeur, on ne décroche jamais.

## La philosophie

Les services de streaming recommandent avec un algorithme qu'on ne voit pas,
qu'on ne comprend pas et qu'on ne corrige pas. Forkstify n'est pas un
recommandeur plus malin : c'est un recommandeur **dont on peut lire et
corriger chaque raison**. Ses raisons sont dans des fichiers texte, dans un
dépôt qu'on forke, qu'on améliore et qu'on partage — une **base artiste
libre**, construite au fur et à mesure par ceux qui s'en servent.

Ce n'est pas qu'une affaire d'auditeurs. Les **labels indépendants**
souffrent de l'opacité des algorithmes de Spotify et des autres services —
jusqu'aux plateformes qui poussent des contenus générés par IA pour plus de
rentabilité, au détriment des artistes qu'elles sont censées faire
découvrir. Un catalogue libre, lisible et forkable, où une connexion se
voit, s'explique et se propose, est aussi une réponse pour eux : personne
n'a besoin de payer ou de deviner pour exister dans les branches.

La règle qui en découle, applicable à chaque fonctionnalité : **toute
décision automatique doit être explicable en une phrase et modifiable en un
commit.** Une branche proposée sait dire « parce que connexion `filiation`
dans ta fiche The Cure », ou « parce que voisin dans l'espace vectoriel,
connexion non écrite » — et dans ce second cas, offre de l'écrire.

## Le pitch d'origine

> Je pense à créer une application de musique sur Linux, câblée à mon Spotify,
> où on choisit un morceau pour commencer, et où cela fonctionne ensuite par
> branches. Ex : je sélectionne « The Cure – A Forest », cela commence à me
> jouer le morceau, et me propose 3 branches :
>
> - Branche 1 : « Poncer » The Cure (d'autres morceaux de The Cure)
> - Branche 2 : « The Cure, Cocteau Twins, New Order, … »
> - Branche 3 : etc.
>
> Avec éventuellement un choix ou un indicateur « zone de confort ».
>
> Une fois qu'on a choisi une branche, au bout de quelques morceaux cela
> repropose plusieurs embranchements.

## Principes

1. **Ça ne s'arrête jamais et ça n'exige jamais de décision.** Si l'utilisateur
   ne choisit pas, l'application choisit pour lui selon sa zone de confort.
   Reprendre la main est toujours possible, jamais obligatoire.
2. **Le catalogue est le cœur du produit ; le moteur de branches le fait
   vivre.** On fournit une base — des fiches et leurs vecteurs — qu'on
   s'approprie, qu'on améliore, et qui **apprend de l'usage**. L'interface
   et la lecture audio sont au service de ça. Face à un choix, on privilégie
   ce qui rend le catalogue plus juste et les branches plus lisibles.
3. **L'intelligence est dans des fichiers texte**, pas dans un service. Le
   catalogue se partage, se forke, se corrige, se relit. Spotify est le
   tuyau : il sert à retrouver et jouer les morceaux, rien de plus — et la
   base doit pouvoir lui survivre.
4. **Le nom dit le programme** : *fork* — on forke le catalogue pour se
   l'approprier, on forke un parcours à chaque embranchement.

## Vocabulaire

Les mots du projet. Les utiliser tels quels, dans le code comme dans la doc.

| Terme               | Sens                                                                                   |
|---------------------|----------------------------------------------------------------------------------------|
| **Graine**          | Le morceau de départ choisi par l'utilisateur.                                         |
| **Branche**         | Une direction d'écoute proposée : un titre lisible (« Siouxsie → Cult Hero → Joy Division ») + une règle de sélection des morceaux. |
| **Embranchement**   | Le moment où l'application propose plusieurs branches et attend — ou n'attend pas — un choix. |
| **Segment**         | Les quelques morceaux joués entre deux embranchements.                                  |
| **Parcours**        | La session d'écoute complète : graine, suite des branches choisies, morceaux joués.     |
| **Zone de confort** | Réglage de 0 à 5 : à quel point on accepte de s'éloigner de ce qu'on connaît. Voir [décision 0001](decisions/0001-confort-familiarite.md). |
| **Catalogue**       | L'ensemble des fiches d'artistes (fichiers texte versionnés) qui alimentent le moteur de branches. |
| **Fiche**           | Le fichier d'un artiste dans le catalogue : identité (MBID), tags, tops, liens, description. |
| **Top**             | Un morceau du réservoir par défaut d'un artiste — ce qu'on joue quand on le « ponce ». |
| **Door** (`doors`)  | Un morceau ciblé, occasionnel, qui reçoit un bonus quand on quitte l'artiste vers la direction (tags) qu'il indique. Critère additionnel, jamais principal. |
| **Lien** (`links`)  | Un lien explicite entre deux fiches : un type fermé (`member`, `family`, `collab`, `scene`, `similar`, `influence`), une note qui l'explique, une proximité (par défaut selon le type, corrigeable). |
| **Base / mien / appris** | Les trois états d'une information du catalogue : venue de l'amont ; écrite ou validée par moi ; produite par mon usage. Voir [conception/catalogue.md](conception/catalogue.md). |
| **Promotion**       | Transformer un signal d'usage (*appris*) en connaissance lisible (*mien*) : un commit sur une fiche. |

Types de branches identifiés jusqu'ici (liste ouverte) :

- **Poncer** — rester sur l'artiste courant. Plus une branche depuis le
  03/09/2026 : c'est un geste, la touche `e` (commande `:encore`) —
  « poncer » reste l'argot français du projet pour ce geste.
- **Voisinage** — artistes proches (même scène, même époque, mêmes influences).
- **Décalage** — un pas de côté : même ambiance, autre genre ou autre époque.
- **Retour** — revenir vers la zone de confort quand on s'en est éloigné.
