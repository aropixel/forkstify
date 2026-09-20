//! The listener's Spotify library, harvested by the setup (chantier A of
//! `docs/conception/sortie.md`): liked tracks, liked albums, followed
//! artists, and the playlists one ticks. Written to `learned/library.toml`
//! — one file, English vocabulary (0014, 0022) — which replaces
//! `classement.json` and the five `artistes-*.json` the Python scripts of
//! `tools/` produced. The ranking keeps the formula of `classement.py`:
//! liked tracks ×1, liked albums ×3, followed +8, playlist tracks ×1.
//!
//! Only the **main artist** of a track counts: a guest on a liked track is
//! not a liked artist (Joel, 09/09/2026). The ticked playlists are
//! remembered, so harvesting again is one gesture, nothing to tick again.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const FORMAT: u32 = 1;
/// The ranking's weights — the harvest's own, not the engine's (0023).
const LIKED_TRACK: u32 = 1;
const LIKED_ALBUM: u32 = 3;
const FOLLOWED: u32 = 8;
const PLAYLIST_TRACK: u32 = 1;
/// Liked tracks are read up to here: a library beyond that says nothing
/// more about taste, and every page is a call.
const MAX_TRACKS: usize = 5000;
/// The coverage step (7): the score from which an artist without a card is
/// worth generating, and how many at most in one go.
pub const COVERAGE_FLOOR: u32 = 5;
pub const COVERAGE_CAP: usize = 30;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Library {
    pub format: u32,
    /// ISO date of the last harvest.
    #[serde(default)]
    pub harvested: String,
    /// The ticked playlists, remembered.
    #[serde(default)]
    pub playlists: Vec<Playlist>,
    #[serde(default)]
    pub artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Artist {
    pub name: String,
    pub spotify: String,
    #[serde(default)]
    pub liked_tracks: u32,
    #[serde(default)]
    pub liked_albums: u32,
    #[serde(default)]
    pub followed: bool,
    #[serde(default)]
    pub playlist_tracks: u32,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub sources: u32,
}

impl Artist {
    fn rescore(&mut self) {
        self.score = self.liked_tracks * LIKED_TRACK
            + self.liked_albums * LIKED_ALBUM
            + self.playlist_tracks * PLAYLIST_TRACK
            + if self.followed { FOLLOWED } else { 0 };
        self.sources = u32::from(self.liked_tracks > 0)
            + u32::from(self.liked_albums > 0)
            + u32::from(self.followed)
            + u32::from(self.playlist_tracks > 0);
    }

    /// Liked on Spotify, in the sense of the home's collection: a track,
    /// an album, or a follow.
    pub fn liked(&self) -> bool {
        self.liked_tracks > 0 || self.liked_albums > 0 || self.followed
    }
}

pub fn path(catalog_dir: &Path) -> PathBuf {
    catalog_dir.join("learned").join("library.toml")
}

impl Library {
    pub fn load(catalog_dir: &Path) -> Option<Library> {
        let text = std::fs::read_to_string(path(catalog_dir)).ok()?;
        toml::from_str(&text).ok()
    }

    pub fn save(&self, catalog_dir: &Path) -> Result<(), String> {
        let file = path(catalog_dir);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("no learned/ folder ({e})"))?;
        }
        let text = toml::to_string(self).map_err(|e| format!("library not serialized ({e})"))?;
        std::fs::write(&file, text).map_err(|e| format!("library.toml not written ({e})"))
    }

    /// Recompute every score and sort, best first.
    pub fn rescore(&mut self) {
        for artist in &mut self.artists {
            artist.rescore();
        }
        self.artists.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    }

    pub fn liked_tracks(&self) -> u32 {
        self.artists.iter().map(|a| a.liked_tracks).sum()
    }
    pub fn liked_albums(&self) -> u32 {
        self.artists.iter().map(|a| a.liked_albums).sum()
    }
    pub fn followed(&self) -> usize {
        self.artists.iter().filter(|a| a.followed).count()
    }
    pub fn playlist_tracks(&self) -> u32 {
        self.artists.iter().map(|a| a.playlist_tracks).sum()
    }

    fn entry(&mut self, id: &str, name: &str) -> &mut Artist {
        let index = match self.artists.iter().position(|a| a.spotify == id) {
            Some(index) => index,
            None => {
                self.artists.push(Artist { name: name.to_string(), spotify: id.to_string(), ..Artist::default() });
                self.artists.len() - 1
            }
        };
        &mut self.artists[index]
    }
}

/// What the harvest says as it goes — one bar per source on the screen.
#[derive(Clone, Debug)]
pub enum Progress {
    Tracks { done: usize, total: usize },
    Albums { done: usize, total: usize },
    Followed { done: usize },
    Playlist { name: String, done: usize, total: usize },
}

type Report = tokio::sync::mpsc::UnboundedSender<Progress>;

/// The main artist of a track or album object: the first one, only.
fn main_artist(object: &serde_json::Value) -> Option<(String, String)> {
    let artist = object["artists"].as_array()?.first()?;
    Some((artist["id"].as_str()?.to_string(), artist["name"].as_str()?.to_string()))
}

/// Liked tracks, liked albums, followed artists — the three sources that
/// need no choice. The playlists' counts are kept as they were: they are
/// harvested apart (`harvest_playlists`), once ticked.
pub async fn harvest(web: &mut crate::spotify::WebApi, previous: Option<&Library>, report: &Report) -> Result<Library, String> {
    let mut library = Library { format: FORMAT, ..Library::default() };
    if let Some(previous) = previous {
        library.playlists = previous.playlists.clone();
        // the playlist counts survive: they are re-harvested in their step
        for artist in &previous.artists {
            if artist.playlist_tracks > 0 {
                library.entry(&artist.spotify, &artist.name).playlist_tracks = artist.playlist_tracks;
            }
        }
    }

    // liked tracks, paged by 50, the main artist only
    let mut offset = 0;
    let mut total = 0;
    loop {
        let page = web
            .get_json(&format!("https://api.spotify.com/v1/me/tracks?limit=50&offset={offset}"))
            .await?;
        total = page["total"].as_u64().unwrap_or(total as u64) as usize;
        let items = page["items"].as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            break;
        }
        for item in &items {
            if let Some((id, name)) = main_artist(&item["track"]) {
                library.entry(&id, &name).liked_tracks += 1;
            }
        }
        offset += items.len();
        let _ = report.send(Progress::Tracks { done: offset, total: total.min(MAX_TRACKS) });
        if page["next"].is_null() || offset >= MAX_TRACKS {
            break;
        }
    }

    // liked albums
    let mut offset = 0;
    let mut total = 0;
    loop {
        let page = web
            .get_json(&format!("https://api.spotify.com/v1/me/albums?limit=50&offset={offset}"))
            .await?;
        total = page["total"].as_u64().unwrap_or(total as u64) as usize;
        let items = page["items"].as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            break;
        }
        for item in &items {
            if let Some((id, name)) = main_artist(&item["album"]) {
                library.entry(&id, &name).liked_albums += 1;
            }
        }
        offset += items.len();
        let _ = report.send(Progress::Albums { done: offset, total });
        if page["next"].is_null() {
            break;
        }
    }

    // followed artists, a cursor rather than an offset
    let mut after: Option<String> = None;
    let mut done = 0;
    loop {
        let mut url = "https://api.spotify.com/v1/me/following?type=artist&limit=50".to_string();
        if let Some(cursor) = &after {
            url.push_str("&after=");
            url.push_str(cursor);
        }
        let page = web.get_json(&url).await?;
        let block = &page["artists"];
        let items = block["items"].as_array().cloned().unwrap_or_default();
        for artist in &items {
            if let (Some(id), Some(name)) = (artist["id"].as_str(), artist["name"].as_str()) {
                library.entry(id, name).followed = true;
                done += 1;
            }
        }
        let _ = report.send(Progress::Followed { done });
        after = block["cursors"]["after"].as_str().map(str::to_string);
        if after.is_none() || items.is_empty() {
            break;
        }
    }

    library.harvested = crate::learned::today_iso();
    library.rescore();
    Ok(library)
}

/// One of the listener's playlists, as the step lists them.
#[derive(Clone, Debug)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub tracks: usize,
    /// "you", "spotify", or "someone else".
    pub owner: String,
}

/// The listener's playlists, theirs first.
pub async fn list_playlists(web: &mut crate::spotify::WebApi, me: &str) -> Result<Vec<PlaylistInfo>, String> {
    let mut out = Vec::new();
    let mut url = Some("https://api.spotify.com/v1/me/playlists?limit=50".to_string());
    while let Some(next) = url.take() {
        let page = web.get_json(&next).await?;
        for item in page["items"].as_array().cloned().unwrap_or_default() {
            let (Some(id), Some(name)) = (item["id"].as_str(), item["name"].as_str()) else { continue };
            let owner_id = item["owner"]["id"].as_str().unwrap_or("");
            let owner = if owner_id == me {
                "you"
            } else if owner_id == "spotify" {
                "spotify"
            } else {
                "someone else"
            };
            out.push(PlaylistInfo {
                id: id.to_string(),
                name: name.to_string(),
                tracks: item["tracks"]["total"].as_u64().unwrap_or(0) as usize,
                owner: owner.to_string(),
            });
        }
        url = page["next"].as_str().map(str::to_string);
    }
    let rank = |p: &PlaylistInfo| match p.owner.as_str() {
        "you" => 0,
        "someone else" => 1,
        _ => 2,
    };
    out.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(out)
}

/// The tracks of the ticked playlists, counted to their main artist. The
/// previous counts are dropped first: a playlist unticked no longer counts.
pub async fn harvest_playlists(
    web: &mut crate::spotify::WebApi,
    library: &mut Library,
    ticked: &[PlaylistInfo],
    report: &Report,
) -> Result<(), String> {
    for artist in &mut library.artists {
        artist.playlist_tracks = 0;
    }
    library.playlists = ticked.iter().map(|p| Playlist { id: p.id.clone(), name: p.name.clone() }).collect();
    for playlist in ticked {
        let mut done = 0;
        let base = format!("https://api.spotify.com/v1/playlists/{}", playlist.id);
        // the newer path first, the older one when Spotify answers nothing
        let mut url = Some(format!("{base}/items?limit=50"));
        let mut fell_back = false;
        while let Some(next) = url.take() {
            let page = match web.get_json(&next).await {
                Ok(page) => page,
                Err(_) if !fell_back => {
                    fell_back = true;
                    url = Some(format!("{base}/tracks?limit=50"));
                    continue;
                }
                Err(why) => return Err(why),
            };
            for item in page["items"].as_array().cloned().unwrap_or_default() {
                let track = if item["track"].is_null() { &item["item"] } else { &item["track"] };
                if let Some((id, name)) = main_artist(track) {
                    library.entry(&id, &name).playlist_tracks += 1;
                    done += 1;
                }
            }
            let _ = report.send(Progress::Playlist { name: playlist.name.clone(), done, total: playlist.tracks });
            url = page["next"].as_str().map(str::to_string);
        }
    }
    library.harvested = crate::learned::today_iso();
    library.rescore();
    Ok(())
}

/// The ranking crossed with the catalog: how much of the top has a card,
/// and who is worth generating (step 7).
pub struct Coverage {
    /// (label, with a card, total) — score ≥ 20, 10–19, 5–9.
    pub tiers: Vec<(String, usize, usize)>,
    /// The fifty most present: how many have a card.
    pub top: (usize, usize),
    /// Score ≥ `COVERAGE_FLOOR` without a card, best first, capped.
    pub missing: Vec<(String, u32)>,
    /// How many were above the floor without a card, before the cap.
    pub missing_total: usize,
}

pub fn coverage(library: &Library, has_card: impl Fn(&str) -> bool) -> Coverage {
    let mut tiers: Vec<(String, usize, usize)> = vec![
        ("score ≥ 20".to_string(), 0, 0),
        ("score 10–19".to_string(), 0, 0),
        ("score 5–9".to_string(), 0, 0),
    ];
    let mut missing = Vec::new();
    let mut top = (0, 0);
    for (rank, artist) in library.artists.iter().enumerate() {
        let known = has_card(&artist.name);
        if rank < 50 {
            top.1 += 1;
            top.0 += usize::from(known);
        }
        let tier = match artist.score {
            s if s >= 20 => Some(0),
            s if s >= 10 => Some(1),
            s if s >= COVERAGE_FLOOR => Some(2),
            _ => None,
        };
        if let Some(tier) = tier {
            tiers[tier].2 += 1;
            tiers[tier].1 += usize::from(known);
            if !known {
                missing.push((artist.name.clone(), artist.score));
            }
        }
    }
    let missing_total = missing.len();
    missing.truncate(COVERAGE_CAP);
    Coverage { tiers, top, missing, missing_total }
}

/// The name → slug rule the catalog and the seed share, plus the names the
/// cards carry: a card whose name moved on ("Ye" at `kanye-west`) still
/// counts as known under either.
pub fn card_finder(catalog: &crate::catalog::Catalog) -> impl Fn(&str) -> bool + '_ {
    let names: HashMap<String, ()> =
        catalog.cards.values().map(|card| (crate::generate::slugify(&card.name), ())).collect();
    move |name: &str| {
        let slug = crate::generate::slugify(name);
        catalog.cards.contains_key(&slug) || names.contains_key(&slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> Library {
        let mut library = Library { format: FORMAT, ..Library::default() };
        library.entry("1", "Calexico").liked_tracks = 66;
        library.entry("1", "Calexico").liked_albums = 4;
        library.entry("1", "Calexico").playlist_tracks = 6;
        library.entry("2", "Guilhem Valayé").followed = true;
        library.entry("3", "Odezenne").liked_tracks = 41;
        library.entry("3", "Odezenne").liked_albums = 10;
        library.rescore();
        library
    }

    /// The formula of classement.py, unchanged: tracks ×1, albums ×3,
    /// followed +8, playlist tracks ×1.
    #[test]
    fn the_ranking_keeps_the_formula() {
        let library = library();
        assert_eq!(library.artists[0].name, "Calexico");
        assert_eq!(library.artists[0].score, 66 + 12 + 6);
        assert_eq!(library.artists[0].sources, 3);
        assert_eq!(library.artists[1].name, "Odezenne");
        assert_eq!(library.artists[1].score, 41 + 30);
        assert_eq!(library.artists[2].score, 8);
        assert!(library.artists[2].liked());
    }

    #[test]
    fn the_file_round_trips() {
        let mut library = library();
        library.harvested = "2026-09-20".into();
        library.playlists.push(Playlist { id: "abc".into(), name: "#fipway".into() });
        let text = toml::to_string(&library).expect("serializable");
        assert!(text.contains("[[artists]]"), "{text}");
        assert!(text.contains("liked_tracks = 66"), "{text}");
        let back: Library = toml::from_str(&text).expect("readable");
        assert_eq!(back.artists.len(), 3);
        assert_eq!(back.playlists[0].name, "#fipway");
        assert_eq!(back.harvested, "2026-09-20");
    }

    /// Coverage: the tiers, the top fifty, and who is worth generating —
    /// above the floor, without a card, best first.
    #[test]
    fn coverage_names_who_to_generate() {
        let library = library();
        let coverage = coverage(&library, |name| name == "Calexico");
        assert_eq!(coverage.top, (1, 3));
        assert_eq!(coverage.tiers[0], ("score ≥ 20".to_string(), 1, 2));
        assert_eq!(coverage.tiers[2], ("score 5–9".to_string(), 0, 1));
        assert_eq!(coverage.missing, vec![("Odezenne".to_string(), 71), ("Guilhem Valayé".to_string(), 8)]);
        assert_eq!(coverage.missing_total, 2);
    }

    #[test]
    fn the_main_artist_only() {
        let track = serde_json::json!({"artists": [{"id": "a", "name": "Bosh"}, {"id": "b", "name": "Guest"}]});
        assert_eq!(main_artist(&track), Some(("a".to_string(), "Bosh".to_string())));
        assert_eq!(main_artist(&serde_json::json!({})), None);
    }
}
