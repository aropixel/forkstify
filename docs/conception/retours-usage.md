# Retours d'usage et chantier clavier

Note vivante, ouverte le **05/09/2026** après les premières sessions
longues d'`ecouter` (Joel, plusieurs sessions le matin du 05/09). Elle
tient la liste des retours, l'inventaire de ce qui existe vraiment, et ce
qui reste à trancher avant d'ajouter quoi que ce soit.

Elle se traite **au fur et à mesure** : chaque entrée porte son statut, et
ce qui est fait descend dans `docs/avancement.md`.

**Point au soir du 05/09/2026 — 5 faits, 2 bloqués par `learned/`, 4 non
commencés, et rien de vérifié en écoute réelle.** Les statuts ci-dessous
sont relus contre le code, pas contre le souvenir.

## L'inventaire d'abord (retour n° 4)

Joel : « je commence à me mélanger, je ne veux pas rajouter et que ça
devienne inutilisable ». Voici donc l'état réel, relevé dans
`src/listen.rs` (`on_input`, `on_control`) le 05/09/2026 — à distinguer de
la table *projetée* de `forme-de-l-application.md`, qui reste une
orientation largement non implémentée.

### Ce qui marche aujourd'hui dans `ecouter`

Onze gestes câblés, listés avec tout le reste dans
**[`docs/keybindings.md`](../keybindings.md)** — la référence unique
depuis le 05/09/2026.

### Ce qui manque et qu'on croit parfois avoir

- **Aucune touche pause/lecture au clavier.** ⏯ ne passe que par les
  touches multimédia (MPRIS). Dans le terminal, rien.
- **Aucune des touches d'affinage de 0013** : `t`/`T` (tops), `x`/`X`
  (sauter/écarter), `a` (aimer), `d` (door), `m` (marquer), `E` (éditer la
  fiche), `-` (moins souvent), `z` (confort), `?` (pourquoi), `y`/`n`. La
  table est écrite, rien n'est câblé. C'est exactement le retour n° 7.
- **Aucune commande `:`**, alors que 0013 en fait le socle (« chaque touche
  n'est que le raccourci d'une commande `:` »).
- **Aucune écriture dans `learned/`** : la boucle d'apprentissage (0014)
  n'est pas fermée, donc aucune mesure n'est encore enregistrable.

### Deux conflits à trancher avant d'ajouter

**1. `u` a deux sens.** La décision **0013** (acceptée) dit : « `u` annule
la dernière action, quelle qu'elle soit ». L'implémentation actuelle en a
fait « revenir à la branche précédente ». Tant qu'il n'y avait ni mesure ni
édition, l'ambiguïté ne coûtait rien ; dès que `da`/`dt` et les tops
arrivent (retours n° 8 et 9), il faut un vrai undo. **À trancher** : `u` =
undo (0013) et la navigation arrière passe sur autre chose (`h` ? déjà
évoqué dans la table projetée), ou 0013 est révisée par une nouvelle
décision.

**2. Les touches proposées recouvrent des touches déjà réservées.** Le
`d` de 0013 est **door** (une édition qui produit un commit) ; le `da`/`dt`
du retour n° 9 en ferait un préfixe *dislike*. Et surtout : `X` (« plus
jamais celui-là ») **est déjà** le dislike d'un morceau, `-` (« cet
artiste, moins souvent ») est déjà le tiède sur l'artiste. **À trancher** :
`da`/`dt` sont-ils de nouveaux gestes, ou les noms `:` des gestes `X` et
`-` déjà prévus ? Le risque, sinon, est d'avoir deux touches pour la même
chose — précisément ce que le retour n° 4 veut éviter.

## Les retours, un par un

### 1. Valider sans « Entrée », à la neovim

**Statut** : **fait** (05/09/2026, `src/keys.rs`) — non vérifié en écoute
réelle.

Aujourd'hui l'entrée est ligne par ligne (`std::io::stdin().lines()`,
choix assumé du 04/09 : « le temps réel appartient à l'étape TUI »).

**Point dur repéré** : ce retour et les retours n° 2, 3, 5 et 9 se
contredisent en apparence — on ne peut pas taper `2en!`, `p1n!`, `da` ou
`pr` avec une lecture « une touche = une action ». **Neovim résout
exactement ça** et c'est la référence citée : lecture en mode brut avec un
**tampon d'attente**, les chiffres sont des *counts*, les lettres des
opérateurs, `!` un modificateur, et la séquence s'exécute dès qu'elle est
non ambiguë. `/` et `:` basculent en mode ligne (avec Entrée), ce dont la
recherche a de toute façon besoin.

C'est ce qui a été fait : termios via `libc`, garde RAII qui rend le
terminal même sur panique, flèches ← → reconnues, `/` et `:` qui ouvrent
une ligne éditable. La grammaire est **sans préfixe**, donc tout se
déclenche sans délai ni `timeoutlen` — propriété vérifiée par un test
exhaustif sur toutes les séquences de trois touches.

### 2. Encore, en trois nuances

**Statut** : **fait** (05/09/2026) — non vérifié en écoute réelle.

Demandé : ajouter n morceaux en fin de branche · après le morceau en
cours · après le morceau en cours en retirant ce qui était prévu.

Câblé sous la forme `e<n>` / `en<n>` / `e!<n>` : le modificateur précède le
compte, contrainte de la grammaire sans préfixe (voir 0015). L'ancien
comportement était déjà la variante « now » ; les deux autres sont
nouvelles.

### 3. Choisir une branche : même grammaire

**Statut** : **fait** (05/09/2026), **bug compris** — non vérifié en
écoute réelle.

Joel : « quand on choisit une branche à l'avance, elle se joue après le
morceau en cours, pas après la branche en cours ».

Vérifié : `choose()` met la branche en attente pour la fin du **morceau**,
et `start_segment()` fait `self.queue = stops.into()` — le reste du segment
est **jeté**. Le comportement actuel est donc la variante la plus
destructrice des trois, et c'est le défaut.

Câblé sous la forme `f<n>` / `fn<n>` / `f!<n>` (le préfixe est `f`, pas
`p` : voir 0015). Le défaut est désormais « après la branche en cours » —
**le segment n'est plus jeté**, ce qui était le bug.

**Observation qui sert le retour n° 4** : les retours 2 et 3 décrivent
**la même grammaire** — un geste, puis les modificateurs `n` (« maintenant »)
et `n!` (« maintenant, et tant pis pour la suite »). Une seule règle à
apprendre pour deux commandes, et elle se généralisera aux suivantes. C'est
la piste à tenir pour que la surface reste petite.

**À trancher** : si `p1` existe, faut-il garder `1` tout court ? Deux
façons de faire la même chose, c'est ce qu'on veut éviter. Proposition :
`1` reste le geste rapide (= `p1`), `p` devient le préfixe qui accepte les
modificateurs.

### 4. Faire le point sur les raccourcis

**Statut** : **fait**, et allé plus loin que demandé. L'inventaire a
produit [`docs/keybindings.md`](../keybindings.md), la table unique, puis
la refonte complète de la grammaire (décision
[0015](../decisions/0015-grammaire-clavier-namespaces.md)) : quatre
namespaces, huit collisions résolues, chaque touche adossée à un mot
anglais.

### 5. Reproposer des branches

**Statut** : **fait** (05/09/2026) — `fr`, dans le namespace des
branches ; `pr` est abandonné, il entrait en collision avec `p<n>`.

Une touche qui retire trois nouvelles branches quand aucune ne convient.
Joel propose `pr` ou `r` (refresh/reload).

**Remarque** : c'est exactement ce que faisait `auto_advance()` par accident
avant la correction du 05/09 — le tirage existe déjà (`recompute()`), il
suffit de l'exposer.

**À trancher** : `pr` entre en collision avec le schéma `p<n>` du retour
n° 3 (`p1`, `p1n`). `r` seul est plus net et laisse `p` cohérent.

### 6. « Partir sur complètement autre chose »

**Statut** : **toujours à clarifier** — la touche `fw` est réservée et
répond « pas encore câblé », mais la question de fond n'est pas tranchée.

Deux lectures possibles, et elles ne mènent pas au même travail :

- **(a)** une branche/touche qui **sort de l'univers courant** — ignorer le
  plancher de la branche aventureuse (cosinus ≥ 0.72 + un tag commun) pour
  sauter loin volontairement ;
- **(b)** repartir d'une **nouvelle graine** en cours de session, sans
  quitter l'application (ce que `/texte` fait déjà à moitié).

Ma lecture penche pour **(a)**, vu la place de la note dans une liste de
gestes d'écoute — mais je ne tranche pas à ta place.

### 7. Câbler les raccourcis manquants (tops, édition de fiche…)

**Statut** : **bloqué par `learned/`**, comme prévu. La grammaire est
décidée et les touches réservées (`tt`/`tT` tops, `ae` éditer la fiche,
`td` door…), elles répondent « décidé, pas encore câblé ». Douze gestes
attendent la boucle d'apprentissage.

La table est déjà décidée (0013) et détaillée dans
`forme-de-l-application.md`. Mais **les mesures n'ont nulle part où
s'écrire** tant que la boucle d'apprentissage (0014, `learned/`) n'est pas
fermée — c'est l'étape 2 de `docs/avancement.md`. Les **éditions** (tops,
`E`) touchent les fiches et peuvent, elles, se faire tout de suite.

**Ordre proposé** : d'abord les éditions (`t`/`T`, `E`) qui ne dépendent que
du catalogue, ensuite les mesures quand `learned/` est branché.

### 8. Lier l'artiste en cours à un autre

**Statut** : touche décidée (`aL`, *artist link*), **pas câblée**, et le
mode de désignation de la cible reste à spécifier.

Une touche qui crée un **link typé** depuis l'artiste du morceau en cours
vers un autre artiste — donc une **édition** au sens de 0013 (commit dans
le catalogue, format des links de 0010).

**À spécifier** : comment on désigne la cible (recherche `/` réutilisée ?),
quel type de link par défaut, et si la proximité se saisit ou se déduit.

### 9. Dire qu'on n'aime pas (`da` / `dt`)

**Statut** : **conflit réglé**, câblage bloqué par `learned/`.

Le doublon que craignait le conflit n° 2 est levé : `tb` (ban track) et
`ab` (ban artist) absorbent à la fois le `dt`/`da` demandé et les `X` et
`-` déjà décidés. Sur l'artiste, les trois verbes forment une échelle —
`al` plus souvent, `as` moins souvent, `ab` plus jamais. Reste à câbler,
avec le reste du namespace.

### 10. Synchroniser par git (`gh`) entre machines

**Statut** : **non commencé.** Seules les commandes sont réservées
(`:sync`, `:push`, `:pull`). Axe différent des autres — c'est de
l'infrastructure, pas du clavier.

Idée de Joel : commits et push réguliers de l'usage et des fiches, `pull` au
démarrage pour retrouver son usage d'une machine à l'autre.

Cohérent avec 0008 (« le fork est la surcouche ») et 0002 (catalogue
versionné). Points à traiter : que faire des **conflits** sur `learned/`
(compteurs décrus, demi-vie 6 mois — une fusion additive a du sens, un
`git merge` textuel non), à quelle **fréquence** pousser sans transformer
l'écoute en machine à commits, et le comportement **hors ligne**.

`gh` est installé et authentifié sur cette machine (compte `kbyjoel`).

### 11. Mode file d'attente

**Statut** : **non commencé.** La touche `Q` (*queue*) est réservée et
répond « pas encore câblé ». **Le plus gros morceau de la liste.**

Un mode à part entière : préparer les branches à l'avance, retirer des
morceaux, retirer une branche entière (**seulement elle, ou toute la
profondeur qui en découle**), intercaler un morceau.

C'est une **deuxième vue** sur l'état du parcours, avec sa propre table de
touches — à ne pas confondre avec la vue d'écoute. La distinction
« retirer cette branche » / « retirer toute la profondeur » suppose que
l'arbre du parcours soit manipulable, alors que `rounds` est aujourd'hui
une **liste plate**. C'est le point structurant à regarder en premier.

Lié à la « vraie prévisualisation + choix à l'avance » déjà notée comme
appartenant à l'interface (04/09/2026).

## À trancher — récapitulatif

Les cinq points du matin ont tous été tranchés le 05/09 (voir
[0015](../decisions/0015-grammaire-clavier-namespaces.md) et
[`keybindings.md`](../keybindings.md)) : `u` annule un geste et `fu`
remonte d'une branche · `da`/`dt` sont absorbés par `tb`/`ab` · `p1` et
`1` sont devenus `f<n>` et le raccourci `1`…`9` · reproposer est `fr`.

Restent ouverts :

1. **Retour n° 6** : « partir sur complètement autre chose » — sortir de
   l'univers courant (a), ou repartir d'une nouvelle graine (b) ? La touche
   `fw` attend la réponse.
2. **`ts` (skip track) vs `l` (suivant)** : deux gestes pour passer un
   morceau, la différence — noter ou non dans `learned/` — étant invisible
   dans les doigts.
3. **`aL` ou `ac`** pour lier deux artistes.

## Ce qui n'a pas été vérifié

**Rien de ce qui a été câblé le 05/09 n'a tourné dans une session
d'écoute.** Le mode brut, le leader, les trois variantes, `fr`, `fu`, la
pause sur `p` et `:size` sont vérifiés à la compilation et par les tests de
grammaire — pas sous les doigts. C'est la première chose à faire à la
prochaine session, avant d'ajouter quoi que ce soit.

Deux points ne se jugeront qu'à ce moment-là : le **coût des deux frappes**
sur les gestes fréquents, et le **sens de `h`/`l`** — la navigation est
passée à l'horizontale alors que la file s'affiche verticalement.
