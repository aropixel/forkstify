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

#[derive(Serialize, Deserialize, Clone)]
pub struct TailTrack {
    pub title: String,
    pub uri: String,
    pub album: String,
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

    pub fn of(&self, slug: &str) -> &[TailTrack] {
        self.artists.get(slug).map_or(&[], |v| v.as_slice())
    }

    /// Keep a harvest, on disk and in memory.
    pub fn keep(&mut self, slug: &str, tracks: Vec<TailTrack>) {
        if let Err(e) = std::fs::create_dir_all(&self.root) {
            eprintln!("traîne non enregistrée ({e})");
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

#[cfg(test)]
mod tests {
    use super::*;

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
