//! The learned layer (0014): what listening teaches, kept beside the
//! catalogue in `learned/`, one TOML per artist, never pushed upstream.
//!
//! Counters carry their own decay instead of a history: on each play,
//! `plays = plays × ½^((today − last)/half-life) + 1`. A float and a date
//! per artist and per top are enough, and a play from three years ago
//! weighs almost nothing. Half-life: six months.
//!
//! Every gesture here is a **measure** (0013): it writes silently, without
//! confirmation, and never enters a commit of the catalogue proper.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Plays at which familiarity reaches half — beyond, it saturates.
const PLAYS_REFERENCE: f64 = 5.0;

/// Six months, in days (0014).
const HALF_LIFE: f64 = 182.5;

/// The cooldown (0012 §2): a track played today keeps this share of its
/// weight, and gets it back with a one-week half-life — a week later 55 %,
/// two weeks 78 %, a month 94 %. Both are "to be tuned as the PoC goes".
const COOLDOWN_FLOOR: f32 = 0.1;
const COOLDOWN_HALF_LIFE: f32 = 7.0;
/// The artist-level cooldown (Joel, 14/09/2026): an artist heard lately
/// steps back as a branch head and recovers over a few days, so a large
/// library stops circling the same faces. Longer floor than a track's — a
/// face returns less readily than one of its songs.
const ARTIST_COOLDOWN_HALF_LIFE: f32 = 4.0;
const ARTIST_COOLDOWN_FLOOR: f32 = 0.3;

/// "less often" multiplies the weight by this, down to the floor.
const LESS_OFTEN: f32 = 0.7;
const WEIGHT_FLOOR: f32 = 0.1;
/// "more often" is its mirror, capped so one key cannot run away.
const MORE_OFTEN: f32 = 1.0 / LESS_OFTEN;
const WEIGHT_CEILING: f32 = 3.0;

fn one() -> f32 {
    1.0
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Top {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub plays: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub liked: bool,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub skipped: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub blacklisted: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Artist {
    #[serde(default)]
    pub plays: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    #[serde(default = "one")]
    pub weight: f32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub blacklisted: bool,
    /// "Not one of my liked" — set by `as`, cleared by `al`. Explicit,
    /// because the Spotify seed says otherwise and would come back with
    /// the next harvest (a son's likes, Joel, 09/09/2026); the weight alone
    /// would not do, it climbs back.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unliked: bool,
    /// Sorted on disk: a stable order keeps diffs honest between machines.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tops: BTreeMap<String, Top>,
}

impl Default for Artist {
    fn default() -> Self {
        Artist { plays: 0.0, last: None, weight: 1.0, blacklisted: false, unliked: false, tops: BTreeMap::new() }
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_zero(f: &f64) -> bool {
    *f == 0.0
}
fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

pub struct Learned {
    root: PathBuf,
    artists: HashMap<String, Artist>,
    /// The best seed score, to bring the ranking onto the same 0–1 scale as
    /// our own plays — they are counts, it is a composite score.
    seed_max: f64,
    /// The name as written in the ranking — the key is lowercased to
    /// compare, but the collection shows it as is.
    seed_names: HashMap<String, String>,
    /// `classement.json`: the familiarity an artist starts with, before any
    /// listening of our own (0014). Keyed by slug of the name — the seed file
    /// predates slugs.
    seed: HashMap<String, f64>,
    /// Who the Spotify account already likes — a liked track or album, or
    /// a followed artist — by lowercase name, from the same seed file.
    seed_liked: HashSet<String>,
    today: i64,
}

impl Learned {
    /// Read what is on disk. A missing `learned/` is not an error: it is a
    /// catalogue nobody has listened to yet.
    pub fn load(catalog_dir: &Path) -> Learned {
        let root = catalog_dir.join("learned");
        let mut artists = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(root.join("artists")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                match std::fs::read_to_string(&path).map(|t| toml::from_str::<Artist>(&t)) {
                    Ok(Ok(artist)) => {
                        artists.insert(slug.to_string(), artist);
                    }
                    Ok(Err(e)) => eprintln!("learned unreadable ({}): {e}", path.display()),
                    Err(_) => {}
                }
            }
        }

        let mut seed = HashMap::new();
        let mut seed_names: HashMap<String, String> = HashMap::new();
        let mut seed_liked = HashSet::new();
        if let Ok(text) = std::fs::read_to_string(root.join("classement.json")) {
            if let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                for row in rows {
                    // the seed file still has its French keys (0014: to be
                    // translated along with the tooling scripts)
                    if let (Some(name), Some(score)) =
                        (row["nom"].as_str(), row["score"].as_f64())
                    {
                        // keyed by slug, so that a card whose name moved on
                        // ("Ye", at `kanye-west`) still meets the seed
                        // row Spotify keeps under the old name (Joel,
                        // 10/09/2026)
                        let key = crate::generate::slugify(name);
                        seed.insert(key.clone(), score);
                        seed_names.insert(key.clone(), name.to_string());
                        let liked = row["titres_aimes"].as_u64().unwrap_or(0) > 0
                            || row["albums_aimes"].as_u64().unwrap_or(0) > 0
                            || row["suivi"].as_bool().unwrap_or(false);
                        if liked {
                            seed_liked.insert(key);
                        }
                    }
                }
            }
        }

        let seed_max = seed.values().copied().fold(1.0, f64::max);
        Learned { root, artists, seed, seed_max, seed_names, seed_liked, today: today() }
    }

    pub fn known(&self) -> usize {
        self.artists.len()
    }

    /// Liked, here or on Spotify: the artist itself ("more often",
    /// a follow), or one of its tracks (♥ here, a liked track or album
    /// there). What the home's collection shows by default (Joel,
    /// 09/09/2026).
    pub fn liked(&self, slug: &str, name: &str) -> bool {
        let artist = self.artists.get(slug);
        if artist.is_some_and(|a| a.unliked || a.blacklisted) {
            return false;
        }
        let here = artist.is_some_and(|a| a.weight > 1.0 || a.tops.values().any(|t| t.liked));
        here || self.seed_liked.contains(slug) || self.seed_liked.contains(&crate::generate::slugify(name))
    }

    pub fn seeded(&self) -> usize {
        self.seed.len()
    }

    /// Familiarity on a 0–1 scale, which is what the comfort dial needs
    /// (0001: comfort *is* familiarity). Our own plays saturate — the tenth
    /// listen says much less than the first — and the seed ranking is
    /// brought onto the same scale by its own maximum, since one is a count
    /// and the other a composite score.
    pub fn familiarity01(&self, slug: &str, name: &str) -> f32 {
        // by the card's slug or by the name as shown: either may be the one
        // the seed knows
        let seeded = self
            .seed
            .get(slug)
            .or_else(|| self.seed.get(&crate::generate::slugify(name)))
            .map_or(0.0, |s| s / self.seed_max);
        let ours = self.artists.get(slug).map_or(0.0, |artist| {
            let plays = decay(artist.plays, artist.last.as_deref(), self.today);
            1.0 - 0.5f64.powf(plays / PLAYS_REFERENCE)
        });
        // the stronger of the two, never the latest known: a first play
        // must not *replace* a whole library. Without this, playing one's
        // favourite artist once dropped it from 100 % to 13 % (seen on the
        // home, 06/09/2026).
        seeded.max(ours).clamp(0.0, 1.0) as f32
    }

    /// Artists we used to play and no longer do — the count stored at the
    /// last listen was high, and that listen is old. 0014's decay is what
    /// makes this readable: "you loved them, you no longer play them".
    /// Returns (slug, months since the last listen), oldest neglect first.
    pub fn neglected(&self, min_plays: f64, min_days: i64) -> Vec<(String, i64)> {
        let mut out: Vec<(String, i64)> = self
            .artists
            .iter()
            .filter(|(_, a)| a.plays >= min_plays && !a.blacklisted)
            .filter_map(|(slug, a)| {
                let days = self.today - from_iso(a.last.as_deref()?)?;
                (days >= min_days).then(|| (slug.clone(), days / 30))
            })
            .collect();
        out.sort_by_key(|(_, months)| std::cmp::Reverse(*months));
        out
    }

    /// The tracks this listener liked anywhere — a seed that is a track
    /// rather than an artist (ruling of 05/09: the seed can be either).
    pub fn liked_anywhere(&self) -> Vec<(String, String)> {
        self.artists
            .iter()
            .flat_map(|(slug, a)| {
                a.tops
                    .iter()
                    .filter(|(_, t)| t.liked && !t.blacklisted)
                    .map(move |(title, _)| (slug.clone(), title.clone()))
            })
            .collect()
    }

    pub fn artist_is_banned(&self, slug: &str) -> bool {
        self.artists.get(slug).is_some_and(|a| a.blacklisted)
    }

    /// Every name in the ranking, as written there.
    pub fn ranked_names(&self) -> impl Iterator<Item = &String> {
        self.seed_names.values()
    }

    /// How many days since this artist last played? `None` if it never
    /// played at all.
    pub fn days_since(&self, slug: &str) -> Option<i64> {
        let artist = self.artists.get(slug)?;
        Some((self.today - from_iso(artist.last.as_deref()?)?).max(0))
    }

    /// What the listening knows of one track: plays (decayed to today),
    /// days since it last sounded, and how often it was skipped. `None`
    /// when it never sounded here.
    pub fn track_stats(&self, slug: &str, title: &str) -> Option<(f64, Option<i64>, u32)> {
        let top = self.artists.get(slug)?.tops.get(title)?;
        let plays = decay(top.plays, top.last.as_deref(), self.today);
        let days = top.last.as_deref().and_then(from_iso).map(|d| (self.today - d).max(0));
        Some((plays, days, top.skipped))
    }

    /// The cooldown of one track (0012 §2): 1.0 when it never sounded here,
    /// `COOLDOWN_FLOOR` the day it did, and back up with a one-week
    /// half-life. The reservoir multiplies its weight by this.
    pub fn freshness(&self, slug: &str, title: &str) -> f32 {
        let Some(days) = self.track_stats(slug, title).and_then(|(_, days, _)| days) else {
            return 1.0;
        };
        let recovered = 1.0 - 0.5f32.powf(days as f32 / COOLDOWN_HALF_LIFE);
        COOLDOWN_FLOOR + (1.0 - COOLDOWN_FLOOR) * recovered
    }

    /// How ready an artist is to head a branch again, by how long since we
    /// last heard them (Joel, 14/09/2026): the artist counterpart of track
    /// freshness (0012 §2). 1.0 for one not heard recently (or ever); down
    /// to a floor for one heard today, recovering over a few days. It
    /// discourages, never forbids — a branch is never shut.
    pub fn artist_freshness(&self, slug: &str) -> f32 {
        let Some(days) = self.days_since(slug) else { return 1.0 };
        let recovered = 1.0 - 0.5f32.powf(days as f32 / ARTIST_COOLDOWN_HALF_LIFE);
        ARTIST_COOLDOWN_FLOOR + (1.0 - ARTIST_COOLDOWN_FLOOR) * recovered
    }

    /// What listening knows of **every** track of an artist: title as it
    /// was played, plays decayed to today, days since the last one, liked,
    /// banned. The discography screen then matches by normalized title —
    /// Spotify and the cards do not always spell them the same.
    pub fn track_table(&self, slug: &str) -> Vec<(String, f64, Option<i64>, bool, bool)> {
        let Some(artist) = self.artists.get(slug) else { return Vec::new() };
        artist
            .tops
            .iter()
            .map(|(title, top)| {
                (
                    title.clone(),
                    decay(top.plays, top.last.as_deref(), self.today),
                    top.last.as_deref().and_then(from_iso).map(|d| (self.today - d).max(0)),
                    top.liked,
                    top.blacklisted,
                )
            })
            .collect()
    }

    pub fn track_banned(&self, slug: &str, title: &str) -> bool {
        self.artists
            .get(slug)
            .and_then(|a| a.tops.get(title))
            .is_some_and(|t| t.blacklisted)
    }

    /// The tracks this listener has liked at an artist — they join the
    /// reservoir beside the tops (0012 §1).
    pub fn liked_tracks(&self, slug: &str) -> Vec<&String> {
        self.artists
            .get(slug)
            .map(|a| a.tops.iter().filter(|(_, t)| t.liked).map(|(title, _)| title).collect())
            .unwrap_or_default()
    }

    /// How many times this track was skipped: it pushes it back in the draw.
    pub fn skipped(&self, slug: &str, title: &str) -> u32 {
        self.artists.get(slug).and_then(|a| a.tops.get(title)).map_or(0, |t| t.skipped)
    }

    /// Every banned track of every artist, as the engine excludes by title.
    pub fn banned_tracks(&self) -> impl Iterator<Item = &String> {
        self.artists
            .values()
            .flat_map(|a| a.tops.iter().filter(|(_, t)| t.blacklisted).map(|(title, _)| title))
    }

    pub fn banned_artists(&self) -> impl Iterator<Item = &String> {
        self.artists.iter().filter(|(_, a)| a.blacklisted).map(|(slug, _)| slug)
    }

    pub fn weight(&self, slug: &str) -> f32 {
        self.artists.get(slug).map_or(1.0, |a| a.weight)
    }

    // --- measures ---------------------------------------------------------

    /// A track played through to the end: the only thing that counts as a
    /// listen (a skip is a skip, and says something else).
    pub fn played(&mut self, slug: &str, title: &str) {
        let today = self.today;
        let artist = self.entry(slug);
        artist.plays = decay(artist.plays, artist.last.as_deref(), today) + 1.0;
        artist.last = Some(iso(today));
        let top = artist.tops.entry(title.to_string()).or_default();
        top.plays = decay(top.plays, top.last.as_deref(), today) + 1.0;
        top.last = Some(iso(today));
        self.save(slug);
    }

    /// "More often" — the one gesture of taste (0018). It also forgives
    /// the skips: the opposite gesture cancels the previous one.
    pub fn like_track(&mut self, slug: &str, title: &str) {
        let top = self.entry(slug).tops.entry(title.to_string()).or_default();
        top.liked = true;
        top.skipped = 0;
        self.save(slug);
    }

    /// Is this track one of the listener's liked ones? For the `tl` toggle
    /// (Joel, 14/09/2026).
    pub fn track_liked(&self, slug: &str, title: &str) -> bool {
        self.liked_tracks(slug).iter().any(|t| *t == title)
    }

    /// Take the like back, and only that — no penalty, unlike `ts` which
    /// also pushes the track down (Joel, 14/09/2026).
    pub fn unlike_track(&mut self, slug: &str, title: &str) {
        if let Some(artist) = self.artists.get_mut(slug) {
            if let Some(top) = artist.tops.get_mut(title) {
                top.liked = false;
                self.save(slug);
            }
        }
    }

    /// "Less often" — this one does not interest me. Each skip pushes
    /// the track further back, and takes the like away.
    pub fn skip_track(&mut self, slug: &str, title: &str) {
        let top = self.entry(slug).tops.entry(title.to_string()).or_default();
        top.skipped += 1;
        top.liked = false;
        self.save(slug);
    }

    pub fn ban_track(&mut self, slug: &str, title: &str) {
        self.entry(slug).tops.entry(title.to_string()).or_default().blacklisted = true;
        self.save(slug);
    }

    pub fn like_artist(&mut self, slug: &str) -> f32 {
        let artist = self.entry(slug);
        artist.weight = (artist.weight * MORE_OFTEN).min(WEIGHT_CEILING);
        artist.unliked = false;
        let weight = artist.weight;
        self.save(slug);
        weight
    }

    pub fn skip_artist(&mut self, slug: &str) -> f32 {
        let artist = self.entry(slug);
        artist.weight = (artist.weight * LESS_OFTEN).max(WEIGHT_FLOOR);
        artist.unliked = true;
        let weight = artist.weight;
        self.save(slug);
        weight
    }

    pub fn ban_artist(&mut self, slug: &str) {
        self.entry(slug).blacklisted = true;
        self.save(slug);
    }

    /// A harvest to sort later (0014): transverse to artists, so it lives
    /// on its own, appended to.
    pub fn mark(&self, artist: &str, title: &str) -> std::io::Result<()> {
        let dir = self.root.join("marks");
        std::fs::create_dir_all(&dir)?;
        let file = dir.join("inbox.toml");
        let mut text = std::fs::read_to_string(&file).unwrap_or_default();
        text.push_str(&format!(
            "[[mark]]\nartist = {}\ntitle = {}\nat = {}\n\n",
            quote(artist),
            quote(title),
            quote(&iso(self.today))
        ));
        std::fs::write(file, text)
    }

    fn entry(&mut self, slug: &str) -> &mut Artist {
        self.artists.entry(slug.to_string()).or_default()
    }

    /// One file per artist, written on every measure: they are tiny, and a
    /// crash must never cost what the ear just said.
    fn save(&self, slug: &str) {
        let Some(artist) = self.artists.get(slug) else {
            return;
        };
        let dir = self.root.join("artists");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("learned not saved ({e})");
            return;
        }
        match toml::to_string_pretty(artist) {
            Ok(text) => {
                if let Err(e) = std::fs::write(dir.join(format!("{slug}.toml")), text) {
                    eprintln!("learned not saved ({e})");
                }
            }
            Err(e) => eprintln!("learned not serializable ({e})"),
        }
    }
}

// --- merging what two machines learned (0017) --------------------------

/// Three-way merge of one artist's file: what each side counted since the
/// common ancestor adds up (decayed to today), a ban on either side holds,
/// a weight follows the side that moved it, a top new on one side comes in
/// as it is. A textual merge would be meaningless on decayed floats; this
/// is what git calls through the `learned` merge driver.
pub fn merge_artist(base: Option<&str>, ours: &str, theirs: &str) -> Result<String, String> {
    merge_artist_at(base, ours, theirs, today())
}

fn merge_artist_at(base: Option<&str>, ours: &str, theirs: &str, today: i64) -> Result<String, String> {
    let parse = |text: &str| toml::from_str::<Artist>(text).map_err(|e| e.to_string());
    let base = match base {
        Some(text) => parse(text)?,
        None => Artist::default(),
    };
    let a = parse(ours)?;
    let b = parse(theirs)?;

    let (plays, last) = merge_count(
        (base.plays, base.last.as_deref()),
        (a.plays, a.last.as_deref()),
        (b.plays, b.last.as_deref()),
        today,
    );
    let weight = if (a.weight - base.weight).abs() > 1e-6 { a.weight } else { b.weight };
    let blacklisted = merge_flag(base.blacklisted, a.blacklisted, b.blacklisted);
    let unliked = merge_flag(base.unliked, a.unliked, b.unliked);

    let mut keys: Vec<&String> = base.tops.keys().chain(a.tops.keys()).chain(b.tops.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut tops = BTreeMap::new();
    for key in keys {
        let bt = base.tops.get(key).cloned().unwrap_or_default();
        // a side that never saw this top left it as the base had it
        let at = a.tops.get(key).cloned().unwrap_or_else(|| bt.clone());
        let tt = b.tops.get(key).cloned().unwrap_or_else(|| bt.clone());
        let (plays, last) = merge_count(
            (bt.plays, bt.last.as_deref()),
            (at.plays, at.last.as_deref()),
            (tt.plays, tt.last.as_deref()),
            today,
        );
        tops.insert(
            key.clone(),
            Top {
                plays,
                last,
                liked: merge_flag(bt.liked, at.liked, tt.liked),
                skipped: (at.skipped + tt.skipped).saturating_sub(bt.skipped),
                blacklisted: merge_flag(bt.blacklisted, at.blacklisted, tt.blacklisted),
            },
        );
    }
    toml::to_string_pretty(&Artist { plays, last, weight, blacklisted, unliked, tops })
        .map_err(|e| e.to_string())
}

/// A counter and its date: the side that did not move yields to the other;
/// when both moved, what each added since the base adds up — all three
/// decayed to today first, so the half-life is applied once.
fn merge_count(
    base: (f64, Option<&str>),
    a: (f64, Option<&str>),
    b: (f64, Option<&str>),
    today: i64,
) -> (f64, Option<String>) {
    let same = |x: (f64, Option<&str>), y: (f64, Option<&str>)| (x.0 - y.0).abs() < 1e-9 && x.1 == y.1;
    if same(a, base) {
        return (b.0, b.1.map(str::to_string));
    }
    if same(b, base) {
        return (a.0, a.1.map(str::to_string));
    }
    let plays = (decay(a.0, a.1, today) + decay(b.0, b.1, today) - decay(base.0, base.1, today)).max(0.0);
    // ISO dates compare as strings; None sorts first
    let last = a.1.max(b.1).map(str::to_string);
    (plays, last)
}

/// A flag set on either side since the base holds: a ban or a like is a
/// decision, and two machines cannot un-decide each other silently.
fn merge_flag(base: bool, a: bool, b: bool) -> bool {
    if a != base {
        a
    } else if b != base {
        b
    } else {
        base
    }
}

/// `plays` as it stands today, the half-life applied to the gap.
fn decay(plays: f64, last: Option<&str>, today: i64) -> f64 {
    match last.and_then(from_iso) {
        Some(day) => plays * 0.5f64.powf((today - day).max(0) as f64 / HALF_LIFE),
        None => plays,
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn today() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    secs / 86_400
}

/// Today's date, in ISO. An edit carries it in its note: that is what
/// later tells where a line of a card came from.
pub fn today_iso() -> String {
    iso(today())
}

/// Days since the epoch → `YYYY-MM-DD`, and back. Hinnant's civil-date
/// algorithm: no dependency for what is two dozen lines of arithmetic.
fn iso(days: i64) -> String {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn from_iso(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

#[cfg(test)]
mod taste_tests {
    use super::*;

    /// 14/09/2026: an artist heard today steps back as a branch head, one
    /// never heard is fully ready, and the step-back fades over days.
    #[test]
    fn artist_freshness_steps_back_the_recent_and_recovers() {
        let mut learned = Learned::blank();
        assert_eq!(learned.artist_freshness("the-cure"), 1.0, "never heard: ready");
        learned.played("the-cure", "A Forest");
        let today = learned.artist_freshness("the-cure");
        assert!((today - ARTIST_COOLDOWN_FLOOR).abs() < 1e-6, "heard today: floor");
        // a few days on, more than half recovered
        learned.artists.get_mut("the-cure").unwrap().last = Some(iso(learned.today - 8));
        assert!(learned.artist_freshness("the-cure") > 0.7, "a week later: mostly back");
    }

    /// 0012 §2: played today, a track keeps only a tenth of its weight;
    /// never played, it keeps it whole; a week later, it has got more than
    /// half of it back.
    #[test]
    fn the_cooldown_penalizes_the_recent_and_fades_with_time() {
        let mut learned = Learned::blank();
        assert_eq!(learned.freshness("the-cure", "A Forest"), 1.0);
        learned.played("the-cure", "A Forest");
        assert!((learned.freshness("the-cure", "A Forest") - COOLDOWN_FLOOR).abs() < 1e-6);
        // a week later
        learned.today += 7;
        let week = learned.freshness("the-cure", "A Forest");
        assert!((week - 0.55).abs() < 0.01, "{week}");
        learned.today += 23;
        assert!(learned.freshness("the-cure", "A Forest") > 0.9);
    }

    /// 0018: liking and skipping are the two gestures of taste, and one
    /// undoes the other.
    #[test]
    fn the_opposite_gesture_cancels_the_previous_one() {
        let mut learned = Learned::blank();
        learned.skip_track("the-cure", "A Forest");
        learned.skip_track("the-cure", "A Forest");
        assert_eq!(learned.skipped("the-cure", "A Forest"), 2);
        learned.like_track("the-cure", "A Forest");
        assert_eq!(learned.skipped("the-cure", "A Forest"), 0);
        assert_eq!(learned.liked_tracks("the-cure"), vec!["A Forest"]);
        learned.skip_track("the-cure", "A Forest");
        assert!(learned.liked_tracks("the-cure").is_empty());
    }
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    const BASE: &str = "plays = 3.0\nlast = \"2026-09-06\"\nweight = 1.0\n\n[tops.\"A Forest\"]\nplays = 1.0\nlast = \"2026-09-06\"\n";

    #[test]
    fn plays_from_two_machines_add_up() {
        let ours = "plays = 5.0\nlast = \"2026-09-07\"\nweight = 1.0\n\n[tops.\"A Forest\"]\nplays = 2.0\nlast = \"2026-09-07\"\n\n[tops.Push]\nplays = 1.0\nlast = \"2026-09-07\"\n";
        let theirs = "plays = 4.0\nlast = \"2026-09-07\"\nweight = 1.0\n\n[tops.\"A Forest\"]\nplays = 1.0\nlast = \"2026-09-06\"\n\n[tops.Lullaby]\nplays = 1.0\nlast = \"2026-09-07\"\nliked = true\n";
        let today = from_iso("2026-09-07").unwrap();
        let merged = merge_artist_at(Some(BASE), ours, theirs, today).unwrap();
        let artist: Artist = toml::from_str(&merged).unwrap();
        // 5 + 4 − 3 (decayed one day): each side counted two plays
        assert!((artist.plays - 6.0).abs() < 0.02, "{}", artist.plays);
        assert_eq!(artist.last.as_deref(), Some("2026-09-07"));
        // one side did not touch A Forest: the other wins as is
        assert_eq!(artist.tops["A Forest"].plays, 2.0);
        // a top new on each side comes in as is
        assert_eq!(artist.tops["Push"].plays, 1.0);
        assert_eq!(artist.tops["Lullaby"].plays, 1.0);
        assert!(artist.tops["Lullaby"].liked);
        // and the file comes out sorted, like every other
        let a = merged.find("A Forest").unwrap();
        let l = merged.find("Lullaby").unwrap();
        let p = merged.find("Push").unwrap();
        assert!(a < l && l < p, "{merged}");
    }

    #[test]
    fn a_ban_on_one_side_wins_and_the_weight_follows_who_moved() {
        let ours = "plays = 3.0\nlast = \"2026-09-06\"\nweight = 0.7\n\n[tops.\"A Forest\"]\nplays = 1.0\nlast = \"2026-09-06\"\n";
        let theirs = "plays = 3.0\nlast = \"2026-09-06\"\nweight = 1.0\nblacklisted = true\n\n[tops.\"A Forest\"]\nplays = 1.0\nlast = \"2026-09-06\"\nblacklisted = true\n";
        let today = from_iso("2026-09-07").unwrap();
        let artist: Artist =
            toml::from_str(&merge_artist_at(Some(BASE), ours, theirs, today).unwrap()).unwrap();
        assert!(artist.blacklisted);
        assert!(artist.tops["A Forest"].blacklisted);
        assert!((artist.weight - 0.7).abs() < 1e-6);
        // nothing more was played: the count does not move
        assert_eq!(artist.plays, 3.0);
    }

    #[test]
    fn without_an_ancestor_both_sides_add_up() {
        let ours = "plays = 1.0\nlast = \"2026-09-07\"\nweight = 1.0\n";
        let theirs = "plays = 2.0\nlast = \"2026-09-07\"\nweight = 1.0\n";
        let today = from_iso("2026-09-07").unwrap();
        let artist: Artist = toml::from_str(&merge_artist_at(None, ours, theirs, today).unwrap()).unwrap();
        assert_eq!(artist.plays, 3.0);
    }
}

#[cfg(test)]
impl Learned {
    /// A blank layer for tests. Its root is a temp dir: measures may write,
    /// nobody reads it back.
    pub fn blank() -> Learned {
        Learned {
            root: std::env::temp_dir().join("forkstify-tests"),
            artists: HashMap::new(),
            seed: HashMap::new(),
            seed_max: 1.0,
            seed_names: HashMap::new(),
            seed_liked: HashSet::new(),
            today: 20_000,
        }
    }
}

#[cfg(test)]
mod tests {
    /// The home's default view: an artist is "liked" here by a ♥ on one
    /// of its tracks or a "more often" on itself; a skip undoes it.
    #[test]
    fn liked_here_by_a_track_or_by_the_artist() {
        let mut learned = Learned::blank();
        assert!(!learned.liked("the-cure", "The Cure"));
        learned.like_track("the-cure", "A Forest");
        assert!(learned.liked("the-cure", "The Cure"));
        learned.skip_track("the-cure", "A Forest");
        assert!(!learned.liked("the-cure", "The Cure"));
        learned.like_artist("the-cure");
        assert!(learned.liked("the-cure", "The Cure"));
        // "less often" takes the artist out of the liked, even with
        // a ♥ on a track; "more often" brings it back
        learned.like_track("the-cure", "A Forest");
        learned.skip_artist("the-cure");
        assert!(!learned.liked("the-cure", "The Cure"));
        learned.like_artist("the-cure");
        assert!(learned.liked("the-cure", "The Cure"));
    }

    use super::*;

    #[test]
    fn dates_round_trip() {
        for day in [0, 1, 19_000, 20_700, 25_000, -1, -700] {
            assert_eq!(from_iso(&iso(day)), Some(day), "jour {day}");
        }
        assert_eq!(iso(0), "1970-01-01");
        assert_eq!(from_iso("2026-09-05").map(iso).as_deref(), Some("2026-09-05"));
    }

    #[test]
    fn the_half_life_is_six_months() {
        let day = from_iso("2026-01-01").unwrap();
        // same day: nothing has decayed
        assert!((decay(8.0, Some("2026-01-01"), day) - 8.0).abs() < 1e-9);
        // one half-life later: half of it is left
        let later = day + HALF_LIFE as i64;
        assert!((decay(8.0, Some("2026-01-01"), later) - 4.0).abs() < 0.02);
        // two half-lives: a quarter
        let much_later = day + 2 * HALF_LIFE as i64;
        assert!((decay(8.0, Some("2026-01-01"), much_later) - 2.0).abs() < 0.02);
        // no date is no decay
        assert_eq!(decay(3.0, None, later), 3.0);
    }

    /// The on-disk shape is a public interface (0014): it must survive a
    /// round trip, and stay readable by hand.
    #[test]
    fn the_on_disk_format_round_trips() {
        let mut artist = Artist { plays: 12.4, last: Some("2026-09-04".into()), weight: 0.8, ..Default::default() };
        artist.tops.insert(
            "A Forest".into(),
            Top { plays: 5.0, last: Some("2026-09-04".into()), liked: true, skipped: 2, blacklisted: false },
        );
        let text = toml::to_string_pretty(&artist).expect("serializable");
        assert!(text.contains("plays = 12.4"), "{text}");
        assert!(text.contains("[tops.\"A Forest\"]"), "{text}");
        // default values do not clutter the file
        assert!(!text.contains("blacklisted"), "{text}");

        let back: Artist = toml::from_str(&text).expect("relisible");
        assert_eq!(back.plays, 12.4);
        assert_eq!(back.weight, 0.8);
        assert_eq!(back.tops["A Forest"].skipped, 2);
        assert!(back.tops["A Forest"].liked);

        // an empty file is still a neutral artist, not an error
        let neuf: Artist = toml::from_str("").expect("empty reads back");
        assert_eq!(neuf.weight, 1.0);
        assert!(!neuf.blacklisted);
    }

    #[test]
    fn the_seed_is_found_by_slug_when_the_name_changed() {
        // "Kanye West" in the Spotify library, "Ye" on the `kanye-west`
        // card: same artist, same familiarity (Joel, 10/09/2026)
        let mut seed = HashMap::new();
        seed.insert("kanye-west".to_string(), 10.0);
        let mut seed_liked = HashSet::new();
        seed_liked.insert("kanye-west".to_string());
        let learned = Learned {
            root: PathBuf::from("/nonexistent"),
            artists: HashMap::new(),
            seed,
            seed_max: 10.0,
            seed_names: HashMap::new(),
            seed_liked,
            today: 20_000,
        };
        assert_eq!(learned.familiarity01("kanye-west", "Ye"), 1.0);
        assert_eq!(learned.familiarity01("", "Kanye West"), 1.0);
        assert!(learned.liked("kanye-west", "Ye"));
        assert!(learned.liked("", "Kanye West"));
        assert_eq!(learned.familiarity01("drake", "Drake"), 0.0);
    }

    #[test]
    fn weights_stay_within_bounds() {
        let mut learned =
            Learned { root: PathBuf::from("/nonexistent"), artists: HashMap::new(),
                      seed: HashMap::new(), seed_max: 1.0,
                      seed_names: HashMap::new(), seed_liked: HashSet::new(), today: 20_000 };
        for _ in 0..40 {
            learned.skip_artist("x");
        }
        assert!(learned.weight("x") >= WEIGHT_FLOOR);
        for _ in 0..80 {
            learned.like_artist("x");
        }
        assert!(learned.weight("x") <= WEIGHT_CEILING);
    }
}
