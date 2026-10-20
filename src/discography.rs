//! The long tail: the fourth source of the reservoir (0012 §1).
//!
//! Everything the engine draws otherwise comes from the card — tops, doors —
//! or from `learned/`. The tail is the rest of an artist's discography.
//! **What exists** is catalog material since 0027: `discography/<slug>.toml`,
//! MusicBrainz's word (`releases.rs`). **What plays it** is this cache: the
//! artist's Spotify albums matched to those releases (0028 §5), with their
//! track uris — local, regenerable, never committed, kept a month
//! (`spotify_match_days`) as Spotify's terms ask, then matched again the
//! next time the artist is needed.
//!
//! The match reads the card's own Spotify artist, so a tail track can never
//! be a namesake's, and it needs no title → id resolution: it can never turn
//! into a "not found on Spotify".

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TailTrack {
    pub title: String,
    pub uri: String,
    pub album: String,
    /// The album's release date as Spotify gives it — "1998" or
    /// "1998-09-22". Without it there is no chronological order, which is
    /// how one remembers an artist (maquette 1a, 07/09/2026).
    #[serde(default)]
    pub released: String,
    /// Rank inside its album. Ordering a folded album by anything else
    /// would make it unrecognisable.
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub duration_ms: u32,
    /// A single or an EP rather than an album: shown after the albums, so a
    /// discography does not turn into a list of one-track records.
    #[serde(default)]
    pub single: bool,
    /// The disc, on a release of several: the number restarts on each.
    #[serde(default = "first_disc")]
    pub disc: u32,
}

fn first_disc() -> u32 {
    1
}

/// One artist's match, as the cache keeps it. `complete` is false when a
/// call failed on the way: what came is used, and the next need resumes.
#[derive(Serialize, Deserialize)]
struct Matched {
    complete: bool,
    tracks: Vec<TailTrack>,
}

impl TailTrack {
    /// The year, when the date says one.
    pub fn year(&self) -> Option<u16> {
        self.released.get(..4).and_then(|y| y.parse().ok())
    }
}

/// Every discography harvested so far, keyed by slug. Loaded once at
/// startup: the engine reads it synchronously, the application fills it.
pub struct Tail {
    root: PathBuf,
    artists: HashMap<String, Vec<TailTrack>>,
    /// The artists whose match is due again: incomplete, or older than
    /// `spotify_match_days`. Their tracks still play meanwhile.
    stale: std::collections::HashSet<String>,
    /// The harvests from before 0027 — Spotify's own albums, a bare list:
    /// converted on first need against `discography/`, with no call.
    legacy: HashMap<String, Vec<TailTrack>>,
}

impl Tail {
    pub fn load() -> Tail {
        let root = cache_dir();
        let mut artists = HashMap::new();
        let mut stale = std::collections::HashSet::new();
        let mut legacy = HashMap::new();
        let days = crate::config::tuning().spotify_match_days as u64;
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Ok(text) = std::fs::read_to_string(&path) else { continue };
                let Ok(matched) = serde_json::from_str::<Matched>(&text) else {
                    // a harvest from before 0027: kept aside, converted
                    // when the artist is first needed
                    if let Ok(tracks) = serde_json::from_str::<Vec<TailTrack>>(&text) {
                        legacy.insert(slug.to_string(), tracks);
                    }
                    continue;
                };
                let age = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .map_or(u64::MAX, |d| d.as_secs() / 86_400);
                if !matched.complete || age >= days {
                    stale.insert(slug.to_string());
                }
                artists.insert(slug.to_string(), matched.tracks);
            }
        }
        Tail { root, artists, stale, legacy }
    }

    /// An empty tail, for tests and for the dry journey on a cold machine.
    #[cfg(test)]
    pub fn blank() -> Tail {
        Tail { root: std::env::temp_dir().join("forkstify-tests-tail"), artists: HashMap::new(), stale: Default::default(), legacy: HashMap::new() }
    }

    pub fn known(&self) -> usize {
        self.artists.len()
    }

    pub fn has(&self, slug: &str) -> bool {
        self.artists.contains_key(slug)
    }

    /// A harvest from before 0027, turned into a tail by the releases of
    /// `discography/` without a call to Spotify (Joel, 2026-10-20: the
    /// first launch matched every artist again, two minutes each, and `ad`
    /// waited). Kept incomplete: the real match follows, behind. False when
    /// there was nothing to convert.
    pub fn convert(&mut self, slug: &str, releases: &[crate::releases::Release]) -> bool {
        let Some(old) = self.legacy.remove(slug) else { return false };
        let tracks = from_legacy(releases, &old);
        self.keep(slug, tracks, false);
        true
    }

    /// Known, complete, and matched within the month.
    pub fn fresh(&self, slug: &str) -> bool {
        self.has(slug) && !self.stale.contains(slug)
    }

    /// A harvest made before the dates were kept: it can still feed the
    /// reservoir, but not the discography screen, which needs the year and
    /// the track number. The cache is regenerable and outside the repo, so
    /// the answer to an old one is simply to harvest again.
    pub fn dated(&self, slug: &str) -> bool {
        self.of(slug).iter().any(|track| !track.released.is_empty())
    }

    /// Forget one artist's harvest, so the next one goes back to Spotify.
    pub fn forget(&mut self, slug: &str) {
        self.artists.remove(slug);
        self.stale.remove(slug);
        let _ = std::fs::remove_file(self.root.join(format!("{slug}.json")));
    }

    pub fn of(&self, slug: &str) -> &[TailTrack] {
        self.artists.get(slug).map_or(&[], |v| v.as_slice())
    }

    /// Keep a match, on disk and in memory. An incomplete one stays due:
    /// the next need resumes it.
    pub fn keep(&mut self, slug: &str, tracks: Vec<TailTrack>, complete: bool) {
        let matched = Matched { complete, tracks };
        if let Err(e) = std::fs::create_dir_all(&self.root) {
            eprintln!("tail not saved ({e})");
        } else if let Ok(text) = serde_json::to_string(&matched) {
            let _ = std::fs::write(self.root.join(format!("{slug}.json")), text);
        }
        if complete {
            self.stale.remove(slug);
        } else {
            self.stale.insert(slug.to_string());
        }
        self.artists.insert(slug.to_string(), matched.tracks);
    }
}

/// One of the artist's albums as Spotify lists them.
#[derive(Debug, Clone)]
pub struct SpotifyAlbum {
    pub id: String,
    pub name: String,
    pub date: String,
    /// `album_type` is `single` — Spotify files EPs there too.
    pub single: bool,
}

/// The words that name an edition, not a work (0028 §5).
const EDITION_WORDS: [&str; 6] = ["remaster", "deluxe", "expanded", "anniversary", "bonus", "edition"];

fn names_an_edition(text: &str) -> bool {
    let lower = text.to_lowercase();
    EDITION_WORDS.iter().any(|w| lower.contains(w))
}

/// A title with its edition mentions dropped — "(Deluxe Edition)",
/// "[2009 Remaster]", "- 25th Anniversary" — and nothing else: "(I Can't
/// Get No) Satisfaction" keeps its parenthesis.
fn without_editions(title: &str) -> &str {
    let mut text = title.trim_end();
    loop {
        let cut = if text.ends_with(')') || text.ends_with(']') {
            let open = if text.ends_with(')') { '(' } else { '[' };
            text.rfind(open)
        } else {
            text.rfind(" - ")
        };
        match cut {
            Some(i) if names_an_edition(&text[i..]) => text = text[..i].trim_end(),
            _ => return text,
        }
    }
}

/// Is this Spotify album an edition of its work — a deluxe, a remaster?
pub fn is_edition(name: &str) -> bool {
    without_editions(name).len() != name.trim_end().len()
}

/// The key two titles are compared by (0028 §5): editions dropped, `&`
/// read as `and`, accents, case and punctuation gone.
pub fn album_key(title: &str) -> String {
    crate::generate::fold_text(&without_editions(title).replace('&', " and "))
}

fn year(date: &str) -> Option<i32> {
    date.get(..4).and_then(|y| y.parse().ok())
}

fn distance(a: &str, b: &str) -> i32 {
    match (year(a), year(b)) {
        (Some(a), Some(b)) => (a - b).abs(),
        _ => 1000,
    }
}

/// Which Spotify album plays each release (0028 §5): its own link when
/// MusicBrainz has one; otherwise the same title once the editions are
/// dropped — the date only breaks a tie between two releases of the same
/// title —, the original ahead of a deluxe or a remaster, then the same
/// kind (album or single), then the nearest year. A release with no match
/// does not play. Returns `(release index, album id)`.
pub fn match_albums(releases: &[crate::releases::Release], albums: &[SpotifyAlbum]) -> Vec<(usize, String)> {
    let keys: Vec<String> = releases.iter().map(|r| album_key(&r.title)).collect();
    // every album to the one release it belongs to: the only one of its
    // title, or the nearest in date among several
    let mut candidates: Vec<Vec<&SpotifyAlbum>> = vec![Vec::new(); releases.len()];
    for album in albums {
        let key = album_key(&album.name);
        let owner = (0..releases.len()).filter(|&i| keys[i] == key && releases[i].spotify.is_none()).min_by_key(|&i| {
            let release = &releases[i];
            (distance(&release.date, &album.date), (release.kind != "album") != album.single)
        });
        if let Some(i) = owner {
            candidates[i].push(album);
        }
    }
    let mut matched = Vec::new();
    for (i, release) in releases.iter().enumerate() {
        if let Some(id) = &release.spotify {
            matched.push((i, id.clone()));
            continue;
        }
        let best = candidates[i].iter().min_by_key(|a| {
            (is_edition(&a.name), (release.kind != "album") != a.single, distance(&release.date, &a.date), a.name.clone())
        });
        if let Some(album) = best {
            matched.push((i, album.id.clone()));
        }
    }
    matched
}

/// An old harvest's albums matched to the releases by the same rules —
/// its tracks already carry their uris, so nothing is asked of Spotify.
/// MusicBrainz's own links are left aside: an old harvest has no album
/// ids to meet them with.
fn from_legacy(releases: &[crate::releases::Release], old: &[TailTrack]) -> Vec<TailTrack> {
    let mut albums: Vec<SpotifyAlbum> = Vec::new();
    for track in old {
        if !albums.iter().any(|a| a.name == track.album) {
            albums.push(SpotifyAlbum { id: track.album.clone(), name: track.album.clone(), date: track.released.clone(), single: track.single });
        }
    }
    let unlinked: Vec<crate::releases::Release> =
        releases.iter().cloned().map(|r| crate::releases::Release { spotify: None, ..r }).collect();
    let mut tracks = Vec::new();
    for (index, album) in match_albums(&unlinked, &albums) {
        let release = &releases[index];
        tracks.extend(old.iter().filter(|t| t.album == album).map(|t| TailTrack {
            album: release.title.clone(),
            released: release.date.clone(),
            single: release.kind != "album",
            ..t.clone()
        }));
    }
    tracks
}

/// The tracks of one matched release, as the tail keeps them: the whole
/// Spotify edition plays (0028 §5), dated and titled by the release.
pub fn tail_of(release: &crate::releases::Release, album: &serde_json::Value) -> Vec<TailTrack> {
    album["tracks"]["items"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        // the tail goes to the queue without being resolved, so an
        // unplayable track here is a silence later
        .filter(|t| t["is_playable"].as_bool() != Some(false))
        .filter_map(|t| {
            Some(TailTrack {
                title: t["name"].as_str()?.to_string(),
                uri: t["uri"].as_str()?.to_string(),
                album: release.title.clone(),
                released: release.date.clone(),
                number: t["track_number"].as_u64().unwrap_or(0) as u32,
                duration_ms: t["duration_ms"].as_u64().unwrap_or(0) as u32,
                single: release.kind != "album",
                disc: t["disc_number"].as_u64().unwrap_or(1) as u32,
            })
        })
        .collect()
}

/// The track of a harvested discography that a card's top names: same
/// title once the version noise is off, the plain-titled copy ahead of a
/// live or a remaster, a studio album ahead of a live one — a live record
/// titles its tracks plainly, "Coffee Cold" with nothing to say it was
/// recorded on stage (Joel, 2026-10-05) — the earliest release ahead of
/// the rest. The
/// discography comes from the artist's own Spotify id, so a hit here
/// cannot be a namesake's — which the title search can (Joel, 23/09/2026,
/// "The Answer" by Boo played The Boo Radleys).
pub fn find<'a>(tracks: &'a [TailTrack], title: &str) -> Option<&'a TailTrack> {
    let wanted = normalize(title);
    tracks
        .iter()
        .filter(|t| normalize(&t.title) == wanted)
        .min_by_key(|t| {
            (clean_title(&t.title) != t.title, crate::spotify::is_live(&t.album), t.single, t.released.clone())
        })
}

fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache")
        });
    base.join("forkstify").join("discography")
}

/// Two titles are the same track when they differ only by a remaster tag, a
/// live suffix or punctuation — Spotify ships the same song a dozen times.
pub fn normalize(title: &str) -> String {
    let lower = title.to_lowercase();
    // everything from an opening bracket on is version noise more often
    // than not: "(Remastered 2010)", "- 2004 Remaster", "(Live)"
    let cut = lower
        .find(" - ")
        .or_else(|| lower.find(" ("))
        .map_or(lower.as_str(), |i| &lower[..i]);
    cut.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// The **readable** title of a track: the one written into a card when it
/// is promoted to a top. `normalize` is for comparing and returns a key;
/// this one strips the same version noise but keeps the title's case,
/// spaces and punctuation — a card is read by a human ([0002]: the format
/// is a public interface).
pub fn clean_title(title: &str) -> String {
    let cut = title
        .find(" - ")
        .or_else(|| title.find(" ("))
        .map_or(title, |i| &title[..i]);
    cut.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, uri: &str, released: &str, single: bool) -> TailTrack {
        TailTrack {
            title: title.into(),
            uri: uri.into(),
            album: "Listen".into(),
            released: released.into(),
            number: 1,
            duration_ms: 0,
            single,
            disc: 1,
        }
    }

    /// The card says "The Answer": the album copy, not the live one nor
    /// the later single, and nothing when the tail has no such title.
    #[test]
    fn the_top_finds_its_own_copy_in_the_tail() {
        let tail = vec![
            track("The Answer (Live)", "spotify:track:live", "2005", false),
            track("The Answer", "spotify:track:single", "2003", true),
            track("The Answer", "spotify:track:album", "2002-11-11", false),
            track("Listen", "spotify:track:listen", "2002-11-11", false),
        ];
        assert_eq!(find(&tail, "The Answer").map(|t| t.uri.as_str()), Some("spotify:track:album"));
        assert_eq!(find(&tail, "the answer - 2010 remaster").map(|t| t.uri.as_str()), Some("spotify:track:album"));
        assert!(find(&tail, "Stones").is_none());
    }

    /// A live album's copy carries a plain title: the album gives it away,
    /// and the studio take wins even when the live one came out first.
    #[test]
    fn a_live_album_comes_after_the_studio_one() {
        let live = TailTrack { album: "Live at the Village Gate".into(), ..track("Coffee Cold", "spotify:track:live", "1965", false) };
        let studio = TailTrack { album: "Shapes of Rhythm".into(), ..track("Coffee Cold", "spotify:track:studio", "1966", false) };
        assert_eq!(find(&[live, studio], "Coffee Cold").map(|t| t.uri.as_str()), Some("spotify:track:studio"));
    }

    #[test]
    fn the_title_written_in_the_card_stays_readable() {
        assert_eq!(clean_title("Metal Heart - 2015 Remaster"), "Metal Heart");
        assert_eq!(clean_title("Colors and the Kids (Live)"), "Colors and the Kids");
        assert_eq!(clean_title("(I Can't Get No) Satisfaction"), "(I Can't Get No) Satisfaction");
        assert_eq!(clean_title("Cross Bones Style"), "Cross Bones Style");
    }

    fn release(title: &str, kind: &str, date: &str) -> crate::releases::Release {
        crate::releases::Release { mbid: title.into(), title: title.into(), kind: kind.into(), date: date.into(), spotify: None, tracks: Vec::new() }
    }

    fn album(id: &str, name: &str, date: &str, single: bool) -> SpotifyAlbum {
        SpotifyAlbum { id: id.into(), name: name.into(), date: date.into(), single }
    }

    #[test]
    fn an_edition_is_dropped_from_the_title_and_nothing_else() {
        assert_eq!(album_key("Romance (Deluxe Edition)"), album_key("Romance"));
        assert_eq!(album_key("Loaded [2015 Remaster]"), album_key("Loaded"));
        assert_eq!(album_key("Electro-Shock Blues - 25th Anniversary Expanded"), album_key("Electro-Shock Blues"));
        assert_eq!(album_key("Ici et là‐bas"), album_key("Ici et la bas"));
        assert_eq!(album_key("Hall & Oates"), album_key("Hall and Oates"));
        // a parenthesis that is part of the title stays
        assert_ne!(album_key("(I Can't Get No) Satisfaction"), album_key("Satisfaction"));
        assert_ne!(album_key("Blinking Lights (and Other Revelations)"), album_key("Blinking Lights"));
        assert!(is_edition("Romance (Deluxe Edition)"));
        assert!(!is_edition("Romance"));
    }

    /// 0028 §5, case by case.
    #[test]
    fn each_release_finds_the_edition_that_plays_it() {
        let mut linked = release("Dogrel", "album", "2019-04-12");
        linked.spotify = Some("linked".into());
        let releases = vec![
            // the remaster of a 1967 album, dated 2012 by Spotify: the
            // title is unique, the date does not matter
            release("The Velvet Underground & Nico", "album", "1967-03-12"),
            // the original ahead of the deluxe
            release("Romance", "album", "2024-08-23"),
            // MusicBrainz's own link wins, whatever Spotify lists
            linked,
            // the single and the album of one title: each its own kind
            release("White Light/White Heat", "single", "1967-11"),
            release("White Light/White Heat", "album", "1968-01-30"),
            // only the deluxe exists: it plays
            release("Shadow People", "album", "2018-01-26"),
            // nothing on Spotify: it does not play
            release("Evil Mothers", "album", "1974"),
        ];
        let albums = vec![
            album("vu-remaster", "The Velvet Underground & Nico (45th Anniversary)", "2012-01-01", false),
            album("romance-deluxe", "Romance (Deluxe Edition)", "2025-03-01", false),
            album("romance", "Romance", "2024-08-23", false),
            album("dogrel-other", "Dogrel", "2019-04-12", false),
            album("wlwh-album", "White Light/White Heat", "1968-01-30", false),
            album("wlwh-single", "White Light / White Heat", "1967-11-01", true),
            album("shadow-deluxe", "Shadow People (Deluxe)", "2019-05-01", false),
        ];
        let matched: HashMap<usize, String> = match_albums(&releases, &albums).into_iter().collect();
        assert_eq!(matched[&0], "vu-remaster");
        assert_eq!(matched[&1], "romance");
        assert_eq!(matched[&2], "linked");
        assert_eq!(matched[&3], "wlwh-single");
        assert_eq!(matched[&4], "wlwh-album");
        assert_eq!(matched[&5], "shadow-deluxe");
        assert!(!matched.contains_key(&6));
    }

    /// The whole edition plays, dated and titled by the release.
    #[test]
    fn a_matched_album_gives_its_whole_edition() {
        let album = serde_json::json!({ "tracks": { "items": [
            { "name": "Sweet Jane", "uri": "spotify:track:a", "track_number": 1, "disc_number": 1, "is_playable": true },
            { "name": "Sweet Jane - Demo", "uri": "spotify:track:b", "track_number": 1, "disc_number": 2 },
            { "name": "Gone", "uri": "spotify:track:c", "track_number": 2, "disc_number": 2, "is_playable": false },
        ]}});
        let tail = tail_of(&release("Loaded", "album", "1970-11"), &album);
        assert_eq!(tail.len(), 2);
        assert!(tail.iter().all(|t| t.album == "Loaded" && t.released == "1970-11" && !t.single));
        assert_eq!((tail[1].disc, tail[1].number), (2, 1));
    }

    /// An old harvest becomes a tail by the releases, with no call: what
    /// MusicBrainz does not list — a compilation — stays out.
    #[test]
    fn an_old_harvest_is_converted_without_spotify() {
        let old = vec![
            TailTrack { album: "Loaded (Fully Loaded Edition)".into(), released: "2015".into(), ..track("Sweet Jane", "spotify:track:a", "2015", false) },
            TailTrack { album: "The Best of".into(), released: "2003".into(), ..track("Heroin", "spotify:track:b", "2003", false) },
        ];
        let mut loaded = release("Loaded", "album", "1970-11");
        loaded.spotify = Some("an-id-the-old-harvest-cannot-meet".into());
        let tail = from_legacy(&[loaded], &old);
        assert_eq!(tail.len(), 1);
        assert_eq!((tail[0].uri.as_str(), tail[0].album.as_str(), tail[0].released.as_str()), ("spotify:track:a", "Loaded", "1970-11"));
    }

    #[test]
    fn variants_of_the_same_title_merge() {
        let a = normalize("A Forest");
        assert_eq!(a, normalize("A Forest - 2005 Remaster"));
        assert_eq!(a, normalize("A Forest (Remastered)"));
        assert_eq!(a, normalize("a  forest!"));
        // but two different titles stay different
        assert_ne!(a, normalize("A Forest Fire"));
        assert_ne!(normalize("Lullaby"), normalize("Lovesong"));
    }
}
