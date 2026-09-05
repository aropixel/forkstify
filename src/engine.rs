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
use crate::learned::Learned;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

/// Where a track came from — 0012 §1: « le top est un poids, pas une liste
/// fermée ». The reservoir of an artist cumulates several sources, and the
/// display says which one won, so a journey stays explainable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Top,
    /// Liked by this listener at this artist (`learned/`).
    Liked,
    /// A door (0011): singled out as the way out towards a direction.
    Door,
    /// Known artist, none of the above — came in through a search.
    Outside,
    /// No card at all.
    Offmap,
}

impl Source {
    /// One glyph, so a queue stays scannable.
    pub fn mark(self) -> char {
        match self {
            Source::Top => '♪',
            Source::Liked => '♥',
            Source::Door => '↳',
            Source::Outside => '+',
            Source::Offmap => '~',
        }
    }
}

#[derive(Clone)]
pub struct Stop {
    pub slug: String,
    pub artist: String,
    pub title: String,
    pub source: Source,
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

/// Below this cosine, the vector space is not trusted for an adventurous
/// jump. A constant for now — meant to be driven by the comfort zone
/// (decision 0001) once it enters the navigation.
const VECTOR_FLOOR: f32 = 0.72;
/// Above this cosine, the space may bridge without any shared genre tag.
const VECTOR_TRUST: f32 = 0.80;

fn shared_tags(a: &Card, b: &Card) -> Vec<String> {
    a.tags.iter().filter(|t| b.tags.contains(t)).cloned().collect()
}

/// Genre tags only: countries (2 letters) and decades ("80s", "2010s")
/// are context, not kinship — they must not justify a bridge alone.
fn genre_tags(card: &Card) -> impl Iterator<Item = &str> {
    card.tags.iter().map(String::as_str).filter(|t| {
        let decade = t.ends_with('s') && t[..t.len() - 1].chars().all(|c| c.is_ascii_digit());
        let country = t.len() == 2 && t.chars().all(|c| c.is_ascii_alphabetic());
        !decade && !country
    })
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

/// Weights of the reservoir (0012 §1). A top is the norm, a liked track
/// nearly as much, a door on its own is thinner — until the direction we
/// are heading towards matches it, and then it jumps ahead. None of this is
/// a rule: it is what the weighted draw is given to chew on.
const W_TOP: f32 = 1.0;
const W_LIKED: f32 = 0.8;
const W_DOOR: f32 = 0.4;
const DOOR_BONUS: f32 = 2.5;

/// The reservoir of one artist — 0012 §1, « le top est un poids, pas une
/// liste fermée ». Cumulates the tops, the tracks this listener liked here,
/// and the doors, each with its weight; a door only gets its bonus when
/// `towards` (the direction the branch is heading) meets its tags (0011).
/// The long tail of the discography — the fourth source — needs an API
/// cache that does not exist yet.
fn reservoir(
    card: &Card,
    slug: &str,
    learned: &Learned,
    played: &HashSet<String>,
    towards: &[String],
) -> Vec<(String, f32, Source)> {
    let mut pool: Vec<(String, f32, Source)> = Vec::new();
    for title in &card.tops {
        pool.push((title.clone(), W_TOP, Source::Top));
    }
    for title in learned.liked_tracks(slug) {
        if !pool.iter().any(|(t, ..)| t == title) {
            pool.push((title.clone(), W_LIKED, Source::Liked));
        }
    }
    for door in &card.doors {
        let opens = door.to.iter().any(|tag| towards.iter().any(|t| t == tag));
        match pool.iter_mut().find(|(t, ..)| *t == door.track) {
            // a door that is also a top keeps its place and gains the bonus
            Some(entry) => {
                if opens {
                    entry.1 *= DOOR_BONUS;
                    entry.2 = Source::Door;
                }
            }
            None => pool.push((
                door.track.clone(),
                if opens { W_DOOR * DOOR_BONUS } else { W_DOOR },
                Source::Door,
            )),
        }
    }
    pool.retain(|(title, ..)| !played.contains(title) && !learned.track_banned(slug, title));
    for entry in &mut pool {
        // a track often skipped falls back in the draw, it is not banned
        entry.1 /= 1.0 + learned.skipped(slug, &entry.0) as f32;
    }
    pool
}

/// One track from the reservoir, drawn by weight, with where it came from.
fn fresh_track(
    card: &Card,
    slug: &str,
    learned: &Learned,
    played: &HashSet<String>,
    towards: &[String],
    rng: &mut impl Rng,
) -> Option<(String, Source)> {
    let pool = reservoir(card, slug, learned, played, towards);
    let dist = WeightedIndex::new(pool.iter().map(|(_, w, _)| w.max(0.01))).ok()?;
    let (title, _, source) = &pool[dist.sample(rng)];
    Some((title.clone(), *source))
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

/// `:encore` (keybind `e`, "poncer" in the French slang of the project):
/// n more tracks from one artist, inserted right after the current track.
/// Not a branch — branches propose futures, encore reacts to the moment.
pub fn encore(
    catalog: &Catalog,
    artist: &str,
    learned: &Learned,
    played: &HashSet<String>,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<Stop> {
    let card = &catalog.cards[artist];
    // staying put is not heading anywhere: a door earns no bonus here
    let mut pool = reservoir(card, artist, learned, played, &[]);
    let mut stops = Vec::new();
    while stops.len() < count && !pool.is_empty() {
        let Ok(dist) = WeightedIndex::new(pool.iter().map(|(_, w, _)| w.max(0.01))) else {
            break;
        };
        // without replacement, as 0012 asks of a second sanding
        let (title, _, source) = pool.swap_remove(dist.sample(rng));
        stops.push(Stop { slug: artist.to_string(), artist: card.name.clone(), title, source });
    }
    stops
}

/// A direction branch: start at a neighbor, then keep walking to the
/// closest next artist — one track per artist along the way.
fn walk(
    catalog: &Catalog,
    current: &str,
    head: String,
    head_reason: String,
    head_weight: f32,
    learned: &Learned,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Branch {
    // the direction this branch heads towards: what a door has to match
    let towards: Vec<String> = catalog.cards[&head].tags.clone();
    let mut artists = vec![head];
    let mut stops = Vec::new();
    let mut hops = 0;

    loop {
        let last = artists.last().unwrap().clone();
        let card = &catalog.cards[&last];
        if let Some((title, source)) = fresh_track(card, &last, learned, played, &towards, rng) {
            stops.push(Stop { slug: last.clone(), artist: card.name.clone(), title, source });
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
                .filter(|(_, score)| *score >= VECTOR_FLOOR)
                .take(3)
                .map(|(slug, score)| (slug, (score - 0.5).max(0.05).powi(3)))
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
    learned: &Learned,
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
        if let Some((title, source)) = fresh_track(card, slug.as_str(), learned, played, &[], rng) {
            stops.push(Stop { slug: slug.clone(), artist: card.name.clone(), title, source });
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

/// Up to three branches, all directions, each a segment of `size` tracks:
/// staying in the journey's universe, then directions proposed from the
/// WHOLE previous branch — the graph for the reassuring side, the vector
/// space outside the graph for the adventurous one. Heads are drawn
/// (weighted) from the best candidates, not fixed, so two journeys from
/// the same seed differ. Staying on the artist is not a branch anymore:
/// that is `encore` (keybind `e`).
pub fn propose(
    catalog: &Catalog,
    context: &[String],
    universe: &[String],
    learned: &Learned,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Vec<Branch> {
    let current = context.last().unwrap().as_str();
    let card = &catalog.cards[current];
    let mut branches = Vec::new();
    if let Some(branch) = stay(catalog, universe, current, learned, played, size, rng) {
        branches.push(branch);
    }
    let slots = 3 - branches.len();

    let graph = graph_neighbors_of(catalog, context, visited);
    let in_graph: HashSet<String> = graph.iter().map(|(slug, ..)| slug.clone()).collect();
    // adventurous candidates: outside the graph, close enough in absolute
    // terms, and sharing a genre tag with the branch unless very close
    let context_genres: HashSet<&str> = context
        .iter()
        .flat_map(|slug| genre_tags(&catalog.cards[slug]))
        .collect();
    let outside: Vec<(String, f32)> = vector_neighbors_of(catalog, context, visited)
        .into_iter()
        .filter(|(slug, score)| {
            !in_graph.contains(slug)
                && *score >= VECTOR_FLOOR
                && (*score >= VECTOR_TRUST
                    || genre_tags(&catalog.cards[slug]).any(|t| context_genres.contains(t)))
        })
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
        .map(|(slug, score)| (slug.clone(), (score - 0.5).max(0.05).powi(3)))
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
        branches.push(walk(
            catalog, current, slug, why, weight, learned, visited, played, size, rng,
        ));
    }
    branches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Door;

    fn the_cure() -> Card {
        Card {
            name: "The Cure".into(),
            tags: vec!["post-punk".into(), "80s".into()],
            tops: vec!["Boys Don't Cry".into(), "A Forest".into()],
            doors: vec![Door {
                track: "A Forest".into(),
                to: vec!["post-punk".into(), "atmospherique".into()],
                note: None,
            }],
            links: Vec::new(),
        }
    }

    fn weight_of(pool: &[(String, f32, Source)], title: &str) -> f32 {
        pool.iter().find(|(t, ..)| t == title).map(|(_, w, _)| *w).expect(title)
    }

    /// 0012 §1 : le top est un poids, pas une liste fermée.
    #[test]
    fn le_reservoir_cumule_les_sources() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.like_track("the-cure", "Killing an Arab");
        let pool = reservoir(&card, "the-cure", &learned, &HashSet::new(), &[]);

        // les deux tops, plus le titre aimé qui n'en est pas un
        assert_eq!(pool.len(), 3, "{pool:?}");
        assert_eq!(weight_of(&pool, "Boys Don't Cry"), W_TOP);
        assert_eq!(weight_of(&pool, "Killing an Arab"), W_LIKED);
        let liked = pool.iter().find(|(t, ..)| t == "Killing an Arab").unwrap();
        assert_eq!(liked.2, Source::Liked);
    }

    /// 0011 : une door est un critère additionnel, jamais principal — son
    /// bonus ne tombe que si la direction recoupe ses tags.
    #[test]
    fn la_door_ne_gagne_que_vers_sa_direction() {
        let card = the_cure();
        let learned = Learned::blank();

        let ailleurs = reservoir(&card, "the-cure", &learned, &HashSet::new(), &["rap".into()]);
        assert_eq!(weight_of(&ailleurs, "A Forest"), W_TOP, "aucune direction commune");

        let vers = reservoir(&card, "the-cure", &learned, &HashSet::new(), &["post-punk".into()]);
        assert_eq!(weight_of(&vers, "A Forest"), W_TOP * DOOR_BONUS);
        let door = vers.iter().find(|(t, ..)| t == "A Forest").unwrap();
        assert_eq!(door.2, Source::Door, "la provenance doit se voir à l'affichage");
        // et le bonus reste local : l'autre top ne bouge pas
        assert_eq!(weight_of(&vers, "Boys Don't Cry"), W_TOP);
    }

    #[test]
    fn un_banni_sort_et_un_passe_recule() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.ban_track("the-cure", "Boys Don't Cry");
        learned.skip_track("the-cure", "A Forest");
        learned.skip_track("the-cure", "A Forest");
        let pool = reservoir(&card, "the-cure", &learned, &HashSet::new(), &[]);

        assert!(!pool.iter().any(|(t, ..)| t == "Boys Don't Cry"), "banni : hors du tirage");
        // deux sauts : le poids est divisé par trois, sans jamais s'annuler
        assert!((weight_of(&pool, "A Forest") - W_TOP / 3.0).abs() < 1e-6);
    }

    /// Ce qu'un parcours a déjà joué ne revient pas (0012 §3).
    #[test]
    fn le_deja_joue_ne_revient_pas() {
        let card = the_cure();
        let learned = Learned::blank();
        let played: HashSet<String> = ["A Forest".to_string()].into_iter().collect();
        let pool = reservoir(&card, "the-cure", &learned, &played, &[]);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].0, "Boys Don't Cry");
    }
}
