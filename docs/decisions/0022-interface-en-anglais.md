# 0022 — L'interface est en anglais

- **Date** : 2026-09-10
- **Statut** : accepté

## Contexte

Joel veut publier une première version de forkstify sous peu, et qu'elle
soit **accessible et utilisable par le plus grand nombre**. Le code, le
format de fiche, les chemins et les commits de l'application étaient déjà
en anglais ([0017](0017-synchronisation-de-l-appris.md), AGENTS.md) ; seule
l'interface — écrans, toasts, aides, sorties de la ligne de commande,
widget Omarchy — restait en français.

## Décision

1. **Tout ce que l'utilisateur voit est en anglais** : la TUI (accueil,
   session, discographie, recherche, tables des touches), les messages et
   erreurs, les sorties de la ligne de commande, le fichier de
   configuration écrit au premier lancement, le widget de barre et son
   script d'installation. Les libellés courts du moteur (sources `liked` ·
   `tail` · `non-top` · `off-catalog`, raisons `shared members` · `family
   ties` · `same scene` · `close to the branch's center`) en font partie.
2. **Les sous-commandes suivent** : `parcours` devient `journey`, `ecouter`
   devient `listen` ; `check`, `import`, `vectors`, `merge-learned` ne
   changent pas.
3. **La section `[catalogue]` du fichier de configuration devient
   `[catalog]`** ; l'ancien nom reste lu (alias serde), rien à changer sur
   les postes existants.
4. **Les notes que l'application écrit dans les fiches** (`set while
   listening, <date>`, `linked while listening, <date>`) sont en anglais,
   comme ses commits : c'est l'application qui parle, pas Joel.
5. **Le texte vectorisé ne bouge pas** (`src/embed.rs` : « Genres : … Pays :
   … ») : il n'est pas une interface et le changer invaliderait l'index
   ([0019](0019-vectorisation-par-l-application.md)). Les spikes de
   `src/bin/` non plus : ce sont des outils de développement.

## Conséquences

- La doc, les décisions, les commits humains et les notes de conception
  restent en français ; la ligne « Langue » d'AGENTS.md est mise à jour.
- Un seul langage à l'écran : le vocabulaire anglais de l'interface (card,
  catalog, learned, seed, journey, branch, fork, tail, comfort zone, cocoon
  → exploration) devient la référence pour tout nouveau texte visible.
