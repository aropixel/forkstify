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
use crate::discography::{self, Tail};
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
    /// The long tail: the rest of the known discography (0012 §1).
    Tail,
    /// No card at all.
    Offmap,
}

impl Source {
    /// The word behind the glyph — interface text, hence French.
    pub fn word(self) -> &'static str {
        match self {
            Source::Top => "top",
            Source::Liked => "aimé",
            Source::Door => "door",
            Source::Tail => "traîne",
            Source::Outside => "hors tops",
            Source::Offmap => "hors catalogue",
        }
    }

    /// One glyph, so a queue stays scannable.
    pub fn mark(self) -> char {
        match self {
            Source::Top => '♪',
            Source::Liked => '♥',
            Source::Door => '↳',
            Source::Tail => '·',
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
    /// Posé sur le **premier** morceau d'une branche ajoutée à la file : son
    /// nom et sa raison. C'est ce qui permet de voir, dans la liste de
    /// lecture, où une branche commence et pourquoi — la file en enchaîne
    /// plusieurs (Joel, 06/09/2026), et chaque maillon dit sa raison à
    /// droite (maquette 3a, 07/09/2026).
    pub head: Option<Head>,
}

/// What opens a link of the playlist: the branch's label and its reason.
#[derive(Clone)]
pub struct Head {
    pub label: String,
    pub reason: String,
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
/// The adventurous floor, at the cocon and wide open. The old fixed 0.72
/// and 0.80 sit at comfort 2 — today's tuning becomes the middle of the
/// dial rather than a constant.
const FLOOR_COCON: f32 = 0.80;
const FLOOR_OPEN: f32 = 0.60;
const TRUST_COCON: f32 = 0.86;
const TRUST_OPEN: f32 = 0.70;

/// Above this cosine, the space may bridge without any shared genre tag.

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

/// The comfort dial (0001): **5 = cocon, 0 = exploration**.
///
/// **The scale was turned round on 06/09/2026**, at Joel's first real use:
/// « si je veux le cocon, je devrais mettre le confort à 5 — le confort,
/// c'est ce qu'on connaît bien ». He is right, and the repository was the
/// odd one out: [0012](../decisions/0012-rotation-des-morceaux.md) §4 says
/// « confort haut : tirage serré sur les tops », which now reads literally.
/// Only `zone-de-confort.md` said the opposite, and a conception note gives
/// way to use.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Comfort(u8);

impl Comfort {
    pub fn new(value: u8) -> Comfort {
        Comfort(value.min(5))
    }

    pub fn value(self) -> u8 {
        self.0
    }

    /// 0.0 in the cocon, 1.0 wide open. The dial counts the other way —
    /// comfort *is* familiarity — so openness is its mirror.
    fn openness(self) -> f32 {
        1.0 - self.0 as f32 / 5.0
    }

    /// How far the adventurous branch may leap. The floor drops as the dial
    /// opens — the constants avancement.md already flagged as « à piloter
    /// par le confort ».
    fn floor(self) -> f32 {
        FLOOR_COCON + (FLOOR_OPEN - FLOOR_COCON) * self.openness()
    }

    fn trust(self) -> f32 {
        TRUST_COCON + (TRUST_OPEN - TRUST_COCON) * self.openness()
    }

    /// The pull of what we already know (0001: comfort *is* familiarity):
    /// +1 in the cocon, 0 in the middle, −1 wide open, where the unknown is
    /// what we are after.
    fn pull(self) -> f32 {
        1.0 - 2.0 * self.openness()
    }

    /// The share the long tail gets in the reservoir — 0012 §4: « confort
    /// haut : tirage serré sur les tops ; confort bas : la longue traîne
    /// pèse davantage ». Read with the polarity above, that means **zero in
    /// the cocon** and full weight wide open.
    fn tail_share(self) -> f32 {
        self.openness()
    }

    /// What this dial does to a candidate of that familiarity. Never zero:
    /// a branch is discouraged, never forbidden — the application does not
    /// decide for the ear.
    fn favours(self, familiarity: f32) -> f32 {
        (1.0 + self.pull() * (2.0 * familiarity - 1.0)).clamp(0.25, 2.0)
    }
}

/// Weights of the reservoir (0012 §1). A top is the norm, a liked track
/// nearly as much, a door on its own is thinner — until the direction we
/// are heading towards matches it, and then it jumps ahead. None of this is
/// a rule: it is what the weighted draw is given to chew on.
const W_TOP: f32 = 1.0;
const W_LIKED: f32 = 0.8;
const W_DOOR: f32 = 0.4;
const DOOR_BONUS: f32 = 2.5;
/// The tail's own weight, before the comfort dial scales it. Low per track,
/// but a discography has ten times more tracks than a card has tops — so
/// cumulatively it takes over as the dial opens, which is what 0012 §4 asks.
const W_TAIL: f32 = 0.25;

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
    tail: &Tail,
    comfort: Comfort,
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
    // the long tail, scaled by the dial: nothing at the cocon, plenty open
    let share = comfort.tail_share();
    if share > 0.0 {
        let known: HashSet<String> =
            pool.iter().map(|(t, ..)| discography::normalize(t)).collect();
        let mut seen = HashSet::new();
        for track in tail.of(slug) {
            let key = discography::normalize(&track.title);
            // the tail is what is *not* already in the reservoir, and
            // Spotify ships the same song under a dozen version names
            if known.contains(&key) || !seen.insert(key) {
                continue;
            }
            pool.push((track.title.clone(), W_TAIL * share, Source::Tail));
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
    tail: &Tail,
    comfort: Comfort,
    played: &HashSet<String>,
    towards: &[String],
    rng: &mut impl Rng,
) -> Option<(String, Source)> {
    let pool = reservoir(card, slug, learned, tail, comfort, played, towards);
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
    tail: &Tail,
    comfort: Comfort,
    played: &HashSet<String>,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<Stop> {
    let card = &catalog.cards[artist];
    // staying put is not heading anywhere: a door earns no bonus here
    let mut pool = reservoir(card, artist, learned, tail, comfort, played, &[]);
    let mut stops = Vec::new();
    while stops.len() < count && !pool.is_empty() {
        let Ok(dist) = WeightedIndex::new(pool.iter().map(|(_, w, _)| w.max(0.01))) else {
            break;
        };
        // without replacement, as 0012 asks of a second sanding
        let (title, _, source) = pool.swap_remove(dist.sample(rng));
        stops.push(Stop {
            slug: artist.to_string(),
            artist: card.name.clone(),
            title,
            source,
            head: None,
        });
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
    tail: &Tail,
    comfort: Comfort,
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
        if let Some((title, source)) = fresh_track(card, &last, learned, tail, comfort, played, &towards, rng) {
            stops.push(Stop {
                slug: last.clone(),
                artist: card.name.clone(),
                title,
                source,
                head: None,
            });
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
                .filter(|(_, score)| *score >= comfort.floor())
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
    tail: &Tail,
    comfort: Comfort,
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
        if let Some((title, source)) = fresh_track(card, slug.as_str(), learned, tail, comfort, played, &[], rng) {
            stops.push(Stop {
                slug: slug.clone(),
                artist: card.name.clone(),
                title,
                source,
                head: None,
            });
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
    tail: &Tail,
    comfort: Comfort,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Vec<Branch> {
    let current = context.last().unwrap().as_str();
    let card = &catalog.cards[current];
    let mut branches = Vec::new();
    if let Some(branch) = stay(catalog, universe, current, learned, tail, comfort, played, size, rng) {
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
                && *score >= comfort.floor()
                && (*score >= comfort.trust()
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
    // 0001 : le confort *est* la familiarité. Il ne filtre pas, il penche —
    // vers ce qu'on connaît au cocon, vers ce qu'on ne connaît pas ouvert.
    let favours = |slug: &String| {
        comfort.favours(learned.familiarity01(slug, &catalog.cards[slug].name))
    };
    let mut graph_pool: Vec<(String, f32)> = graph
        .iter()
        .take(6)
        .map(|(slug, weight, _)| (slug.clone(), weight * weight * favours(slug)))
        .collect();
    while heads.len() < graph_slots {
        let Some(slug) = draw_weighted(&mut graph_pool, rng) else { break };
        let (weight, why) = by_slug[&slug];
        heads.push((slug.clone(), why.clone(), *weight));
    }
    let mut outside_pool: Vec<(String, f32)> = outside
        .iter()
        .take(6)
        .map(|(slug, score)| (slug.clone(), (score - 0.5).max(0.05).powi(3) * favours(slug)))
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
            catalog, current, slug, why, weight, learned, tail, comfort, visited, played, size,
            rng,
        ));
    }
    branches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Door;

    /// A blank tail: the tests that predate it must keep meaning the same
    /// thing, and a cocon draws none of it anyway.
    fn no_tail() -> Tail {
        Tail::blank()
    }

    fn the_cure() -> Card {
        Card {
            generated: false,
            name: "The Cure".into(),
            spotify: None,
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
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &[]);

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

        let ailleurs = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &["rap".into()]);
        assert_eq!(weight_of(&ailleurs, "A Forest"), W_TOP, "aucune direction commune");

        let vers = reservoir(
            &card,
            "the-cure",
            &learned,
            &no_tail(),
            Comfort::new(0),
            &HashSet::new(),
            &["post-punk".into()],
        );
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
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &[]);

        assert!(!pool.iter().any(|(t, ..)| t == "Boys Don't Cry"), "banni : hors du tirage");
        // deux sauts : le poids est divisé par trois, sans jamais s'annuler
        assert!((weight_of(&pool, "A Forest") - W_TOP / 3.0).abs() < 1e-6);
    }

    /// 0001 : le confort *est* la familiarité — et la polarité de l'échelle
    /// est le piège de cette décision (voir la doc de `Comfort`).
    #[test]
    fn le_cocon_penche_vers_le_connu_et_l_exploration_vers_l_inconnu() {
        // 5 = cocon, 0 = exploration (retourné le 06/09/2026)
        let cocon = Comfort::new(5);
        let milieu = Comfort::new(3);
        let ouvert = Comfort::new(0);

        // au cocon, un artiste familier passe devant un inconnu
        assert!(cocon.favours(1.0) > cocon.favours(0.0));
        // ouvert, c'est l'inverse — sinon le curseur ne sert à rien
        assert!(ouvert.favours(0.0) > ouvert.favours(1.0));
        // et jamais une exclusion : on décourage, on n'interdit pas
        assert!(cocon.favours(0.0) >= 0.25 && ouvert.favours(1.0) >= 0.25);

        // le plancher de l'aventureuse s'abaisse quand on ouvre
        assert!(cocon.floor() > milieu.floor());
        assert!(milieu.floor() > ouvert.floor());
        // le confort 3 reproduit le réglage fixe d'avant le curseur
        let trois_ = Comfort::new(3);
        assert!((trois_.floor() - 0.72).abs() < 1e-6, "{}", trois_.floor());
        assert!((trois_.trust() - 0.796).abs() < 1e-3, "{}", trois_.trust());

        // six valeurs entières : il n'y a pas de milieu exact. 3 penche
        // encore vers le connu, 2 déjà vers l'inconnu — la bascule tombe
        // entre les deux, et c'est une propriété, pas un défaut.
        assert!(milieu.favours(1.0) > milieu.favours(0.0));
        let deux = Comfort::new(2);
        assert!(deux.favours(0.0) > deux.favours(1.0));
        // et la valeur est bornée
        assert_eq!(Comfort::new(9).value(), 5);
    }

    /// 0012 §4 : c'est le confort qui règle la profondeur du tirage. Au
    /// cocon la traîne ne pèse rien ; ouvert, elle prend le dessus.
    #[test]
    fn la_traine_ne_pese_que_quand_on_ouvre() {
        let card = the_cure();
        let learned = Learned::blank();
        let mut tail = Tail::blank();
        tail.keep(
            "the-cure",
            vec![
                crate::discography::TailTrack {
                    title: "Killing an Arab".into(),
                    uri: "spotify:track:x".into(),
                    album: "Three Imaginary Boys".into(),
                },
                // la même chanson, remasterisée : elle ne doit pas compter deux fois
                crate::discography::TailTrack {
                    title: "Killing an Arab - 2004 Remaster".into(),
                    uri: "spotify:track:y".into(),
                    album: "Boys Don't Cry".into(),
                },
                // et un titre déjà dans les tops n'entre pas dans la traîne
                crate::discography::TailTrack {
                    title: "A Forest (Remastered)".into(),
                    uri: "spotify:track:z".into(),
                    album: "Seventeen Seconds".into(),
                },
            ],
        );

        // 5 = cocon depuis le 06/09/2026
        let cocon = reservoir(&card, "the-cure", &learned, &tail, Comfort::new(5), &HashSet::new(), &[]);
        assert!(!cocon.iter().any(|(_, _, s)| *s == Source::Tail), "au cocon, pas de traîne");
        assert_eq!(cocon.len(), 2, "les deux tops, rien d'autre");

        let ouvert = reservoir(&card, "the-cure", &learned, &tail, Comfort::new(0), &HashSet::new(), &[]);
        let traine: Vec<&(String, f32, Source)> =
            ouvert.iter().filter(|(_, _, s)| *s == Source::Tail).collect();
        assert_eq!(traine.len(), 1, "une seule fois Killing an Arab : {ouvert:?}");
        assert_eq!(traine[0].0, "Killing an Arab");
        assert!(traine[0].1 > 0.0 && traine[0].1 < W_TOP, "moins qu'un top, mais présente");
    }

    /// Ce qu'un parcours a déjà joué ne revient pas (0012 §3).
    #[test]
    fn le_deja_joue_ne_revient_pas() {
        let card = the_cure();
        let learned = Learned::blank();
        let played: HashSet<String> = ["A Forest".to_string()].into_iter().collect();
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &played, &[]);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].0, "Boys Don't Cry");
    }
}
