//! The long tail: the fourth source of the reservoir (0012 §1).
//!
//! Everything the engine draws otherwise comes from the card — tops, doors —
//! or from `learned/`. The tail is the rest of an artist's known
//! discography, and it is **not** catalogue material: 0012 calls it « hors
//! catalogue », and `catalogue.md` files API caches outside the repo. So it
//! lives in the user's cache, is regenerable, is never committed, and never
//! syncs between machines.
//!
//! It is harvested from **Spotify**, not Deezer, because the cards already
//! carry a verified `spotify` id (212 of 214) and none carries a Deezer one:
//! going through Deezer would mean a search per artist, with the homonym
//! risk the catalogue has already paid once. Spotify also hands back
//! `spotify:track:` uris directly, so a tail track needs no title → id
//! resolution and can never turn into a « introuvable sur Spotify ».

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TailTrack {
    pub title: String,
    pub uri: String,
    pub album: String,
    /// The album's release date as Spotify gives it — « 1998 » or
    /// « 1998-09-22 ». Without it there is no chronological order, which is
    /// how one remembers an artist (maquette 1a, 07/09/2026).
    #[serde(default)]
    pub released: String,
    /// Rank inside its album. Ordering a folded album by anything else
    /// would make it unrecognisable.
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub duration_ms: u32,
    /// A single or an EP rather than an album: shown after the albums, so a
    /// discography does not turn into a list of one-track records.
    #[serde(default)]
    pub single: bool,
}

impl TailTrack {
    /// The year, when the date says one.
    pub fn year(&self) -> Option<u16> {
        self.released.get(..4).and_then(|y| y.parse().ok())
    }
}

/// Every discography harvested so far, keyed by slug. Loaded once at
/// startup: the engine reads it synchronously, the application fills it.
pub struct Tail {
    root: PathBuf,
    artists: HashMap<String, Vec<TailTrack>>,
}

impl Tail {
    pub fn load() -> Tail {
        let root = cache_dir();
        let mut artists = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(tracks) = serde_json::from_str::<Vec<TailTrack>>(&text) {
                        artists.insert(slug.to_string(), tracks);
                    }
                }
            }
        }
        Tail { root, artists }
    }

    /// An empty tail, for tests and for the dry journey on a cold machine.
    #[cfg(test)]
    pub fn blank() -> Tail {
        Tail { root: std::env::temp_dir().join("forkstify-tests-tail"), artists: HashMap::new() }
    }

    pub fn known(&self) -> usize {
        self.artists.len()
    }

    pub fn has(&self, slug: &str) -> bool {
        self.artists.contains_key(slug)
    }

    /// A harvest made before the dates were kept: it can still feed the
    /// reservoir, but not the discography screen, which needs the year and
    /// the track number. The cache is regenerable and outside the repo, so
    /// the answer to an old one is simply to harvest again.
    pub fn dated(&self, slug: &str) -> bool {
        self.of(slug).iter().any(|track| !track.released.is_empty())
    }

    /// Forget one artist's harvest, so the next one goes back to Spotify.
    pub fn forget(&mut self, slug: &str) {
        self.artists.remove(slug);
        let _ = std::fs::remove_file(self.root.join(format!("{slug}.json")));
    }

    pub fn of(&self, slug: &str) -> &[TailTrack] {
        self.artists.get(slug).map_or(&[], |v| v.as_slice())
    }

    /// Keep a harvest, on disk and in memory.
    pub fn keep(&mut self, slug: &str, tracks: Vec<TailTrack>) {
        if let Err(e) = std::fs::create_dir_all(&self.root) {
            eprintln!("tail not saved ({e})");
        } else if let Ok(text) = serde_json::to_string(&tracks) {
            let _ = std::fs::write(self.root.join(format!("{slug}.json")), text);
        }
        self.artists.insert(slug.to_string(), tracks);
    }
}

fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache")
        });
    base.join("forkstify").join("discography")
}

/// Two titles are the same track when they differ only by a remaster tag, a
/// live suffix or punctuation — Spotify ships the same song a dozen times.
pub fn normalize(title: &str) -> String {
    let lower = title.to_lowercase();
    // everything from an opening bracket on is version noise more often
    // than not: "(Remastered 2010)", "- 2004 Remaster", "(Live)"
    let cut = lower
        .find(" - ")
        .or_else(|| lower.find(" ("))
        .map_or(lower.as_str(), |i| &lower[..i]);
    cut.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Le titre **lisible** d'un morceau : celui qu'on écrit dans une fiche
/// quand on le promeut en top. `normalize` sert à comparer et rend un
/// mot-clé ; celui-ci coupe le même bruit de version mais garde la casse,
/// les espaces et la ponctuation du titre — une fiche est lue par un
/// humain ([0002] : le format est une interface publique).
pub fn clean_title(title: &str) -> String {
    let cut = title
        .find(" - ")
        .or_else(|| title.find(" ("))
        .map_or(title, |i| &title[..i]);
    cut.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_titre_ecrit_dans_la_fiche_reste_lisible() {
        assert_eq!(clean_title("Metal Heart - 2015 Remaster"), "Metal Heart");
        assert_eq!(clean_title("Colors and the Kids (Live)"), "Colors and the Kids");
        assert_eq!(clean_title("(I Can't Get No) Satisfaction"), "(I Can't Get No) Satisfaction");
        assert_eq!(clean_title("Cross Bones Style"), "Cross Bones Style");
    }

    #[test]
    fn les_variantes_d_un_meme_titre_se_confondent() {
        let a = normalize("A Forest");
        assert_eq!(a, normalize("A Forest - 2005 Remaster"));
        assert_eq!(a, normalize("A Forest (Remastered)"));
        assert_eq!(a, normalize("a  forest!"));
        // mais deux titres différents restent différents
        assert_ne!(a, normalize("A Forest Fire"));
        assert_ne!(normalize("Lullaby"), normalize("Lovesong"));
    }
}
