# Explorer la discographie d'un artiste

Note ouverte le **07/09/2026**, sur un retour de Joel (retour n° 12 de
[`retours-usage.md`](retours-usage.md)) :

> « J'ai lancé une graine "Cat Power", et cela m'a sorti 3 morceaux de cet
> artiste et le premier s'est lancé. Il se trouve que je n'aime quasiment
> que des morceaux de l'album *What Would the Community Think*. J'aurais
> aimé avoir une commande (`:explore` ?) pour avoir la liste visuelle des
> morceaux disponibles classés par albums, et pouvoir faire des `tt` sur
> les morceaux que j'aime et `tT` sur les morceaux tops existants que je
> veux enlever. »

**Câblée le 07/09/2026**, forme **1a** de la maquette
`Discographie.dc.html` (arbitrage de Joel). Ce qui suit garde le
raisonnement ; ce qui a été tranché est marqué comme tel, et la table des
touches vit dans [`keybindings.md`](../keybindings.md).

## Ce que le retour dit vraiment

Trois morceaux sont sortis parce que la **taille de branche** vaut 3
(`:size`, `src/listen.rs`), pas parce que la fiche a trois tops — c'est un
détail, mais il déplace la question : le problème n'est pas le nombre, c'est
que **le réservoir de Cat Power ne ressemble pas à ce que Joel aime d'elle**.

Or c'est exactement le geste que 0013 promet : `tt` promeut, `tT` retire.
Ce qui manque n'est pas l'édition — elle est câblée depuis le 06/09 et elle
commite — c'est **le fait de ne pouvoir l'exercer que sur le morceau qui
sonne**. Pour redresser un artiste, il faudrait le poncer entièrement, une
écoute par correction. La commande demandée retourne le rapport : on montre
tout, on corrige d'un coup d'œil.

C'est aussi le premier écran où l'on **regarde le catalogue au lieu de
l'écouter**. Il vaut donc mieux qu'il dise tout ce que le moteur sait d'un
morceau, pas seulement son titre : c'est là qu'une fiche se relit.

## Ce sur quoi ça repose — presque tout est déjà là

| Brique | Où | État |
|---|---|---|
| La discographie complète, albums et singles, avec l'album de chaque titre et son `uri` | `src/spotify.rs::discography`, `src/discography.rs` | ✅ récoltée par `:warm`, en cache hors dépôt, régénérable |
| Promouvoir / retirer un top, et le commit qui va avec | `src/edit.rs::add_top`, `remove_top` | ✅ |
| Ce que l'usage sait d'un morceau (écoutes, passages, aimé, banni) | `src/learned.rs` | ✅ |
| Confondre « A Forest » et « A Forest - 2005 Remaster » | `discography::normalize` | ✅ |
| Une liste qui se parcourt à la flèche, plein écran | `tui.rs::render_collection` (la collection de l'accueil) | ✅ le modèle est écrit |

**La fonctionnalité est donc un écran, pas un moteur.** C'est ce qui la rend
petite — et ce qui plaide pour la faire avant les chantiers lourds (le
cooldown daté, l'arbre de la file).

## La forme retenue

**Arbitrage de Joel, 07/09/2026 : la touche est `ad`, et c'est une
modale.** Pas un écran qui remplace l'écoute — une **modale posée sur
l'écran d'écoute**, qui prend le clavier tant qu'elle est ouverte et le
rend à `échap`, comme le réglage du confort prend la main sur les touches
(`comfort_before`). Ce qui joue reste visible derrière : on ne quitte pas
l'écoute pour redresser une fiche, et le son n'a de toute façon jamais
cessé.

`ad` se lit **a**rtist **d**iscography ; `d` était libre dans le namespace
`a`, et 0013 veut que toute touche soit le raccourci d'une commande — c'est
`:discography`.

```
 Cat Power — 214 titres, 19 albums · 3 tops · confort 2
 ────────────────────────────────────────────────────────
  What Would the Community Think (1996)
   ♪  Nude As the News                        12 écoutes
   ♥  Good Clean Fun                           4 écoutes
      They Tell Me                             ·
   ⊘  Enough                                   2 passages
  Moon Pix (1998)
   ♪  Cross Bones Style                        8 écoutes
      Metal Heart                              ·
 ────────────────────────────────────────────────────────
  ↑↓ parcourir · tt top · tT retirer · tl ♥ · tb ⊘ · échap
```

- **Groupé par album, du plus ancien au plus récent** — c'est ainsi qu'on
  se souvient d'un artiste, et c'est la demande.
- **Les glyphes sont ceux de la table** (`keybindings.md`, « d'où vient
  chaque morceau ») : `♪` un top de la fiche, `♥` aimé ici, `↳` une door,
  `⊘` banni, rien pour le reste. Un glyphe ne porte qu'un sens, ici comme
  ailleurs.
- **La colonne de droite est ce que l'appris sait** : écoutes, passages,
  dernière fois. C'est ce qui permet de trancher « je crois que je n'aime
  que cet album » en le vérifiant.
- **Une section finale, « tops introuvables dans la discographie »** : les
  titres que la fiche déclare et que Spotify ne rend pas sous ce nom —
  coquille, live, compilation. C'est la moitié « audit » de l'écran, et
  `tT` doit y fonctionner comme ailleurs.

### Les gestes, dans l'écran

| Touche | Effet | Pourquoi |
|---|---|---|
| ↑ ↓, `gg`, `G` | Déplacer la ligne courante | Comme partout |
| `tt` / `tT` | Promouvoir / retirer des tops **la ligne** | La demande, et les mêmes doigts qu'en écoute |
| `tl` / `tb` | Aimer / bannir **la ligne** | Des mesures : elles n'écrivent que dans `learned/`, rien à commiter |
| `/texte` | Filtrer la liste | Habitude déjà prise pour chercher |
| `échap` | Fermer | Comme un bloc |

`td` (door) n'y est **pas** : une door pointe vers la direction où l'on va
(0011), et cet écran n'a pas de « suivant ». Le geste garde son sens en
écoute, où il en a un.

## Ce que le code a gagné

1. **Quatre champs à `TailTrack`** (faits) : `album_id`, `release_date`,
   `track_number`, `group` (album/single). Sans la date, pas d'ordre
   chronologique ; sans le numéro, pas d'ordre dans l'album. Le cache est
   **régénérable et hors dépôt** — une entrée sans date se relit avec
   `#[serde(default)]` et déclenche une nouvelle récolte, sans migration.
2. **La déduplication à l'affichage.** Spotify livre le même morceau cinq
   fois (album, single, réédition). `normalize` sait déjà les confondre :
   on garde **la plus ancienne occurrence de type album**, on masque les
   autres. Sans cela, Cat Power fait trois cents lignes de doublons.
3. **Le titre écrit dans la fiche doit être propre.** `tt` sur « Nude As
   the News - 2015 Remaster » ne doit pas inscrire ce titre-là : il faut la
   coupe que `normalize` fait déjà, mais qui rend le titre lisible plutôt
   qu'un mot-clé (`clean_title`).
4. **`tT` retire la chaîne de la fiche, pas celle de Spotify.** Elles ne
   sont pas toujours identiques ; la ligne doit donc porter le titre du top
   qu'elle a reconnu, apparié par `normalize`. Sinon `remove_top` ne trouve
   rien et dit « n'est pas dans les tops » alors que le `♪` est affiché.
5. **La récolte à l'ouverture** : si la traîne de l'artiste n'est pas en
   cache, l'écran la récolte (`harvest`) au lieu d'exiger un `:warm`
   préalable. Hors ligne ou sans identifiant Spotify, il le dit et n'ouvre
   que ce qu'il a : les tops de la fiche.

Soit, en volume : `discography.rs` et `spotify.rs` retouchés, un état de
plus dans `Live`, un `render_explore` calqué sur `render_collection`, et
rien de neuf dans `edit.rs` hormis le titre propre.

## Tranché le 07/09/2026, et câblé

1. **La cible.** `ad` vise l'artiste de la ligne **surlignée** s'il y en a
   une, celui du morceau en cours sinon — et l'en-tête de la modale nomme
   l'artiste ouvert. Le reste du namespace `t`/`a` agissait sur le morceau
   en cours, sauf `tx` ; **unifié le 09/09/2026**
   ([0020](../decisions/0020-la-cible-d-un-geste.md)) : tout geste vise la
   ligne surlignée, sinon ce qui sonne.
2. **Le nom et la forme.** `ad` / `:discography`, et une **modale** posée
   sur l'écran d'écoute — pas un écran qui le remplace. La lecture n'a
   jamais cessé, et on la voit derrière.
3. **Un commit pour la fournée.** Les `tt`/`tT` s'accumulent en bas de la
   modale et partent à ⏎ en **une écriture, un commit**
   (`edit::set_tops`) : cinq commits pour une seule pensée ne se relisent
   pas. `u` défait la dernière tant que rien n'est écrit, et le premier
   échap prévient s'il en reste.

   *Question de Joel, ce jour : « pour les commits, il me semblait qu'on
   avait dit à la fermeture, et toutes les dix minutes ».* C'est
   [0017](../decisions/0017-synchronisation-de-l-appris.md), et elle porte
   sur **l'appris** — mesuré, silencieux, jamais relu ligne à ligne. Les
   **éditions** relèvent de 0013 : écrites et commitées au geste, parce
   qu'elles laissent une trace lisible et annulable. La fournée ne change
   pas cette règle, elle en groupe les gestes d'un même écran.
4. **Entrée écrit** — et c'est `e` qui met à la file, sans fermer. Une
   édition ne compte pour le moteur qu'au prochain lancement : `e` est la
   réponse à « je veux l'entendre maintenant ». **Depuis le 11/09/2026,
   entrée sur un morceau part aussi de lui** (Joel : « démarrer une
   nouvelle graine depuis une chanson de l'écran de discographie ») : la
   fournée est écrite d'abord s'il y en a une, puis la graine remplace le
   parcours — le morceau joue, les branches partent de son artiste, comme
   la modale de recherche. Sur une ligne d'album, entrée écrit seulement ;
   un morceau banni ne part pas. Choix réversible : si écrire sans partir
   manque sur un morceau, une touche à part (`w`) le rendra.
5. **Les quatre ajouts retenus** (Joel) : `s` bascule l'ordre
   (chronologique ⇄ mes écoutes d'abord), `v` cycle la vue (tout, ♪ tops,
   ♥ aimés, ⊘ bannis), `A` promeut les quatre titres les plus écoutés de
   l'album, `e` met à la file.

## Ce qui reste ouvert

- **Le repli des albums** ne se juge qu'à l'usage : pas de pliage par album
  mémorisé, `h` plie tout et `l` rouvre celui du curseur.
- **Les compilations et les participations** restent hors récolte
  (`include_groups=album,single`) : un titre qui n'existe que sur une
  compilation n'apparaît pas — sauf s'il est un top de la fiche, auquel cas
  il tombe dans « tops hors discographie ».
- **Le cache de la traîne** a gagné quatre champs (date, rang, durée,
  single). Une récolte d'avant est **refaite en silence** à l'ouverture :
  le cache est régénérable et hors dépôt, il n'y a rien à migrer.

## Ce que ça ne fait pas

- **Ni renommer, ni éditer une fiche à la main** : `ae` reste le geste pour
  ça, et il attend toujours une saisie interrogée.
- **Ni toucher aux liens ni aux tags** : cet écran est celui des morceaux.
- **Ni proposer l'amont** : ce qui est corrigé ici part dans le fork, et
  `:mine` le montre déjà ([0008](../decisions/0008-le-fork-est-la-surcouche.md)).

`ad`, `:discography` et la table de la modale sont dans
[`keybindings.md`](../keybindings.md), marqués ✅. Ce que le code a coûté :
quatre champs de plus dans le cache de la traîne, `edit::set_tops` (la
fournée), `explore.rs` (l'état, dix tests), une table de touches modale
dans `keys.rs`, et `render_explore` dans `tui.rs`. **Non vérifié en session
réelle** — comme tout ce qui a été livré ces deux jours.
