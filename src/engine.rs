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
use crate::config::tuning;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

/// Where a track came from — 0012 §1: "the top is a weight, not a closed
/// list". The reservoir of an artist cumulates several sources, and the
/// display says which one won, so a journey stays explainable.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
    /// The word behind the glyph — interface text.
    pub fn word(self) -> &'static str {
        match self {
            Source::Top => "top",
            Source::Liked => "liked",
            Source::Door => "door",
            Source::Tail => "tail",
            Source::Outside => "non-top",
            Source::Offmap => "off-catalog",
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Stop {
    pub slug: String,
    pub artist: String,
    pub title: String,
    pub source: Source,
    /// Set on the **first** track of a branch added to the queue: its name
    /// and its reason. That is what shows, in the playlist, where a branch
    /// starts and why — the queue chains several of them (Joel,
    /// 06/09/2026), and each link says its reason on the right (mockup 3a,
    /// 07/09/2026).
    pub head: Option<Head>,
    /// Added by an encore (`e<n>`): the list shows it with "↻" instead of
    /// "→", which is how the gesture is verified — no notice needed
    /// (Joel, 07/09/2026).
    pub encore: bool,
}

/// What opens a link of the playlist: the branch's label and its reason.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

// Reason labels are interface text.
const LABELS: [(&str, &str); 7] = [
    (crate::catalog::MINE, "yours"),
    ("member", "shared members"),
    ("collab", "collaboration"),
    ("similar", "similar"),
    ("family", "family ties"),
    ("scene", "same scene"),
    ("influence", "influence"),
];

fn label(kind: &str) -> &str {
    LABELS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, l)| *l)
        .unwrap_or(kind)
}

// The adventurous leap's thresholds — under the floor the vector space is
// not trusted for a jump outside the graph, above the trust it may bridge
// without a shared genre tag — slide with the dial, and their values are
// the listener's (`[tuning]`, 0023): `leap_floor_*`, `leap_trust_*`.

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
        text.push_str(" · shared tags: ");
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

/// A link pointing at a card that does not exist — a direction the catalog
/// names but cannot walk yet.
///
/// The engine has always dropped those (`graph_neighbors` below). Since
/// 09/09/2026 they are a proposal instead: generating the card is one
/// keystroke, and that is how a catalog grows along its own edges
/// (`docs/design/on-the-fly-generation.md`). They stay out of
/// `graph_neighbors` on purpose — every walk indexes `catalog.cards`, and a
/// neighbor without a card would be a panic waiting to happen.
pub struct Missing {
    pub slug: String,
    /// The name to show, and the one to ask MusicBrainz for: a slug spelled
    /// back out, since nothing else is known of an artist without a card.
    pub name: String,
    pub kind: String,
    pub proximity: u8,
    pub why: String,
    /// Generation is underway: the column says so rather than letting one
    /// think an `f<n>` did nothing.
    pub pending: bool,
}

/// The context's links that point nowhere, closest first.
pub fn missing_neighbors(
    catalog: &Catalog,
    context: &[String],
    excluded: &HashSet<String>,
) -> Vec<Missing> {
    let mut best: HashMap<String, (u8, String, String)> = HashMap::new();
    for source in context {
        let Some(card) = catalog.cards.get(source) else { continue };
        for link in &card.links {
            if catalog.cards.contains_key(&link.to) || excluded.contains(&link.to) {
                continue;
            }
            let mut why = label(&link.kind).to_string();
            if let Some(note) = &link.note {
                why.push_str(" — ");
                why.push_str(note);
            }
            why.push_str(&format!(" · around {}", card.name));
            let proximity = catalog.proximity(link);
            let entry = best
                .entry(link.to.clone())
                .or_insert((0, link.kind.clone(), String::new()));
            if proximity > entry.0 {
                *entry = (proximity, link.kind.clone(), why);
            }
        }
    }
    let mut sorted: Vec<Missing> = best
        .into_iter()
        .map(|(slug, (proximity, kind, why))| Missing {
            name: crate::generate::pretty(&slug),
            slug,
            kind,
            proximity,
            why,
            pending: false,
        })
        .collect();
    sorted.sort_by(|a, b| b.proximity.cmp(&a.proximity).then(a.slug.cmp(&b.slug)));
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
                format!("via {}: {}", catalog.cards[source].name, why)
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
                why.push_str(&format!(" · linked to {count} artists of the branch"));
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

/// The comfort dial (0001): **5 = cocoon, 0 = exploration**.
///
/// **The scale was turned round on 06/09/2026**, at Joel's first real use:
/// "if I want the cocoon, I should set comfort to 5 — comfort is what we
/// know well". He is right, and the repository was the odd one out:
/// [0012](../decisions/0012-track-rotation.md) §4 says "high
/// comfort: a tight draw on the tops", which now reads literally.
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

    /// 0.0 in the cocoon, 1.0 wide open. The dial counts the other way —
    /// comfort *is* familiarity — so openness is its mirror.
    fn openness(self) -> f32 {
        1.0 - self.0 as f32 / 5.0
    }

    /// How far the adventurous branch may leap. The floor drops as the dial
    /// opens — the constants progress.md already flagged as "to be driven
    /// by comfort".
    fn floor(self) -> f32 {
        let t = tuning();
        t.leap_floor_cocoon + (t.leap_floor_open - t.leap_floor_cocoon) * self.openness()
    }

    fn trust(self) -> f32 {
        let t = tuning();
        t.leap_trust_cocoon + (t.leap_trust_open - t.leap_trust_cocoon) * self.openness()
    }

    /// The pull of what we already know (0001: comfort *is* familiarity):
    /// +1 in the cocoon, 0 in the middle, −1 wide open, where the unknown is
    /// what we are after.
    fn pull(self) -> f32 {
        1.0 - 2.0 * self.openness()
    }

    /// The share the long tail gets in the reservoir — 0012 §4: "high
    /// comfort: a tight draw on the tops; low comfort: the long tail weighs
    /// more". Read with the polarity above, that means **zero in the
    /// cocoon** and full weight wide open.
    fn tail_share(self) -> f32 {
        self.openness()
    }

    /// Whether the tail is worth fetching at all: at the cocoon it weighs
    /// nothing, so a harvest would only cost the network.
    pub fn wants_tail(self) -> bool {
        self.tail_share() > 0.0
    }

    /// What this dial does to a candidate of that familiarity. Never zero:
    /// a branch is discouraged, never forbidden — the application does not
    /// decide for the ear.
    pub fn favours(self, familiarity: f32) -> f32 {
        (1.0 + self.pull() * (2.0 * familiarity - 1.0)).clamp(0.25, 2.0)
    }
}

// Weights of the reservoir (0012 §1) — a top is the norm, a liked track
// outweighs it by the dial (×10 in the cocoon, ×2 wide open, 0018), a door
// on its own is thinner until the direction matches it, one track of the
// tail is light but a discography has ten times more of them. None of
// this is a rule: it is what the weighted draw is given to chew on, and
// the numbers are the listener's (`[tuning]`, 0023).

/// The reservoir of one artist — 0012 §1, "the top is a weight, not a
/// closed list". Cumulates the tops, the tracks this listener liked here,
/// and the doors, each with its weight; a door only gets its bonus when
/// `towards` (the direction the branch is heading) meets its tags (0011).
/// The long tail of the discography — the fourth source — needs an API
/// cache that does not exist yet.
/// The weight of a liked track, by the dial (0018).
fn liked_weight(comfort: Comfort) -> f32 {
    let t = tuning();
    t.top_weight * (t.liked_weight_open + (t.liked_weight_cocoon - t.liked_weight_open) * (1.0 - comfort.openness()))
}

/// The card a stop belongs to: its own slug, or the one its artist's name
/// resolves to. A track inserted from Spotify carries no slug until its
/// card lands (0016), and a card written under another name — "Ye" for
/// Kanye West — is still found by the slug.
pub fn card_of(cards: &HashMap<String, Card>, stop: &Stop) -> Option<String> {
    if cards.contains_key(&stop.slug) {
        return Some(stop.slug.clone());
    }
    let by_name = crate::generate::slugify(&stop.artist);
    cards.contains_key(&by_name).then_some(by_name)
}

/// What a track's glyph would be if it were drawn now, likes aside: a top
/// of the card, a track of its tail, or neither.
pub fn plain_source(catalog: &Catalog, tail: &Tail, slug: &str, title: &str) -> Source {
    let Some(card) = catalog.cards.get(slug) else { return Source::Offmap };
    if card.tops.iter().any(|top| top == title) {
        return Source::Top;
    }
    let key = discography::normalize(title);
    if tail.of(slug).iter().any(|t| discography::normalize(&t.title) == key) {
        Source::Tail
    } else {
        Source::Outside
    }
}

/// The glyph a track deserves as things stand. The like outranks the rest
/// (0018); a door keeps its own, which says where it leads rather than how
/// it was picked. A source is fixed when a track is drawn, so liking one
/// afterwards left the old glyph in the list (Joel, 2026-09-23).
pub fn current_source(catalog: &Catalog, tail: &Tail, learned: &Learned, stop: &Stop) -> Source {
    if learned.track_liked(&stop.slug, &stop.title) {
        Source::Liked
    } else if stop.source == Source::Door {
        Source::Door
    } else {
        plain_source(catalog, tail, &stop.slug, &stop.title)
    }
}

fn reservoir(
    card: &Card,
    slug: &str,
    learned: &Learned,
    tail: &Tail,
    comfort: Comfort,
    played: &HashSet<String>,
    towards: &[String],
) -> Vec<(String, f32, Source)> {
    let t = tuning();
    let mut pool: Vec<(String, f32, Source)> = Vec::new();
    for title in &card.tops {
        pool.push((title.clone(), t.top_weight, Source::Top));
    }
    // what the listener likes comes first, top or not: a liked top takes
    // the like's weight and wears its mark (0018)
    let liked_weight = liked_weight(comfort);
    for title in learned.liked_tracks(slug) {
        match pool.iter_mut().find(|(t, ..)| t == title) {
            Some(entry) => {
                entry.1 = liked_weight;
                entry.2 = Source::Liked;
            }
            None => pool.push((title.clone(), liked_weight, Source::Liked)),
        }
    }
    for door in &card.doors {
        let opens = door.to.iter().any(|tag| towards.iter().any(|t| t == tag));
        match pool.iter_mut().find(|(t, ..)| *t == door.track) {
            // a door that is also a top keeps its place and gains the bonus
            Some(entry) => {
                if opens {
                    entry.1 *= t.door_bonus;
                    entry.2 = Source::Door;
                }
            }
            None => pool.push((
                door.track.clone(),
                if opens { t.door_weight * t.door_bonus } else { t.door_weight },
                Source::Door,
            )),
        }
    }
    // the long tail, scaled by the dial AND by how familiar this artist is
    // (Joel, 14/09/2026): deep cuts for an artist we know, tops for a new
    // one, so lowering comfort widens the artists without drowning in the
    // tail. A brand-new artist (familiarity 0) is led by its tops.
    let familiarity = learned.familiarity01(slug, &card.name);
    let share = comfort.tail_share() * familiarity;
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
            pool.push((track.title.clone(), t.tail_weight * share, Source::Tail));
        }
    }
    pool.retain(|(title, ..)| !played.contains(title) && !learned.track_banned(slug, title));
    for entry in &mut pool {
        // a track often skipped falls back in the draw, it is not banned
        entry.1 /= 1.0 + learned.skipped(slug, &entry.0) as f32;
        // and one that sounded lately steps back too, for a while (0012 §2)
        entry.1 *= learned.freshness(slug, &entry.0);
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
            encore: false,
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
                encore: false,
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

/// Where the journey stands, in the vector space. Not its plain average:
/// an evening drifts, and what was played lately says where we are better
/// than where we started — from Can to M83 by way of Paul McCartney, the
/// flat centroid still sat back at the beginning (Joel, 2026-09-23). Each
/// artist counts half as much as the one `universe_half_life` artists
/// after it. `universe` comes oldest first.
fn drifting_centroid(catalog: &Catalog, universe: &[String]) -> Option<Vec<f32>> {
    let half_life = tuning().universe_half_life;
    let last = universe.len().saturating_sub(1) as f32;
    let known: Vec<(&Vec<f32>, f32)> = universe
        .iter()
        .enumerate()
        .filter_map(|(i, slug)| {
            catalog.vectors.get(slug).map(|v| (v, 0.5f32.powf((last - i as f32) / half_life)))
        })
        .collect();
    let total: f32 = known.iter().map(|(_, weight)| weight).sum();
    let (first, _) = known.first()?;
    if !(total > 0.0) {
        return None;
    }
    let mut centroid = vec![0.0f32; first.len()];
    for (vector, weight) in &known {
        for (c, x) in centroid.iter_mut().zip(vector.iter()) {
            *c += x * weight / total;
        }
    }
    Some(centroid)
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
    let centroid = drifting_centroid(catalog, universe)?;

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
    let mut weighted: Vec<(String, f32)> = pool
        .into_iter()
        .map(|(slug, c)| {
            let fresh = learned.artist_freshness(&slug);
            (slug, (c - 0.5).max(0.05) * fresh)
        })
        .collect();

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
                encore: false,
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
        reason: "stay within the journey's universe".to_string(),
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
    // 0001: comfort *is* familiarity. It does not filter, it leans —
    // towards what we know in the cocoon, towards what we don't wide open.
    // comfort leans by familiarity; artist freshness rotates who leads, so
    // the same faces do not head every branch (Joel, 14/09/2026)
    let favours = |slug: &String| {
        comfort.favours(learned.familiarity01(slug, &catalog.cards[slug].name))
            * learned.artist_freshness(slug)
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
        let mut why = format!("close to the branch's center ({score:.2})");
        if !shared.is_empty() {
            why.push_str(" · shared tags: ");
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

/// The branch of an imposed head — a gap that just got its card (`f<n>`
/// or `fg<n>` on a "○ no card yet" line): the **same walk as a proposed
/// branch**, the fresh card leading, then a track per artist crossed in
/// its neighbourhood. Not an encore of the head: a gap taken used to give
/// n tracks of the one artist, a segment in disguise (Joel, 20/09/2026).
/// `weight` is the link's proximity, so the gauge and the auto draw read
/// it like any branch. None when nothing of it can play.
pub fn branch_from(
    catalog: &Catalog,
    context: &[String],
    head: &str,
    reason: String,
    weight: f32,
    learned: &Learned,
    tail: &Tail,
    comfort: Comfort,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Option<Branch> {
    catalog.cards.get(head)?;
    // at the seed's opening there is no segment yet: the head is its own
    // starting point
    let current = context.last().map(String::as_str).unwrap_or(head);
    let branch =
        walk(catalog, current, head.to_string(), reason, weight, learned, tail, comfort, visited, played, size, rng);
    (!branch.stops.is_empty()).then_some(branch)
}

/// `fw` — wander: leave the universe on purpose (feedback no. 6, Joel,
/// 11/09/2026). With a `target`, the head is that artist, whatever the
/// distance; without, the head is drawn among the artists **farthest**
/// from the journey's centre — outside the journey and its graph
/// neighbourhood, with unplayed tops — leaning by the dial as any head
/// does. Then the same walk as a branch, so the wander has a direction.
pub fn wander(
    catalog: &Catalog,
    context: &[String],
    universe: &[String],
    target: Option<&str>,
    learned: &Learned,
    tail: &Tail,
    comfort: Comfort,
    visited: &HashSet<String>,
    played: &HashSet<String>,
    size: usize,
    rng: &mut impl Rng,
) -> Option<Branch> {
    let current = context.last()?.as_str();
    let (head, reason, weight) = match target {
        Some(slug) => {
            let name = &catalog.cards.get(slug)?.name;
            (slug.to_string(), format!("wander — to {name}, as asked"), 3.0)
        }
        None => {
            // the universe and everything one link away from it: not far
            let nobody = HashSet::new();
            let mut near: HashSet<String> = visited.clone();
            for slug in universe {
                near.insert(slug.clone());
                near.extend(graph_neighbors(catalog, slug, &nobody).into_iter().map(|(s, ..)| s));
            }
            let mut far: Vec<(String, f32)> = vector_neighbors_of(catalog, universe, &near)
                .into_iter()
                .filter(|(slug, _)| catalog.cards[slug].tops.iter().any(|t| !played.contains(t)))
                .collect();
            // far is what the adventurous branch refuses: under the dial's
            // floor. Farthest first — the exact opposite of a branch; if
            // nothing is under the floor, the farthest there is will do
            far.sort_by(|a, b| a.1.total_cmp(&b.1));
            let under: Vec<(String, f32)> =
                far.iter().filter(|(_, score)| *score < comfort.floor()).cloned().collect();
            if !under.is_empty() {
                far = under;
            }
            far.truncate(12);
            let mut pool: Vec<(String, f32)> = far
                .iter()
                .map(|(slug, score)| {
                    let familiarity = learned.familiarity01(slug, &catalog.cards[slug].name);
                    (slug.clone(), (1.0 - score).max(0.05) * comfort.favours(familiarity))
                })
                .collect();
            let slug = draw_weighted(&mut pool, rng)?;
            let score = far.iter().find(|(s, _)| *s == slug).map(|(_, c)| *c).unwrap_or(0.0);
            (slug, format!("wander — far from the journey ({score:.2})"), 1.0)
        }
    };
    let branch = walk(catalog, current, head, reason, weight, learned, tail, comfort, visited, played, size, rng);
    (!branch.stops.is_empty()).then_some(branch)
}

/// The artists of the playlist's last segment, in order: from the last
/// stop that opens one (a branch's head, a `ti` insertion) to the end of
/// the axis — or the whole axis when nothing opens a segment (the seed's
/// opening). Only artists with a card count: the next directions have to
/// start from somewhere the catalog knows. That is what the playlist as
/// it stands says the next directions start from (Joel, 17/09/2026): a
/// `ti`, an `e` in the discography, a moved line, all count.
pub fn segment_of<'a>(axis: impl Iterator<Item = &'a Stop>, known: impl Fn(&str) -> bool) -> Vec<String> {
    let mut segment: Vec<String> = Vec::new();
    for stop in axis {
        if stop.head.is_some() {
            segment.clear();
        }
        if known(&stop.slug) && !segment.contains(&stop.slug) {
            segment.push(stop.slug.clone());
        }
    }
    segment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Door, Link};

    /// `fw` (11/09/2026): without a target the head is the farthest artist
    /// from the journey, outside its graph; with one, it is that artist.
    #[test]
    fn wander_leaves_the_universe_or_goes_where_asked() {
        let mut cure = the_cure();
        cure.links = vec![Link { to: "siouxsie".into(), kind: "similar".into(), note: None, proximity: None }];
        let named = |name: &str| {
            let mut card = the_cure();
            card.name = name.into();
            card.links = Vec::new();
            card.tops = vec![format!("{name} — one")];
            card.doors = Vec::new();
            card
        };
        let catalog = Catalog {
            cards: HashMap::from([
                ("the-cure".to_string(), cure),
                ("siouxsie".to_string(), named("Siouxsie")),
                ("joy-division".to_string(), named("Joy Division")),
                ("fela-kuti".to_string(), named("Fela Kuti")),
            ]),
            proximities: HashMap::from([("similar".to_string(), 4)]),
            vectors: HashMap::from([
                ("the-cure".to_string(), vec![1.0, 0.0]),
                ("siouxsie".to_string(), vec![0.9, 0.1]),
                ("joy-division".to_string(), vec![0.8, 0.2]),
                ("fela-kuti".to_string(), vec![0.0, 1.0]),
            ]),
        };
        let learned = Learned::blank();
        let context = vec!["the-cure".to_string()];
        let visited: HashSet<String> = context.iter().cloned().collect();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        // far: Siouxsie is one link away, so not far; Fela is the farthest
        for _ in 0..10 {
            let branch = wander(&catalog, &context, &context, None, &learned, &no_tail(), Comfort::new(3), &visited, &HashSet::new(), 1, &mut rng)
                .expect("somewhere far");
            assert_eq!(branch.artists[0], "fela-kuti", "{}", branch.label);
            assert!(branch.reason.starts_with("wander — far"), "{}", branch.reason);
        }

        // asked: the head is the artist named, however close
        let branch = wander(&catalog, &context, &context, Some("siouxsie"), &learned, &no_tail(), Comfort::new(3), &visited, &HashSet::new(), 1, &mut rng)
            .expect("where asked");
        assert_eq!(branch.artists[0], "siouxsie");
        assert_eq!(branch.stops[0].artist, "Siouxsie");
    }

    /// A blank tail: the tests that predate it must keep meaning the same
    /// thing, and a cocoon draws none of it anyway.
    fn no_tail() -> Tail {
        Tail::blank()
    }

    fn the_cure() -> Card {
        Card {
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
            begin: None,
            end: None,
            origin: None,
            description: None,
        }
    }

    /// 0016: a link to a missing card is no longer dropped, it proposes
    /// itself. That is what was missing on 09/09/2026 — Brel pointed at
    /// three artists, and the engine saw none of them.
    #[test]
    fn a_link_without_a_card_becomes_a_proposal() {
        let mut brel = the_cure();
        brel.name = "Jacques Brel".into();
        brel.links = vec![
            Link { to: "georges-brassens".into(), kind: "similar".into(), note: None, proximity: None },
            Link { to: "georges-moustaki".into(), kind: "similar".into(), note: None, proximity: None },
        ];
        let mut brassens = the_cure();
        brassens.name = "Georges Brassens".into();
        let cards =
            HashMap::from([("jacques-brel".to_string(), brel), ("georges-brassens".to_string(), brassens)]);
        let catalog = Catalog {
            cards,
            proximities: HashMap::from([("similar".to_string(), 4)]),
            vectors: HashMap::new(),
        };

        // the neighbor that has a card stays a real branch…
        let walkable = graph_neighbors(&catalog, "jacques-brel", &HashSet::new());
        assert_eq!(walkable.len(), 1);
        assert_eq!(walkable[0].0, "georges-brassens");

        // …and the one without becomes a gap, named and weighted
        let missing =
            missing_neighbors(&catalog, &["jacques-brel".to_string()], &HashSet::new());
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].slug, "georges-moustaki");
        assert_eq!(missing[0].name, "Georges Moustaki");
        assert_eq!(missing[0].proximity, 4);
        assert!(missing[0].why.contains("around Jacques Brel"), "{}", missing[0].why);
        assert!(!missing[0].pending);
    }

    /// A gap that got its card is walked like any branch: the head leads,
    /// then its neighbourhood — not n tracks of the head alone.
    #[test]
    fn the_branch_of_a_fresh_head_walks_on() {
        let mut moustaki = the_cure();
        moustaki.name = "Georges Moustaki".into();
        moustaki.tops = vec!["Le Métèque".into()];
        moustaki.links =
            vec![Link { to: "georges-brassens".into(), kind: "similar".into(), note: None, proximity: None }];
        let mut brassens = the_cure();
        brassens.name = "Georges Brassens".into();
        brassens.tops = vec!["Les Copains d'abord".into()];
        let mut brel = the_cure();
        brel.name = "Jacques Brel".into();
        let catalog = Catalog {
            cards: HashMap::from([
                ("georges-moustaki".to_string(), moustaki),
                ("georges-brassens".to_string(), brassens),
                ("jacques-brel".to_string(), brel),
            ]),
            proximities: HashMap::from([("similar".to_string(), 4)]),
            vectors: HashMap::new(),
        };
        let learned = Learned::blank();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let branch = branch_from(
            &catalog,
            &["jacques-brel".to_string()],
            "georges-moustaki",
            "similar · around Jacques Brel".into(),
            4.0,
            &learned,
            &no_tail(),
            Comfort::new(3),
            &HashSet::new(),
            &HashSet::new(),
            3,
            &mut rng,
        )
        .expect("something to play");
        assert_eq!(branch.artists[0], "georges-moustaki");
        assert!(branch.artists.contains(&"georges-brassens".to_string()), "{:?}", branch.artists);
        assert_eq!(branch.stops[0].slug, "georges-moustaki");
        assert!(branch.stops.iter().any(|s| s.slug == "georges-brassens"));
        assert_eq!(branch.weight, 4.0);
        // a head nobody knows gives no branch
        assert!(branch_from(
            &catalog, &[], "nobody", String::new(), 1.0, &learned, &no_tail(), Comfort::new(3),
            &HashSet::new(), &HashSet::new(), 3, &mut rng
        )
        .is_none());
    }

    /// An artist already visited does not come back as a card to generate.
    #[test]
    fn an_excluded_gap_is_not_proposed() {
        let mut brel = the_cure();
        brel.links = vec![Link {
            to: "georges-moustaki".into(),
            kind: "similar".into(),
            note: None,
            proximity: None,
        }];
        let catalog = Catalog {
            cards: HashMap::from([("jacques-brel".to_string(), brel)]),
            proximities: HashMap::new(),
            vectors: HashMap::new(),
        };
        let excluded = HashSet::from(["georges-moustaki".to_string()]);
        assert!(missing_neighbors(&catalog, &["jacques-brel".to_string()], &excluded).is_empty());
    }

    fn weight_of(pool: &[(String, f32, Source)], title: &str) -> f32 {
        pool.iter().find(|(t, ..)| t == title).map(|(_, w, _)| *w).expect(title)
    }

    /// 0012 §1: the top is a weight, not a closed list.
    #[test]
    fn the_pool_accumulates_the_sources() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.like_track("the-cure", "Killing an Arab");
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &[]);

        // the two tops, plus the liked track that is not one
        assert_eq!(pool.len(), 3, "{pool:?}");
        assert_eq!(weight_of(&pool, "Boys Don't Cry"), tuning().top_weight);
        assert_eq!(weight_of(&pool, "Killing an Arab"), liked_weight(Comfort::new(0)));
        let liked = pool.iter().find(|(t, ..)| t == "Killing an Arab").unwrap();
        assert_eq!(liked.2, Source::Liked);
    }

    /// 0018: liking is the one gesture of taste, and it outranks the tops —
    /// a liked top takes the like's weight and its mark, and in the cocoon
    /// a like weighs ten tops.
    #[test]
    fn liked_beats_top() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.like_track("the-cure", "A Forest");
        let cocon = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(5), &HashSet::new(), &[]);
        let forest = cocon.iter().find(|(t, ..)| t == "A Forest").unwrap();
        assert_eq!(forest.2, Source::Liked, "a liked top wears ♥");
        assert!((forest.1 - 10.0 * tuning().top_weight).abs() < 1e-6, "{}", forest.1);
        assert_eq!(weight_of(&cocon, "Boys Don't Cry"), tuning().top_weight);
        // wide open, a like still weighs two tops: never less
        let ouvert = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &[]);
        assert!((weight_of(&ouvert, "A Forest") - 2.0 * tuning().top_weight).abs() < 1e-6);
    }

    /// 0011: a door is an additional criterion, never the main one — its
    /// bonus only lands when the direction overlaps its tags.
    #[test]
    fn a_door_only_wins_toward_its_direction() {
        let card = the_cure();
        let learned = Learned::blank();

        let ailleurs = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &["rap".into()]);
        assert_eq!(weight_of(&ailleurs, "A Forest"), tuning().top_weight, "aucune direction commune");

        let vers = reservoir(
            &card,
            "the-cure",
            &learned,
            &no_tail(),
            Comfort::new(0),
            &HashSet::new(),
            &["post-punk".into()],
        );
        assert_eq!(weight_of(&vers, "A Forest"), tuning().top_weight * tuning().door_bonus);
        let door = vers.iter().find(|(t, ..)| t == "A Forest").unwrap();
        assert_eq!(door.2, Source::Door, "the source must show on screen");
        // and the bonus stays local: the other top does not move
        assert_eq!(weight_of(&vers, "Boys Don't Cry"), tuning().top_weight);
    }

    #[test]
    fn a_banned_one_leaves_and_a_skipped_one_steps_back() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.ban_track("the-cure", "Boys Don't Cry");
        learned.skip_track("the-cure", "A Forest");
        learned.skip_track("the-cure", "A Forest");
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &HashSet::new(), &[]);

        assert!(!pool.iter().any(|(t, ..)| t == "Boys Don't Cry"), "banned: out of the draw");
        // two skips: the weight is divided by three, never down to zero
        assert!((weight_of(&pool, "A Forest") - tuning().top_weight / 3.0).abs() < 1e-6);
    }

    /// 0012 §2: a track played yesterday steps back, even liked — that is
    /// the cooldown, and it does not exclude it.
    #[test]
    fn a_recently_played_one_steps_back() {
        let card = the_cure();
        let mut learned = Learned::blank();
        learned.like_track("the-cure", "A Forest");
        learned.played("the-cure", "A Forest");
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(3), &HashSet::new(), &[]);
        let forest = weight_of(&pool, "A Forest");
        let liked = liked_weight(Comfort::new(3));
        assert!(forest < liked / 5.0, "played today: {forest} against {liked} fresh");
        assert!(forest > 0.0, "stepped back, not excluded");
        assert_eq!(weight_of(&pool, "Boys Don't Cry"), tuning().top_weight, "never played: intact");
    }

    /// 0001: comfort *is* familiarity — and the scale's polarity is the
    /// trap of that decision (see the doc of `Comfort`).
    #[test]
    fn the_cocoon_leans_to_the_known_and_exploration_to_the_unknown() {
        // 5 = cocoon, 0 = exploration (turned round on 06/09/2026)
        let cocon = Comfort::new(5);
        let milieu = Comfort::new(3);
        let ouvert = Comfort::new(0);

        // in the cocoon, a familiar artist goes ahead of an unknown one
        assert!(cocon.favours(1.0) > cocon.favours(0.0));
        // wide open, the reverse — otherwise the dial is useless
        assert!(ouvert.favours(0.0) > ouvert.favours(1.0));
        // and never an exclusion: we discourage, we don't forbid
        assert!(cocon.favours(0.0) >= 0.25 && ouvert.favours(1.0) >= 0.25);

        // the adventurous floor drops as the dial opens
        assert!(cocon.floor() > milieu.floor());
        assert!(milieu.floor() > ouvert.floor());
        // comfort 3 reproduces the fixed tuning from before the dial
        let trois_ = Comfort::new(3);
        assert!((trois_.floor() - 0.72).abs() < 1e-6, "{}", trois_.floor());
        assert!((trois_.trust() - 0.796).abs() < 1e-3, "{}", trois_.trust());

        // six integer values: there is no exact middle. 3 still leans
        // towards the known, 2 already towards the unknown — the tipping
        // point falls between them, and that is a property, not a flaw.
        assert!(milieu.favours(1.0) > milieu.favours(0.0));
        let deux = Comfort::new(2);
        assert!(deux.favours(0.0) > deux.favours(1.0));
        // and the value is clamped
        assert_eq!(Comfort::new(9).value(), 5);
    }

    /// 0012 §4: comfort sets the depth of the draw. In the cocoon the tail
    /// weighs nothing; wide open, it takes over.
    #[test]
    fn the_tail_weighs_by_comfort_and_by_familiarity() {
        let card = the_cure();
        // a familiar artist: the tail is for the ones we know (Joel, 14/09/2026)
        let mut learned = Learned::blank();
        for _ in 0..10 {
            learned.played("the-cure", "warmup");
        }
        let mut tail = Tail::blank();
        tail.keep(
            "the-cure",
            vec![
                crate::discography::TailTrack {
                    title: "Killing an Arab".into(),
                    uri: "spotify:track:x".into(),
                    album: "Three Imaginary Boys".into(),
                    ..Default::default()
                },
                // the same song, remastered: it must not count twice
                crate::discography::TailTrack {
                    title: "Killing an Arab - 2004 Remaster".into(),
                    uri: "spotify:track:y".into(),
                    album: "Boys Don't Cry".into(),
                    ..Default::default()
                },
                // and a track already in the tops does not enter the tail
                crate::discography::TailTrack {
                    title: "A Forest (Remastered)".into(),
                    uri: "spotify:track:z".into(),
                    album: "Seventeen Seconds".into(),
                    ..Default::default()
                },
            ],
        );

        // 5 = cocoon since 06/09/2026
        let cocon = reservoir(&card, "the-cure", &learned, &tail, Comfort::new(5), &HashSet::new(), &[]);
        assert!(!cocon.iter().any(|(_, _, s)| *s == Source::Tail), "in the cocoon, no tail");
        assert_eq!(cocon.len(), 2, "the two tops, nothing else");

        let ouvert = reservoir(&card, "the-cure", &learned, &tail, Comfort::new(0), &HashSet::new(), &[]);
        let traine: Vec<&(String, f32, Source)> =
            ouvert.iter().filter(|(_, _, s)| *s == Source::Tail).collect();
        assert_eq!(traine.len(), 1, "Killing an Arab only once: {ouvert:?}");
        assert_eq!(traine[0].0, "Killing an Arab");
        assert!(traine[0].1 > 0.0 && traine[0].1 < tuning().top_weight, "less than a top, but present");

        // a new artist (familiarity 0) is led by its tops, even wide open:
        // the tail is depth for the ones we know (Joel, 14/09/2026)
        let stranger = Learned::blank();
        let unknown = reservoir(&card, "the-cure", &stranger, &tail, Comfort::new(0), &HashSet::new(), &[]);
        assert!(
            !unknown.iter().any(|(_, _, s)| *s == Source::Tail),
            "a new artist: tops only, no tail, even open: {unknown:?}"
        );
    }

    /// 2026-09-23: a source is fixed when a track is drawn, so the glyph
    /// has to be recomputed after a like — and a door keeps its own.
    #[test]
    fn the_glyph_follows_the_like_but_leaves_a_door_alone() {
        let catalog = Catalog {
            cards: HashMap::from([("the-cure".to_string(), the_cure())]),
            proximities: HashMap::new(),
            vectors: HashMap::new(),
        };
        let mut tail = Tail::blank();
        tail.keep(
            "the-cure",
            vec![crate::discography::TailTrack {
                title: "Killing an Arab".into(),
                ..Default::default()
            }],
        );
        let stop = |title: &str, source: Source| Stop {
            slug: "the-cure".into(),
            artist: "The Cure".into(),
            title: title.into(),
            source,
            head: None,
            encore: false,
        };
        let mut learned = Learned::blank();
        let mark = |learned: &Learned, s: &Stop| current_source(&catalog, &tail, learned, s);

        // drawn as a top, and a top it stays
        assert_eq!(mark(&learned, &stop("Boys Don't Cry", Source::Top)), Source::Top);
        // a tail track keeps its dot…
        assert_eq!(mark(&learned, &stop("Killing an Arab", Source::Tail)), Source::Tail);
        // …until it is liked, whatever it was drawn as
        learned.like_track("the-cure", "Killing an Arab");
        assert_eq!(mark(&learned, &stop("Killing an Arab", Source::Tail)), Source::Liked);
        // a door says where it leads: the like does not take that away
        learned.like_track("the-cure", "A Forest");
        assert_eq!(mark(&learned, &stop("A Forest", Source::Door)), Source::Liked);
        // and an unliked door keeps its arrow rather than falling back to top
        assert_eq!(mark(&learned, &stop("Boys Don't Cry", Source::Door)), Source::Door);
        // nothing known of the artist: off the map
        let orphan = Stop { slug: "nobody".into(), ..stop("x", Source::Top) };
        assert_eq!(mark(&learned, &orphan), Source::Offmap);
    }

    /// 2026-09-23: an evening drifts. "Stay in the universe" follows what
    /// was played lately rather than the plain average of the whole thing.
    #[test]
    fn the_universe_centre_follows_the_drift() {
        // two clusters at right angles: the start, then where we ended up
        let catalog = Catalog {
            cards: HashMap::new(),
            proximities: HashMap::new(),
            vectors: HashMap::from([
                ("can".to_string(), vec![1.0, 0.0]),
                ("lou-reed".to_string(), vec![1.0, 0.0]),
                ("the-beatles".to_string(), vec![1.0, 0.0]),
                ("metronomy".to_string(), vec![0.0, 1.0]),
                ("m83".to_string(), vec![0.0, 1.0]),
                ("the-xx".to_string(), vec![0.0, 1.0]),
            ]),
        };
        // three artists at the start, three where we ended up: two branches
        let universe: Vec<String> = [
            "can", "lou-reed", "the-beatles", "metronomy", "m83", "the-xx",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let centre = drifting_centroid(&catalog, &universe).expect("a centre");

        // the plain average would sit halfway, three against three, and say
        // nothing of where the evening went
        assert!(centre[1] > centre[0], "the recent half leads: {centre:?}");
        assert!(centre[0] > 0.0, "and the journey still counts for something");

        // the order is what matters, not the names: reversed, so is the centre
        let reversed: Vec<String> = universe.iter().rev().cloned().collect();
        let back = drifting_centroid(&catalog, &reversed).expect("a centre");
        assert!(back[0] > back[1], "played last, Can's side leads instead: {back:?}");
    }

    /// 2026-09-25: a gesture aims at the row under the needle (0020), and
    /// a track with no card of its own still finds one under its artist's
    /// name — "Ye" is filed as `kanye-west`.
    #[test]
    fn a_stop_finds_its_card_by_name_when_it_has_no_slug() {
        let mut ye = the_cure();
        ye.name = "Ye".into();
        let cards = HashMap::from([("kanye-west".to_string(), ye)]);
        let named = |slug: &str, artist: &str| Stop {
            slug: slug.into(),
            artist: artist.into(),
            title: "Flashing Lights".into(),
            source: Source::Offmap,
            head: None,
            encore: false,
        };
        // its own slug when it has one
        assert_eq!(card_of(&cards, &named("kanye-west", "Kanye West")).as_deref(), Some("kanye-west"));
        // inserted from Spotify, no slug: the name resolves it, even though
        // the card is called something else
        assert_eq!(card_of(&cards, &named("", "Kanye West")).as_deref(), Some("kanye-west"));
        // and nobody's card is nobody's: the gesture must say so rather
        // than quietly aim at someone else
        assert_eq!(card_of(&cards, &named("", "Nobody At All")), None);
    }

    /// What a journey already played does not come back (0012 §3).
    #[test]
    fn the_already_played_does_not_come_back() {
        let card = the_cure();
        let learned = Learned::blank();
        let played: HashSet<String> = ["A Forest".to_string()].into_iter().collect();
        let pool = reservoir(&card, "the-cure", &learned, &no_tail(), Comfort::new(0), &played, &[]);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].0, "Boys Don't Cry");
    }

    /// The playlist as it stands (17/09/2026): the last segment starts at
    /// the last head; what `ti` or the discography's `e` queued counts,
    /// an off-catalog stop does not.
    #[test]
    fn the_last_segment_of_the_playlist() {
        let stop = |slug: &str, head: bool| Stop {
            slug: slug.into(),
            artist: slug.into(),
            title: format!("{slug} — one"),
            source: Source::Top,
            head: head.then(|| Head { label: slug.into(), reason: String::new() }),
            encore: false,
        };
        let known = |slug: &str| slug != "nobody";
        // the seed's opening: no head at all, the whole axis is the segment
        let opening = [stop("the-cure", false), stop("the-cure", false), stop("siouxsie", false)];
        assert_eq!(segment_of(opening.iter(), known), vec!["the-cure", "siouxsie"]);
        // a branch, then an `e` from the discography and a `ti`: the
        // insertion opens the last segment, the off-catalog track is skipped
        let axis = [
            stop("the-cure", false),
            stop("siouxsie", true),
            stop("joy-division", false),
            stop("bauhaus", false),
            stop("nobody", true),
            stop("bauhaus", false),
        ];
        assert_eq!(segment_of(axis.iter(), known), vec!["bauhaus"]);
        let axis = [stop("the-cure", false), stop("siouxsie", true), stop("joy-division", false)];
        assert_eq!(segment_of(axis.iter(), known), vec!["siouxsie", "joy-division"]);
    }
}
