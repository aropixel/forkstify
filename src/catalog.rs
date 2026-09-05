//! Reads the active catalog: artist cards (TOML), proximity grid, vectors.

use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

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
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tops: Vec<String>,
    #[serde(default)]
    pub doors: Vec<Door>,
    #[serde(default)]
    pub links: Vec<Link>,
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
        let dir = path.join("fiches");
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("pas de dossier fiches/ dans {}", path.display()))?
        {
            let file = entry?.path();
            if file.extension().is_some_and(|e| e == "toml") {
                let slug = file.file_stem().unwrap().to_string_lossy().to_string();
                let card: Card = toml::from_str(&std::fs::read_to_string(&file)?)
                    .with_context(|| format!("fiche illisible : {}", file.display()))?;
                cards.insert(slug, card);
            }
        }

        let mut proximities: HashMap<String, u8> = DEFAULT_PROXIMITIES
            .iter()
            .map(|(kind, value)| (kind.to_string(), *value))
            .collect();
        if let Ok(text) = std::fs::read_to_string(path.join("catalogue.toml")) {
            let settings: Settings = toml::from_str(&text).context("catalogue.toml illisible")?;
            proximities.extend(settings.proximity);
        }

        let mut vectors = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(path.join("vecteurs/vecteurs.jsonl")) {
            for line in text.lines() {
                let parsed: VectorLine =
                    serde_json::from_str(line).context("vecteurs.jsonl illisible")?;
                vectors.insert(parsed.slug, parsed.v);
            }
        } else {
            eprintln!("(pas de vecteurs/vecteurs.jsonl : navigation sur le seul graphe)");
        }

        Ok(Catalog { cards, proximities, vectors })
    }

    /// Slugs whose card name matches the query (case-insensitive substring),
    /// for the `/` search — best-effort, name only.
    pub fn search_names(&self, query: &str, limit: usize) -> Vec<String> {
        let needle = query.to_lowercase();
        let mut hits: Vec<(&String, &String)> = self
            .cards
            .iter()
            .filter(|(_, card)| card.name.to_lowercase().contains(&needle))
            .map(|(slug, card)| (slug, &card.name))
            .collect();
        // exact-ish first (shorter names rank higher), then alphabetical
        hits.sort_by(|a, b| a.1.len().cmp(&b.1.len()).then(a.0.cmp(b.0)));
        hits.into_iter().take(limit).map(|(slug, _)| slug.clone()).collect()
    }

    /// A link's proximity, resolved in cascade: the link itself, the
    /// catalog's grid, the built-in defaults (decision 0010).
    pub fn proximity(&self, link: &Link) -> u8 {
        link.proximity
            .or_else(|| self.proximities.get(&link.kind).copied())
            .unwrap_or(3)
    }
}
