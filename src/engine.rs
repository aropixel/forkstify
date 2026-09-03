//! The branch engine: propose readable directions from an artist. The
//! graph first (typed links, both ways), vectors to fill the gaps. Every
//! proposal carries its reason — one sentence (project rule: any automatic
//! decision must be explainable in one sentence).

use crate::catalog::{Card, Catalog};
use std::collections::{HashMap, HashSet};

pub struct Branch {
    pub slug: String,
    pub name: String,
    pub reason: String,
    /// On the proximity scale (1–5); vector scores are mapped onto it.
    pub weight: f32,
}

// Reason labels are interface text, hence in French.
const LABELS: [(&str, &str); 6] = [
    ("member", "membres en commun"),
    ("collab", "collaboration"),
    ("similar", "similaires"),
    ("family", "liens familiaux"),
    ("scene", "même scène"),
    ("influence", "influence"),
];

fn label(kind: &str) -> &str {
    LABELS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, l)| *l)
        .unwrap_or(kind)
}

fn shared_tags(a: &Card, b: &Card) -> Vec<String> {
    a.tags.iter().filter(|t| b.tags.contains(t)).cloned().collect()
}

fn reason(kind: &str, note: Option<&str>, a: &Card, b: &Card) -> String {
    let mut text = label(kind).to_string();
    if let Some(note) = note {
        text.push_str(" — ");
        text.push_str(note);
    }
    let shared = shared_tags(a, b);
    if !shared.is_empty() {
        text.push_str(" · tags communs : ");
        text.push_str(&shared.join(", "));
    }
    text
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norms = a.iter().map(|x| x * x).sum::<f32>().sqrt()
        * b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / norms
}

/// Neighbors through the graph: the card's links plus links pointing at it
/// (a relation holds both ways). Per slug, the best proximity wins.
pub fn graph_neighbors(
    catalog: &Catalog,
    current: &str,
    excluded: &HashSet<String>,
) -> Vec<(String, u8, String)> {
    let card = &catalog.cards[current];
    let mut candidates: HashMap<String, (u8, String)> = HashMap::new();
    let mut offer = |slug: &str, proximity: u8, why: String| {
        let entry = candidates.entry(slug.to_string()).or_insert((0, String::new()));
        if proximity > entry.0 {
            *entry = (proximity, why);
        }
    };

    for link in &card.links {
        if let Some(target) = catalog.cards.get(&link.to) {
            if !excluded.contains(&link.to) {
                let why = reason(&link.kind, link.note.as_deref(), card, target);
                offer(&link.to, catalog.proximity(link), why);
            }
        }
    }
    for (slug, other) in &catalog.cards {
        if slug == current || excluded.contains(slug) {
            continue;
        }
        for link in &other.links {
            if link.to == current {
                let why = reason(&link.kind, link.note.as_deref(), card, other);
                offer(slug, catalog.proximity(link), why);
            }
        }
    }

    let mut sorted: Vec<(String, u8, String)> = candidates
        .into_iter()
        .map(|(slug, (proximity, why))| (slug, proximity, why))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted
}

/// Neighbors through the vector space, closest first.
pub fn vector_neighbors(
    catalog: &Catalog,
    current: &str,
    excluded: &HashSet<String>,
) -> Vec<(String, f32)> {
    let Some(reference) = catalog.vectors.get(current) else {
        return Vec::new();
    };
    let mut scores: Vec<(String, f32)> = catalog
        .vectors
        .iter()
        .filter(|(slug, _)| {
            slug.as_str() != current
                && !excluded.contains(*slug)
                && catalog.cards.contains_key(*slug)
        })
        .map(|(slug, vector)| (slug.clone(), cosine(reference, vector)))
        .collect();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores
}

/// Three branches: the two best from the graph (the reassuring side), then
/// the vector space outside the graph (the adventurous one). Vectors also
/// fill in when the graph falls short.
pub fn propose(catalog: &Catalog, current: &str, excluded: &HashSet<String>) -> Vec<Branch> {
    let card = &catalog.cards[current];
    let graph = graph_neighbors(catalog, current, excluded);

    let mut branches: Vec<Branch> = graph
        .iter()
        .take(2)
        .map(|(slug, proximity, why)| Branch {
            slug: slug.clone(),
            name: catalog.cards[slug].name.clone(),
            reason: why.clone(),
            weight: *proximity as f32,
        })
        .collect();

    let in_graph: HashSet<String> = graph.iter().map(|(slug, ..)| slug.clone()).collect();
    for (slug, score) in vector_neighbors(catalog, current, excluded) {
        if branches.len() >= 3 {
            break;
        }
        // The third branch must come from outside the graph; when the graph
        // could not fill its two slots, vectors complete freely.
        let outside_required = branches.len() == 2;
        if branches.iter().any(|b| b.slug == slug) || (outside_required && in_graph.contains(&slug))
        {
            continue;
        }
        let target = &catalog.cards[&slug];
        let shared = shared_tags(card, target);
        let mut why = format!("proche dans l'espace ({score:.2})");
        if !shared.is_empty() {
            why.push_str(" · tags communs : ");
            why.push_str(&shared.join(", "));
        }
        branches.push(Branch {
            slug,
            name: target.name.clone(),
            reason: why,
            weight: score * 5.0,
        });
    }

    branches
}
