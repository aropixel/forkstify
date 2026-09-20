# Avant de sortir : les trois derniers chantiers

Note ouverte le **19/09/2026**, à la demande de Joel : « affiner les
dernières choses avant de pouvoir sortir le projet ». Trois sujets, un
cahier. Chaque chantier distingue ce qui est **demandé** (Joel, 19/09/2026,
sauf mention), ce qui est **proposé** (l'agent, à valider) et ce qui reste
**à trancher**. Rien n'est codé tant que la section « à trancher » du
chantier n'est pas vidée.

Les notes de fond restent celles-ci, et cette note y renvoie plutôt que
de les recopier : [premiere-installation.md](premiere-installation.md)
pour le modèle base / mien / appris et l'amorce,
[catalogue.md](catalogue.md) pour ce qui est partageable,
[generation-a-la-volee.md](generation-a-la-volee.md) pour les creux.

## L'état de départ, relu dans le code le 19/09/2026

- **Pas de setup.** `forkstify` lit `[catalog] path`, vide = `~/Work/forkstify-catalog`,
  et `Catalog::load` échoue si le dossier manque. La bibliothèque Spotify
  entre par cinq scripts Python de `tools/` lancés à la main, qui lisent le
  trousseau GNOME, puis `classement.py` écrit `learned/classement.json`
  (clés françaises, lues par `learned.rs`). Ce qui existe déjà et servira :
  l'écran **non connecté** (`home::disconnected_rows`) qui guide les deux
  autorisations, l'OAuth PKCE navigateur (`spotify.rs`, cinq scopes),
  la découverte zeroconf du téléphone, `WebApi` qui pagine, `import.rs` qui
  sait ajouter un remote et prendre des fiches, `generate.rs` et
  `embed.rs` pour les fiches manquantes.
- **Le catalogue, côté git.** Le fork de Joel a **217 commits d'avance** sur
  la référence, zéro de retard : 165 fichiers d'appris, 49 fiches ajoutées
  ou retouchées (`git diff --stat upstream/main -- cards`), et l'index de
  vecteurs entièrement réécrit (normalisation 0019). `:mine` calcule le
  diff des fiches contre `upstream/main` et l'affiche en overlay. Les
  commits de l'application sont faits par `git commit` sans identité
  explicite : ils prennent le `user.name` global de la machine.
- **La référence porte l'appris de Joel** : `learned/classement.json`,
  ses cinq récoltes `artistes-*.json`, `faits-mb.json`, `mbid.json` et
  18 fiches d'appris sont sur `aropixel/forkstify-catalog`. Un fork neuf
  hériterait de la bibliothèque de Joel. À nettoyer avant la sortie (voir
  la liste finale).
- **Les creux.** `engine::missing_neighbors` rend les liens du contexte
  qui pointent vers une fiche absente ; la colonne en montre trois au plus,
  en gris « ○ no card yet », numérotés après les branches. `f<n>` sur un
  creux appelle `generate(…, After::Branch)` : la fiche naît, **et la
  branche part** — il n'y a pas de chemin qui génère sans brancher. Les
  creux sont fréquents parce qu'une fiche générée garde ses quatre
  `similar` Deezer même vers le vide, ce qui est voulu (0016).

---

## Chantier A — le setup après l'installation

**Fait le 20/09/2026**, d'après la maquette `Installation.dc.html` (Claude
Design, projet « Accueil Forkstify », neuf écrans) : `src/setup.rs` (les
écrans et le fil), `src/library.rs` (la récolte, `learned/library.toml`,
le classement, la couverture), `keys::parse_setup` (la table : chiffres,
`j`/`k`, `o`, espace coche, `⏎`, échap), `tui::render_setup` (une colonne,
le pas et sa jauge à droite, l'invite en dernière ligne). **La maquette a
tranché les points ouverts** : le mode local reste (« il évite un mur le
premier soir »), un seul `library.toml`, seuil ≥ 5 et 30 fiches au plus,
les deux scopes acceptés avec la ré-autorisation qu'ils entraînent. Ce
qui s'écarte de la maquette, faute de mieux : à l'étape 7 la génération
**se regarde** (échap l'arrête après la fiche en cours, ce qui est écrit
est commité) au lieu de continuer derrière l'accueil ; l'étape 4 se
regarde aussi (échap annule). Et `:setup` / `:library` **ferment la
session** — le son s'arrête — puis l'accueil revient sur le catalogue
rejoué. Le premier lancement, c'est `forkstify` sans catalogue lisible.
Les scripts de `tools/` ne sont pas encore retirés du catalogue : après
que Joel a rejoué `:library` sur son fork. **Non éprouvé en session
réelle** : la récolte et la génération demandent le réseau et les jetons ;
le fil (clone, identité, confort, récapitulatif) a tourné dans un test de
fumée sur des dossiers XDG temporaires.

### Demandé

Un setup **au premier lancement et rejouable** (Joel, 09/09/2026,
[premiere-installation.md](premiere-installation.md) § À trancher, 5) :
connexion, import de la bibliothèque, playlists à cocher, classement
calculé par l'application, scripts Python retirés. Joel fournit une
maquette Claude Design avant qu'on code les écrans. Ce chantier fixe **les
étapes à présenter et les informations à recueillir**, pour que la
maquette parte d'une liste arrêtée.

### Proposé : sept étapes, dans cet ordre

Le fil conducteur : **on n'a rien à taper qu'on ne sache déjà**, et chaque
étape peut être sautée puis rejouée seule. L'écran est un écran de la
session comme l'accueil (0021, « l'accueil est un écran de la session »),
tout se dit en toast, la grammaire du clavier reste celle des modales
(`j`/`k`, `espace` coche, `⏎` valide, `échap` saute).

| # | Étape | Ce qu'on recueille | Ce qu'on écrit |
|---|---|---|---|
| 1 | **Le catalogue** | L'URL de **son fork** de la référence (ou rien) | le clone dans `~/.local/share/forkstify/catalog`, `origin` = le fork, `upstream` = la référence, `[catalog] path` dans `config.toml` |
| 2 | **L'identité git** | `user.name` / `user.email` s'ils manquent | `git config --local` dans le clone, jamais global |
| 3 | **La connexion** | rien à taper : le téléphone (zeroconf) et le navigateur (OAuth) | les deux jetons, là où ils vivent déjà (`~/.local/state/forkstify`) |
| 4 | **La bibliothèque** | un « oui » | `learned/library.toml` : titres aimés (artiste principal seul), albums aimés, artistes suivis |
| 5 | **Les playlists** | celles à **cocher** dans la liste de ses playlists | leurs identifiants dans `learned/library.toml`, pour que rejouer soit un seul geste |
| 6 | **Le confort** | un chiffre 0–5, à la jauge (`cc` existe) — défaut 3 | l'état `comfort`, comme aujourd'hui |
| 7 | **La couverture** | un « oui » pour générer les fiches manquantes du haut de sa bibliothèque | des fiches `generated = true`, leurs vecteurs, un commit |

Détail par étape :

1. **Le catalogue.** Le cas normal est **un fork** : c'est ce que 0016
   et 0008 supposent, et ce que la synchronisation de l'appris (0017)
   exige — elle pousse sur `origin`, ce qu'un clone de la référence ne
   permet pas. Trois entrées : (a) l'URL de son fork, collée ; (b) si `gh`
   est installé et connecté, forkstify propose de **forker lui-même**
   (`gh repo fork aropixel/forkstify-catalog --clone`) ; (c) rien —
   forkstify clone la référence en **mode local** : tout marche, l'appris
   se commite mais ne se pousse pas, et l'accueil le dit (`⇅ local`).
   `:catalog fork <url>` (chantier B) fait passer de (c) à (a) plus tard
   sans rien perdre : on ajoute le remote, on pousse. Le chemin par défaut
   quitte `~/Work` : `~/.local/share/forkstify/catalog` (XDG), Joel
   gardant son `[catalog] path` actuel.
2. **L'identité git.** Un commit sans `user.name` échoue, et forkstify
   commite en permanence (0017). Si la config globale les a, rien n'est
   demandé. Sinon on demande le nom et le mail et on les écrit
   **localement** dans le clone : c'est l'identité des commits du catalogue,
   pas celle de la machine.
3. **La connexion.** C'est l'écran non connecté d'aujourd'hui, inséré
   dans la suite : rien de nouveau, sinon **deux scopes de plus** pour les
   étapes 4 et 5 — `user-follow-read` (artistes suivis) et
   `playlist-read-private` (ses playlists, dont les privées). Conséquence :
   les utilisateurs déjà autorisés — Joel — repasseront **une fois** par le
   navigateur, et le produit doit le dire au lieu de laisser croire à une
   panne.
4. **La bibliothèque.** La récolte des scripts, réécrite sur `WebApi` :
   `/me/tracks`, `/me/albums`, `/me/following?type=artist`, paginés
   (plusieurs centaines d'appels pour une grosse bibliothèque, avec
   `Retry-After` respecté — ~2 min pour 5 000 titres). Une barre de
   progression par source. **L'artiste principal seul** compte (Joel,
   09/09/2026 : les invités d'un titre aimé ne sont pas des aimés).
5. **Les playlists.** La liste de ses playlists (nom, nombre de titres,
   propriétaire), les siennes d'abord, à cocher. Une playlist cochée
   compte ses artistes ×1 comme aujourd'hui (`#fipway`, road trip). Les
   identifiants cochés sont **mémorisés** dans `learned/library.toml`, si
   bien que « re-récolter » est un seul geste sans rien recocher — la
   question laissée ouverte le 09/09 est tranchée par là si Joel est
   d'accord.
6. **Le confort.** La jauge de `cc`, avec ses mots (cocon → exploration),
   et la phrase de 0001 : « elle choisit seule quand tu ne choisis pas ».
7. **La couverture.** On croise le classement et le catalogue : « 48 de
   tes 50 artistes les plus présents ont une fiche ; en générer 12 de plus
   pour couvrir tout ce qui a un score ≥ 5 ? (~3 s chacune) ». C'est la
   base large **et** la génération (0016) à l'échelle d'une installation :
   le nouveau venu au goût éloigné de la référence a de quoi démarrer
   dès la première soirée, sans attendre d'arriver chez chacun. Plafond
   (30 ?) et seuil (score ≥ 5, celui de l'écran d'accueil) à régler ;
   **un seul commit** pour la fournée, comme `import` — pas un par fiche.

**Le classement calculé par l'application.** `learned/library.toml`
remplace `classement.json` et les cinq `artistes-*.json`, en **vocabulaire
anglais** (0014 l'attendait, 0022 l'impose) : par artiste, `name`,
`spotify`, `liked_tracks`, `liked_albums`, `followed`, `playlist_tracks`,
`score`, `sources` ; en tête, la date de récolte et les playlists cochées.
Le score garde la formule de `classement.py` — titres ×1, albums ×3, suivi
+8, playlists ×1 — en constantes du code, pas dans `[tuning]` : 0023 règle
les indices du moteur, pas la récolte. `learned.rs` lit le nouveau fichier
et **encore l'ancien** tant qu'il existe, pour que le fork de Joel ne
change pas de comportement le jour du basculement ; la résolution des MBID
(`resoudre-mbid.py`, `mbid.json`) n'a plus d'objet — la génération
résout par le nom, avec Deezer en secours.

**Rejouable.** `:setup` depuis l'accueil rejoue la suite, chaque étape
déjà faite s'affichant cochée et se sautant d'un `⏎` ; `:library` rejoue
les seules étapes 4-5-7. En ligne de commande, `forkstify setup` fait la
même chose pour le plugin Omarchy, qui installe le binaire et pourra le
lancer. Le premier lancement, c'est simplement `forkstify` **sans
catalogue lisible** : il ouvre le setup au lieu d'échouer.

**Ce qui se retire ensuite.** Les scripts de récolte et de classement
(`bibliotheque-*.py`, `playlist-spotify.py`, `classement.py`,
`resoudre-mbid.py`), `generate-cards.py` et `vectoriser.py` (déjà
remplacés, 0019), `voisins.py` (`forkstify check`). Restent `amis-*.py`,
qui n'ont pas d'équivalent dans l'application et attendent des amis
consentants — à traduire (étape 11 de l'avancement) ou à déplacer hors
du catalogue de référence.

### À trancher

Tranché en bloc par la maquette du 20/09/2026 :

1. ~~**Le fork obligatoire ou non**~~ — le mode local est fait.
2. ~~**Le format de `learned/library.toml`**~~ — un seul fichier.
3. ~~**Plafond et seuil de l'étape 7**~~ — score ≥ 5, 30 au plus,
   reproposée à chaque `:library` (l'écran 9 la marque « to do »).
4. ~~**Les deux scopes de plus**~~ — acceptés ; l'écran 3 dit que ce
   n'est pas une panne.
5. ~~La maquette~~ — `Installation.dc.html`.

Reste, à l'usage : si la génération de l'étape 7 doit un jour continuer
derrière l'accueil comme la maquette le montre. Les scripts de `tools/`
sont retirés le 20/09/2026 (Joel) ; `classement.json` reste lu dans le
fork tant que `:library` n'a pas écrit `library.toml`.

---

## Chantier B — le namespace « catalogue »

**Fait le 20/09/2026**, d'après la maquette `Catalogue.dc.html` (sept
écrans) : `src/fork.rs` (`status`, `diff`, `propose`, `update`, `resume`,
`fork`), la table (`C` pending, `Cd` `Cp` `Cu`, `o`), les jobs hors de la
boucle dans `listen.rs`, `:catalog` et ses sous-commandes. `:mine` et
`edit::mine` sont partis. Ce qui s'écarte de la maquette : `:catalog`
s'affiche en overlay comme `Cd`, pas dans le flux ; l'overlay de `Cd` ne
déroule pas (douze nouvelles, puis « … n more ») et n'ouvre pas la fiche ;
`o` sur un conflit ouvre la fiche dans l'éditeur du bureau (`xdg-open`),
faute de pouvoir rendre `stdin` à `$EDITOR`. **Non éprouvé en vrai** :
`Cp` jusqu'à `gh pr create`, `Cu` sur le vrai fork de Joel ; mais `diff`,
`propose` (deux fois, la branche réécrite), `update` qui fusionne, `update`
qui s'arrête sur une fiche et `resume` sont éprouvés par un **test
d'intégration sur trois dépôts git temporaires** (la référence, le fork,
le clone) — `git` est entré dans l'image `forkstify-build` pour cela.

### Demandé

Regrouper sous une lettre les gestes sur le catalogue : `:mine` (renommé
en *diff*), une commande qui fait une **PR des nouvelles fiches vers la
référence**, une commande qui **rebase le fork sur la référence**.

### Proposé

**La lettre : `C`, majuscule** (arbitrage de Joel, 19/09/2026). `c`
est prise par le confort depuis le 08/09/2026 (`c<n>`, `cc`) ; la
partager entre deux sujets, à la façon de `f` (chiffre = branche, lettre
= opération), a été proposé et **écarté** par Joel — deux sens sous une
lettre ne se lisent pas. `m` (*mine*) a été envisagé, `z` pour déplacer le
confort aussi. `C` garde le mot *catalog*, reste libre (`G`, `J`, `K` sont
les seules majuscules nues), et la majuscule marque le geste **rare et
lourd**, comme `A` promeut un album entier dans la modale. C'est le
premier namespace en majuscule ; la grammaire reste sans préfixe, le
test `grammar_is_prefix_free` le vérifie. Trois gestes, du mot anglais
comme partout :

| Touche | Mot | Commande | Action |
|---|---|---|---|
| `Cd` | catalog **diff** | `:catalog diff` | Ce que ce catalogue a de plus que la référence — l'actuel `:mine`, renommé ; les fiches seulement |
| `Cp` | catalog **propose** | `:catalog propose` | Proposer ces fiches à la référence : une branche, un push, la PR ouverte dans le navigateur |
| `Cu` | catalog **update** | `:catalog update` | Rapatrier la référence dans le fork, régénérer l'index, recharger le catalogue de la session |
| — | | `:catalog fork <url>` | Faire d'un clone local un fork (chantier A, sortie du mode local) — rare, pas de touche ; remplace le `:fork` 📋 de la table |
| — | | `:catalog` | L'état en une ligne : n commits d'avance / de retard, dernière mise à jour, remotes |

Ce sont des gestes **rares** — 0015 leur donne une commande `:` ; les
touches sont un confort pour les trois du quotidien, et la ligne `C` de
l'aide (`espace`, `C`) les montre. `:mine`
disparaît sans alias (sobriété : un nom).

**`Cd` — diff.** Même calcul qu'aujourd'hui (`git diff upstream/main --
cards/`), même overlay, deux ajouts : chaque fiche dit si elle est
**nouvelle** (`+ generated`, `+ written`) ou **retouchée** (`~ +3 −1`), et
l'en-tête donne le compte et la date de la dernière mise à jour. Toujours
les fiches seulement : ni `learned/`, ni `vectors/`.

**`Cp` — propose.** Le fork de Joel montre le problème : `main` mêle 165
commits d'appris aux fiches, une PR de `main` serait illisible et
reverserait l'appris — ce que 0014 interdit. On ne propose donc **pas des
commits, mais l'état des fiches**, comme `import` le fait dans l'autre
sens (« the state of their cards, never their history ») :

1. `git fetch upstream` ;
2. une branche `proposal` **depuis `upstream/main`**, dans un **worktree**
   à part (`~/.local/state/forkstify/proposal`) — le clone que la session
   lit ne change jamais de branche, l'appris continue de se commiter sur
   `main` toutes les dix minutes ;
3. `git checkout main -- cards/` dans ce worktree : les fiches telles
   qu'elles sont, sans `learned/` ni `vectors/` ;
4. un commit `Propose N cards` dont le corps est écrit **pour le
   relecteur**, en deux listes (voir « La relecture côté référence »
   ci-dessous) : les fiches nouvelles générées d'abord, une ligne chacune
   — nom, MBID, tags —, puis les fiches retouchées avec le diff résumé et
   la note de provenance des liens (catalogue.md § « Tout le mien n'est
   pas également partageable ») ; trailer `Forkstify: proposal <version>` ;
5. `git push --force origin proposal` — **une seule proposition ouverte à
   la fois**, la branche se réécrit et la PR ouverte se met à jour ;
6. la PR elle-même, **deux voies selon le poste** (arbitrage de Joel,
   19/09/2026) : si `gh` est installé **et connecté** (`gh auth status`),
   forkstify montre le titre, le corps et le compte des fiches, et
   **demande confirmation** — `y` crée la PR (`gh pr create --head
   <compte>:proposal --title … --body …`), toute autre touche n'envoie
   rien, la branche poussée restant là ; le toast donne l'URL de la PR.
   Sinon, le navigateur s'ouvre sur la page de comparaison GitHub, titre
   et corps pré-remplis dans l'URL, comme `ag` ouvre un artiste
   (`xdg-open`) : on relit, on clique. Dans les deux cas c'est la « PR
   pré-mâchée » de catalogue.md, et rien ne part sans un geste de plus.
   Une proposition déjà ouverte n'en crée pas une seconde : le push de
   l'étape 5 l'a mise à jour, et le toast le dit avec son URL
   (`gh pr list --head proposal`, ou rien à faire côté navigateur).

**Les vecteurs n'entrent pas dans la PR.** L'index est dérivé (0019) et
réécrit en entier à chaque régénération : dans une PR il ne serait que du
bruit et des conflits. C'est l'action GitHub de la référence qui régénère
à la fusion (ci-dessous).

**La relecture côté référence** (Joel, 20/09/2026 : « j'ai peur que les
validations de PR soient un peu laborieuses de mon côté »). La mesure sur
son propre fork, après un mois d'usage : **46 fiches nouvelles, toutes
`generated = true`, 3 fiches retouchées à la main** (14 lignes ajoutées,
4 retirées). Une fiche générée est la sortie du pipeline, MusicBrainz puis
Deezer — la référence aurait produit la même : il n'y a rien à y relire,
il y a des choses à **vérifier**, et une machine le fait mieux. Ce qui
demande une oreille, ce sont les retouches, rares. D'où trois pièces,
tranchées par Joel le 20/09/2026 :

1. **Une action GitHub sur la référence** (en place le 20/09/2026,
   `forkstify validate`), qui vérifie chaque PR : TOML lisible et `format = 1` ; `mbid` présent et **unique dans tout le
   catalogue** (c'est elle qui attrape un « Ye » proposé alors que
   `kanye-west` existe) ; slug conforme au nom ; cibles des `links` en
   slugs valides ; aucun fichier hors de `cards/`. À la fusion sur `main`,
   elle **régénère l'index** (`forkstify vectors`) et le commite — le
   mainteneur n'y touche plus. L'action tourne le binaire dans le
   conteneur `forkstify-build`, comme `bin/build`.
2. **`Cp` compose la PR pour le relecteur** : les deux listes de l'étape
   4. On survole la première, on lit la seconde.
3. **Une règle de fusion écrite dans le dépôt de la référence**
   (`CONTRIBUTING.md`, en anglais — 0022) : une PR qui **n'apporte que
   des fiches générées** se fusionne sur un coup d'œil — nom et MBID,
   pour l'homonyme que l'action ne voit pas, comme Les Thugs — dès que
   l'action est verte. Une PR qui **retouche** des fiches existantes se
   lit : les faits (`member`, `collab`, `family`) se prennent ; un
   `similar` se prend s'il porte sa note de provenance ; un changement de
   tops se prend s'il **corrige une erreur** (mauvais titre, version
   live, identifiant Spotify faux), pas s'il exprime un goût — les tops
   de la référence ne sont que les portes d'entrée d'un fork vierge
   ([0018](../decisions/0018-un-seul-geste-pour-le-gout.md)).

Écarté pour l'instant : l'auto-fusion GitHub des PR « fiches nouvelles
seulement » vertes — le coup d'œil sur le nom et le MBID vaut d'être
gardé tant que le rythme le permet. En réserve si même cela pèse : `Cp`
ne proposerait par défaut que les fiches nouvelles, les retouches ne
partant qu'en cochant les fiches dans une modale.

**`Cu` — update : une fusion, pas un rebase.** Joel dit « rebase » ; je
propose **merge**, pour une raison de 0017 : `main` est partagé par deux
postes qui tirent en `pull --rebase` et poussent au fil de l'eau. Rebaser
`main` sur `upstream/main` réécrit des commits déjà poussés, et l'autre
poste se retrouve avec une histoire qui a divergé sous ses pieds. Une
fusion ne réécrit rien, et la structure du catalogue la rend presque
toujours triviale : une fiche par artiste (les nouvelles fiches de la
référence arrivent sans conflit), l'appris à part. Le résultat est le même
pour l'utilisateur — les fiches de la référence sont là — et `Cd` reste
juste puisqu'il compare des états, pas des histoires. Les étapes :

1. l'appris sale est commité d'abord, comme `:sync` ;
2. `git fetch upstream` puis `git merge --no-edit upstream/main` ;
3. `vectors/` en conflit : on prend n'importe lequel et on **régénère**
   l'index sur place (0019), dans un commit qui suit ; `cards/` en conflit
   — les deux côtés ont touché la même fiche — on s'arrête, on nomme les
   fiches, et on laisse la main (« l'amont a enrichi la description de
   The Cure, tu as changé les tops » : le guidage champ par champ imaginé
   dans catalogue.md est un chantier à part, pas celui-ci) ;
4. la session **recharge son catalogue** — elle le possède depuis le
   09/09, une fiche arrivée de la référence peut donc combler un creux
   affiché sans relancer — l'appris ne bouge pas ;
5. un toast : « ⇅ 41 cards from upstream · index regenerated ».

`git rebase` reste possible à la main pour qui n'a qu'un poste ; le
produit ne le propose pas.

### À trancher

1. ~~**La lettre**~~ — tranché le 19/09/2026 : `C`.
2. ~~**Merge plutôt que rebase** pour `Cu`~~ — tranché le 19/09/2026 :
   la fusion.
3. ~~**Une seule proposition ouverte à la fois**~~ — tranché le
   20/09/2026 : la branche `proposal`, réécrite.
4. ~~**Le navigateur plutôt que `gh`**~~ — tranché le 19/09/2026 : `gh`
   avec confirmation quand il est là et connecté, le navigateur sinon.
5. ~~Le nom `Cp`~~ — tranché le 20/09/2026 : `Cp`, *propose*.

**Le chantier B n'a plus rien à trancher** : la lettre `C`, les trois
gestes, la fusion pour `Cu`, `gh` avec confirmation sinon le navigateur,
une seule branche `proposal`, et la relecture côté référence.

---

## Chantier C — générer un creux sans le prendre

**Fait le 20/09/2026** (`src/keys.rs`, `src/engine.rs::branch_from`,
`src/listen.rs`) : `fg<n>` génère la fiche du creux n avec l'intention
`After::Gap` ; à l'arrivée, la fiche devient une branche **à la suite des
branches affichées** — donc au numéro du creux quand il était le premier
—, les autres ne bougent pas, et les creux se rafraîchissent autour du
contexte *et* de la fiche fraîche. La branche est une marche
(`engine::branch_from` → `walk`), pour `f<n>` comme pour `fg<n>` ; le
`f<n>` d'un creux ne donne plus un encore déguisé. Trois tests : `fg2`
se lit et la grammaire reste sans préfixe ; la branche d'une tête
fraîche traverse plus d'un artiste ; une tête inconnue ne donne rien.
Non éprouvé en session réelle. Reste en réserve : `fga`, et la
génération d'avance en option.

### Demandé

Dans les branches proposées, un artiste **sans fiche** ne se prend
aujourd'hui qu'en le mettant à la file. Joel veut **générer la fiche
seule** — `fg<n>` — et qu'à l'arrivée de la fiche, les branches se
proposent **en tenant compte** de la nouvelle fiche.

### Proposé

**`fg<n>` — fork generate.** Après `f`, `g` est une opération, comme `r`,
`w`, `u` ; `fg` attend son chiffre, la grammaire reste sans préfixe. Sur
un creux affiché, `fg<n>` lance la génération avec une intention
nouvelle, `After::Gap`, à côté de `After::Branch` : la fiche est composée,
vectorisée, commitée et adoptée par la session exactement comme
aujourd'hui (0013 : une génération est une édition) — **mais rien n'est
mis à la file**. Sur un numéro de branche jouable, `fg<n>` répond « n has
a card already » ; sur un creux en cours de génération, « already
underway » (la garde `generating` existe).

**À l'arrivée : le creux devient une branche, à sa place — et une vraie
branche.** Plutôt que de rejouer trois branches — ce qui remélangerait ce
que l'utilisateur était en train de lire, la raison même pour laquelle
`⏎` tire parmi les branches affichées (Joel, 05/09/2026) — la ligne
« ○ no card yet » se change en **branche jouable au même numéro**, la
marque ○ remplacée par celle de la source du morceau. Les deux autres
branches ne bougent pas.

**Ce qu'elle contient** (Joel, 20/09/2026 : « pas uniquement des
morceaux de l'artiste qui vient d'être généré — une branche régénérée
comme les autres, avec un morceau de l'artiste généré et d'autres
morceaux d'autres artistes ») : une **marche**, comme toute branche
proposée — la fiche fraîche en tête, puis un morceau par artiste
traversé, tirés dans son voisinage de graphe et de vecteurs, à la taille
`:size`. C'est `engine::walk`, celui de `propose` et de `wander`, avec la
fiche fraîche pour tête, la raison du lien pour raison et sa proximité
pour poids. Et c'est une **correction au passage** : aujourd'hui,
`branch_to` — le chemin du `f<n>` sur un creux — construit la branche
avec `engine::encore`, donc n morceaux du seul artiste généré ; un creux
pris donnait un « encore » déguisé en branche. `f<n>` et `fg<n>` passent
tous deux par la marche. Un voisin de la fiche fraîche qui n'a pas de
fiche est ignoré par la marche, comme partout — mais il apparaît en
creux, ci-dessous.

Puis la liste des creux se **rafraîchit** depuis le contexte : les liens
de la fiche fraîche vers le vide apparaissent à leur tour en gris — c'est
le catalogue qui grandit le long de ses liens, un cran plus loin. Le
toast : « ✓ Georges Moustaki — card ready · branch 3 ». `fr` reste là
pour qui veut trois autres branches, et la fiche fraîche est alors dans
le vivier comme les autres ; `⏎` peut désormais tirer la branche, ce
qu'il ne fait jamais sur un creux.

Ce que ça change dans le code, pour mesurer : une variante d'`After`, un
cas dans `keys::parse` et son test, `walk` exposé (ou un `branch_from`
sur le modèle de `wander`) et `branch_to` qui l'appelle à la place
d'`encore`, `recompute` des seuls creux après adoption, la ligne `fg<n>`
dans l'aide et la table. Un test : la branche d'un creux généré traverse
plus d'un artiste quand le voisinage le permet.

**Un pas de plus, à discuter : la génération d'avance.** Si les creux
gênent, c'est qu'ils attendent un geste. Une option `[generation]
prefetch = true` ferait générer **en fond et sans bruit** les creux
affichés (trois au plus, ~3 s chacun), comme `harvest_proposed` va
chercher la traîne des branches proposées : le ○ disparaîtrait de
lui-même, et `fg<n>` ne servirait plus qu'à forcer. Le prix : des appels
MusicBrainz à chaque recalcul, et un catalogue qui grossit de fiches
qu'on n'a jamais visitées — moins gênant qu'il n'y paraît, puisque `Cp`
les proposera à la référence et que chaque fiche née enrichit le commun
(catalogue.md § La mutualisation). Je propose `fg<n>` d'abord, l'option
ensuite si l'usage le demande, **désactivée par défaut**.

### À trancher

1. ~~**En place plutôt que rejoué**~~ — tranché le 20/09/2026 : en
   place, au numéro du creux, et la branche est une **marche** comme les
   autres, pas un encore de l'artiste généré.
2. ~~**Le nom**~~ — tranché le 20/09/2026 : `fg`.
3. **`fga` — tout générer** (les trois creux) : pas avant que le besoin
   se montre deux fois.
4. La **génération d'avance**, en option, plus tard.

---

## L'ordre proposé

1. **Chantier C** — le plus petit, aucune décision lourde, il rend
   l'écoute plus fluide tout de suite et Joel l'éprouve dès le lendemain.
2. **Chantier B** — `Cd` et `Cu` d'abord (Joel en a besoin pour suivre la
   référence quand elle sera nettoyée), `Cp` ensuite : il demande que la
   référence soit prête à recevoir.
3. **Chantier A** — le plus gros ; il attend la maquette et il touche au
   format de l'appris. Ses étapes 1-3 (catalogue, identité, connexion)
   peuvent se faire avant la maquette : elles n'ont pas d'écran à
   dessiner, ce sont des questions posées l'une après l'autre.

## Ce que la sortie demande en plus, hors de ces trois chantiers

Relevé au passage, pour ne pas le perdre — chaque point est une ligne,
à trancher ailleurs :

- ~~**La référence porte l'appris de Joel**~~ — retiré le 20/09/2026,
  avec `tools/` des deux dépôts. Le premier `Cu` du fork règle seul les
  conflits *modify/delete* sur `learned/` (les siens gardés).
- **Le chemin par défaut** `~/Work/forkstify-catalog` est celui du poste
  de Joel — chantier A, étape 1.
- **Un `README.md`** dans le dépôt de l'application (il n'y a
  qu'`AGENTS.md`), et celui du catalogue en anglais (0022).
- ~~**L'action GitHub de la référence** et son `CONTRIBUTING.md`~~ — en
  place le 20/09/2026 : `.github/workflows/catalog.yml` (`check` sur
  chaque PR — fiches seules, `forkstify validate` ; `index` sur `main` —
  `forkstify vectors` commité), l'action composite qui construit forkstify
  depuis `aropixel/forkstify`, `CONTRIBUTING.md` en anglais. Tant que
  l'application est privée, le secret `FORKSTIFY_TOKEN` (PAT à grain fin,
  *Contents: read* sur `aropixel/forkstify`) est à créer sur la référence.
- **La licence du catalogue** (catalogue.md § À trancher : ODbL ou
  CC BY-SA) et celle du code (`manifest.json` dit MIT).
- **Les commits d'édition parlent français** (corrections en attente de
  l'avancement) — `Cp` les rendra visibles à la référence.
- Les deux dépôts sont **privés** ; la commande de comptage des forks
  (premiere-installation.md § Mesurer l'usage) ne compte rien avant.
