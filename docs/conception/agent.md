# L'agent — une IA qui pilote forkstify de l'extérieur

Note ouverte le **10/09/2026** sur une idée de Joel : une commande `:agent`
qui transmet un prompt à une IA connectée — le Claude Code de son poste, par
exemple — du genre « crée-moi une playlist de 20 titres dans l'ambiance
Kanye West, Drake, Kendrick Lamar ». L'IA se sert des outils de forkstify,
accède au catalogue et lance une session d'écoute, après un échange
éventuel pour éclaircir la demande.

Rien n'est codé. Cette note fixe l'orientation discutée le jour même et
liste ce qui reste à trancher ; l'arbitrage revient à Joel.

## Décidé — ce qu'on ne rouvre pas

Rien n'est acté dans `docs/decisions/`. Mais l'idée hérite de règles qui la
cadrent :

- **Toute décision automatique doit être explicable en une phrase et
  modifiable en un commit** ([vision.md](../vision.md)). Un morceau mis en
  file par l'agent est une décision automatique comme une autre.
- **Chaque touche n'est que le raccourci d'une commande `:`**, et les
  commandes « rendent tout découvrable et scriptable »
  ([0013](../decisions/0013-affinage-clavier-mesure-ou-edition.md)).
- **Base large *et* génération à la volée** : arriver chez un artiste sans
  fiche, c'est la lui créer
  ([0016](../decisions/0016-base-large-et-generation-a-la-volee.md)).
- **Aucun secret dans le dépôt**, et **pas de dépendance sans besoin
  établi** ([AGENTS.md](../../AGENTS.md)).
- **Choisir une branche l'ajoute à la file** au lieu de la remplacer
  ([retours-usage.md](retours-usage.md), 06/09/2026).

## Le point qui tranche — l'agent pilote, il ne choisit pas dans sa tête

Une IA qui aligne vingt titres de mémoire, c'est le DJ IA de Spotify : une
boîte noire de plus, l'inverse du pitch. La même IA qui **pilote
forkstify** — cherche, génère des fiches, demande des branches, lit leurs
raisons et met en file — reste dans la règle : chaque morceau porte une
raison lisible, et son passage **laisse une trace dans le catalogue**.

Sur l'exemple de Joel, ça donne :

1. L'agent cherche Kanye West, Drake et Kendrick Lamar dans le catalogue.
   Ils n'y sont pas. Il lance `:generate` pour chacun : fiche MusicBrainz +
   Deezer, similaires en cascade, vecteur, un commit chacun.
2. Il demande les branches proposées depuis ces fiches, lit les raisons,
   en retient, ajoute à la file. Il peut compléter avec des pistes Spotify
   « de sa tête », mais elles arrivent **étiquetées** : une branche nommée
   par lui, avec sa phrase de justification — comme un résultat `[spotify]`
   de la recherche aujourd'hui, qui se distingue d'un `[catalogue]`.
3. Résultat : une playlist de vingt titres, et **trois à dix fiches de plus
   dans le fork**. La playlist est le produit dérivé ; le catalogue a grandi.
   C'est 0016 avec un ouvrier de plus.

## Orientation — l'agent est dehors, pas dedans

**Forkstify n'embarque ni client LLM, ni clé d'API, ni choix de modèle.**
Trois raisons : les secrets (aucun dans le dépôt, aucun dans le binaire), la
sobriété (une dépendance HTTP et un SDK pour une fonctionnalité de confort),
et l'interchangeabilité — l'IA est un tuyau comme Spotify l'est, elle doit
pouvoir changer sans toucher au cœur.

Deux étages, dans cet ordre :

### Étage 1 — forkstify devient pilotable de l'extérieur, sans `:agent`

- **Un socket de contrôle** pour la TUI en cours (dans le répertoire
  `XDG_RUNTIME_DIR`), et une sous-commande du type `forkstify cmd
  ':generate Drake'` qui lui parle — ou qui fait tourner le moteur à sec
  s'il n'y a pas de TUI vivante.
- **Quelques lectures en JSON** : l'état (ce qui joue, la file, le confort,
  la taille), une fiche, les voisins (`check`), les branches proposées
  depuis un artiste ou depuis le bout de la file, la discographie.
- **L'API de l'agent, ce sont les commandes `:`.** Tout ce que l'agent fait
  est une commande que l'utilisateur aurait pu taper ; le journal se lit
  comme une session au clavier. C'est la promesse « scriptable » de 0013,
  tenue une fois pour toutes — et elle sert aussi sans IA (un script, un
  raccourci Hyprland, `playerctl`-like).
- **La conversation se tient dans le terminal de l'agent.** Joel tape sa
  demande dans Claude Code, qui appelle le binaire par Bash et sait déjà
  poser une question avant d'agir. Une skill ou une section d'`AGENTS.md`
  documente les commandes et le contrat (ci-dessous).
- **Pas de serveur MCP à ce stade.** Il n'a de sens que si un deuxième
  hôte que Claude Code apparaît — pas d'abstraction avant le deuxième
  usage.

### Étage 2 — `:agent <prompt>` dans la TUI

- La commande lance **en fond** l'agent configuré dans `config.toml`
  (`[agent] command = "claude -p …"`, par exemple), avec le prompt et le
  socket comme outil. L'agent est une commande externe : Claude Code
  aujourd'hui, autre chose demain, sans changer forkstify.
- **La musique ne s'arrête pas** pendant les trente secondes à deux minutes
  que ça prend (principe 1 de la vision). L'agent **ajoute au bout de la
  file**, il ne remplace pas — la file « qui s'enchaîne » du 06/09 est
  faite pour ça. Le pied d'écran signale « agent en cours ».
- Ses **questions** remontent dans une modale, sur le patron de la
  discographie (`keys::parse_modal`) ; sa réponse repart au même agent
  (session reprise). L'étage 2 ne se code que si l'étage 1 a prouvé que
  l'aller-retour par le terminal de Claude Code est trop lourd à l'usage.

### Le contrat de l'agent

- **Chaque branche qu'il met en file porte son nom et sa raison** en une
  phrase, affichés comme les raisons du moteur. Les morceaux tirés par le
  moteur gardent les raisons du moteur ; ceux « de sa tête » sont marqués
  comme tels.
- **`u` annule toute sa contribution d'un coup**, comme une seule édition
  (0013) — la file revient à ce qu'elle était avant sa demande.
- **Ses commits sur le catalogue se reconnaissent** : le trailer de
  [0017](../decisions/0017-synchronisation-de-l-appris.md) porte un `kind`
  qui le nomme (`Forkstify: agent <version>`, à préciser), pour qu'on sache
  toujours ce qui vient de lui — et pour le relire.
- **Un plafond de fiches générées par demande**, sinon « ambiance rap US »
  en génère quarante et fait tomber MusicBrainz et Deezer sur nos
  backoffs.
- **Il préfère le catalogue à sa mémoire** : générer et brancher avant de
  citer. Sa mémoire sert à *choisir* entre des branches et à nommer une
  ambiance, pas à remplacer le moteur.

## Ce que ça rouvre

- **Sauvegarder la playlist** (question 0 de [retours-usage.md](retours-usage.md))
  revient sur la table : une playlist demandée en une phrase voudra être
  gardée.
- **La relecture par un LLM**, que [moteur-de-branches.md](moteur-de-branches.md)
  pose pour nommer une connexion vectorielle sans lien écrit : c'est
  peut-être le vrai emploi de l'agent. Relire les fiches générées, écrire
  les descriptions absentes, proposer un lien avec sa note — des commits
  relus par un humain, exactement ce que le catalogue attend. La playlist
  est un prétexte agréable ; la relecture rend les branches plus justes.

## À trancher

1. **Le plafond** de fiches générées par demande (5 ? 10 ?), et s'il se
   règle par le confort ou par l'appel.
2. **La trace dans la file** : un nom de branche par demande (« agent :
   ambiance Kanye / Drake / Kendrick ») avec les raisons du moteur sous
   chaque morceau, ou une branche par direction trouvée ?
3. **Le sort des morceaux « de sa tête »** : autorisés et marqués (proposé),
   ou interdits — l'agent ne peut mettre en file que ce que le moteur ou la
   recherche lui rend.
4. **Le nom de la commande** : `:agent` (proposé) ou `:ask`, plus neutre sur
   qui répond.
5. **Le `kind` du trailer** pour les commits de l'agent, et si une fiche
   générée à sa demande se distingue d'une fiche générée à la main (le
   champ `generated = true` suffit peut-être).
6. **L'étage 2 est-il nécessaire** — à décider après avoir vécu l'étage 1
   depuis Claude Code.
