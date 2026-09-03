//! The branch engine: propose readable directions from an artist. The
//! graph first (typed links, both ways), vectors to fill the gaps. Every
//! proposal carries its reason — one sentence (project rule: any automatic
//! decision must be explainable in one sentence).
//!
//! A branch IS a segment: a handful of tracks, possibly across several
//! artists (asked by Joel while testing, 03/09/2026). The first branch
//! sands the current artist; the others walk a direction, one track per
//! artist along the way.

use crate::catalog::{Card, Catalog};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct Stop {
    pub artist: String,
    pub title: String,
}

pub struct Branch {
    pub label: String,
    pub reason: String,
    /// Slugs walked by this branch (empty when sanding the current artist).
    pub artists: Vec<String>,
    pub stops: Vec<Stop>,
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

/// Neighbors of a whole branch through the graph: the neighbors of each
/// of its artists, merged. A candidate linked to several of them fits the
/// branch's direction better and climbs.
fn graph_neighbors_of(
    catalog: &Catalog,
    context: &[String],
    excluded: &HashSet<String>,
) -> Vec<(String, f32, String)> {
    let mut merged: HashMap<String, (u8, String, usize)> = HashMap::new();
    for source in context {
        for (slug, proximity, why) in graph_neighbors(catalog, source, excluded) {
            let why = if source == context.last().unwrap() {
                why
            } else {
                format!("via {} : {}", catalog.cards[source].name, why)
            };
            let entry = merged.entry(slug).or_insert((0, String::new(), 0));
            entry.2 += 1;
            if proximity > entry.0 {
                entry.0 = proximity;
                entry.1 = why;
            }
        }
    }
    let mut sorted: Vec<(String, f32, String)> = merged
        .into_iter()
        .map(|(slug, (proximity, mut why, count))| {
            if count > 1 {
                why.push_str(&format!(" · lié à {count} artistes de la branche"));
            }
            // half a point per extra artist of the branch backing it
            (slug, proximity as f32 + 0.5 * (count as f32 - 1.0), why)
        })
        .collect();
    sorted.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted
}

/// Neighbors of a whole branch through the vector space: closest to the
/// centroid of its artists' vectors — the branch's center of gravity.
fn vector_neighbors_of(
    catalog: &Catalog,
    context: &[String],
    excluded: &HashSet<String>,
) -> Vec<(String, f32)> {
    let known: Vec<&Vec<f32>> =
        context.iter().filter_map(|slug| catalog.vectors.get(slug)).collect();
    let Some(first) = known.first() else {
        return Vec::new();
    };
    let mut centroid = vec![0.0f32; first.len()];
    for vector in &known {
        for (c, x) in centroid.iter_mut().zip(vector.iter()) {
            *c += x / known.len() as f32;
        }
    }
    let mut scores: Vec<(String, f32)> = catalog
        .vectors
        .iter()
        .filter(|(slug, _)| {
            !context.contains(slug)
                && !excluded.contains(*slug)
                && catalog.cards.contains_key(*slug)
        })
        .map(|(slug, vector)| (slug.clone(), cosine(&centroid, vector)))
        .collect();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores
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

/// One unplayed top, at random (decision 0012: no repetition endured).
fn fresh_track(card: &Card, played: &HashSet<String>, rng: &mut impl Rng) -> Option<String> {
    let fresh: Vec<&String> = card.tops.iter().filter(|t| !played.contains(*t)).collect();
    fresh.choose(rng).map(|t| (*t).clone())
}

/// Weighted draw without replacement — decision 0012 applied to the
/// branches themselves: good candidates rotate instead of the best one
/// winning every time.
fn draw_weighted(pool: &mut Vec<(String, f32)>, rng: &mut impl Rng) -> Option<String> {
    if pool.is_empty() {
        return None;
    }
    let weights = pool.iter().map(|(_, w)| w.max(0.01));
    let dist = WeightedIndex::new(weights).ok()?;
    Some(pool.swap_remove(dist.sample(rng)).0)
}

/// The sanding branch: stay on the current artist, more of its tops.
fn sand(catalog: &Catalog, current: &str, played: &HashSet<String>, size: usize, rng: &mut impl Rng) -> Option<Branch> {
    let card = &catalog.cards[current];
    let fresh: Vec<&String> = card.tops.iter().filter(|t| !played.contains(*t)).collect();
    if fresh.is_empty() {
        return None;
    }
    let stops: Vec<Stop> = fresh
        .choose_multiple(rng, size)
        .map(|title| Stop { artist: card.name.clone(), title: (*title).clone() })
        .collect();
    Some(Branch {
        label: format!("Poncer {}", card.name),
        reason: String::new(),
        artists: Vec::new(),
        stops,
        weight: 4.0,
    })
}

/// A direction branch: start at a neighbor, then keep walking to the
/// closest next artist — one track per artist along the way.
fn walk(
    catalog: &Catalog,
    current: &str,
    head: String,
    head_reason: String,
    head_weight: f32,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Branch {
    let mut artists = vec![head];
    let mut stops = Vec::new();
    let mut hops = 0;

    loop {
        let last = artists.last().unwrap().clone();
        let card = &catalog.cards[&last];
        if let Some(title) = fresh_track(card, played, rng) {
            stops.push(Stop { artist: card.name.clone(), title });
        }
        hops += 1;
        if stops.len() >= size || hops >= size * 3 {
            break;
        }
        let mut excluded: HashSet<String> = visited.clone();
        excluded.insert(current.to_string());
        excluded.extend(artists.iter().cloned());
        // draw the next hop among the closest few, not always the closest
        let mut nexts: Vec<(String, f32)> = graph_neighbors(catalog, &last, &excluded)
            .into_iter()
            .take(3)
            .map(|(slug, proximity, _)| (slug, proximity as f32))
            .collect();
        if nexts.is_empty() {
            nexts = vector_neighbors(catalog, &last, &excluded)
                .into_iter()
                .take(3)
                .map(|(slug, score)| (slug, (score - 0.5).max(0.05)))
                .collect();
        }
        match draw_weighted(&mut nexts, rng) {
            Some(slug) => artists.push(slug),
            None => break,
        }
    }

    let label = artists
        .iter()
        .map(|slug| catalog.cards[slug].name.as_str())
        .collect::<Vec<_>>()
        .join(" → ");
    Branch { label, reason: head_reason, artists, stops, weight: head_weight }
}

/// The "stay in this universe" branch (asked by Joel while testing): a
/// segment drawn from the whole journey's neighborhood — its artists and
/// their graph neighbors, ranked by closeness to the journey's centroid.
/// Already-visited artists may come back as long as they still hold
/// unplayed tops.
fn stay(
    catalog: &Catalog,
    universe: &[String],
    current: &str,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Option<Branch> {
    if universe.len() < 2 {
        return None;
    }
    let known: Vec<&Vec<f32>> =
        universe.iter().filter_map(|slug| catalog.vectors.get(slug)).collect();
    let first = known.first()?;
    let mut centroid = vec![0.0f32; first.len()];
    for vector in &known {
        for (c, x) in centroid.iter_mut().zip(vector.iter()) {
            *c += x / known.len() as f32;
        }
    }

    let nobody = HashSet::new();
    let mut seen = HashSet::new();
    let mut pool: Vec<(String, f32)> = Vec::new();
    for slug in universe {
        let around = std::iter::once(slug.clone())
            .chain(graph_neighbors(catalog, slug, &nobody).into_iter().map(|(s, ..)| s));
        for candidate in around {
            if candidate == current || !seen.insert(candidate.clone()) {
                continue;
            }
            let card = &catalog.cards[&candidate];
            if !card.tops.iter().any(|t| !played.contains(t)) {
                continue;
            }
            let closeness = catalog
                .vectors
                .get(&candidate)
                .map(|v| cosine(&centroid, v))
                .unwrap_or(0.6);
            pool.push((candidate, closeness));
        }
    }
    pool.sort_by(|a, b| b.1.total_cmp(&a.1));
    pool.truncate(12);
    let mut weighted: Vec<(String, f32)> =
        pool.into_iter().map(|(slug, c)| (slug, (c - 0.5).max(0.05))).collect();

    let mut artists = Vec::new();
    let mut stops = Vec::new();
    while stops.len() < size {
        let Some(slug) = draw_weighted(&mut weighted, rng) else { break };
        let card = &catalog.cards[&slug];
        if let Some(title) = fresh_track(card, played, rng) {
            stops.push(Stop { artist: card.name.clone(), title });
            artists.push(slug);
        }
    }
    if stops.is_empty() {
        return None;
    }
    let label = artists
        .iter()
        .map(|slug| catalog.cards[slug].name.as_str())
        .collect::<Vec<_>>()
        .join(" → ");
    Some(Branch {
        label,
        reason: "rester dans l'univers du parcours".to_string(),
        artists,
        stops,
        weight: 4.0,
    })
}

/// Up to four branches, each a segment of `size` tracks: sanding the
/// current artist while it has unplayed tops, staying in the journey's
/// universe, then directions proposed from the WHOLE previous branch —
/// the graph for the reassuring side, the vector space outside the graph
/// for the adventurous one. Heads are drawn (weighted) from the best
/// candidates, not fixed, so two journeys from the same seed differ.
pub fn propose(
    catalog: &Catalog,
    context: &[String],
    universe: &[String],
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Vec<Branch> {
    let current = context.last().unwrap().as_str();
    let card = &catalog.cards[current];
    let mut branches = Vec::new();
    if let Some(branch) = sand(catalog, current, played, size, rng) {
        branches.push(branch);
    }
    if let Some(branch) = stay(catalog, universe, current, played, size, rng) {
        branches.push(branch);
    }
    // 4 branches when sanding and staying are both on the table, never
    // more than 3 direction branches
    let slots = (4usize.saturating_sub(branches.len())).min(3);

    let graph = graph_neighbors_of(catalog, context, visited);
    let in_graph: HashSet<String> = graph.iter().map(|(slug, ..)| slug.clone()).collect();
    let outside: Vec<(String, f32)> = vector_neighbors_of(catalog, context, visited)
        .into_iter()
        .filter(|(slug, _)| !in_graph.contains(slug))
        .collect();
    let graph_slots = if outside.is_empty() { slots } else { slots.saturating_sub(1) };

    // draw the heads from a reservoir of good candidates (0012 again)
    let by_slug: HashMap<&String, (&f32, &String)> = graph
        .iter()
        .map(|(slug, weight, why)| (slug, (weight, why)))
        .collect();
    let mut heads: Vec<(String, String, f32)> = Vec::new();
    let mut graph_pool: Vec<(String, f32)> = graph
        .iter()
        .take(6)
        .map(|(slug, weight, _)| (slug.clone(), weight * weight))
        .collect();
    while heads.len() < graph_slots {
        let Some(slug) = draw_weighted(&mut graph_pool, rng) else { break };
        let (weight, why) = by_slug[&slug];
        heads.push((slug.clone(), why.clone(), *weight));
    }
    let mut outside_pool: Vec<(String, f32)> = outside
        .iter()
        .take(6)
        .map(|(slug, score)| (slug.clone(), (score - 0.5).max(0.05)))
        .collect();
    let outside_scores: HashMap<&String, &f32> =
        outside.iter().map(|(slug, score)| (slug, score)).collect();
    while heads.len() < slots {
        let Some(slug) = draw_weighted(&mut outside_pool, rng) else { break };
        let score = *outside_scores[&slug];
        let target = &catalog.cards[&slug];
        let shared = shared_tags(card, target);
        let mut why = format!("proche du centre de la branche ({score:.2})");
        if !shared.is_empty() {
            why.push_str(" · tags communs : ");
            why.push_str(&shared.join(", "));
        }
        heads.push((slug, why, score * 5.0));
    }

    for (slug, why, weight) in heads {
        branches.push(walk(catalog, current, slug, why, weight, visited, played, size, rng));
    }
    branches
}
