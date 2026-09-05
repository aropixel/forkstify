# La longue traîne — la quatrième source du réservoir

Note ouverte le **05/09/2026** sur question de Joel (« comment gérer la
longue traîne *behind the scene* ? »), après l'ouverture du réservoir
([0012](../decisions/0012-rotation-des-morceaux.md) §1) qui en implémente
trois sources sur quatre.

Rien n'est codé. Cette note propose, compare et recommande ; l'arbitrage
revient à Joel.

## Décidé — ce qu'on ne rouvre pas

- **La traîne est la quatrième source du réservoir**, à poids faible : le
  top est un poids, pas une liste fermée (0012 §1).
- **C'est la zone de confort qui règle sa profondeur** — confort haut,
  tirage serré sur les tops ; confort bas, la traîne pèse davantage
  (0012 §4). Il n'y a **pas de second réglage** :
  [0001](../decisions/0001-confort-familiarite.md) tient déjà ce curseur.
- **Elle vit hors du catalogue.** 0012 la dit « hors catalogue », et
  [catalogue.md](catalogue.md) range explicitement les caches d'API
  (« résolution titre → identifiant, pochettes ») **hors du dépôt**. Le cas
  des vecteurs ne fait pas jurisprudence : ils sont livrés parce que tout le
  monde doit avoir *les mêmes* pour que « proche » veuille dire la même
  chose partout, et parce qu'ils demandent un modèle de 100 Mo. Une
  discographie n'a ni l'un ni l'autre problème — et redistribuer des
  données d'un tiers dans un dépôt public poserait en plus une question de
  conditions d'usage qu'on n'a pas à se créer.

## Le fait qui tranche la source

Les fiches portent **`mbid` (214/214)** et **`spotify` (212/214)**.
**Aucune ne porte d'identifiant Deezer.**

Passer par Deezer — ce que fait `outillage/generer-fiches.py` avec
`artist/{id}/top` — imposerait donc un `search/artist` par artiste, avec le
risque d'homonymie que le catalogue a déjà payé une fois (« Experience »
résolu à tort en The Jimi Hendrix Experience, corrigé le 02/09).

Passer par Spotify part d'un identifiant **déjà présent et déjà vérifié**,
et rend des `spotify:track:` directement — donc **pas de résolution
titre → identifiant**, et pas le « introuvable sur Spotify » qui l'accompagne.

**Recommandation : la traîne vient de Spotify**, par le `spotify` de la
fiche.

Conséquence acceptée : `parcours` (le mode à sec, sans Premium) ne
*récolte* pas la traîne. Mais il **lit le cache** que les sessions
d'écoute ont rempli, donc il continue de fonctionner, avec la traîne des
artistes déjà rencontrés.

## Orientation — la forme proposée

- **Où** : `~/.cache/forkstify/discography/<slug>.json` (XDG). Régénérable,
  jamais commité, non synchronisé (le retour n° 10 ne le concerne pas).
- **Quoi** : titre + `spotify:track:` + album, dédupliqué par titre
  normalisé, **les tops de la fiche retirés** — la traîne, c'est ce qui
  n'est *pas* déjà dans le réservoir.
- **Le moteur reste synchrone.** Il lit une discographie déjà chargée ; la
  remplir est le travail de l'application, en fond. C'est la même règle que
  « séparer le cerveau du son » : le moteur produit, l'application va
  chercher.
- **Marque d'affichage** : une cinquième provenance à côté de `♪ ♥ ↳ + ~`.

## Câblé le 05/09/2026

Joel a validé les arbitrages ; les deux points que la note laissait ouverts
ont été pris au plus sobre, et dits comme tels : **récolte au moment du
besoin** et **pas de péremption**.

- `src/discography.rs` — le cache, `~/.cache/forkstify/discography/<slug>.json`,
  chargé une fois au démarrage. Le moteur le lit **synchroniquement** ;
  l'application le remplit.
- `WebApi::discography()` — albums et singles, puis leurs pistes par lots de
  vingt : quelques appels par artiste, une fois. Une récolte partielle est
  gardée — la traîne est un réservoir, pas un inventaire.
- **Quand** : automatiquement quand `e<n>` demande plus de profondeur que la
  fiche n'en a, et `:warm` pour la forcer sur l'artiste en cours.
- **Le curseur trouve son troisième levier** : la part de la traîne dans le
  réservoir est exactement l'ouverture du confort (0012 §4). **Zéro au
  cocon**, pleine à l'exploration. Un test le fige.
- **Déduplication par titre normalisé** : Spotify livre la même chanson sous
  dix habillages (« - 2004 Remaster », « (Remastered) »), et un titre déjà
  dans les tops n'entre pas dans la traîne — la traîne, c'est ce qui n'est
  *pas* déjà dans le réservoir.
- **Marque `·`**, à côté de `♪ ♥ ↳ + ~`.

Le champ `spotify` des fiches, présent depuis toujours et jamais lu par le
code, l'est enfin : c'est lui qui ouvre la porte.

## À trancher — ce qui reste

*(Les trois points ci-dessous sont tranchés ; conservés pour mémoire du
raisonnement.)*

1. **La profondeur.** ~~Le top élargi~~ (`/v1/artists/{id}/top-tracks`, 10
   titres, un appel) recoupe largement les tops de la fiche et n'est donc
   presque pas une traîne. La vraie traîne demande la **discographie** :
   `/v1/artists/{id}/albums` puis `/v1/albums?ids=` par lots de 20 — de
   l'ordre de 3 à 10 appels par artiste, une fois, puis c'est en cache.
2. **Le moment de la récolte.** En fond dès qu'une branche est affichée
   (les artistes proposés sont connus d'avance) ; ou seulement à
   `e<n>` quand les tops d'un artiste sont épuisés — le moment où la
   profondeur sert vraiment ; ou une commande de préchauffage
   (`:warm`) qui fait tout le catalogue en une fois.
3. **La péremption.** Jamais (une discographie bouge peu, et un `:warm`
   force la mise à jour), ou un TTL.

## L'ordre, et pourquoi il compte

**La zone de confort (0001) devrait venir avant la traîne.**

0012 §4 est explicite : le confort *est* le volume de la traîne, et il n'y
a pas de second réglage. Livrer la traîne sans lui, c'est lui donner un
poids fixe arbitraire — donc soit elle ne s'entend jamais, soit elle
déborde, et dans les deux cas on ne peut pas la régler autrement qu'en
recompilant.

0001 est déjà décidée et `learned/` fournit depuis aujourd'hui la
familiarité dont elle a besoin. La brancher d'abord donne à la traîne son
bouton de volume le jour où elle arrive.

**Fait le 05/09/2026** (arbitrage de Joel) : `engine::Comfort` existe, il
pilote le plancher de l'aventureuse et le tirage des têtes. Il lui manque
son troisième levier — **la profondeur du tirage dans le réservoir**
(0012 §4), qui n'a rien à régler tant que la traîne n'existe pas. Le
bouton attend son volume.
