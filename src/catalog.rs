//! Reads the active catalog: artist cards (TOML), proximity grid, vectors.

use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// The kind of a link the listener drew themselves (`ac`, 2026-09-23).
/// No card ever carries it: it is woven into the catalog in memory from
/// `learned/`, so the engine walks it while `cards/` stays untouched.
pub const MINE: &str = "mine";

#[derive(Deserialize)]
pub struct Link {
    pub to: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub note: Option<String>,
    pub proximity: Option<u8>,
}

/// A door (0011): one track singled out as the way *out* of an artist
/// towards a direction. `to` points at tags, never at artists. It is an
/// additional criterion, never the main one.
#[derive(Deserialize)]
pub struct Door {
    pub track: String,
    #[serde(default)]
    pub to: Vec<String>,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct Card {
    pub name: String,
    /// The artist's Spotify id, when the card carries one (212 of 214) —
    /// the way in to the long tail.
    pub spotify: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tops: Vec<String>,
    #[serde(default)]
    pub doors: Vec<Door>,
    #[serde(default)]
    pub links: Vec<Link>,
    /// Dates and origin as MusicBrainz gives them, and the human prose —
    /// what the vector's text is composed from (embed.rs), nothing else
    /// reads them.
    pub begin: Option<String>,
    pub end: Option<String>,
    pub origin: Option<String>,
    pub description: Option<String>,
}

pub struct Catalog {
    pub cards: HashMap<String, Card>,
    pub proximities: HashMap<String, u8>,
    pub vectors: HashMap<String, Vec<f32>>,
}

// Built-in defaults, identical to the reference catalog's grid.
const DEFAULT_PROXIMITIES: [(&str, u8); 6] = [
    ("member", 5),
    ("collab", 4),
    ("similar", 4),
    ("family", 3),
    ("scene", 3),
    ("influence", 2),
];

#[derive(Deserialize)]
struct Settings {
    #[serde(default)]
    proximity: HashMap<String, u8>,
}

#[derive(Deserialize)]
struct VectorLine {
    slug: String,
    v: Vec<f32>,
}

impl Catalog {
    pub fn load(path: &Path) -> anyhow::Result<Catalog> {
        let mut cards = HashMap::new();
        let dir = path.join("cards");
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("no cards/ folder in {}", path.display()))?
        {
            let file = entry?.path();
            if file.extension().is_some_and(|e| e == "toml") {
                let slug = file.file_stem().unwrap().to_string_lossy().to_string();
                let card: Card = toml::from_str(&std::fs::read_to_string(&file)?)
                    .with_context(|| format!("card unreadable: {}", file.display()))?;
                cards.insert(slug, card);
            }
        }

        let mut proximities: HashMap<String, u8> = DEFAULT_PROXIMITIES
            .iter()
            .map(|(kind, value)| (kind.to_string(), *value))
            .collect();
        if let Ok(text) = std::fs::read_to_string(path.join("catalog.toml")) {
            let settings: Settings = toml::from_str(&text).context("catalog.toml unreadable")?;
            proximities.extend(settings.proximity);
        }

        let mut vectors = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(path.join("vectors/vectors.jsonl")) {
            for line in text.lines() {
                let parsed: VectorLine =
                    serde_json::from_str(line).context("vectors.jsonl unreadable")?;
                vectors.insert(parsed.slug, parsed.v);
            }
        } else {
            eprintln!("(no vectors/vectors.jsonl: navigating on the graph alone)");
        }

        Ok(Catalog { cards, proximities, vectors })
    }

    /// Slugs matching the query, **folded**: accents down to ASCII, case
    /// and punctuation gone (`generate::fold_text`), against the card's
    /// name *and* its slug — the name of its TOML file. So `rosalia`
    /// finds ROSALÍA and `the cure` finds `the-cure` (Joel, 2026-09-29).
    /// An exact match comes first, then the shorter names, then
    /// alphabetical.
    pub fn search_names(&self, query: &str, limit: usize) -> Vec<String> {
        let needle = crate::generate::fold_text(query);
        if needle.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<(&String, &String, bool)> = self
            .cards
            .iter()
            .filter_map(|(slug, card)| {
                let name = crate::generate::fold_text(&card.name);
                let file = crate::generate::fold_text(slug);
                let exact = name == needle || file == needle;
                (exact || name.contains(&needle) || file.contains(&needle))
                    .then_some((slug, &card.name, exact))
            })
            .collect();
        hits.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.len().cmp(&b.1.len())).then(a.0.cmp(b.0)));
        hits.into_iter().take(limit).map(|(slug, ..)| slug.clone()).collect()
    }

    /// A link's proximity, resolved in cascade: the link itself, the
    /// catalog's grid, the built-in defaults (decision 0010).
    pub fn proximity(&self, link: &Link) -> u8 {
        link.proximity
            .or_else(|| self.proximities.get(&link.kind).copied())
            .unwrap_or(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog(names: &[(&str, &str)]) -> Catalog {
        let cards = names
            .iter()
            .map(|(slug, name)| {
                let text = format!("format = 1\nname = \"{name}\"\nmbid = \"x\"\n");
                (slug.to_string(), toml::from_str::<Card>(&text).expect("card"))
            })
            .collect();
        Catalog { cards, proximities: HashMap::new(), vectors: HashMap::new() }
    }

    /// `fw rosalia` found nothing: the card is spelled ROSALÍA, and the
    /// search compared the accent (Joel, 2026-09-29). Folded, it matches —
    /// and so does the name of the TOML file.
    #[test]
    fn the_search_folds_accents_case_and_punctuation() {
        let catalog = catalog(&[
            ("rosalia", "ROSALÍA"),
            ("the-cure", "The Cure"),
            ("motorhead", "Motörhead"),
            ("cat-power", "Cat Power"),
        ]);
        let one = |query: &str| catalog.search_names(query, 1).into_iter().next();
        assert_eq!(one("rosalia").as_deref(), Some("rosalia"));
        assert_eq!(one("ROSALÍA").as_deref(), Some("rosalia"));
        assert_eq!(one("motorhead").as_deref(), Some("motorhead"));
        // the slug is searched too: the name of the file works as typed
        assert_eq!(one("the-cure").as_deref(), Some("the-cure"));
        // punctuation and spaces are out on both sides
        assert_eq!(one("catpower").as_deref(), Some("cat-power"));
        assert_eq!(one("zzz"), None);
    }

    /// An exact name wins over a longer one that merely contains it: `fw`
    /// takes the first hit, and it has to be the obvious one.
    #[test]
    fn an_exact_name_comes_first() {
        let catalog = catalog(&[("boo", "Boo!"), ("the-boo-radleys", "The Boo Radleys")]);
        assert_eq!(catalog.search_names("boo", 2), vec!["boo", "the-boo-radleys"]);
    }
}
