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
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Six months, in days (0014).
const HALF_LIFE: f64 = 182.5;

/// « moins souvent » multiplies the weight by this, down to the floor.
const LESS_OFTEN: f32 = 0.7;
const WEIGHT_FLOOR: f32 = 0.1;
/// « plus souvent » is its mirror, capped so one key cannot run away.
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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tops: HashMap<String, Top>,
}

impl Default for Artist {
    fn default() -> Self {
        Artist { plays: 0.0, last: None, weight: 1.0, blacklisted: false, tops: HashMap::new() }
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
    /// `classement.json`: the familiarity an artist starts with, before any
    /// listening of our own (0014). Keyed by display name — the seed file
    /// predates slugs.
    seed: HashMap<String, f64>,
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
                    Ok(Err(e)) => eprintln!("appris illisible ({}) : {e}", path.display()),
                    Err(_) => {}
                }
            }
        }

        let mut seed = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(root.join("classement.json")) {
            if let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                for row in rows {
                    // the seed file still has its French keys (0014: to be
                    // translated with the outillage scripts)
                    if let (Some(name), Some(score)) =
                        (row["nom"].as_str(), row["score"].as_f64())
                    {
                        seed.insert(name.to_lowercase(), score);
                    }
                }
            }
        }

        Learned { root, artists, seed, today: today() }
    }

    pub fn known(&self) -> usize {
        self.artists.len()
    }

    pub fn seeded(&self) -> usize {
        self.seed.len()
    }

    /// How familiar an artist is: our own decayed plays, or the seed score
    /// when we have never played them.
    pub fn familiarity(&self, slug: &str, name: &str) -> f64 {
        match self.artists.get(slug) {
            Some(artist) if artist.plays > 0.0 => decay(artist.plays, artist.last.as_deref(), self.today),
            _ => self.seed.get(&name.to_lowercase()).copied().unwrap_or(0.0),
        }
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

    pub fn like_track(&mut self, slug: &str, title: &str) {
        self.entry(slug).tops.entry(title.to_string()).or_default().liked = true;
        self.save(slug);
    }

    pub fn skip_track(&mut self, slug: &str, title: &str) {
        let top = self.entry(slug).tops.entry(title.to_string()).or_default();
        top.skipped += 1;
        self.save(slug);
    }

    pub fn ban_track(&mut self, slug: &str, title: &str) {
        self.entry(slug).tops.entry(title.to_string()).or_default().blacklisted = true;
        self.save(slug);
    }

    pub fn like_artist(&mut self, slug: &str) -> f32 {
        let artist = self.entry(slug);
        artist.weight = (artist.weight * MORE_OFTEN).min(WEIGHT_CEILING);
        let weight = artist.weight;
        self.save(slug);
        weight
    }

    pub fn skip_artist(&mut self, slug: &str) -> f32 {
        let artist = self.entry(slug);
        artist.weight = (artist.weight * LESS_OFTEN).max(WEIGHT_FLOOR);
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
            eprintln!("appris non enregistré ({e})");
            return;
        }
        match toml::to_string_pretty(artist) {
            Ok(text) => {
                if let Err(e) = std::fs::write(dir.join(format!("{slug}.toml")), text) {
                    eprintln!("appris non enregistré ({e})");
                }
            }
            Err(e) => eprintln!("appris non sérialisable ({e})"),
        }
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
impl Learned {
    /// A blank layer for tests. Its root is a temp dir: measures may write,
    /// nobody reads it back.
    pub fn blank() -> Learned {
        Learned {
            root: std::env::temp_dir().join("forkstify-tests"),
            artists: HashMap::new(),
            seed: HashMap::new(),
            today: 20_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_dates_font_l_aller_retour() {
        for day in [0, 1, 19_000, 20_700, 25_000, -1, -700] {
            assert_eq!(from_iso(&iso(day)), Some(day), "jour {day}");
        }
        assert_eq!(iso(0), "1970-01-01");
        assert_eq!(from_iso("2026-09-05").map(iso).as_deref(), Some("2026-09-05"));
    }

    #[test]
    fn la_demi_vie_est_de_six_mois() {
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
    fn le_format_sur_disque_fait_l_aller_retour() {
        let mut artist = Artist { plays: 12.4, last: Some("2026-09-04".into()), weight: 0.8, ..Default::default() };
        artist.tops.insert(
            "A Forest".into(),
            Top { plays: 5.0, last: Some("2026-09-04".into()), liked: true, skipped: 2, blacklisted: false },
        );
        let text = toml::to_string_pretty(&artist).expect("sérialisable");
        assert!(text.contains("plays = 12.4"), "{text}");
        assert!(text.contains("[tops.\"A Forest\"]"), "{text}");
        // les valeurs par défaut ne salissent pas le fichier
        assert!(!text.contains("blacklisted"), "{text}");

        let back: Artist = toml::from_str(&text).expect("relisible");
        assert_eq!(back.plays, 12.4);
        assert_eq!(back.weight, 0.8);
        assert_eq!(back.tops["A Forest"].skipped, 2);
        assert!(back.tops["A Forest"].liked);

        // un fichier vide reste un artiste neutre, pas une erreur
        let neuf: Artist = toml::from_str("").expect("vide relisible");
        assert_eq!(neuf.weight, 1.0);
        assert!(!neuf.blacklisted);
    }

    #[test]
    fn les_poids_restent_dans_leurs_bornes() {
        let mut learned =
            Learned { root: PathBuf::from("/nonexistent"), artists: HashMap::new(),
                      seed: HashMap::new(), today: 20_000 };
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
