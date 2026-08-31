# 0007 — Les fiches sont écrites en TOML

**Date** : 2026-08-30 · **Statut** : acceptée

## Contexte

Le format des fiches est une interface publique
([0002](0002-catalogue-partage-forkable.md)) : il doit être lisible et
modifiable à la main par des gens qui ne sont pas les auteurs de
l'application. Candidats : YAML, TOML, Markdown avec en-tête.

En Rust ([0006](0006-rust.md)), TOML est de première classe (`serde` +
`toml`) ; `serde_yaml` n'est plus maintenu par son auteur. Contre TOML : les
listes d'objets (portes, connexions) s'écrivent en `[[table]]`, plus verbeux
qu'en YAML.

## Décision

**TOML.** Une fiche = un fichier `.toml`, avec `format = 1` en tête.

## Conséquences

- Pas d'ambiguïté d'indentation ni de typage implicite (le `no` qui devient
  `false` en YAML) — un bon point pour des fiches écrites à la main par des
  inconnus.
- Portes et connexions s'écrivent en `[[portes]]` / `[[connexions]]`. Voir
  l'esquisse dans [conception/catalogue.md](../conception/catalogue.md).
- Parsing avec `serde` + `toml`, sans dépendance exotique.
