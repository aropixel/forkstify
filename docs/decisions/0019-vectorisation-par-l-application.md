# 0019 — L'application vectorise elle-même

- **Date** : 2026-09-09
- **Statut** : accepté

## Contexte

[0016](0016-base-large-et-generation-a-la-volee.md) a fait de la génération
à la volée le chemin principal par lequel un catalogue grandit, et a laissé
ouvert le sort du **vecteur** d'une fiche générée. Elle naissait sans :
navigable par ses liens et ses tags, mais sourde à tout ce qui passe par le
sens — `vector_neighbors`, le centroïde d'une branche, la branche
aventureuse. Le dégât était silencieux et cumulatif : plus on écoute, moins
l'espace vectoriel couvre le catalogue.

Deux sorties ont été comparées dans
[conception/generation-a-la-volee.md](../conception/generation-a-la-volee.md) :
une commande qui lance le conteneur Python en fond, ou le calcul dans
l'application avec `fastembed` en Rust. La première est une corvée qu'on
oublie, et l'automatiser ferait de docker et python une dépendance
d'exécution d'un lecteur de musique. La seconde a été chiffrée par un essai
le 09/09/2026 : cosinus ≥ 0,999999 avec l'index Python sur les 316 fiches à
condition de tronquer à 128 jetons, build à froid 26 s, binaire +35 Mo,
`g++` dans l'image de build, modèle de 241 Mo téléchargé au premier usage.
L'essai a aussi révélé que l'index Python n'était **pas normalisé** (normes
de 2,5 à 3,6), ce que le centroïde du moteur subissait.

## Décision

- **L'application calcule les vecteurs elle-même** (`embed.rs`) : même
  modèle (`paraphrase-multilingual-MiniLM-L12-v2`, quantifié, 384
  dimensions, mean pooling), même texte composé depuis la structure de la
  fiche que `tools/vectoriser.py`, **troncature à 128 jetons**, vecteurs
  **normalisés**.
- **Une fiche générée naît avec son vecteur, dans le même commit**
  `Forkstify: edit`. Si le modèle est injoignable, la fiche entre quand
  même, l'écran dit « sans vecteur », et `forkstify vectors` rattrape.
- **`forkstify vectors` régénère tout l'index** depuis les fiches et écrit
  `vectors/meta.toml`. Il remplace `tools/vectoriser.py` : la référence et
  les forks vectorisent avec le même binaire, c'est la forme la plus forte
  de « les mêmes vecteurs partout » ([catalogue.md](../conception/catalogue.md)).
- **L'import régénère l'index dans son commit**, sans docker.

## Conséquences

- L'image de build embarque `g++` (ONNX Runtime, lié en statique) ; le
  binaire dépend de `libstdc++.so.6` à l'exécution, présente partout. Les
  features par défaut de `fastembed` tirent OpenSSL : on prend les
  variantes `rustls`, comme tout le reste du binaire.
- Le modèle vit dans `$XDG_CACHE_HOME/forkstify/fastembed`, jamais dans le
  catalogue. Son premier téléchargement demande le réseau, que la
  génération exige déjà, et l'écran le dit avant.
- L'index de référence a été régénéré une fois par l'application
  (09/09/2026) : mêmes directions, normes à 1.
- **Reste à faire** : une édition de lien en session (`aL`) change le texte
  des deux fiches et ne recalcule pas encore leurs vecteurs — l'édition ne
  compte de toute façon pour le moteur qu'au prochain lancement ;
  `forkstify vectors` couvre le cas en attendant.
