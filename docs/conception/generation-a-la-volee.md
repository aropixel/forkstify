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

- **La vectorisation d'une fiche générée** — *ouvert, Joel décidera
  (09/09/2026)*. Une fiche naît aujourd'hui sans vecteur : elle navigue
  par ses liens et ses tags, ce qui n'est pas absurde puisqu'elle vient
  d'être composée à partir d'eux, mais elle reste absente de tout ce qui
  passe par le sens — `vector_neighbors`, le centroïde de `stay`, la
  branche aventureuse. Et depuis que la génération est le chemin principal
  par lequel le catalogue grandit, le trou s'élargit à chaque écoute, sans
  bruit : plus on écoute, moins l'espace vectoriel couvre le catalogue. Deux
  sorties :

  | | Ce que ça donne | Ce que ça coûte |
  |---|---|---|
  | **(a) Graphe seul, et le dire** | La fiche marche tout de suite par ses liens ; le vecteur se rattrape par lots, quand on veut, par une commande `:vectors` qui lance le conteneur en fond — `import.rs` sait déjà le faire | Rien de neuf. Mais c'est une corvée qu'on oublie, et l'automatiser ferait de docker et python une dépendance d'exécution d'un lecteur de musique |
  | **(b) `fastembed` en Rust** | Le vecteur se calcule dans forkstify, à la génération — et à toute édition, ce que [catalogue.md](catalogue.md) promet déjà (« l'application recalcule localement le vecteur d'une fiche modifiée ») ; la fiche est complète du premier coup | ONNX Runtime en dépendance. Chiffré ci-dessous |

  **L'essai (09/09/2026, `~/Work/tries/fastembed-spike`)** — un binaire
  jetable qui vectorise les 316 textes composés par `vectoriser.py
  --textes` et les compare à `vectors/vectors.jsonl` :

  | Mesure | Valeur |
  |---|---|
  | Crate | `fastembed` 6.0.3, variante `ParaphraseMLMiniLML12V2Q`, features `hf-hub-rustls-tls` + `ort-download-binaries-rustls-tls` (les features par défaut tirent OpenSSL) |
  | Fidélité | cosinus Rust/Python **≥ 0,999999** sur les 316 fiches, **à condition de `max_length = 128`** (défaut du crate : 512 ; Python tronque à 128, et à 512 The Cure tombe à 0,82 — les fiches riches dépassent 128 jetons) |
  | Modèle | même dépôt HF que Python (`qdrant/…-onnx-Q`, 241 Mo), téléchargé au premier usage dans le cache indiqué (le crate écrit `Qdrant` avec une majuscule : le cache Python n'est pas réutilisé) |
  | Build à froid | **26 s** dans le conteneur, registre cargo déjà chaud ; `target/` 557 Mo |
  | Image de build | il faut **`g++`** en plus (ONNX Runtime est en C++, lié en statique) ; à l'exécution une seule dépendance dynamique de plus, `libstdc++.so.6` |
  | Binaire | **+35 Mo** environ (le spike seul fait 37 Mo, forkstify 23 Mo aujourd'hui) |
  | Exécution | chargement du modèle **0,8 s** à chaud, **14 ms** par texte, 6 s pour les 316 |

  **Découverte au passage** : les 316 vecteurs de la référence **ne sont pas
  normalisés** (normes de 2,5 à 3,6 — le fastembed Python ne normalise pas
  ce modèle, le Rust si). Le cosinus n'y voit rien, mais le centroïde de
  `vector_neighbors_of` additionne les vecteurs bruts : un artiste pèse
  selon sa norme, et un vecteur venu du Rust pèserait trois fois moins que
  ses voisins. Quel que soit le choix, **l'index se régénère une fois par
  un seul vectoriseur**, et si c'est l'application, `forkstify vectors`
  remplace `tools/vectoriser.py`.

  **Orientation de l'agent** : (b). Le coût mesuré est raisonnable (26 s,
  35 Mo, `g++` dans l'image), la fidélité est totale à 128 jetons, et
  c'est la seule sortie où le trou se referme sans corvée ni docker. Le
  vecteur d'une fiche générée part dans le **même commit** `Forkstify: edit`
  que la fiche, sinon l'index livré diverge côté fork ; le fichier unique
  trié par slug s'y prête. Repli si Joel juge le binaire trop lourd : une
  feature cargo `embed`, avec le message « sans vecteur » actuel sans elle.

- **Une fiche, ou son voisinage ?** Brel généré seul avait trois liens vers
  le vide ; il a fallu générer Brassens derrière pour que la branche mène
  quelque part. Faut-il générer en cascade (l'artiste et ses similaires
  manquants), au risque de quatre fois le réseau et quatre fiches qu'on n'a
  pas demandées ? Le déclencheur « à l'arrivée » rend la question moins
  urgente : le voisinage se génère au fur et à mesure qu'on y va.

- **Reverser à l'amont.** [catalogue.md](catalogue.md) § La mutualisation
  veut qu'une fiche générée absente de la référence soit proposée en PR
  pré-mâchée. Rien ne le fait ; à concevoir avec `:mine`.
