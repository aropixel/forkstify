//! **Generating a card on the fly** (decision 0016, settled 09/09/2026 —
//! see `docs/conception/generation-a-la-volee.md`).
//!
//! Same pipeline as `tools/generate-cards.py`, which seeded the 316 cards of
//! the reference: MusicBrainz for identity, dates, origin, genres and typed
//! relations; Deezer, keyless, for the top tracks and the similar artists.
//! "One pipeline, three moments" — seeding, importing, and now listening.
//!
//! Two deliberate departures from the script:
//!
//! * **A similar link is kept even when its target has no card.** The script
//!   drops those, because a link to nothing is dead weight; here it is a
//!   branch waiting to be generated, which is how the catalog grows along
//!   its own edges. MusicBrainz relations stay reserved for existing cards —
//!   otherwise every session musician would become a direction.
//! * **Nothing is cached on disk.** The script harvests thousands of cards
//!   and needs it; a listening session generates one card at a time.
//!
//! Everything here blocks (ureq, and MusicBrainz's one-request-per-second):
//! call it from `spawn_blocking`, never on the reactor.

use std::time::Duration;

/// MusicBrainz asks every client to identify itself and to stay under one
/// request per second. Both are honoured here — an application that gets
/// itself blocked takes the whole ecosystem's goodwill with it.
const AGENT: &str = concat!(
    "forkstify/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/aropixel/forkstify)"
);
const MB_PAUSE: Duration = Duration::from_millis(1100);

/// What the generation produced: the card as text, ready to be written and
/// committed, plus what a listener should be told about it.
pub struct Draft {
    pub slug: String,
    pub name: String,
    pub toml: String,
    pub tops: usize,
    pub links: usize,
    /// What the sources could not give. Said on screen, never fatal — the
    /// format makes everything but `format`/`name`/`mbid` optional.
    pub caveats: Vec<String>,
}

/// The catalog, reduced to what a background job may carry: the match key of
/// every card and the slug it belongs to. `Catalog` itself cannot cross into
/// `spawn_blocking`, and nothing else of it is needed here.
pub struct Known(Vec<(String, String)>);

impl Known {
    pub fn of(catalog: &crate::catalog::Catalog) -> Known {
        Known(catalog.cards.keys().map(|slug| (match_key(slug), slug.clone())).collect())
    }

    fn slug_of(&self, name: &str) -> Option<&str> {
        let key = match_key(&slugify(name));
        self.0.iter().find(|(k, _)| *k == key).map(|(_, slug)| slug.as_str())
    }
}

/// The catalog's slug rule (`docs/conception/catalogue.md`): ASCII, lower
/// case, words joined by hyphens. `&` becomes `and` so "Hall & Oates" and
/// "Hall and Oates" land on the same file.
pub fn slugify(name: &str) -> String {
    let expanded = name.replace('&', " and ");
    let folded: String = expanded
        .chars()
        .map(|c| match c {
            '\u{2019}' | '\u{2018}' | '\'' => ' ',
            c => fold(c),
        })
        .collect();
    folded
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/// Accented latin letters down to ASCII. Enough for the artist names the
/// two sources return; anything else falls out at the slug's separators.
fn fold(c: char) -> char {
    match c {
        'à'..='å' | 'À'..='Å' => 'a',
        'è'..='ë' | 'È'..='Ë' => 'e',
        'ì'..='ï' | 'Ì'..='Ï' => 'i',
        'ò'..='ö' | 'Ò'..='Ö' => 'o',
        'ù'..='ü' | 'Ù'..='Ü' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' | 'Ñ' => 'n',
        'ý' | 'ÿ' | 'Ý' => 'y',
        'ø' | 'Ø' => 'o',
        'æ' | 'Æ' => 'a',
        c => c,
    }
}

/// A slug spelled back out for the screen: « georges-moustaki » becomes
/// « Georges Moustaki ». It is all anyone knows of an artist without a card,
/// and it is also what MusicBrainz gets asked for.
pub fn pretty(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A slug reduced to what makes two spellings the same artist: "the" of the
/// head and the linking words go, separators go — so "gun-club" finds "The
/// Gun Club" and "j-p-nataf" finds "jp-nataf".
fn match_key(slug: &str) -> String {
    let mut words: Vec<&str> = slug.split('-').filter(|w| !matches!(*w, "and" | "et")).collect();
    if words.len() > 1 && words[0] == "the" {
        words.remove(0);
    }
    words.concat()
}

/// A GET that tells **« no »** from **« not now »**. MusicBrainz answers
/// 503 as soon as it is pressed — often, for a minute at a time — and
/// Deezer 429: that is not an absence, and giving up at once would fail a
/// generation that is perfectly possible. `Ok(None)` is an answer (404: the
/// artist is not there); `Err` is the server still busy after half a
/// minute, or unreachable — worth saying, and worth retrying later.
fn get(url: &str) -> Result<Option<serde_json::Value>, String> {
    let host = url.split('/').nth(2).unwrap_or("the server");
    const ATTEMPTS: u64 = 6;
    for attempt in 1..=ATTEMPTS {
        match ureq::get(url).set("User-Agent", AGENT).call() {
            Ok(response) => return Ok(response.into_json().ok()),
            Err(ureq::Error::Status(429 | 503, _)) if attempt < ATTEMPTS => {
                std::thread::sleep(Duration::from_secs(2 * attempt));
            }
            Err(ureq::Error::Status(429 | 503, _)) => {
                return Err(format!("{host} is busy — try again in a moment"));
            }
            // 404 and the rest are answers: the artist is not there
            Err(ureq::Error::Status(..)) => return Ok(None),
            Err(ureq::Error::Transport(e)) => return Err(format!("{host} unreachable ({e})")),
        }
    }
    Err(format!("{host} is busy — try again in a moment"))
}

fn musicbrainz(path: &str) -> Result<Option<serde_json::Value>, String> {
    std::thread::sleep(MB_PAUSE);
    get(&format!("https://musicbrainz.org/ws/2/{path}"))
}

/// Deezer's answers are all optional to a card (tops, neighbours, a
/// respelling): a silence is a silence.
fn deezer(path: &str) -> Option<serde_json::Value> {
    get(&format!("https://api.deezer.com/{path}")).ok().flatten()
}

fn encode(text: &str) -> String {
    crate::spotify::encode(text)
}

/// A MusicBrainz id as typed: 8-4-4-4-12 hex digits.
pub fn is_mbid(text: &str) -> bool {
    text.len() == 36
        && text.char_indices().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// Find the artist by name. A slug spelled back out lost its apostrophes
/// and accents — « Lojo » for Lo’Jo — and MusicBrainz's search does not
/// bridge that gap; Deezer's does, so when the first try finds nobody it
/// lends the real spelling, checked against the same key (Joel,
/// 09/09/2026). `Err` is a server that would not answer.
fn search_mbid(name: &str) -> Result<Option<String>, String> {
    if let Some(mbid) = mbid_among(name)? {
        return Ok(Some(mbid));
    }
    let wanted = match_key(&slugify(name));
    let respelled = deezer(&format!("search/artist?q={}&limit=5", encode(name))).and_then(|body| {
        body["data"].as_array()?.iter().find_map(|hit| {
            let hit = hit["name"].as_str()?;
            (hit != name && match_key(&slugify(hit)) == wanted).then(|| hit.to_string())
        })
    });
    match respelled {
        Some(other) => mbid_among(&other),
        None => Ok(None),
    }
}

/// One MusicBrainz search. The score alone is not enough — "destinys child"
/// must land on "Destiny's Child" — so the slugs are compared too.
fn mbid_among(name: &str) -> Result<Option<String>, String> {
    let Some(body) = musicbrainz(&format!("artist?query={}&limit=5&fmt=json", encode(name)))? else {
        return Ok(None);
    };
    let wanted = match_key(&slugify(name));
    for hit in body["artists"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
        if hit["score"].as_u64().unwrap_or(0) < 90 {
            break;
        }
        let Some(found) = hit["name"].as_str() else { continue };
        if match_key(&slugify(found)) == wanted {
            return Ok(hit["id"].as_str().map(String::from));
        }
    }
    Ok(None)
}

/// A typed relation of the format (0010), and who it points at.
struct Relation {
    kind: &'static str,
    name: String,
    begin: Option<String>,
    end: Option<String>,
}

struct Facts {
    name: String,
    kind: Option<String>,
    country: Option<String>,
    area: Option<String>,
    begin: Option<String>,
    end: Option<String>,
    genres: Vec<String>,
    relations: Vec<Relation>,
    spotify: Option<String>,
    deezer: Option<String>,
}

/// MusicBrainz relation → link type of format 1. Anything not listed is not
/// a kinship the format knows how to say.
fn relation_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "member of band" | "founder" => "member",
        "collaboration" | "supporting musician" => "collab",
        "instrumental supporting musician" | "vocal supporting musician" => "collab",
        "sibling" | "parent" | "married" => "family",
        _ => return None,
    })
}

/// Pull an id out of an external url of the artist — Spotify and Deezer both
/// hang off MusicBrainz's url relations, which spares two searches.
fn id_after(url: &str, marker: &str, numeric: bool) -> Option<String> {
    let rest = &url[url.find(marker)? + marker.len()..];
    let id: String = rest
        .chars()
        .take_while(|c| if numeric { c.is_ascii_digit() } else { c.is_ascii_alphanumeric() })
        .collect();
    (!id.is_empty()).then_some(id)
}

/// `Ok(None)`: MusicBrainz does not know this id. `Err`: it would not say.
fn facts(mbid: &str) -> Result<Option<Facts>, String> {
    let Some(body) = musicbrainz(&format!("artist/{mbid}?inc=genres+artist-rels+url-rels&fmt=json"))?
    else {
        return Ok(None);
    };

    let mut genres: Vec<(String, u64)> = body["genres"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|g| {
                    Some((g["name"].as_str()?.to_string(), g["count"].as_u64().unwrap_or(0)))
                })
                .collect()
        })
        .unwrap_or_default();
    genres.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let (mut relations, mut spotify, mut deezer_id) = (Vec::new(), None, None);
    for relation in body["relations"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
        let url = relation["url"]["resource"].as_str().unwrap_or("");
        if url.contains("open.spotify.com/artist/") {
            spotify = spotify.or_else(|| id_after(url, "open.spotify.com/artist/", false));
        }
        if url.contains("deezer.com") && url.contains("/artist/") {
            deezer_id = deezer_id.or_else(|| id_after(url, "/artist/", true));
        }
        if let (Some(target), Some(kind)) =
            (relation["artist"]["name"].as_str(), relation_kind(relation["type"].as_str().unwrap_or("")))
        {
            relations.push(Relation {
                kind,
                name: target.to_string(),
                begin: relation["begin"].as_str().map(String::from),
                end: relation["end"].as_str().map(String::from),
            });
        }
    }

    let ended = body["life-span"]["ended"].as_bool().unwrap_or(false);
    let Some(name) = body["name"].as_str() else { return Ok(None) };
    Ok(Some(Facts {
        name: name.to_string(),
        kind: body["type"].as_str().map(String::from),
        country: body["country"].as_str().map(String::from),
        area: body["begin-area"]["name"].as_str().map(String::from),
        begin: body["life-span"]["begin"].as_str().map(String::from),
        end: ended.then(|| body["life-span"]["end"].as_str().map(String::from)).flatten(),
        genres: genres.into_iter().map(|(name, _)| name).collect(),
        relations,
        spotify,
        deezer: deezer_id,
    }))
}

fn deezer_id_by_name(name: &str) -> Option<String> {
    let body = deezer(&format!("search/artist?q={}&limit=5", encode(name)))?;
    let wanted = match_key(&slugify(name));
    for hit in body["data"].as_array()? {
        if match_key(&slugify(hit["name"].as_str()?)) == wanted {
            return Some(hit["id"].as_u64()?.to_string());
        }
    }
    None
}

/// « Vesoul (2011 Remaster) » is still Vesoul. Only the remastering suffix
/// goes — a live or a version is a different recording and keeps its name.
fn clean_top(title: &str) -> String {
    let lower = title.to_lowercase();
    for opener in [" (", " [", " - "] {
        if let Some(at) = lower.rfind(opener) {
            if lower[at..].contains("master") {
                return title[..at].trim_end_matches([' ', '-']).to_string();
            }
        }
    }
    title.to_string()
}

/// The five top tracks and the neighbourhood, in two keyless calls.
fn tops_and_similar(deezer_id: &str) -> (Vec<String>, Vec<String>) {
    let mut tops: Vec<String> = Vec::new();
    if let Some(body) = deezer(&format!("artist/{deezer_id}/top?limit=10")) {
        for track in body["data"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let Some(title) = track["title"].as_str().map(clean_top) else { continue };
            if !tops.iter().any(|kept| kept.eq_ignore_ascii_case(&title)) {
                tops.push(title);
            }
            if tops.len() == 5 {
                break;
            }
        }
    }
    let similar = deezer(&format!("artist/{deezer_id}/related?limit=20"))
        .and_then(|body| {
            Some(
                body["data"]
                    .as_array()?
                    .iter()
                    .filter_map(|a| a["name"].as_str().map(String::from))
                    .collect(),
            )
        })
        .unwrap_or_default();
    (tops, similar)
}

const COUNTRIES: [(&str, &str); 1] = [("GB", "uk")];

/// The decade of the *band*'s beginning. For a person `begin` is a birth
/// date, which says nothing about when they played.
fn decade(begin: Option<&str>) -> Option<String> {
    let year: u32 = begin?.get(..4)?.parse().ok()?;
    Some(if year < 2000 {
        format!("{}s", year % 100 / 10 * 10)
    } else {
        format!("{}s", year / 10 * 10)
    })
}

fn compose_tags(facts: &Facts) -> Vec<String> {
    let mut tags: Vec<String> = facts.genres.iter().take(4).map(|g| slugify(g)).collect();
    if let Some(country) = &facts.country {
        let code = COUNTRIES
            .iter()
            .find(|(from, _)| *from == country)
            .map(|(_, to)| to.to_string())
            .unwrap_or_else(|| country.to_lowercase());
        tags.push(code);
    }
    if facts.kind.as_deref() != Some("Person") {
        if let Some(decade) = decade(facts.begin.as_deref()) {
            tags.push(decade);
        }
    }
    tags
}

struct Link {
    to: String,
    kind: String,
    note: Option<String>,
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Compose the card's text. Written by hand rather than serialized: a card
/// is read and corrected by humans (0002), and the shape below is the one
/// the 316 cards of the reference already have.
fn compose(
    name: &str,
    mbid: &str,
    facts: &Facts,
    tags: &[String],
    tops: &[String],
    links: &[Link],
) -> String {
    let mut lines = vec![
        "format = 1".to_string(),
        "generated = true".to_string(),
        format!("name = {}", quoted(name)),
        format!("mbid = {}", quoted(mbid)),
    ];
    let head: [(&str, Option<&str>); 4] = [
        ("spotify", facts.spotify.as_deref()),
        ("begin", facts.begin.as_deref().and_then(|d| d.get(..4))),
        ("end", facts.end.as_deref().and_then(|d| d.get(..4))),
        ("origin", facts.area.as_deref()),
    ];
    for (key, value) in head {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            lines.push(format!("{key} = {}", quoted(value)));
        }
    }

    let joined: Vec<String> = tags.iter().map(|t| quoted(t)).collect();
    lines.push(String::new());
    lines.push(format!("tags = [{}]", joined.join(", ")));

    if !tops.is_empty() {
        lines.push(String::new());
        lines.push("tops = [".to_string());
        lines.extend(tops.iter().map(|t| format!("  {},", quoted(t))));
        lines.push("]".to_string());
    }
    if !links.is_empty() {
        lines.push(String::new());
        lines.push("links = [".to_string());
        for link in links {
            let mut parts =
                vec![format!("to = {}", quoted(&link.to)), format!("type = {}", quoted(&link.kind))];
            if let Some(note) = &link.note {
                parts.push(format!("note = {}", quoted(note)));
            }
            lines.push(format!("  {{ {} }},", parts.join(", ")));
        }
        lines.push("]".to_string());
    }
    lines.join("\n") + "\n"
}

/// Generate the card of `slug`. `hint` is the artist's real name when the
/// caller has it (a Spotify search result); without one the slug is spelled
/// back out, which is how a link into the void names its target. `mbid`
/// is the id found by hand when the search by name fails (Joel,
/// 09/09/2026): it skips the search, and if MusicBrainz then stays silent
/// the card is born minimal — name, id, Deezer tops — rather than not at
/// all.
///
/// Blocking, and slow on purpose (MusicBrainz's rate limit): four network
/// calls, about three seconds.
pub fn draft(slug: &str, hint: Option<&str>, mbid: Option<&str>, known: &Known) -> Result<Draft, String> {
    let asked = match hint {
        Some(name) => name.to_string(),
        None => slug.replace('-', " "),
    };
    let mut caveats = Vec::new();
    let (mbid, facts) = match mbid {
        Some(mbid) => {
            let facts = match facts(mbid) {
                Ok(Some(facts)) => facts,
                Ok(None) => return Err(format!("MusicBrainz does not know the id {mbid}")),
                Err(why) => {
                    caveats.push(format!("{why}: minimal card, to review"));
                    Facts {
                        name: asked.clone(),
                        kind: None,
                        country: None,
                        area: None,
                        begin: None,
                        end: None,
                        genres: Vec::new(),
                        relations: Vec::new(),
                        spotify: None,
                        deezer: None,
                    }
                }
            };
            (mbid.to_string(), facts)
        }
        None => {
            let mbid = search_mbid(&asked)?.ok_or_else(|| {
                format!("\"{asked}\" not found on MusicBrainz — :generate {asked} <mbid> with an id found by hand")
            })?;
            let facts = facts(&mbid)?
                .ok_or_else(|| format!("MusicBrainz no longer knows the id {mbid}"))?;
            (mbid, facts)
        }
    };
    let deezer_id = facts.deezer.clone().or_else(|| deezer_id_by_name(&facts.name));
    let (tops, similar) = match &deezer_id {
        Some(id) => {
            let (tops, similar) = tops_and_similar(id);
            // a stale or duplicate MusicBrainz id points at an empty page:
            // ask Deezer by name before giving up on the tops
            if tops.is_empty() && facts.deezer.is_some() {
                match deezer_id_by_name(&facts.name).filter(|other| other != id) {
                    Some(other) => tops_and_similar(&other),
                    None => (tops, similar),
                }
            } else {
                (tops, similar)
            }
        }
        None => (Vec::new(), Vec::new()),
    };
    if tops.is_empty() {
        caveats.push("no tops: Deezer does not know this artist".to_string());
    }

    let mut links: Vec<Link> = Vec::new();
    // MusicBrainz relations first: factual, typed, and pointing at cards we
    // already have — a relation towards nobody is not a direction
    for relation in &facts.relations {
        let Some(target) = known.slug_of(&relation.name) else { continue };
        if target == slug || links.iter().any(|l| l.to == target) {
            continue;
        }
        let note = (relation.kind == "member").then(|| {
            let begin = relation.begin.as_deref().and_then(|d| d.get(..4)).unwrap_or("…");
            let end = relation.end.as_deref().and_then(|d| d.get(..4)).unwrap_or("…");
            format!("member ({begin}–{end})")
        });
        links.push(Link { to: target.to_string(), kind: relation.kind.to_string(), note });
    }
    // then Deezer's neighbourhood, kept whether or not the target has a card
    for name in &similar {
        if links.iter().filter(|l| l.kind == "similar").count() == 4 {
            break;
        }
        let target = known.slug_of(name).map(String::from).unwrap_or_else(|| slugify(name));
        if target == slug || target.is_empty() || links.iter().any(|l| l.to == target) {
            continue;
        }
        links.push(Link { to: target, kind: "similar".to_string(), note: None });
    }
    if links.is_empty() {
        caveats.push("no links: this card will only branch through its tags".to_string());
    }

    let tags = compose_tags(&facts);
    let toml = compose(&facts.name, &mbid, &facts, &tags, &tops, &links);
    Ok(Draft {
        slug: slug.to_string(),
        name: facts.name,
        toml,
        tops: tops.len(),
        links: links.len(),
        caveats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Network: « lojo » is a slug spelled back out, and MusicBrainz alone
    /// answers « Lojo Russo »; Deezer lends « Lo'jo ».
    #[test]
    #[ignore]
    fn un_slug_sans_apostrophe_se_retrouve_par_deezer() {
        assert_eq!(
            search_mbid("Lojo").expect("MusicBrainz").as_deref(),
            Some("a1c1fb23-38e0-4d7f-8fed-3c81fef5ad0f")
        );
    }

    #[test]
    fn un_mbid_se_reconnait_a_sa_forme() {
        assert!(is_mbid("db6107e1-f692-453a-ab0e-4566faaba298"));
        assert!(!is_mbid("Oai Star"));
        assert!(!is_mbid("db6107e1f692453aab0e4566faaba298"));
        assert!(!is_mbid("db6107e1-f692-453a-ab0e-4566faaba29g"));
    }

    #[test]
    fn le_slug_suit_la_regle_du_catalogue() {
        assert_eq!(slugify("Jacques Brel"), "jacques-brel");
        assert_eq!(slugify("Destiny's Child"), "destiny-s-child");
        assert_eq!(slugify("Hall & Oates"), "hall-and-oates");
        assert_eq!(slugify("Françoise Hardy"), "francoise-hardy");
        assert_eq!(slugify("Sigur Rós"), "sigur-ros");
    }

    /// C'est cette clé qui empêche de créer un doublon d'une fiche existante
    /// sous une autre orthographe.
    #[test]
    fn deux_orthographes_du_meme_artiste_ont_la_meme_cle() {
        assert_eq!(match_key("the-gun-club"), match_key("gun-club"));
        assert_eq!(match_key("j-p-nataf"), match_key("jp-nataf"));
        assert_eq!(match_key(&slugify("Hall & Oates")), match_key(&slugify("Hall and Oates")));
        assert_ne!(match_key("the-cure"), match_key("cure-the-band"));
    }

    #[test]
    fn seul_le_suffixe_de_remasterisation_tombe() {
        assert_eq!(clean_top("Vesoul (2011 Remaster)"), "Vesoul");
        assert_eq!(clean_top("Heroes - 2017 Remaster"), "Heroes");
        assert_eq!(clean_top("Amsterdam (Live, Olympia / 1964)"), "Amsterdam (Live, Olympia / 1964)");
        assert_eq!(clean_top("Ne me quitte pas"), "Ne me quitte pas");
    }

    #[test]
    fn la_decennie_est_celle_du_debut() {
        assert_eq!(decade(Some("1977-06-01")).as_deref(), Some("70s"));
        assert_eq!(decade(Some("2004")).as_deref(), Some("2000s"));
        assert_eq!(decade(None), None);
        assert_eq!(decade(Some("inconnu")), None);
    }

    fn facts_of(name: &str) -> Facts {
        Facts {
            name: name.to_string(),
            kind: Some("Group".to_string()),
            country: Some("GB".to_string()),
            area: Some("Crawley".to_string()),
            begin: Some("1976-04".to_string()),
            end: None,
            genres: vec!["Post-Punk".to_string(), "New Wave".to_string()],
            relations: Vec::new(),
            spotify: Some("7bu3H8JO7d0UbMoVzbo70s".to_string()),
            deezer: None,
        }
    }

    /// Le vrai risque d'une écriture à la main : produire un TOML que le
    /// chargeur du catalogue ne relit pas.
    #[test]
    fn la_fiche_composee_se_relit_comme_une_fiche() {
        let facts = facts_of("The Cure");
        let tags = compose_tags(&facts);
        let links = vec![
            Link { to: "siouxsie".into(), kind: "member".into(), note: Some("member (1979–1980)".into()) },
            Link { to: "joy-division".into(), kind: "similar".into(), note: None },
        ];
        let text = compose("The Cure", "abc", &facts, &tags, &["A Forest".into()], &links);
        let card: crate::catalog::Card = toml::from_str(&text).expect("lisible");
        assert_eq!(card.name, "The Cure");
        assert!(card.generated);
        assert_eq!(card.spotify.as_deref(), Some("7bu3H8JO7d0UbMoVzbo70s"));
        assert_eq!(card.tops, vec!["A Forest"]);
        assert_eq!(card.links.len(), 2);
        assert_eq!(card.tags, vec!["post-punk", "new-wave", "uk", "70s"]);
    }

    /// Un titre à guillemets ne doit pas casser la fiche — Deezer en rend.
    #[test]
    fn un_titre_retors_ne_casse_pas_la_fiche() {
        let facts = facts_of("Nirvana");
        let text = compose("Nirvana", "abc", &facts, &[], &["Smells Like \"Teen\" Spirit".into()], &[]);
        let card: crate::catalog::Card = toml::from_str(&text).expect("lisible");
        assert_eq!(card.tops, vec!["Smells Like \"Teen\" Spirit"]);
    }

    /// Une personne n'a pas de décennie de formation : `begin` est sa
    /// naissance, et « Jacques Brel, 20s » serait faux.
    #[test]
    fn une_personne_n_a_pas_de_tag_de_decennie() {
        let mut facts = facts_of("Jacques Brel");
        facts.kind = Some("Person".to_string());
        facts.country = Some("BE".to_string());
        facts.begin = Some("1929-04-08".to_string());
        assert_eq!(compose_tags(&facts), vec!["post-punk", "new-wave", "be"]);
    }

    /// Le pipeline en entier, contre les deux vraies sources. Ignoré par
    /// défaut — il demande le réseau et quelques secondes :
    /// `bin/test -- --ignored --nocapture le_pipeline`
    #[test]
    #[ignore]
    fn le_pipeline_compose_une_vraie_fiche() {
        let known = Known(vec![
            (match_key("georges-brassens"), "georges-brassens".to_string()),
            (match_key("serge-gainsbourg"), "serge-gainsbourg".to_string()),
        ]);
        let draft = draft("jacques-brel", None, None, &known).expect("Jacques Brel");
        println!("{}", draft.toml);
        assert_eq!(draft.name, "Jacques Brel");
        let card: crate::catalog::Card = toml::from_str(&draft.toml).expect("fiche lisible");
        assert!(card.generated);
        assert_eq!(card.spotify.as_deref(), Some("4RN2vlFWepLa46qQIU2PHs"));
        assert!(!card.tops.is_empty(), "des tops Deezer");
        // le voisin qui a une fiche est visé par son slug…
        assert!(card.links.iter().any(|l| l.to == "georges-brassens"), "{:?}", draft.toml);
        // …et le format se relit avec ses tags
        assert!(card.tags.contains(&"be".to_string()), "{:?}", card.tags);
    }

    #[test]
    fn les_identifiants_se_lisent_dans_les_urls() {
        assert_eq!(
            id_after("https://open.spotify.com/artist/4RN2vlFWepLa46qQIU2PHs", "open.spotify.com/artist/", false),
            Some("4RN2vlFWepLa46qQIU2PHs".to_string())
        );
        assert_eq!(
            id_after("https://www.deezer.com/artist/1039", "/artist/", true),
            Some("1039".to_string())
        );
        assert_eq!(id_after("https://fr.wikipedia.org/wiki/Brel", "/artist/", true), None);
    }
}
