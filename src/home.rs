//! The home screen — what `forkstify` shows when launched with nothing.
//!
//! Two states, as the mockups decide: a **not connected** screen as long as
//! an authorization is missing, and **home** afterwards. Home does not
//! degrade; it only exists once the authorizations are in place.
//!
//! It renders **before any connection**: the catalog and the learned are
//! local, so the screen shows up right away and the network only comes into
//! play when playing.
//!
//! Every block carries its reason in one line (brand rule: every automatic
//! decision explains itself). Numbering runs **across** the blocks, so that
//! picking a seed is the same gesture as picking a branch.

use crate::catalog::Catalog;
use crate::discography::Tail;
use crate::engine::Comfort;
use crate::keys::{self, Cmd};
use crate::learned::Learned;
use crate::tui::{Collection, CollectionRow, HomeView, Row, Tui};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// What was chosen to start. The seed can be either (Joel's call,
/// 05/09/2026): an artist starts a segment on itself, a track plays then
/// branches from its artist.
pub enum Choice {
    Artist(String),
    Track { slug: String, title: String },
}

/// The last journey, to "resume". Lives in the cache, not in the catalog:
/// it is session, not knowledge.
#[derive(Serialize, Deserialize, Clone)]
pub struct LastSession {
    pub slug: String,
    pub name: String,
    pub title: String,
    pub at: String,
}

fn last_path() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache"));
    base.join("forkstify").join("last.json")
}

pub fn remember(last: &LastSession) {
    let path = last_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(last) {
        let _ = std::fs::write(path, text);
    }
}

fn recall() -> Option<LastSession> {
    serde_json::from_str(&std::fs::read_to_string(last_path()).ok()?).ok()
}

/// How the collection is sorted. Three readings of the same list: what we
/// know best, alphabetical order, and what has not been played for a long
/// time.
#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Familiarity,
    Alphabetical,
    LastPlayed,
}

impl Sort {
    fn next(self) -> Sort {
        match self {
            Sort::Familiarity => Sort::Alphabetical,
            Sort::Alphabetical => Sort::LastPlayed,
            Sort::LastPlayed => Sort::Familiarity,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Sort::Familiarity => "familiarity",
            Sort::Alphabetical => "a-z",
            Sort::LastPlayed => "last played",
        }
    }
}

/// What the collection shows: by default **the liked** — here ("more
/// often", a ♥) or on Spotify (track, album, followed) — or the whole
/// catalog (Joel, 09/09/2026). `v`, the view, as in the discography.
#[derive(Clone, Copy, PartialEq)]
enum Scope {
    Liked,
    All,
}

impl Scope {
    fn next(self) -> Scope {
        match self {
            Scope::Liked => Scope::All,
            Scope::All => Scope::Liked,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Scope::Liked => "liked",
            Scope::All => "all",
        }
    }
}

/// "n days ago" in two characters, the way a TUI says it. Used by the home
/// collection and by the playlist notes.
pub(crate) fn age(days: Option<i64>) -> String {
    match days {
        None => "never".into(),
        Some(0) => "today".into(),
        Some(1) => "yday".into(),
        Some(d) if d < 14 => format!("-{d}d"),
        Some(d) if d < 60 => format!("-{}w", d / 7),
        Some(d) if d < 365 => format!("-{}mo", d / 30),
        Some(d) => format!("-{}y", d / 365),
    }
}

/// The whole collection: the catalog **and** the ranking, together. An
/// artist without a card is listed, but cannot serve as a seed — branches
/// come from the card, and saying so beats hiding it.
fn collection(
    catalog: &Catalog,
    learned: &Learned,
    sort: Sort,
    scope: Scope,
    filter: &str,
) -> Vec<(Option<String>, CollectionRow)> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut rows: Vec<(Option<String>, CollectionRow)> = Vec::new();

    for (slug, card) in &catalog.cards {
        seen.insert(card.name.to_lowercase());
        seen.insert(slug.clone());
        let days = learned.days_since(slug);
        rows.push((
            Some(slug.clone()),
            CollectionRow {
                familiarity: (learned.familiarity01(slug, &card.name) * 5.0).round() as u8,
                days,
                name: card.name.clone(),
                carded: true,
                age: age(days),
                neglected: days.is_some_and(|d| d >= 180),
            },
        ));
    }
    for name in learned.ranked_names() {
        // by name, or by slug: Spotify still says "Kanye West" where the
        // card, at `kanye-west`, is named "Ye" — one row, not two (Joel,
        // 10/09/2026)
        if seen.contains(&name.to_lowercase()) || seen.contains(&crate::generate::slugify(name)) {
            continue;
        }
        rows.push((
            None,
            CollectionRow {
                familiarity: (learned.familiarity01("", name) * 5.0).round() as u8,
                days: None,
                name: name.clone(),
                carded: false,
                age: "never".into(),
                neglected: false,
            },
        ));
    }

    match sort {
        Sort::Familiarity => rows.sort_by(|a, b| {
            b.1.familiarity.cmp(&a.1.familiarity).then_with(|| a.1.name.cmp(&b.1.name))
        }),
        Sort::Alphabetical => rows.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())),
        // most recently played first; those that never played close the
        // line, since they have no last play
        Sort::LastPlayed => rows.sort_by(|a, b| {
            match (a.1.days, b.1.days) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then_with(|| a.1.name.cmp(&b.1.name))
        }),
    }
    if scope == Scope::Liked {
        // an artist without a card is written under the slug of its name —
        // the same one the gestures use below
        rows.retain(|(slug, row)| {
            let slug = slug.clone().unwrap_or_else(|| crate::generate::slugify(&row.name));
            learned.liked(&slug, &row.name)
        });
    }
    if !filter.is_empty() {
        let needle = filter.to_lowercase();
        rows.retain(|(_, row)| row.name.to_lowercase().contains(&needle));
    }
    rows
}

/// What is authorized, read from disk without opening anything.
pub struct Status {
    pub librespot: bool,
    pub web: bool,
}

impl Status {
    pub fn read() -> Status {
        Status { librespot: crate::sound::has_credentials(), web: crate::spotify::has_refresh() }
    }
    pub fn connected(&self) -> bool {
        self.librespot && self.web
    }
}

/// A door in: a seed, and the sentence saying why it is there.
struct Entry {
    choice: Choice,
    label: String,
    /// Filled only when the seed is a **track**: the title goes first, the
    /// artist behind.
    artist: Option<String>,
    reason: String,
    preview: Vec<(String, String)>,
}


/// One door among all of them, drawn by weight: what the dial makes of
/// the artist's familiarity (0001). Never a zero weight — a door is
/// discouraged, never shut.
fn draw<'a>(
    doors: &[&'a Entry],
    catalog: &Catalog,
    learned: &Learned,
    comfort: Comfort,
    rng: &mut impl Rng,
) -> Option<&'a Entry> {
    let weights: Vec<f32> = doors
        .iter()
        .map(|entry| {
            let slug = match &entry.choice {
                Choice::Artist(slug) | Choice::Track { slug, .. } => slug,
            };
            let name = catalog.cards.get(slug).map(|c| c.name.as_str()).unwrap_or(slug);
            comfort.favours(learned.familiarity01(slug, name))
        })
        .collect();
    let dist = WeightedIndex::new(&weights).ok()?;
    doors.get(dist.sample(rng)).copied()
}

/// The not-connected screen. Two situations that look nothing alike: never
/// authorized, where both gestures need explaining; authorization lost,
/// which is a passage, not a wall.
pub fn disconnected_rows(status: &Status, catalog: &Catalog) -> Vec<Row> {
    let mut rows = Vec::new();
    if !status.librespot && !status.web {
        rows.push(Row::Rule("first launch".into()));
        rows.push(Row::Text(
            "no authorization given yet. two are needed, and they are independent.".into(),
        ));
        rows.push(Row::Text(String::new()));
        rows.push(Row::Text("  1 librespot — sound".into()));
        rows.push(Row::Dim(
            "     credentials come from the phone over zeroconf — nothing to type here.".into(),
        ));
        rows.push(Row::Text(format!(
            "     open spotify on the phone, \"available devices\",\n     then pick \"{}\" from the list.",
            crate::sound::DEVICE_NAME
        )));
        rows.push(Row::Text(String::new()));
        rows.push(Row::Text("  2 the web api — tracks".into()));
        rows.push(Row::Dim(
            "     an oauth authorization in the browser (ncspot client id, five scopes).".into(),
        ));
    } else {
        rows.push(Row::Rule("incomplete authorization".into()));
        if !status.librespot {
            rows.push(Row::Text("the phone credentials are missing.".into()));
            rows.push(Row::Text(format!(
                "on spotify: \"available devices\", then \"{}\".",
                crate::sound::DEVICE_NAME
            )));
        }
        if !status.web {
            rows.push(Row::Text(
                "the web api token has expired — forkstify will ask for it again on its own.".into(),
            ));
            rows.push(Row::Dim(
                "sound is not cut off; only tracks no longer resolve.".into(),
            ));
        }
    }
    rows.push(Row::Rule("meanwhile".into()));
    rows.push(Row::Dim(format!(
        "the catalog is local: {} cards, their vectors and the learned read offline.\nthis screen is not a dead end.",
        catalog.cards.len()
    )));
    rows.push(Row::Key {
        key: "b".into(),
        what: "dry run".into(),
        note: "branches show up, nothing plays".into(),
        wired: false,
    });
    rows.push(Row::Key {
        key: ":search".into(),
        what: "search a card in the catalog only".into(),
        note: String::new(),
        wired: false,
    });
    rows
}

/// The home doors, in the order comfort decides.
fn entries(catalog: &Catalog, learned: &Learned, comfort: Comfort) -> Vec<(String, Vec<Entry>)> {
    let mut familiar: Vec<(&String, f32)> = catalog
        .cards
        .iter()
        .filter(|(slug, _)| !learned.artist_is_banned(slug))
        .map(|(slug, card)| (slug, learned.familiarity01(slug, &card.name)))
        .collect();
    familiar.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut habitues: Vec<Entry> = familiar
        .iter()
        .filter(|(_, f)| *f > 0.0)
        .take(2)
        .map(|(slug, f)| {
            let card = &catalog.cards[*slug];
            Entry {
                choice: Choice::Artist((*slug).clone()),
                label: card.name.clone(),
                artist: None,
                reason: format!("familiarity {:.0}%", f * 100.0),
                preview: card
                    .tops
                    .iter()
                    .take(2)
                    .map(|top| (top.clone(), card.name.clone()))
                    .collect(),
            }
        })
        .collect();

    // a seed that is a track, not an artist — a liked track if there is one,
    // else a top of the most familiar artist
    let track = learned.liked_anywhere().into_iter().next().or_else(|| {
        familiar.first().and_then(|(slug, _)| {
            catalog.cards[*slug].tops.first().map(|t| ((*slug).clone(), t.clone()))
        })
    });
    if let Some((slug, title)) = track {
        if let Some(card) = catalog.cards.get(&slug) {
            habitues.push(Entry {
                choice: Choice::Track { slug: slug.clone(), title: title.clone() },
                label: title.clone(),
                artist: Some(card.name.clone()),
                reason: format!(
                    "a track, not an artist: it plays, then the branches fork from {}",
                    card.name
                ),
                preview: Vec::new(),
            });
        }
    }

    // the neglected only exist with usage; on day one, they are the cards
    // nothing has ever touched
    let neglected = learned.neglected(3.0, 90);
    let (second_title, second) = if neglected.is_empty() {
        let jamais: Vec<Entry> = familiar
            .iter()
            .rev()
            .filter(|(_, f)| *f == 0.0)
            .take(2)
            .map(|(slug, _)| {
                let card = &catalog.cards[*slug];
                Entry {
                    choice: Choice::Artist((*slug).clone()),
                    label: card.name.clone(),
                    artist: None,
                    reason: "in the catalog, never played".to_string(),
                    preview: card
                        .tops
                        .iter()
                        .take(1)
                        .map(|top| (top.clone(), card.name.clone()))
                        .collect(),
                }
            })
            .collect();
        ("never played", jamais)
    } else {
        let delaisses: Vec<Entry> = neglected
            .iter()
            .filter_map(|(slug, months)| {
                let card = catalog.cards.get(slug)?;
                Some(Entry {
                    choice: Choice::Artist(slug.clone()),
                    label: card.name.clone(),
                    artist: None,
                    reason: format!("last played {months} months ago"),
                    preview: card
                        .tops
                        .iter()
                        .take(1)
                        .map(|top| (top.clone(), card.name.clone()))
                        .collect(),
                })
            })
            .take(2)
            .collect();
        ("neglected", delaisses)
    };

    let habitues_bloc = (
        "your regulars".to_string(),
        habitues,
    );
    let second_bloc = (second_title.to_string(), second);

    // 0012 §4: comfort decides, there is no other setting — the regulars
    // first in the cocoon, the neglected first when opening up. The test
    // was left over from the old polarity (5 = cocoon since 06/09/2026):
    // it put the never-played on top of a cocoon (Joel, 11/09/2026)
    if comfort.value() <= 1 {
        vec![second_bloc, habitues_bloc]
    } else {
        vec![habitues_bloc, second_bloc]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Card;
    use std::collections::HashMap;

    fn catalog() -> Catalog {
        let card = |name: &str| Card {
            generated: false,
            name: name.into(),
            spotify: None,
            tags: Vec::new(),
            tops: vec![format!("{name} — one")],
            doors: Vec::new(),
            links: Vec::new(),
            begin: None,
            end: None,
            origin: None,
            description: None,
        };
        Catalog {
            cards: HashMap::from([("air".to_string(), card("Air")), ("idles".to_string(), card("IDLES"))]),
            proximities: HashMap::new(),
            vectors: HashMap::new(),
        }
    }

    /// ecran-d-accueil.md: the cocoon puts the regulars first, the
    /// exploration the neglected — it was the other way round.
    #[test]
    fn the_cocoon_puts_the_regulars_first() {
        let learned = Learned::blank();
        let titles = |comfort: u8| -> Vec<String> {
            entries(&catalog(), &learned, Comfort::new(comfort)).into_iter().map(|(t, _)| t).collect()
        };
        assert_eq!(titles(5)[0], "your regulars");
        assert_eq!(titles(3)[0], "your regulars");
        assert_eq!(titles(1)[0], "never played");
        assert_eq!(titles(0)[0], "never played");
    }
}

fn rows_of(
    learned: &Learned,
    blocks: &[(String, Vec<Entry>)],
) -> (Vec<Row>, usize) {
    let mut rows = Vec::new();
    if let Some(last) = recall() {
        rows.push(Row::Rule("resume".into()));
        // the title first, the artist behind — the same everywhere
        rows.push(Row::Key {
            key: "r".into(),
            what: format!("{} — {}", last.title, last.name),
            note: format!("interrupted {}", last.at),
            wired: true,
        });
    }

    rows.push(Row::Rule("search".into()));
    rows.push(Row::Key {
        key: ":search".into(),
        what: "an artist or a track".into(),
        note: "catalog and spotify".into(),
        wired: true,
    });
    rows.push(Row::Key {
        key: "/".into(),
        what: "filter the collection".into(),
        note: "on the right; esc clears".into(),
        wired: true,
    });
    rows.push(Row::Dim(
        "     enter starts a journey on what is chosen: the artist,\n     or the track, then the branches of its artist.".into(),
    ));

    let mut n = 0;
    for (title, block) in blocks {
        rows.push(Row::Rule(title.clone()));
        if title == "neglected" {
            rows.push(Row::Dim(
                "     a familiarity that was high and has faded — a reminder, not a discovery."
                    .into(),
            ));
        }
        for entry in block {
            n += 1;
            rows.push(Row::Entry {
                n,
                label: entry.label.clone(),
                artist: entry.artist.clone(),
                reason: entry.reason.clone(),
                tracks: entry.preview.clone(),
            });
        }
    }
    let _ = learned;

    rows.push(Row::Rule("random".into()));
    rows.push(Row::Key {
        key: "enter".into(),
        what: "weighted draw by the comfort zone".into(),
        note: "the door that does not ask you to choose".into(),
        wired: true,
    });
    (rows, n)
}

/// Home. Renders, reads a key, and says what to start.
/// What home answers to a key.
pub enum Outcome {
    Stay,
    /// Start a journey — it replaces the one playing, if any.
    Start(Choice),
    /// Go back to the current session screen, changing nothing.
    Back,
    /// Open the search modal, empty or already filled — it lives on the
    /// session screen, which holds it for both screens (Joel, 09/09/2026).
    Find(String),
    /// `:generate <name>` — bring in an artist missing from the catalog, then
    /// start from them ([0016]).
    Generate(String),
    /// `ad` on the highlighted line: its discography, as a modal over home
    /// (Joel, 10/09/2026).
    Explore { slug: String, name: String },
    /// Any other `a` gesture on the highlighted line (`ag`, `ae`, `aL`): the
    /// session does it, with the same code as when listening. `slug` is
    /// missing when the artist has no card.
    Artist { key: char, slug: Option<String>, name: String },
    Quit,
}

/// Home is a **session screen**, not a separate loop (Joel, 08/09/2026):
/// `q` comes back to it from listening, listening goes on underneath, and
/// `r` or esc return to the session screen. This state is what moves on
/// home between two keys.
pub struct Home {
    /// what is being typed: the only thing that moves at the bottom
    typed: String,
    said: String,
    sort: Sort,
    scope: Scope,
    /// the collection cursor: as long as it does not exist, enter keeps its
    /// usual meaning — "choose for me"
    cursor: Option<usize>,
    /// `/text` filters the collection (Joel, 08/09/2026); esc clears it.
    filter: String,
}

impl Default for Home {
    fn default() -> Self {
        Home {
            typed: String::new(),
            said: String::new(),
            sort: Sort::Familiarity,
            scope: Scope::Liked,
            cursor: None,
            filter: String::new(),
        }
    }
}

impl Home {
    /// What home has to say after a key. The session picks it up after
    /// every gesture and shows it as a toast: **every notification is a
    /// toast** (Joel, 10/09/2026), the bottom line only carries the input
    /// and the keys.
    pub fn take_said(&mut self) -> Option<String> {
        if self.said.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.said))
        }
    }

    /// Draw home. `bar` is the playback footer, when a session plays
    /// underneath; `live` says whether there is a session to go back to.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        catalog: &Catalog,
        learned: &Learned,
        tail: &Tail,
        comfort: Comfort,
        comfort_mode: bool,
        status: &[(String, bool)],
        bar: Option<crate::tui::Bar<'_>>,
        live: bool,
        finder: Option<crate::tui::FinderView>,
        explore: Option<&crate::explore::Explore>,
        overlay: Option<(&str, &[String])>,
        toast: Option<crate::tui::Toast>,
        tui: &mut Tui,
    ) {
        let listing = collection(catalog, learned, self.sort, self.scope, &self.filter);
        let shelf: Vec<CollectionRow> = listing.iter().map(|(_, row)| row.clone()).collect();
        let carded = listing.iter().filter(|(slug, _)| slug.is_some()).count();
        let blocks = entries(catalog, learned, comfort);
        let (rows, count) = rows_of(learned, &blocks);
        let _ = tui.draw_home(&HomeView {
            status: status.to_vec(),
            census: format!(
                "local catalog — {} cards · {} ranked artists · {} discography(ies) cached",
                catalog.cards.len(),
                learned.seeded(),
                tail.known()
            ),
            rows: &rows,
            prompt: if !self.typed.is_empty() {
                self.typed.clone()
            } else if !self.filter.is_empty() {
                format!(
                    "[filter \"{}\" — {} artist(s) · ↑↓ enter · esc clears · :search <text> searches]",
                    self.filter,
                    listing.len()
                )
            } else if live {
                format!(
                    "[1-{count} to start · r back to playing · /filter · :search · enter random · c<n> · q quit]"
                )
            } else {
                format!(
                    "[1-{count} to start · r resume · /filter · :search · enter random · c<n> · q]"
                )
            },
            comfort: comfort.value(),
            comfort_mode,
            comfort_word: crate::listen::comfort_word(comfort.value()),
            collection: Some(Collection {
                rows: &shelf,
                total: listing.len(),
                carded,
                cursor: self.cursor,
                sort: self.sort.label(),
                scope: self.scope.label(),
            }),
            bar,
            finder,
            explore,
            overlay,
            toast,
        });
    }

    /// A key on home. The lists are recomputed on every key: they are
    /// small, and that is what guarantees we choose among what is shown.
    pub fn on_cmd(
        &mut self,
        cmd: Cmd,
        catalog: &Catalog,
        learned: &mut Learned,
        comfort: &mut Comfort,
        live: bool,
    ) -> Outcome {
        match &cmd {
            Cmd::Pending(seq) => {
                self.typed = seq.clone();
                return Outcome::Stay;
            }
            Cmd::Typing(line) => {
                self.typed = line.clone().unwrap_or_default();
                return Outcome::Stay;
            }
            Cmd::Unknown(seq) => {
                self.typed.clear();
                self.said = format!("(unknown: {seq})");
                return Outcome::Stay;
            }
            _ => {
                self.typed.clear();
                self.said.clear();
            }
        }
        let listing = collection(catalog, learned, self.sort, self.scope, &self.filter);
        let blocks = entries(catalog, learned, *comfort);
        let flat: Vec<&Entry> = blocks.iter().flat_map(|(_, b)| b.iter()).collect();
        let pick = |entry: &Entry| match &entry.choice {
            Choice::Artist(slug) => Choice::Artist(slug.clone()),
            Choice::Track { slug, title } => Choice::Track { slug: slug.clone(), title: title.clone() },
        };
        match cmd {
            Cmd::Quit => Outcome::Quit,
            Cmd::Digit(n) => match flat.get(n - 1) {
                Some(entry) => Outcome::Start(pick(entry)),
                None => {
                    self.said = format!("(no entry {n})");
                    Outcome::Stay
                }
            },
            // `r` goes back to the current listening; without one, it resumes
            // the last session
            Cmd::Resume if live => Outcome::Back,
            Cmd::Resume => match recall() {
                Some(last) => Outcome::Start(Choice::Track { slug: last.slug, title: last.title }),
                None => {
                    self.said = "(no journey to resume)".into();
                    Outcome::Stay
                }
            },
            // the collection cursor takes precedence: enter starts what is
            // under it, otherwise it keeps its usual meaning
            Cmd::Auto if self.cursor.is_some() => {
                let index = self.cursor.unwrap();
                match listing.get(index) {
                    Some((Some(slug), _)) => Outcome::Start(Choice::Artist(slug.clone())),
                    // no card: the session generates it, then starts from the
                    // artist — landing on an artist means making them a card
                    // (0016; Joel, 10/09/2026, on Kanye West)
                    Some((None, row)) => Outcome::Generate(row.name.clone()),
                    None => Outcome::Stay,
                }
            }
            // al / as / ab on the highlighted line — 0020: on home, the
            // highlighted line, or nothing. `as` takes the artist out of the
            // liked and writes it in learned/, where it overrides Spotify
            // (Joel, 09/09/2026)
            // ad: the discography of the highlighted line, if it has a card
            Cmd::Artist('d') => {
                let Some(index) = self.cursor else {
                    self.said = "(nothing highlighted — ↑↓ to choose)".into();
                    return Outcome::Stay;
                };
                match listing.get(index) {
                    Some((Some(slug), row)) => Outcome::Explore { slug: slug.clone(), name: row.name.clone() },
                    // no card: the session generates it, then opens (0016)
                    Some((None, row)) => Outcome::Artist { key: 'd', slug: None, name: row.name.clone() },
                    None => Outcome::Stay,
                }
            }
            Cmd::Artist(key) if !matches!(key, 'l' | 's' | 'b') => {
                let Some(index) = self.cursor else {
                    self.said = "(nothing highlighted — ↑↓ to choose)".into();
                    return Outcome::Stay;
                };
                match listing.get(index) {
                    Some((slug, row)) => Outcome::Artist { key, slug: slug.clone(), name: row.name.clone() },
                    None => Outcome::Stay,
                }
            }
            Cmd::Artist(key @ ('l' | 's' | 'b')) => {
                let Some(index) = self.cursor else {
                    self.said = "(nothing highlighted — ↑↓ to choose)".into();
                    return Outcome::Stay;
                };
                let Some((slug, row)) = listing.get(index) else { return Outcome::Stay };
                let slug = slug.clone().unwrap_or_else(|| crate::generate::slugify(&row.name));
                let name = row.name.clone();
                self.said = match key {
                    'l' => format!("↑ {name} — more often (weight {:.2})", learned.like_artist(&slug)),
                    's' => format!(
                        "↓ {name} — less often, out of the liked (weight {:.2}) · al to come back",
                        learned.skip_artist(&slug)
                    ),
                    _ => {
                        learned.ban_artist(&slug);
                        format!("⊘ {name} — never again")
                    }
                };
                // the line may have left the view: the cursor stays on a line
                let left = collection(catalog, learned, self.sort, self.scope, &self.filter).len();
                self.cursor = (left > 0).then(|| index.min(left - 1));
                Outcome::Stay
            }
            // enter, nothing highlighted: "choose for me" — a draw over
            // every door on the page, weighted by the dial: the familiar
            // in the cocoon, the unknown wide open (ecran-d-accueil.md).
            // It took the first line until 11/09/2026 (Joel: "c'est faux").
            Cmd::Auto => match draw(&flat, catalog, learned, *comfort, &mut rand::thread_rng()) {
                Some(entry) => Outcome::Start(pick(entry)),
                None => Outcome::Stay,
            },
            // `/` filters the collection; searching is `:search`
            Cmd::Search(query) => {
                self.filter = query.trim().to_string();
                self.cursor = if self.filter.is_empty() { None } else { Some(0) };
                Outcome::Stay
            }
            Cmd::Comfort(n) => {
                *comfort = Comfort::new(n);
                self.said = format!("comfort {n} — {}", crate::listen::comfort_word(n));
                Outcome::Stay
            }
            Cmd::Colon(text) => {
                let mut words = text.split_whitespace();
                match (words.next(), words.next()) {
                    (Some("comfort"), Some(v)) => {
                        if let Ok(v) = v.parse::<u8>() {
                            if v <= 5 {
                                *comfort = Comfort::new(v);
                            }
                        }
                        Outcome::Stay
                    }
                    // the same modal as when listening, opened from home:
                    // `:search` alone opens it empty, `:search <text>` fills
                    // it (Joel, 09/09/2026)
                    (Some("search"), _) => {
                        Outcome::Find(text.trim().trim_start_matches("search").trim().to_string())
                    }
                    (Some("generate"), Some(_)) => Outcome::Generate(
                        text.trim().trim_start_matches("generate").trim().to_string(),
                    ),
                    _ => Outcome::Stay,
                }
            }
            Cmd::Up => {
                let here = self.cursor.unwrap_or(0);
                self.cursor = Some(here.saturating_sub(1));
                Outcome::Stay
            }
            Cmd::Down => {
                let here = self.cursor.map_or(0, |i| i + 1);
                self.cursor = Some(here.min(listing.len().saturating_sub(1)));
                Outcome::Stay
            }
            // both ends, as in vim
            Cmd::Top => {
                self.cursor = Some(0);
                Outcome::Stay
            }
            Cmd::Bottom => {
                self.cursor = Some(listing.len().saturating_sub(1));
                Outcome::Stay
            }
            // esc clears the filter, then gives up the cursor; with neither,
            // it gives back the session screen
            Cmd::Escape if !self.filter.is_empty() => {
                self.filter.clear();
                self.cursor = None;
                Outcome::Stay
            }
            Cmd::Escape if self.cursor.is_some() => {
                self.cursor = None;
                Outcome::Stay
            }
            Cmd::Escape if live => Outcome::Back,
            Cmd::Sort => {
                self.sort = self.sort.next();
                Outcome::Stay
            }
            Cmd::Filter => {
                self.scope = self.scope.next();
                self.cursor = None;
                Outcome::Stay
            }
            _ => Outcome::Stay,
        }
    }
}

/// The discovery loop, shown while the not-connected screen waits.
pub fn ask_phone() -> Result<String, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    rt.block_on(crate::sound::discover())
}

pub fn reader() -> tokio::sync::mpsc::UnboundedReceiver<Cmd> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Cmd>();
    keys::spawn_reader(tx);
    rx
}
