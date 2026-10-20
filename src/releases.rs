//! **An artist's releases, from MusicBrainz** — the discography as catalog
//! material ([0027](../docs/decisions/0027-the-discography-is-catalog-material.md),
//! [0028](../docs/decisions/0028-the-discography-file-and-its-match.md),
//! [0029](../docs/decisions/0029-soundtracks-and-several-artists-in-a-discography.md)).
//!
//! `discography/<slug>.toml` says what exists: one `[[release]]` per release
//! group that is an album, an EP or a single, with no secondary type but
//! `Soundtrack`, its tracks taken from the group's earliest official
//! release — a group with none (promos, bootlegs) is left out. MusicBrainz
//! data only — what plays it on Spotify is matched on
//! each machine, never written here (`discography.rs` keeps that cache).
//!
//! `forkstify discography [catalog]` harvests every card that has no file
//! yet, and commits by batches of 25. A card with nothing kept gets no
//! file, so the next harvest tries it again.
//!
//! Everything here blocks, at MusicBrainz's one request per second.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Artists written per commit (0028 §2).
const BATCH: usize = 25;

/// The secondary type a kept release group may carry (0029 §1): a
/// soundtrack is a work; a compilation, a live or a remix is a repackaging.
const ALLOWED_SECONDARY: &str = "Soundtrack";

/// A medium that is not a disc of music: the bonus DVD of a deluxe edition
/// is not part of the album.
const VIDEO_FORMATS: [&str; 4] = ["DVD", "Blu-ray", "VHS", "Video"];

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Track {
    /// Only on a release with more than one disc (0028 §1).
    #[serde(default)]
    pub disc: Option<u32>,
    pub n: u32,
    pub title: String,
    pub mbid: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Release {
    /// The release group's.
    pub mbid: String,
    pub title: String,
    /// `album`, `ep` or `single`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The group's first release date, as precise as MusicBrainz has it.
    #[serde(default)]
    pub date: String,
    /// A Spotify album id, only when MusicBrainz itself links one (0027 §3).
    #[serde(default)]
    pub spotify: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
}

/// What a file says, as the player reads it: the releases. `artists` is
/// the harvest's business.
#[derive(Deserialize)]
struct File {
    #[serde(default)]
    release: Vec<Release>,
}

/// An artist's releases from the catalog — `None` when the card has no
/// file yet, or one that does not read.
pub fn read(catalog: &Path, slug: &str) -> Option<Vec<Release>> {
    let text = std::fs::read_to_string(path_of(catalog, slug)).ok()?;
    toml::from_str::<File>(&text).ok().map(|f| f.release)
}

#[derive(Debug, Default, PartialEq)]
pub struct Discography {
    /// The MusicBrainz artists gathered, `(mbid, name)`: the card's first,
    /// then its projects (0029 §2).
    pub artists: Vec<(String, String)>,
    pub releases: Vec<Release>,
}

pub fn path_of(catalog: &Path, slug: &str) -> PathBuf {
    catalog.join("discography").join(format!("{slug}.toml"))
}

/// The bands an artist is a member of whose name begins with the
/// artist's — *Michel Cloup Duo* for Michel Cloup (0029 §3). From an
/// `artist/{mbid}?inc=artist-rels` lookup.
fn projects(artist: &Value) -> Vec<(String, String)> {
    let own = crate::generate::fold_text(artist["name"].as_str().unwrap_or(""));
    if own.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<(String, String)> = artist["relations"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|r| r["type"].as_str() == Some("member of band") && r["direction"].as_str() == Some("forward"))
        .filter_map(|r| {
            let band = &r["artist"];
            let name = band["name"].as_str()?;
            let id = band["id"].as_str()?;
            let folded = crate::generate::fold_text(name);
            (folded != own && folded.starts_with(&own)).then(|| (id.to_string(), name.to_string()))
        })
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The release groups of one browse page that 0027 §2 and 0029 §1 keep,
/// without their tracks yet.
fn kept_groups(page: &Value) -> Vec<Release> {
    page["release-groups"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|group| {
            let kind = match group["primary-type"].as_str()? {
                "Album" => "album",
                "EP" => "ep",
                "Single" => "single",
                _ => return None,
            };
            let secondary = group["secondary-types"].as_array().map(Vec::as_slice).unwrap_or(&[]);
            if secondary.iter().any(|t| t.as_str() != Some(ALLOWED_SECONDARY)) {
                return None;
            }
            let spotify = spotify_link(group);
            Some(Release {
                mbid: group["id"].as_str()?.to_string(),
                title: group["title"].as_str()?.to_string(),
                kind: kind.to_string(),
                date: group["first-release-date"].as_str().unwrap_or("").to_string(),
                spotify,
                tracks: Vec::new(),
            })
        })
        .collect()
}

/// The Spotify album an entity's url relations point at, if any.
fn spotify_link(entity: &Value) -> Option<String> {
    entity["relations"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|r| r["url"]["resource"].as_str())
        .find_map(spotify_album)
}

/// MusicBrainz links Spotify to **releases**, rarely to their group
/// (Fontaines D.C., 2026-10-20: 27 of 70 releases, none of the groups):
/// a group takes the link of its earliest release that has one.
fn spotify_of_groups(pages: &[Value], kept: &HashMap<String, usize>) -> HashMap<String, String> {
    let mut found: HashMap<String, (String, String)> = HashMap::new();
    for release in pages.iter().flat_map(|p| p["releases"].as_array().map(Vec::as_slice).unwrap_or(&[])) {
        let (Some(group), Some(link)) = (release["release-group"]["id"].as_str(), spotify_link(release)) else {
            continue;
        };
        if !kept.contains_key(group) {
            continue;
        }
        let date = release["date"].as_str().unwrap_or("").to_string();
        if found.get(group).map_or(true, |(best, _)| date_key(&date) < date_key(best)) {
            found.insert(group.to_string(), (date, link));
        }
    }
    found.into_iter().map(|(group, (_, link))| (group, link)).collect()
}

/// `https://open.spotify.com/album/<id>?si=…` → `<id>`.
fn spotify_album(url: &str) -> Option<String> {
    let id = url.split("open.spotify.com/album/").nth(1)?;
    let id = id.split(['?', '/', '#']).next()?;
    crate::generate::is_spotify_id(id).then(|| id.to_string())
}

/// The tracks of one release, its video media left out; a disc number
/// only when more than one disc of music is left.
fn tracks_of(release: &Value) -> Vec<Track> {
    let media: Vec<&Value> = release["media"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|m| {
            let format = m["format"].as_str().unwrap_or("");
            !VIDEO_FORMATS.iter().any(|v| format.contains(v))
        })
        .collect();
    let several = media.len() > 1;
    let mut tracks = Vec::new();
    for (index, medium) in media.iter().enumerate() {
        for track in medium["tracks"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let (Some(n), Some(title), Some(mbid)) =
                (track["position"].as_u64(), track["title"].as_str(), track["recording"]["id"].as_str())
            else {
                continue;
            };
            tracks.push(Track {
                disc: several.then_some(index as u32 + 1),
                n: n as u32,
                title: title.to_string(),
                mbid: mbid.to_string(),
            });
        }
    }
    tracks
}

/// A date that sorts: an undated release after every dated one.
fn date_key(date: &str) -> (bool, &str) {
    (date.is_empty(), date)
}

/// From one page of releases, the earliest official one of each kept
/// group, against what earlier pages already chose.
fn choose_earliest<'a>(page: &'a Value, kept: &HashMap<String, usize>, chosen: &mut HashMap<String, (String, &'a Value)>) {
    for release in page["releases"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
        let Some(group) = release["release-group"]["id"].as_str() else { continue };
        if !kept.contains_key(group) {
            continue;
        }
        let date = release["date"].as_str().unwrap_or("").to_string();
        let earlier = chosen
            .get(group)
            .map_or(true, |(best, _)| date_key(&date) < date_key(best));
        if earlier {
            chosen.insert(group.to_string(), (date, release));
        }
    }
}

/// Every page of a MusicBrainz browse, `key` naming the list in each.
fn browse(path: &str, key: &str, count: &str) -> Result<Vec<Value>, String> {
    let mut pages = Vec::new();
    let mut offset = 0;
    loop {
        let Some(page) = crate::generate::musicbrainz(&format!("{path}&limit=100&offset={offset}&fmt=json"))? else {
            return Ok(pages);
        };
        let got = page[key].as_array().map_or(0, Vec::len) as u64;
        let total = page[count].as_u64().unwrap_or(0);
        offset += got;
        pages.push(page);
        if got == 0 || offset >= total {
            return Ok(pages);
        }
    }
}

/// What MusicBrainz says an artist released, their projects gathered.
/// `Err` is a server that would not answer: nothing is written, the next
/// harvest tries again.
pub fn harvest(mbid: &str) -> Result<Discography, String> {
    let Some(artist) = crate::generate::musicbrainz(&format!("artist/{mbid}?inc=artist-rels&fmt=json"))? else {
        return Ok(Discography::default());
    };
    let mut artists = vec![(mbid.to_string(), artist["name"].as_str().unwrap_or("").to_string())];
    artists.extend(projects(&artist));

    let mut releases: Vec<Release> = Vec::new();
    for (id, _) in &artists {
        let groups = browse(
            &format!("release-group?artist={id}&type=album|ep|single&inc=url-rels"),
            "release-groups",
            "release-group-count",
        )?;
        let mut kept: Vec<Release> = groups
            .iter()
            .flat_map(kept_groups)
            .filter(|g| !releases.iter().any(|r| r.mbid == g.mbid))
            .collect();
        if kept.is_empty() {
            continue;
        }
        let index: HashMap<String, usize> = kept.iter().enumerate().map(|(i, g)| (g.mbid.clone(), i)).collect();
        let pages = browse(
            &format!("release?artist={id}&status=official&type=album|ep|single&inc=recordings+release-groups+url-rels"),
            "releases",
            "release-count",
        )?;
        let mut chosen = HashMap::new();
        for page in &pages {
            choose_earliest(page, &index, &mut chosen);
        }
        for (group, (_, release)) in &chosen {
            kept[index[group]].tracks = tracks_of(release);
        }
        for (group, link) in spotify_of_groups(&pages, &index) {
            kept[index[&group]].spotify.get_or_insert(link);
        }
        // a group with no official release — promos, bootlegs — was
        // never on sale: no track list to take, nothing Spotify plays
        releases.extend(kept.into_iter().filter(|g| chosen.contains_key(&g.mbid)));
    }
    sort(&mut releases);
    Ok(Discography { artists, releases })
}

/// 0027 §5: by date, then title — so two forks write the same lines.
fn sort(releases: &mut [Release]) {
    releases.sort_by(|a, b| {
        date_key(&a.date)
            .cmp(&date_key(&b.date))
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.mbid.cmp(&b.mbid))
    });
    for release in releases.iter_mut() {
        release.tracks.sort_by_key(|t| (t.disc, t.n));
    }
}

fn quoted(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

/// The file's text, deterministic: a fixed field order, `artists` only
/// when projects are gathered (0029 §2), one track per line.
pub fn to_toml(discography: &Discography) -> String {
    let mut out = String::from("format = 1\n");
    if discography.artists.len() > 1 {
        out.push_str("artists = [\n");
        for (mbid, name) in &discography.artists {
            out.push_str(&format!("  {},  # {name}\n", quoted(mbid)));
        }
        out.push_str("]\n");
    }
    for release in &discography.releases {
        out.push_str("\n[[release]]\n");
        out.push_str(&format!("mbid = {}\n", quoted(&release.mbid)));
        out.push_str(&format!("title = {}\n", quoted(&release.title)));
        out.push_str(&format!("type = {}\n", quoted(&release.kind)));
        if !release.date.is_empty() {
            out.push_str(&format!("date = {}\n", quoted(&release.date)));
        }
        if let Some(spotify) = &release.spotify {
            out.push_str(&format!("spotify = {}\n", quoted(spotify)));
        }
        out.push_str("tracks = [\n");
        for track in &release.tracks {
            let disc = track.disc.map(|d| format!("disc = {d}, ")).unwrap_or_default();
            out.push_str(&format!(
                "  {{ {disc}n = {}, title = {}, mbid = {} }},\n",
                track.n,
                quoted(&track.title),
                quoted(&track.mbid)
            ));
        }
        out.push_str("]\n");
    }
    out
}

/// Write a file whole or not at all: a harvest cut mid-write must not
/// leave a half file the next one would take for done.
fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let partial = path.with_extension("toml.partial");
    std::fs::write(&partial, text).map_err(|e| e.to_string())?;
    std::fs::rename(&partial, path).map_err(|e| e.to_string())
}

/// Write an artist's discography into the catalog; the path, for the
/// commit that carries it.
pub fn write_file(catalog: &Path, slug: &str, discography: &Discography) -> Result<PathBuf, String> {
    let path = path_of(catalog, slug);
    write(&path, &to_toml(discography)).map_err(|e| format!("discography not written ({e})"))?;
    Ok(path)
}

/// The names of the projects gathered beside the card's artist.
pub fn projects_of(discography: &Discography) -> Vec<String> {
    discography.artists.iter().skip(1).map(|(_, name)| name.clone()).collect()
}

/// `ad` on a card with no file yet (0029 §4): the file, committed alone
/// with `Forkstify: discography`. `Ok(None)` when nothing is kept.
pub fn commit_one(catalog: &Path, slug: &str, discography: &Discography) -> Result<Option<usize>, String> {
    if discography.releases.is_empty() {
        return Ok(None);
    }
    write_file(catalog, slug, discography)?;
    let projects = projects_of(discography);
    let with = if projects.is_empty() { String::new() } else { format!(" — with {}", projects.join(", ")) };
    let body = format!("- {slug} · {} release(s){with}", discography.releases.len());
    crate::sync::commit_folder(catalog, "discography", &format!("Add the discography of {slug}"), Some(&body), "discography")?;
    Ok(Some(discography.releases.len()))
}

/// One written file, as the commit's body lists it.
struct Written {
    slug: String,
    releases: usize,
    projects: Vec<String>,
}

fn commit(dir: &Path, batch: &[Written]) -> Result<(), String> {
    let subject = match batch {
        [one] => format!("Add the discography of {}", one.slug),
        _ => format!("Add the discographies of {} artists", batch.len()),
    };
    let body: Vec<String> = batch
        .iter()
        .map(|w| {
            let with = if w.projects.is_empty() { String::new() } else { format!(" — with {}", w.projects.join(", ")) };
            format!("- {} · {} release(s){with}", w.slug, w.releases)
        })
        .collect();
    crate::sync::commit_folder(dir, "discography", &subject, Some(&body.join("\n")), "discography").map(|_| ())
}

/// `forkstify discography [catalog] [--artist <slug>]…`: harvest every card
/// with no file yet — or only the ones named —, commit by batches.
pub fn run(dir: &Path, only: &[String]) -> Result<(), String> {
    let catalog = crate::catalog::Catalog::load(dir).map_err(|e| e.to_string())?;
    // what an interrupted harvest wrote and did not commit: it is
    // skipped below as done, so it goes in first
    if crate::sync::commit_folder(dir, "discography", "Add the discographies left by an interrupted harvest", None, "discography")? {
        println!("✓ committed what an interrupted harvest had written");
    }
    let mut slugs: Vec<&String> = catalog
        .cards
        .keys()
        .filter(|slug| only.is_empty() || only.contains(slug))
        .filter(|slug| !path_of(dir, slug).exists())
        .collect();
    slugs.sort();
    for wanted in only {
        if !catalog.cards.contains_key(wanted) {
            eprintln!("  ⏹ {wanted}: no such card");
        }
    }
    println!("{} card(s) without a discography", slugs.len());

    let (mut batch, mut written, mut empty) = (Vec::new(), 0, 0);
    let mut stopped = None;
    for slug in slugs {
        let Some(mbid) = catalog.cards[slug].mbid.as_deref() else {
            println!("  ~ {slug}: no mbid");
            continue;
        };
        let discography = match harvest(mbid) {
            Ok(d) => d,
            Err(why) => {
                stopped = Some(why);
                break;
            }
        };
        if discography.releases.is_empty() {
            empty += 1;
            println!("  · {slug}: nothing kept — no file, tried again next time");
            continue;
        }
        write_file(dir, slug, &discography)?;
        let projects = projects_of(&discography);
        let with = if projects.is_empty() { String::new() } else { format!(" — with {}", projects.join(", ")) };
        println!("  ✓ {slug}: {} release(s){with}", discography.releases.len());
        batch.push(Written { slug: slug.clone(), releases: discography.releases.len(), projects });
        written += 1;
        if batch.len() == BATCH {
            commit(dir, &batch)?;
            println!("  ✓ {} committed", batch.len());
            batch.clear();
        }
    }
    if !batch.is_empty() {
        commit(dir, &batch)?;
        println!("  ✓ {} committed", batch.len());
    }
    println!("✓ {written} discography(ies) written, {empty} artist(s) with nothing kept");
    match stopped {
        Some(why) => Err(format!("stopped — {why}; run it again to resume")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn soundtracks_are_kept_compilations_and_lives_are_not() {
        let page = json!({ "release-groups": [
            { "id": "a", "title": "Boom Boom", "primary-type": "Album", "secondary-types": [], "first-release-date": "2023-11-03" },
            { "id": "b", "title": "Thatcher's Not Dead", "primary-type": "Album", "secondary-types": ["Soundtrack"], "first-release-date": "2023-04-22" },
            { "id": "c", "title": "Live", "primary-type": "Album", "secondary-types": ["Live"] },
            { "id": "d", "title": "Best Of", "primary-type": "Album", "secondary-types": ["Compilation", "Soundtrack"] },
            { "id": "e", "title": "Chansons rares", "primary-type": "Other", "secondary-types": [] },
            { "id": "f", "title": "Caravan", "primary-type": "EP", "relations": [
                { "url": { "resource": "https://www.discogs.com/master/1" } },
                { "url": { "resource": "https://open.spotify.com/album/4LH4d3cOWNNsVw41Gqt2kv?si=x" } } ] },
        ]});
        let kept = kept_groups(&page);
        let ids: Vec<&str> = kept.iter().map(|r| r.mbid.as_str()).collect();
        assert_eq!(ids, ["a", "b", "f"]);
        assert_eq!(kept[2].kind, "ep");
        assert_eq!(kept[2].spotify.as_deref(), Some("4LH4d3cOWNNsVw41Gqt2kv"));
        assert_eq!(kept[0].spotify, None);
    }

    #[test]
    fn a_project_is_a_band_whose_name_begins_with_the_artists() {
        let artist = json!({ "name": "Michel Cloup", "relations": [
            { "type": "member of band", "direction": "forward", "artist": { "id": "duo", "name": "Michel Cloup Duo" } },
            { "type": "member of band", "direction": "forward", "artist": { "id": "dia", "name": "Diabologum" } },
            { "type": "member of band", "direction": "backward", "artist": { "id": "x", "name": "Michel Cloup Trio" } },
            { "type": "collaboration", "direction": "forward", "artist": { "id": "y", "name": "Michel Cloup & Friends" } },
        ]});
        assert_eq!(projects(&artist), [("duo".to_string(), "Michel Cloup Duo".to_string())]);
    }

    #[test]
    fn the_earliest_official_release_gives_the_tracks() {
        let kept: HashMap<String, usize> = [("g".to_string(), 0)].into();
        let page = json!({ "releases": [
            { "date": "2005-04-26", "release-group": { "id": "g" }, "media": [] },
            { "date": "", "release-group": { "id": "g" }, "media": [] },
            { "date": "2005-04", "release-group": { "id": "g" }, "media": [{ "tracks": [] }] },
            { "date": "1999", "release-group": { "id": "other" }, "media": [] },
        ]});
        let mut chosen = HashMap::new();
        choose_earliest(&page, &kept, &mut chosen);
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen["g"].0, "2005-04");
    }

    #[test]
    fn a_group_takes_the_spotify_link_of_its_earliest_release_that_has_one() {
        let kept: HashMap<String, usize> = [("g".to_string(), 0)].into();
        let link = |id: &str| json!([{ "url": { "resource": format!("https://open.spotify.com/album/{id}") } }]);
        let pages = [json!({ "releases": [
            { "date": "2019-04-12", "release-group": { "id": "g" }, "relations": [] },
            { "date": "2021-01-01", "release-group": { "id": "g" }, "relations": link("2222222222222222222222") },
            { "date": "2019-04-12", "release-group": { "id": "g" }, "relations": link("1111111111111111111111") },
            { "date": "2019", "release-group": { "id": "other" }, "relations": link("3333333333333333333333") },
        ]})];
        let found = spotify_of_groups(&pages, &kept);
        assert_eq!(found.len(), 1);
        assert_eq!(found["g"], "1111111111111111111111");
    }

    #[test]
    fn a_disc_number_only_with_several_discs_of_music() {
        let track = |n: u64, title: &str| json!({ "position": n, "title": title, "recording": { "id": title } });
        let one = json!({ "media": [
            { "format": "CD", "tracks": [track(1, "a"), track(2, "b")] },
            { "format": "DVD-Video", "tracks": [track(1, "clip")] },
        ]});
        assert!(tracks_of(&one).iter().all(|t| t.disc.is_none()));
        assert_eq!(tracks_of(&one).len(), 2);
        let two = json!({ "media": [
            { "format": "CD", "tracks": [track(1, "a")] },
            { "format": "CD", "tracks": [track(1, "b")] },
        ]});
        let discs: Vec<Option<u32>> = tracks_of(&two).iter().map(|t| t.disc).collect();
        assert_eq!(discs, [Some(1), Some(2)]);
    }

    fn release(mbid: &str, title: &str, date: &str) -> Release {
        Release {
            mbid: mbid.into(),
            title: title.into(),
            kind: "album".into(),
            date: date.into(),
            spotify: None,
            tracks: vec![
                Track { disc: Some(2), n: 1, title: "Dust of Ages".into(), mbid: "r3".into() },
                Track { disc: Some(1), n: 2, title: "Say \"Hi\"".into(), mbid: "r2".into() },
                Track { disc: Some(1), n: 1, title: "Theme".into(), mbid: "r1".into() },
            ],
        }
    }

    #[test]
    fn the_file_is_written_the_same_whatever_the_order_found() {
        let mut releases = vec![release("z", "Undated", ""), release("b", "Blinking Lights", "2005-04-26"), release("a", "Beautiful Freak", "1996")];
        sort(&mut releases);
        let discography = Discography {
            artists: vec![("m1".into(), "Eels".into()), ("m2".into(), "Eels Duo".into())],
            releases,
        };
        let text = to_toml(&discography);
        let mut shuffled = discography.releases.clone();
        shuffled.reverse();
        sort(&mut shuffled);
        assert_eq!(to_toml(&Discography { artists: discography.artists.clone(), releases: shuffled }), text);
        assert!(text.starts_with("format = 1\nartists = [\n  \"m1\",  # Eels\n  \"m2\",  # Eels Duo\n]\n"), "{text}");
        let titles: Vec<&str> = text.lines().filter(|l| l.starts_with("title = ")).collect();
        assert_eq!(titles, ["title = \"Beautiful Freak\"", "title = \"Blinking Lights\"", "title = \"Undated\""]);
        assert!(text.contains("  { disc = 1, n = 1, title = \"Theme\", mbid = \"r1\" },\n  { disc = 1, n = 2,"), "{text}");
        // and it reads back as TOML, quotes included
        let table: toml::Table = toml::from_str(&text).expect("valid TOML");
        assert_eq!(table["release"].as_array().unwrap().len(), 3);
    }

    /// `ad` on a card with no file: the file, alone in its commit, with
    /// the `discography` trailer — and nothing for an artist with nothing.
    #[test]
    fn a_discography_is_committed_alone() {
        let git = |dir: &Path, args: &[&str]| {
            let out = std::process::Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        if std::process::Command::new("git").arg("--version").output().is_err() {
            eprintln!("git absent — test skipped");
            return;
        }
        let dir = std::env::temp_dir().join(format!("forkstify-releases-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cards")).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["config", "user.name", "Test"]);
        git(&dir, &["config", "user.email", "test@example.net"]);
        std::fs::write(dir.join("cards/eels.toml"), "format = 1\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "seed"]);
        // something else in progress stays out of the commit
        std::fs::write(dir.join("cards/eels.toml"), "format = 1\nname = \"Eels\"\n").unwrap();

        let nothing = Discography { artists: vec![("m1".into(), "Eels".into())], releases: Vec::new() };
        assert_eq!(commit_one(&dir, "eels", &nothing), Ok(None));
        assert!(!path_of(&dir, "eels").exists());

        let eels = Discography { artists: vec![("m1".into(), "Eels".into())], releases: vec![release("a", "Beautiful Freak", "1996")] };
        assert_eq!(commit_one(&dir, "eels", &eels), Ok(Some(1)));
        let message = git(&dir, &["log", "-1", "--format=%B"]);
        assert!(message.starts_with("Add the discography of eels"), "{message}");
        assert!(message.contains(&crate::sync::trailer("discography")), "{message}");
        assert_eq!(git(&dir, &["show", "--name-only", "--format=", "HEAD"]), "discography/eels.toml");
        assert_eq!(git(&dir, &["status", "--porcelain"]), "M cards/eels.toml");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_written_file_reads_back() {
        let dir = std::env::temp_dir().join(format!("forkstify-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut freak = release("a", "Beautiful Freak", "1996");
        freak.spotify = Some("4LH4d3cOWNNsVw41Gqt2kv".into());
        let discography = Discography { artists: vec![("m1".into(), "Eels".into())], releases: vec![freak.clone()] };
        write_file(&dir, "eels", &discography).expect("written");
        let mut read_back = read(&dir, "eels").expect("read");
        sort(&mut read_back);
        let mut expected = vec![freak];
        sort(&mut expected);
        assert_eq!(read_back, expected);
        assert!(read(&dir, "nobody").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn one_artist_alone_writes_no_artists_field() {
        let discography = Discography { artists: vec![("m1".into(), "Eels".into())], releases: vec![release("a", "A", "1996")] };
        assert!(!to_toml(&discography).contains("artists"));
    }
}
