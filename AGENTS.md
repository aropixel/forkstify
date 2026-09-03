# forkstify

Lecteur de musique pour Linux, câblé à Spotify, qui fonctionne **par
branches** : on part d'un morceau, l'application en enchaîne quelques-uns,
puis propose plusieurs directions ; on en choisit une — ou on la laisse
choisir — et ainsi de suite. Le pitch, les principes et le vocabulaire sont
dans [`docs/vision.md`](docs/vision.md).

**Nom provisoire.** Projet personnel de Joel Gomez Caballe, en **phase de
conception** : rien n'est encore codé. Sa philosophie tient en une ligne —
**reprendre la main sur l'algorithme** — et sa règle en une phrase : toute
décision automatique doit être explicable en une phrase et modifiable en un
commit.

Ce fichier est le contrat de l'agent sur ce dépôt. Il fait autorité sur ce
dépôt ; `~/Work/chorizo/AGENTS.md` fait autorité sur la machine.

## Où sont les choses

| Chemin             | Contenu                                                                 |
|--------------------|-------------------------------------------------------------------------|
| `AGENTS.md`        | Ce fichier — le contrat de l'agent.                                      |
| `docs/vision.md`   | Ce qu'est le produit : pitch, principes, vocabulaire.                    |
| `docs/decisions/`  | Une décision par fichier, numérotée et datée. **On ne modifie jamais une décision** : pour revenir dessus, on en écrit une nouvelle qui remplace l'ancienne. |
| `docs/conception/` | Notes de travail par sujet, vivantes, réécrites au fil des échanges. Chacune distingue **décidé**, **orientation** (proposé, non contredit) et **à trancher**. |
| `docs/avancement.md` | L'état courant : fait, en attente, prochaines étapes. Point d'entrée d'une session. |

Le reste vient avec les décisions.

## Ce qui est décidé

Le détail est dans `docs/decisions/`. En résumé :

- **Cible** : Linux, usage personnel d'abord, mais le catalogue a vocation à
  être partagé par l'écosystème Linux.
- **Source musicale** : Spotify. Pas de fichiers locaux ni d'autre service.
- **Le catalogue est le cœur du produit**, le moteur de branches le fait
  vivre. On fournit une base (fiches + vecteurs), on se l'approprie, on
  l'améliore, et elle **apprend de l'usage** — modèle base / mien / appris
  dans [`docs/conception/catalogue.md`](docs/conception/catalogue.md).
- **Zone de confort** = familiarité, de 0 à 5, réglée à l'ouverture ; elle
  choisit seule quand l'utilisateur ne choisit pas ([0001](docs/decisions/0001-confort-familiarite.md)).
- **Catalogue** = fichiers texte versionnés, un par artiste, partagé et
  forkable ([0002](docs/decisions/0002-catalogue-partage-forkable.md)).
- **Titres** : tops par défaut ([0003](docs/decisions/0003-titres-tops-et-portes.md)) ;
  portes retirées, liens typés en anglais avec proximité, champs anglais,
  réglages dans `catalogue.toml` ([0010](docs/decisions/0010-format-revise-links-sans-portes.md)).
- **Deux dépôts**, application et catalogue ; importer = cloner, un seul
  catalogue actif, on bascule quand on veut ([0004](docs/decisions/0004-deux-depots-catalogue-ciblable.md)).
- **Les fiches portent la version de leur format** ([0005](docs/decisions/0005-version-dans-les-fiches.md)).
- **L'identité d'un artiste est son MBID** ; Spotify est une implémentation
  parmi d'autres ([0009](docs/decisions/0009-identite-mbid.md)).
- **Concept et PoC avant l'interface** : pas de maquette pour l'instant.
- **Rust** ([0006](docs/decisions/0006-rust.md)).
- **Fiches en TOML**, `format = 1` en tête ([0007](docs/decisions/0007-fiches-en-toml.md)).
- **Pas de surcouche à part : le fork est la surcouche.** Le catalogue actif
  est un clone git, on y commite ses modifications ([0008](docs/decisions/0008-le-fork-est-la-surcouche.md)).
- **La répétition ne doit jamais être subie** : tirage pondéré, cooldown,
  sans remise, confort = profondeur ([0012](docs/decisions/0012-rotation-des-morceaux.md)).
- **Affinage au clavier : chaque touche est une mesure ou une édition**,
  `u` annule, tout est commande `:` ([0013](docs/decisions/0013-affinage-clavier-mesure-ou-edition.md)).
- **Langue** : français partout (doc, commits, interface) — **sauf le
  code : identifiants et commentaires en anglais** (visée open source,
  demandé par Joel le 03/09/2026).

## Ce qui reste à trancher

Plus de grande décision en suspens : le concept, le langage, le format et le
modèle de catalogue sont fixés. Les questions fines sont listées à la fin de
chaque note de `docs/conception/` et se tranchent au fil du PoC.

**L'état courant et les prochaines étapes sont dans
[`docs/avancement.md`](docs/avancement.md)** — c'est le point d'entrée
d'une nouvelle session, à tenir à jour à chaque avancée.

## Contraintes Spotify

Vérifiées le 30/08/2026 dans la documentation officielle et les README de
librespot et spotify-player, sauf mention contraire. Détail et sources dans
[`docs/conception/spotify.md`](docs/conception/spotify.md).

- **Spotify Premium** requis, pour l'API Web comme pour librespot.
- **API Web** : toute application a besoin d'un *client id* enregistré sur le
  dashboard développeur. Une application en **mode développement** est
  limitée à **5 utilisateurs** déclarés à la main. Le **mode étendu** n'est
  accordé depuis mai 2025 qu'à des **organisations** établies avec au moins
  **250 000 utilisateurs actifs mensuels**. Un projet libre ne peut donc pas
  obtenir de client id partageable. L'écosystème libre (spotify-player,
  ncspot) réutilise un client id historique déjà en mode étendu.
- **Recommandations, artistes similaires, audio features** : fermés aux
  nouvelles applications depuis fin 2024. Le projet n'en dépend pas par
  conception ([0002](docs/decisions/0002-catalogue-partage-forkable.md)).
- **librespot** (Rust) fait de l'application un appareil Spotify Connect,
  découvert depuis l'appli du téléphone sans saisie de mot de passe. Il
  n'utilise pas le dashboard développeur. Il n'est pas officiellement
  supporté par Spotify.

## Comment l'agent travaille ici

Les règles de `~/Work/chorizo/AGENTS.md` s'appliquent (français, tout
dockerisé, pas de `sudo`, demander avant tout `push`, toute suppression, toute
action root). En plus, sur ce dépôt :

- **Concevoir avant de coder.** Tant qu'une décision dont dépend un morceau
  de code n'est pas prise, l'agent ne l'écrit pas. Il propose, compare,
  recommande, et attend l'arbitrage de Joel.
- **Consigner au bon endroit.** Une décision prise → un fichier dans
  `docs/decisions/` et une ligne ici. Une orientation ou une question → la
  note de `docs/conception/` concernée. Un mot nouveau → `docs/vision.md`.
  Ce fichier reste court.
- **Le catalogue d'abord.** Face à un choix, privilégier ce qui rend le
  catalogue plus juste et les branches plus lisibles.
- **Vérifier avant d'affirmer**, en particulier tout ce qui concerne l'API
  Spotify : lire la doc officielle du moment, dire ce qui n'a pas pu être
  vérifié.
- **Aucun secret dans le dépôt.** Identifiants Spotify, jetons, clés d'API
  tierces vont dans un fichier ignoré par git.
- **Sobriété.** Pas de fichier « au cas où », pas de dépendance sans besoin
  établi, pas d'abstraction avant le deuxième usage.
- **Commits et push au fil de l'eau** sur les deux dépôts forkstify
  (demandé par Joel, 31/08/2026), messages en français. Le dépôt chorizo
  garde sa règle : demander avant de pousser.
- **Tenir `docs/avancement.md` à jour** à chaque avancée notable — c'est ce
  qui permet de reprendre dans une nouvelle session.
