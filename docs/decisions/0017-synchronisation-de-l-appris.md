# 0017 — L'appris se synchronise tout seul : commit, push, pull, fusion par compteur

- **Date** : 2026-09-07
- **Statut** : accepté

## Contexte

Joel a écouté toute une journée sur un poste, `learned/` s'est enrichi, puis
il a changé de poste : rien n'avait été commité, l'appris était resté
derrière. Sur l'autre poste, dix fichiers de `learned/artists/` attendaient
de même, jamais commités. Deux machines, deux appris divergents, aucun
poussé.

C'était voulu à moitié : [0014](0014-forme-de-l-appris.md) fait de l'appris
une couche **silencieuse**, écrite sans confirmation ; seules les éditions de
fiches commitent ([0013](0013-affinage-clavier-mesure-ou-edition.md)). Le
retour n° 10 avait réservé `:sync`, `:push`, `:pull` sans trancher la
fréquence ni, surtout, la **fusion** : deux machines qui écrivent
`odezenne.toml` finissent en conflit textuel, et un `git merge` n'a aucun
sens sur des compteurs décrus. La machine n'a par ailleurs **aucune
sauvegarde** : un appris non poussé est un appris en risque.

## Décision

1. **L'application commite et pousse l'appris elle-même.** Pull au
   démarrage (après avoir commité ce qui a été appris ici, pour que le
   rebase ait quelque chose à fusionner), commit d'autosauvegarde **toutes
   les dix minutes** d'écoute si `learned/` a bougé, commit et push **à la
   sortie**, et `:sync` à la demande. Le push se fait en tâche de fond ;
   hors ligne, on continue et on le dit, on poussera la prochaine fois.
   Les appels réseau ont un délai court (`ConnectTimeout=5`) pour ne jamais
   suspendre l'écran.
2. **La fusion se fait par compteur, pas par ligne.** Un pilote de fusion
   git (`merge=learned` sur `learned/artists/*.toml`, commandé par
   `forkstify merge-learned <base> <ours> <theirs>`) fusionne à trois voies :
   les écoutes de chaque côté depuis l'ancêtre commun **s'additionnent**,
   décrues au jour de la fusion ; `last` prend la plus récente ; un ban ou un
   aimé posé d'un côté **tient** ; le poids suit le côté qui l'a bougé ; un
   top nouveau d'un côté entre tel quel. Le pilote est déclaré par
   l'application dans la config du clone (jamais versionnée) et l'attribut
   dans `.gitattributes` du dépôt de référence, avec `.git/info/attributes`
   en repli pour un clone qui ne l'aurait pas.
3. **Les fichiers de l'appris sont écrits triés.** L'ordre d'une table de
   hachage produisait de faux diffs à chaque écriture ; les tops sont
   désormais en `BTreeMap`.
4. **Les messages de commit produits par l'application sont en anglais**,
   comme le format de fiche et le vocabulaire sur disque : ce sont une
   interface publique du dépôt (Joel, 07/09/2026). Les commits humains du
   projet restent en français. Sujet : `learned: 3 artists`, `import: 12
   cards from <remote>` ; l'édition garde pour sujet la phrase affichée.
5. **Chaque commit de l'application porte un trailer** `Forkstify: <kind>
   <version>` (`learned`, `edit`, `import`). C'est ce qui permettra de
   compter l'usage à travers les forks publics : la recherche de commits de
   GitHub indexe les messages des dépôts publics —
   `gh api search/commits -f q='"Forkstify:"' --jq .total_count` — et le
   nombre de forks du dépôt de référence compte les utilisateurs.

## Conséquences

- 0014 n'est pas révisée : l'appris reste silencieux à l'écriture et
  n'entre jamais dans une PR ; il est simplement **poussé** sur le fork de
  l'utilisateur, ce qui le rend portable d'une machine à l'autre.
- `:sync` et `:push` sont câblés (même geste) ; `:pull` n'a pas de raison
  d'être à part, le pull se fait au démarrage.
- Un push refusé (l'autre machine a poussé entre-temps) attend le pull
  suivant, qui rebase et fusionne. Rien n'est jamais perdu, rien n'est
  jamais écrasé.
- La toute première réconciliation entre les deux postes de Joel se fait
  en lançant forkstify sur chacun : le premier commite et pousse, le second
  commite, rebase et fusionne par le pilote.
- Limites connues : la recherche de commits GitHub ne voit que les branches
  par défaut des dépôts **publics** ; un fork privé n'est pas compté. Le
  trailer ne dit rien de plus que « forkstify a écrit ceci », pas qui.
