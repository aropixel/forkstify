# 0020 — La cible d'un geste : la ligne surlignée, sinon ce qui sonne

- **Date** : 2026-09-09
- **Statut** : accepté

## Contexte

Depuis que choisir une branche **s'ajoute** à la file (06/09/2026), le
« courant » que le moteur tire du parcours est le dernier artiste de la
dernière branche empilée — le bout de la chaîne, plus ce qui sonne. Joel,
en écoute, a pris plusieurs branches puis tapé `en3` : les morceaux
ajoutés étaient ceux de l'artiste au bout de la chaîne, pas de celui qu'il
écoutait et survolait.

Par ailleurs deux règles cohabitaient sur l'axe : `ad`, `ti` et `tx`
suivaient la ligne surlignée, le reste de `t` et `a` suivait le morceau en
cours (question ouverte depuis le 07/09/2026).

## Décision

Joel, le 09/09/2026 : « par défaut le morceau en cours de lecture, mais si
une ligne est surlignée suite à un déplacement, elle est prioritaire. »

- **Un geste vise la ligne surlignée s'il y en a une, sinon le morceau qui
  sonne.** Une seule règle, pour les trois namespaces qui parlent d'un
  morceau ou de son artiste : `t`, `a` et `e`.
- La sélection ne joue rien et se voit : c'est pourquoi elle commande quand
  elle existe. Échap l'efface et rend la cible au morceau en cours.
- **L'encore se pose près de sa cible** : `e<n>` en fin de file ; `en<n>`
  et `e!<n>` **derrière la ligne surlignée** si elle est à venir, sinon
  juste après le morceau en cours (Joel, même jour). `e!` retire ce qui
  suit cet endroit.
- `ts` et `tb` ne font avancer la musique que s'ils visent ce qui sonne.
  Sur une ligne à venir, `ts` la sort de la file (« passer » un morceau
  prévu, c'est ne pas le jouer) ; sur une ligne passée, il note seulement.

## Conséquences

- `under_needle` et `target` disent la même chose ; `e` s'y raccorde.
- La table des touches ([`keybindings.md`](../keybindings.md)) porte la
  règle en tête du namespace `t`.
