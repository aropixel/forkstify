# La génération de fiche à la volée

Note vivante. La décision [0016](../decisions/0016-base-large-et-generation-a-la-volee.md)
a acté que **l'application génère une fiche quand on arrive chez un artiste
qui n'en a pas** ; elle a laissé ouvert *quand* cela se déclenche
([premiere-installation.md](premiere-installation.md) § À trancher, point 3).
Cette note tient le sujet.

## Ce qui l'a déclenchée

Le 09/09/2026 au matin, Joel : « j'ai envie d'écouter Jacques Brel mais il
n'est pas dans le catalogue. J'ai pu le trouver via la recherche Spotify,
mais je ne peux pas jouer le morceau et cela ne crée pas la fiche artiste. »

Le comportement était conforme au code : depuis l'accueil, un résultat
Spotify sans fiche répond « rien d'où brancher » ; depuis l'écoute,
`:search` le joue en `Offmap` mais n'écrit rien. Le catalogue ne pouvait
grandir que par `tools/generate-cards.py`, hors session.

Deuxième constat, le même matin : la fiche de Brel, une fois générée par le
script, portait trois liens `similar` vers Brassens, Moustaki et Gainsbourg.
Deux avaient une fiche, pas le troisième — et `graph_neighbors`
(`engine.rs:179`) **jette un lien dont la fiche manque**. Une moitié du
voisinage de Brel était donc invisible. C'est le même manque vu de l'autre
bout : le catalogue ne grandit pas le long de ses propres liens.

## Décidé

**Les deux déclencheurs** (Joel, 09/09/2026).

1. **La recherche fait entrer quelqu'un de neuf.** Entrée sur un résultat
   hors catalogue génère la fiche de son artiste, la commite, puis démarre
   chez lui. C'est le geste du matin, sans touche à apprendre.
   `:generate <nom>` fait la même chose sans passer par un morceau.
2. **L'arrivée fait grandir le catalogue le long de ses liens.** Un lien
   vers une fiche absente cesse d'être jeté : il s'affiche comme une branche
   en creux, et s'y engager génère la fiche avant de marcher.

Aucun des deux ne suffit seul : le premier ne sait faire entrer que ce
qu'on nomme, le second que ce que le catalogue pointe déjà.

**C'est une édition au sens de [0013](../decisions/0013-affinage-clavier-mesure-ou-edition.md)** —
un commit lisible, `generated = true` jusqu'à relecture — comme 0016 le
prévoyait.

**Le catalogue devient mutable pendant la session.** Il était prêté en
lecture seule pour toute la durée d'une écoute (`Live<'a> { catalog: &'a
Catalog }`), si bien qu'une fiche écrite n'existait qu'au lancement suivant
— ce que la table des touches assume pour les éditions (« une édition ne
compte pour le moteur qu'au prochain lancement »). Pour une génération c'est
intenable : on la demande pour écouter *maintenant*. La session possède donc
désormais son catalogue et y insère la fiche au retour du travail de fond.

## Orientation

**Les sources sont celles du script**, déjà éprouvées sur 316 fiches :
MusicBrainz pour l'identité, les dates, l'origine, les genres et les
relations typées ; Deezer, sans clé, pour les tops et les similaires. Le
pipeline est celui de [catalogue.md](catalogue.md) § Le démarrage à froid —
« un seul pipeline, trois moments », et l'écoute est le troisième.

**Une différence avec le script, et une seule** : il abandonne les liens
dont la cible est hors de l'univers connu, parce qu'un lien vers rien est
du poids mort. Dans l'application ce n'est plus vrai — un lien vers une
fiche absente est désormais une branche à générer. Les quatre `similar` de
Deezer sont donc conservés tels quels ; les relations MusicBrainz, elles,
restent réservées aux fiches existantes, sinon le moindre musicien de
session deviendrait une direction.

**La génération demande le réseau**, et 0016 veut que l'application le dise
plutôt que d'échouer.

## À trancher

- **La vectorisation d'une fiche générée** — *ouvert, Joel décidera plus
  tard (09/09/2026)*. Une fiche naît aujourd'hui sans vecteur : elle
  navigue par ses liens et ses tags, ce qui n'est pas absurde puisqu'elle
  vient d'être composée à partir d'eux, mais elle reste absente de tout ce
  qui passe par le sens — `vector_neighbors`, le centroïde de `stay`, la
  branche aventureuse. Deux sorties :

  | | Ce que ça donne | Ce que ça coûte |
  |---|---|---|
  | **(a) Graphe seul, et le dire** | La fiche marche tout de suite par ses liens ; le vecteur se rattrape par lots, quand on veut. Il resterait à ajouter une commande `:vectors` qui lance le conteneur en fond — `import.rs` sait déjà le faire pour les fiches reprises | Rien de neuf. Mais l'artiste reste sourd au vecteur tant qu'on n'a pas rattrapé, et il faut docker |
  | **(b) `fastembed` en Rust** | Le vecteur se calcule dans forkstify, à la génération ; plus de docker ni de python dans la boucle, et la fiche est complète du premier coup | ONNX Runtime en dépendance : binaire et temps de compilation qui gonflent. Le modèle (220 Mo) est déjà dans `tools/cache/fastembed` |

  **En attendant, c'est (a) sans la commande** : la fiche naît sans vecteur
  et l'écran le dit (« sans vecteur : navigation par le graphe »), le
  rattrapage se fait à la main avec la commande docker qu'imprime déjà
  `tools/vectoriser.py`. C'est la seule position qui ne préjuge de rien —
  aucune des deux sorties n'a de code à défaire pour être prise.

- **Une fiche, ou son voisinage ?** Brel généré seul avait trois liens vers
  le vide ; il a fallu générer Brassens derrière pour que la branche mène
  quelque part. Faut-il générer en cascade (l'artiste et ses similaires
  manquants), au risque de quatre fois le réseau et quatre fiches qu'on n'a
  pas demandées ? Le déclencheur « à l'arrivée » rend la question moins
  urgente : le voisinage se génère au fur et à mesure qu'on y va.

- **Reverser à l'amont.** [catalogue.md](catalogue.md) § La mutualisation
  veut qu'une fiche générée absente de la référence soit proposée en PR
  pré-mâchée. Rien ne le fait ; à concevoir avec `:mine`.
