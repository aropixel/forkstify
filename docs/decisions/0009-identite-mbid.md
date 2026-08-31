# 0009 — L'identité d'un artiste est son MBID ; Spotify est une implémentation

**Date** : 2026-08-30 · **Statut** : acceptée

## Contexte

Le catalogue est une base artiste **libre** qui doit survivre aux services de
streaming (« reprendre la main sur l'algorithme »). MusicBrainz est la base
musicale libre de référence : identifiants stables (MBID), données CC0/ODbL,
et chaque fiche relie déjà les identifiants Spotify, Deezer, Apple Music,
Tidal, Qobuz (vérifié sur The Cure le 30/08/2026).

## Décision

- **L'identité d'un artiste dans le catalogue est son MBID.**
- **Spotify est une « implémentation » parmi d'autres** — le premier tuyau
  branché, pas le socle. Deezer ou d'autres pourront venir dans un second
  temps, sans toucher au catalogue.

## Conséquences

- La fiche porte le MBID comme clé et les identifiants de services comme
  champs d'implémentation (`spotify = "…"`, un jour `deezer = "…"`).
- Le code sépare le moteur (qui ne connaît que le catalogue) des
  implémentations de lecture et de bibliothèque (qui connaissent un service).
- Les faits d'une fiche générée s'ancrent sur MusicBrainz / Wikidata, qui
  fournissent aussi les identifiants de services.
