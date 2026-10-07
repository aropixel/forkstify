//! `:suggest` — the agent proposes connections, the listener decides
//! (`docs/design/agent.md`, 2026-10-06 and 07).
//!
//! The agent is Claude Code, called as a command: `claude -p` with **no
//! tool at all**, the prompt on its input, a JSON schema for its answer.
//! Text in, JSON out: it can name no artist that was not sent, and it
//! touches nothing. What it proposes waits in a list kept in local state —
//! which is also the cache: an artist with proposals waiting is not asked
//! about again until they are settled, or until what would be sent has
//! changed.
//!
//! This module is the logic only — the prompt, the call, the filter, the
//! list, the screen's state and its lines. `listen.rs` wires it to the
//! keys, the jobs and the writes.

use crate::catalog::{Card, Catalog, AUDIENCE};
use crate::learned::Learned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// The types the agent may propose: a reading, never a fact — `member`,
/// `collab` and `family` come from MusicBrainz, never from a language
/// model (`docs/design/catalog.md`).
pub const KINDS: [&str; 3] = ["similar", "scene", "influence"];

/// One proposal: tie `artist` to `anchor`, of that type, that close, for
/// that reason. `lean` is where the agent would put it — "card" or
/// "connection" —, the listener decides.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Proposal {
    pub artist: String,
    pub anchor: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub proximity: u8,
    pub lean: String,
    pub reason: String,
}

/// One artist a call asks about.
#[derive(Clone)]
pub struct Target {
    pub slug: String,
    /// What the answer depends on — the artist's line and the liked
    /// artists — not the exclusions: declining one proposal must not make
    /// the others look stale.
    pub fingerprint: String,
    /// Who it is already tied to, or was declined for: never an anchor,
    /// even if the shared candidates hold them.
    pub excluded: HashSet<String>,
}

/// What one call sends, and what its answer is checked against: one
/// artist (`aS`), or the orphans (`:suggest`).
#[derive(Clone)]
pub struct Ask {
    pub targets: Vec<Target>,
    pub prompt: String,
    pub candidates: HashSet<String>,
}

/// The answer's shape, handed to `claude --json-schema`.
pub const SCHEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["suggestions"],"properties":{"suggestions":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["artist","anchor","type","proximity","lean","reason"],"properties":{"artist":{"type":"string"},"anchor":{"type":"string"},"type":{"enum":["similar","scene","influence"]},"proximity":{"type":"integer","minimum":1,"maximum":5},"lean":{"enum":["card","connection"]},"reason":{"type":"string"}}}}}}"#;

/// The orphans' instructions, tried on Joel's catalog on 2026-10-07
/// (`agent.md`).
const ORPHANS: &str = "You suggest connections between artists for forkstify, a music player that plays by branches: from an artist, it moves to the artists linked to it.

Each ORPHAN below is an artist the listener likes, but no link leads from any other artist they like to it, so it hardly ever plays. For each orphan, choose one to three ANCHORS from the CANDIDATES list: artists the listener also likes and already reaches, whose music is genuinely close to the orphan's. A good anchor is one a listener who loves the anchor would be glad to hear the orphan right after.

Rules:
- Anchors must come from CANDIDATES, by their slug, exactly as written. Never name any other artist, and never one listed as declined for that orphan. In each suggestion, artist is the orphan's slug.
- Judge by the music: sound, songwriting, era, scene, lineage. The tags are hints from MusicBrainz and are sometimes wrong or missing; trust your knowledge of the artists over them. Ignore shared country or language alone.
- If no candidate is genuinely close, return no suggestion for that orphan. An empty answer is better than a weak one.
- type: \"similar\" (they sound alike, they go together), \"scene\" (same scene, same moment, same circle), or \"influence\" (one descends from the other). Never claim a fact (members, collaborations, family): those come from other sources.
- proximity, 1 to 5: 1 a distant echo, 2 an influence far back, 3 a family or a scene, 4 they go together, 5 almost the same universe.
- lean: \"card\" when the kinship is knowledge anyone could check, \"connection\" when it is more a matter of taste.
- reason: one short sentence in English, saying what the two share musically. No hedging, no filler, at most 15 words.";

/// The instructions for one artist, tried on 2026-10-07 (`agent.md`).
const INSTRUCTIONS: &str = "You suggest connections between artists for forkstify, a music player that plays by branches: from an artist, it moves to the artists linked to it.

The listener asked for suggestions around one TARGET artist. Its links today do not reach far enough into what they listen to. Choose up to five ANCHORS from the CANDIDATES list: artists the listener likes, not yet tied to the target, whose music is genuinely close to the target's. A good anchor is one a listener who loves the anchor would be glad to hear the target right after, and the other way round.

Rules:
- Anchors must come from CANDIDATES, by their slug, exactly as written. Never name any other artist. In each suggestion, artist is the target's slug.
- Judge by the music: sound, songwriting, era, scene, lineage. The tags are hints from MusicBrainz and are sometimes wrong or missing; trust your knowledge of the artists over them. Ignore shared country or language alone.
- Fewer is fine. If no candidate is genuinely close, return no suggestion. An empty answer is better than a weak one.
- type: \"similar\" (they sound alike, they go together), \"scene\" (same scene, same moment, same circle), or \"influence\" (one descends from the other). Never claim a fact (members, collaborations, family): those come from other sources.
- proximity, 1 to 5: 1 a distant echo, 2 an influence far back, 3 a family or a scene, 4 they go together, 5 almost the same universe.
- lean: \"card\" when the kinship is knowledge anyone could check, \"connection\" when it is more a matter of taste.
- reason: one short sentence in English, saying what the two share musically. No hedging, no filler, at most 15 words.";

/// A card in one line, as the prompt shows it.
fn describe(slug: &str, card: &Card) -> String {
    let mut bits = vec![format!("{} [{slug}]", card.name)];
    if !card.tags.is_empty() {
        bits.push(format!("tags: {}", card.tags.join(", ")));
    }
    if let Some(origin) = card.origin.as_deref().filter(|o| !o.is_empty()) {
        bits.push(format!("from {origin}"));
    }
    let years: Vec<&str> = [card.begin.as_deref(), card.end.as_deref()].into_iter().flatten().filter(|y| !y.is_empty()).collect();
    if !years.is_empty() {
        bits.push(years.join("–"));
    }
    bits.join(" · ")
}

/// Who `slug` is tied to through the graph, both ways — card links and
/// the listener's connections, which the session weaves into the cards —
/// `audience` aside: a service's word ties nothing (0026).
fn tied(catalog: &Catalog, slug: &str) -> HashSet<String> {
    let mut out: HashSet<String> = catalog.cards[slug]
        .links
        .iter()
        .filter(|l| l.kind != AUDIENCE)
        .map(|l| l.to.clone())
        .collect();
    for (other, card) in &catalog.cards {
        if card.links.iter().any(|l| l.to == slug && l.kind != AUDIENCE) {
            out.insert(other.clone());
        }
    }
    out
}

/// The artists the listener likes that have a card, sorted.
fn liked(catalog: &Catalog, learned: &Learned) -> Vec<String> {
    let mut out: Vec<String> =
        catalog.cards.iter().filter(|(slug, card)| learned.liked(slug, &card.name)).map(|(slug, _)| slug.clone()).collect();
    out.sort();
    out
}

/// An orphan: liked, and tied to no other liked artist (`agent.md`).
fn orphan(catalog: &Catalog, liked: &HashSet<&String>, slug: &str) -> bool {
    !tied(catalog, slug).iter().any(|other| liked.contains(other))
}

/// What to send about `target`. The candidates are the liked artists not
/// yet tied to it, minus what was declined for it; when the target is an
/// orphan itself, minus the other orphans too — two orphans tied together
/// only make an island.
pub fn ask(catalog: &Catalog, learned: &Learned, target: &str) -> Result<Ask, String> {
    let card = catalog.cards.get(target).ok_or_else(|| format!("{target} has no card"))?;
    let liked = liked(catalog, learned);
    let liked_set: HashSet<&String> = liked.iter().collect();
    let tied_now = tied(catalog, target);
    let declined: HashSet<&String> = learned.declined(target).into_iter().collect();
    let lonely = orphan(catalog, &liked_set, target);
    let candidates: Vec<&String> = liked
        .iter()
        .filter(|slug| {
            slug.as_str() != target
                && !tied_now.contains(*slug)
                && !declined.contains(slug)
                && !(lonely && orphan(catalog, &liked_set, slug))
        })
        .collect();

    let links: Vec<String> = card.links.iter().map(|l| format!("{} ({})", l.to, l.kind)).collect();
    let mut from: Vec<&String> =
        catalog.cards.iter().filter(|(_, c)| c.links.iter().any(|l| l.to == target)).map(|(s, _)| s).collect();
    from.sort();
    let head = format!(
        "- {} · links: {} · linked from: {}",
        describe(target, card),
        if links.is_empty() { "none".to_string() } else { links.join(", ") },
        if from.is_empty() { "none".to_string() } else { from.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ") },
    );
    let mut prompt = format!("{INSTRUCTIONS}\n\nTARGET\n{head}\n\nCANDIDATES\n");
    for slug in &candidates {
        prompt.push_str(&format!("- {}\n", describe(slug, &catalog.cards[*slug])));
    }
    let mut excluded: HashSet<String> = tied_now;
    excluded.extend(declined.into_iter().cloned());
    Ok(Ask {
        targets: vec![Target { slug: target.to_string(), fingerprint: fingerprint(catalog, target, &liked), excluded }],
        prompt,
        candidates: candidates.into_iter().cloned().collect(),
    })
}

/// What an artist's answer depends on: its card, its links both ways, and
/// the liked artists — the same for `aS` and for the orphans, so one
/// answers for the other.
fn fingerprint(catalog: &Catalog, slug: &str, liked: &[String]) -> String {
    let card = &catalog.cards[slug];
    let links: Vec<String> = card.links.iter().map(|l| format!("{}:{}", l.to, l.kind)).collect();
    let mut from: Vec<&String> =
        catalog.cards.iter().filter(|(_, c)| c.links.iter().any(|l| l.to == slug)).map(|(s, _)| s).collect();
    from.sort();
    crate::embed::fnv(&format!("{}\n{}\n{:?}\n{}", describe(slug, card), links.join(","), from, liked.join("\n")))
}

/// `:suggest` — the orphans: the liked artists tied to no other liked
/// one, the most familiar first, at most `limit` of them, skipping those
/// already answered from the same card and the same liked artists. The
/// candidates are the liked artists that are not orphans — an anchor must
/// be reachable, or two orphans only make an island. `None` when no
/// orphan is left to ask about.
pub fn ask_orphans(catalog: &Catalog, learned: &Learned, store: &Store, limit: usize) -> Option<Ask> {
    let liked = liked(catalog, learned);
    let liked_set: HashSet<&String> = liked.iter().collect();
    let orphans: HashSet<&String> = liked.iter().filter(|s| orphan(catalog, &liked_set, s)).collect();
    let mut asked: Vec<&String> = orphans
        .iter()
        .copied()
        .filter(|slug| !store.answered(slug, &fingerprint(catalog, slug, &liked)))
        .collect();
    let familiarity = |slug: &String| learned.familiarity01(slug, &catalog.cards[slug.as_str()].name);
    asked.sort_by(|a, b| familiarity(b).total_cmp(&familiarity(a)).then(a.cmp(b)));
    asked.truncate(limit);
    if asked.is_empty() {
        return None;
    }
    let candidates: Vec<&String> = liked.iter().filter(|s| !orphans.contains(s)).collect();
    let mut prompt = format!("{ORPHANS}\n\nORPHANS\n");
    let mut targets = Vec::new();
    for slug in &asked {
        let card = &catalog.cards[slug.as_str()];
        let links: Vec<String> = card.links.iter().map(|l| format!("{} ({})", l.to, l.kind)).collect();
        let declined: Vec<&String> = learned.declined(slug);
        let mut line = format!(
            "- {} · current links: {}",
            describe(slug, card),
            if links.is_empty() { "none".to_string() } else { links.join(", ") }
        );
        if !declined.is_empty() {
            line.push_str(&format!(" · declined: {}", declined.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        }
        prompt.push_str(&line);
        prompt.push('\n');
        let mut excluded = tied(catalog, slug);
        excluded.extend(declined.into_iter().cloned());
        targets.push(Target { slug: (*slug).clone(), fingerprint: fingerprint(catalog, slug, &liked), excluded });
    }
    prompt.push_str("\nCANDIDATES\n");
    for slug in &candidates {
        prompt.push_str(&format!("- {}\n", describe(slug, &catalog.cards[slug.as_str()])));
    }
    Some(Ask { targets, prompt, candidates: candidates.into_iter().cloned().collect() })
}

/// Call the agent. Blocking — from `spawn_blocking`, never on the loop.
/// `--safe-mode` keeps the listener's own `CLAUDE.md`, skills, hooks and
/// MCP servers out of the call while keeping the login; `--bare` would
/// too, but only takes an API key (checked on Claude Code 2.1.285).
pub fn call(prompt: &str) -> Result<Vec<Proposal>, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("claude")
        .args(["-p", "--safe-mode", "--tools", "", "--output-format", "json", "--json-schema", SCHEMA, "--no-session-persistence"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => "Claude Code (claude) is not on the PATH".to_string(),
            _ => format!("claude did not start ({e})"),
        })?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(prompt.as_bytes()).map_err(|e| format!("claude did not take the prompt ({e})"))?;
    }
    let out = child.wait_with_output().map_err(|e| format!("claude did not answer ({e})"))?;
    read_answer(&String::from_utf8_lossy(&out.stdout))
        .map_err(|why| if out.status.success() { why } else { format!("{why} — {}", String::from_utf8_lossy(&out.stderr).trim()) })
}

/// The answer `claude -p --output-format json` prints: an envelope, the
/// proposals in `structured_output`.
fn read_answer(text: &str) -> Result<Vec<Proposal>, String> {
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default)]
        is_error: bool,
        #[serde(default)]
        result: Option<String>,
        structured_output: Option<Answer>,
    }
    #[derive(Deserialize)]
    struct Answer {
        suggestions: Vec<Proposal>,
    }
    let envelope: Envelope = serde_json::from_str(text.trim()).map_err(|e| format!("claude's answer does not read ({e})"))?;
    if envelope.is_error {
        return Err(format!("claude: {}", envelope.result.unwrap_or_default()));
    }
    envelope.structured_output.map(|a| a.suggestions).ok_or_else(|| "claude gave no structured answer".to_string())
}

/// Keep what holds: the target's own, an anchor that was sent, a type the
/// agent may give, a closeness on the scale — each anchor once.
pub fn keep(ask: &Ask, proposals: Vec<Proposal>) -> Vec<Proposal> {
    let mut seen = HashSet::new();
    proposals
        .into_iter()
        .filter(|p| {
            ask.targets.iter().any(|t| t.slug == p.artist && !t.excluded.contains(&p.anchor))
                && ask.candidates.contains(&p.anchor)
                && p.anchor != p.artist
                && KINDS.contains(&p.kind.as_str())
                && (1..=5).contains(&p.proximity)
                && seen.insert((p.artist.clone(), p.anchor.clone()))
        })
        .collect()
}

/// One artist's proposals, and what they were asked from.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Entry {
    pub fingerprint: String,
    pub asked: String,
    pub pending: Vec<Proposal>,
}

/// The waiting list, which is also the cache: local state, it does not
/// travel — on another machine it costs one call at most.
#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub artists: BTreeMap<String, Entry>,
}

impl Store {
    fn path() -> PathBuf {
        crate::config::state_dir().join("suggestions.json")
    }

    pub fn load() -> Store {
        std::fs::read_to_string(Store::path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Store::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }

    /// Whether to show what waits instead of calling (`agent.md`, the
    /// cache): something still waits, and it was asked from the same
    /// card and the same liked artists.
    pub fn fresh(&self, target: &Target) -> bool {
        self.artists.get(&target.slug).is_some_and(|e| !e.pending.is_empty() && e.fingerprint == target.fingerprint)
    }

    /// Whether this artist was already answered — something waiting or
    /// not — from the same card and the same liked artists: the general
    /// run does not ask again about an orphan the agent found nothing for.
    pub fn answered(&self, slug: &str, fingerprint: &str) -> bool {
        self.artists.get(slug).is_some_and(|e| e.fingerprint == fingerprint)
    }

    /// File a call's answer. What still waited stays, after the new ones:
    /// asking again brings other ideas, it does not drop the ones not yet
    /// settled.
    pub fn put(&mut self, ask: &Ask, proposals: Vec<Proposal>) {
        for target in &ask.targets {
            let mut pending: Vec<Proposal> = proposals.iter().filter(|p| p.artist == target.slug).cloned().collect();
            if let Some(old) = self.artists.get(&target.slug) {
                for p in &old.pending {
                    if !pending.iter().any(|q| q.anchor == p.anchor) {
                        pending.push(p.clone());
                    }
                }
            }
            let entry = Entry { fingerprint: target.fingerprint.clone(), asked: crate::learned::today_iso(), pending };
            self.artists.insert(target.slug.clone(), entry);
        }
        self.save();
    }

    /// Settled — accepted or declined: out of the list.
    pub fn settle(&mut self, artist: &str, anchor: &str) {
        if let Some(entry) = self.artists.get_mut(artist) {
            entry.pending.retain(|p| p.anchor != anchor);
        }
        self.save();
    }

    /// Everything waiting, artist by artist; one artist only when named.
    pub fn waiting(&self, only: Option<&str>) -> Vec<Proposal> {
        self.artists
            .iter()
            .filter(|(artist, _)| only.map_or(true, |o| o == artist.as_str()))
            .flat_map(|(_, e)| e.pending.iter().cloned())
            .collect()
    }
}

/// Where the screen stands in deciding the highlighted proposal
/// (`Propositions.dc.html`, 2a–2c).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Step {
    List,
    /// How close — the `ac` question, starting at the agent's closeness.
    Closeness(u8),
    /// Where: a card link (`true`) or a connection.
    Destination { proximity: u8, card: bool },
    /// For a card link: its type, one of `KINDS`, and the diff shown.
    Kind { proximity: u8, kind: usize },
}

/// The screen: the proposals, the cursor, the step.
pub struct Screen {
    pub rows: Vec<Proposal>,
    pub cursor: usize,
    pub step: Step,
    /// The last thing said on the screen — what was written, declined.
    pub notice: String,
}

impl Screen {
    pub fn new(rows: Vec<Proposal>) -> Screen {
        Screen { rows, cursor: 0, step: Step::List, notice: String::new() }
    }

    pub fn current(&self) -> Option<&Proposal> {
        self.rows.get(self.cursor)
    }

    /// A proposal settled: out of the rows, the cursor stays in range.
    pub fn drop_current(&mut self) {
        if self.cursor < self.rows.len() {
            self.rows.remove(self.cursor);
        }
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
        self.step = Step::List;
    }

    /// The block's title and lines. `name` gives an artist's name from its
    /// slug, `grid` the catalog's closeness by kind, `today` the date.
    pub fn view(&self, name: &dyn Fn(&str) -> String, grid: &std::collections::HashMap<String, u8>, today: &str) -> (String, Vec<String>) {
        let mut lines = Vec::new();
        let Some(p) = self.current() else {
            return ("suggest".to_string(), vec!["(nothing waiting)".to_string(), String::new(), "esc close".to_string()]);
        };
        let pair = format!("{} → {}", name(&p.artist), name(&p.anchor));
        match self.step {
            Step::List => {
                let mut last = "";
                for (i, row) in self.rows.iter().enumerate() {
                    if row.artist != last {
                        if !last.is_empty() {
                            lines.push(String::new());
                        }
                        lines.push(name(&row.artist));
                        last = &row.artist;
                    }
                    let here = if i == self.cursor { "▸" } else { " " };
                    lines.push(format!("{here} → {}  {}  {} — {}", name(&row.anchor), row.proximity, row.kind, row.reason));
                }
                lines.push(String::new());
                if !self.notice.is_empty() {
                    lines.push(self.notice.clone());
                }
                lines.push("j k move · ⏎ accept · x decline · r ask again · esc close".to_string());
                (format!("suggest — {} waiting", self.rows.len()), lines)
            }
            Step::Closeness(proximity) => {
                lines.push(format!("{pair} — how close?"));
                lines.push(format!("agent: {}", p.reason));
                lines.push(String::new());
                for level in (1..=5u8).rev() {
                    let meaning = crate::listen::closeness_meaning(level);
                    let mut kinds: Vec<&str> = grid.iter().filter(|(_, at)| **at == level).map(|(k, _)| k.as_str()).collect();
                    kinds.sort_unstable();
                    let kinds = if kinds.is_empty() { String::new() } else { format!("  ({})", kinds.join(", ")) };
                    let here = if level == proximity { "▸" } else { " " };
                    lines.push(format!("{here} {level}  {meaning}{kinds}"));
                }
                lines.push(String::new());
                lines.push("h l one step · 1–5 jump · ⏎ next: where · esc back to the list".to_string());
                ("suggest — how close".to_string(), lines)
            }
            Step::Destination { proximity, card } => {
                lines.push(format!("{pair} {proximity} — where?"));
                lines.push(String::new());
                let mark = |on: bool| if on { "▸" } else { " " };
                lines.push(format!(
                    "{} c  card link    in {}'s card · committed · can go out with Cp · moves the vector",
                    mark(card),
                    name(&p.artist)
                ));
                lines.push(format!("{} a  connection   in learned/ · private · never sent, never in the vectors", mark(!card)));
                lines.push(String::new());
                lines.push(format!("(agent leans: {})", if p.lean == "card" { "card link" } else { "connection" }));
                lines.push("c card link · a connection · j k ⏎ the same, by moving · esc back to how close".to_string());
                ("suggest — where".to_string(), lines)
            }
            Step::Kind { proximity, kind } => {
                let shown: Vec<String> = KINDS
                    .iter()
                    .enumerate()
                    .map(|(i, k)| if i == kind { format!("▸{k}") } else { format!(" {k}") })
                    .collect();
                lines.push(format!("{pair} {proximity} — type?  {}", shown.join("  ")));
                lines.push(String::new());
                lines.push(format!("cards/{}.toml", p.artist));
                lines.push(format!("+ {}", crate::edit::link_line(&p.anchor, KINDS[kind], written_proximity(grid, KINDS[kind], proximity), &p.reason)));
                lines.push(String::new());
                lines.push(link_summary(&name(&p.artist), &name(&p.anchor), KINDS[kind], proximity));
                lines.push(format!("suggested by the agent, {today}"));
                lines.push(String::new());
                lines.push("h l type · y write and commit · esc back to where".to_string());
                ("suggest — the card link".to_string(), lines)
            }
        }
    }
}

/// The closeness to write on a card link: only when it differs from what
/// the grid gives its kind (`agent.md`, point 5).
pub fn written_proximity(grid: &std::collections::HashMap<String, u8>, kind: &str, proximity: u8) -> Option<u8> {
    (grid.get(kind) != Some(&proximity)).then_some(proximity)
}

/// The commit's subject for a card link — and the screen's preview of it.
pub fn link_summary(from: &str, to: &str, kind: &str, proximity: u8) -> String {
    format!("{from} — link: → {to} ({kind}, {proximity})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Link;
    use std::collections::HashMap;

    fn card(name: &str, tags: &[&str], links: &[(&str, &str)]) -> Card {
        let text = format!("format = 1\nname = \"{name}\"\nmbid = \"x\"\n");
        let mut card: Card = toml::from_str(&text).expect("card");
        card.tags = tags.iter().map(|t| t.to_string()).collect();
        card.links = links
            .iter()
            .map(|(to, kind)| Link { to: to.to_string(), kind: kind.to_string(), note: None, proximity: None })
            .collect();
        card
    }

    /// The answer's envelope, as `claude -p --output-format json` prints
    /// it: the proposals are in `structured_output`; an error says so.
    #[test]
    fn the_answer_is_read_from_its_envelope() {
        let ok = r#"{"type":"result","is_error":false,"result":"{}","structured_output":{"suggestions":[{"artist":"experience","anchor":"miossec","type":"scene","proximity":4,"lean":"card","reason":"Raw French indie rock."}]}}"#;
        let got = read_answer(ok).expect("read");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].anchor, "miossec");
        assert_eq!(got[0].kind, "scene");
        let failed = r#"{"type":"result","is_error":true,"result":"usage limit reached"}"#;
        assert!(read_answer(failed).unwrap_err().contains("usage limit"));
        assert!(read_answer("not json").is_err());
    }

    /// Only what holds is kept: the target's, an anchor that was sent, a
    /// reading and not a fact, a closeness on the scale, each anchor once.
    #[test]
    fn only_what_was_sent_is_kept() {
        let ask = Ask {
            targets: vec![Target { slug: "experience".into(), fingerprint: String::new(), excluded: HashSet::new() }],
            prompt: String::new(),
            candidates: ["miossec".to_string(), "noir-desir".to_string()].into_iter().collect(),
        };
        let p = |anchor: &str, kind: &str, proximity: u8| Proposal {
            artist: "experience".into(),
            anchor: anchor.into(),
            kind: kind.into(),
            proximity,
            lean: "card".into(),
            reason: String::new(),
        };
        let kept = keep(
            &ask,
            vec![
                p("miossec", "scene", 4),
                p("miossec", "similar", 3),
                p("zebda", "scene", 3),
                p("noir-desir", "family", 3),
                p("noir-desir", "scene", 9),
                p("noir-desir", "influence", 3),
            ],
        );
        let anchors: Vec<(&str, &str)> = kept.iter().map(|p| (p.anchor.as_str(), p.kind.as_str())).collect();
        assert_eq!(anchors, vec![("miossec", "scene"), ("noir-desir", "influence")]);
    }

    /// The candidates: liked, not the target, not tied to it (a card link
    /// either way, `audience` aside), not declined; and the fingerprint
    /// does not move when a proposal is declined.
    #[test]
    fn the_candidates_leave_out_what_is_tied_or_declined() {
        let cards = HashMap::from([
            ("experience".to_string(), card("Expérience", &["fr"], &[("zebda", "audience"), ("michel-cloup", "member")])),
            ("michel-cloup".to_string(), card("Michel Cloup", &["fr"], &[])),
            ("zebda".to_string(), card("Zebda", &["fr"], &[])),
            ("miossec".to_string(), card("Miossec", &["fr"], &[("dominique-a", "scene")])),
            ("dominique-a".to_string(), card("Dominique A", &["fr"], &[("experience", "scene")])),
            ("noir-desir".to_string(), card("Noir Désir", &["fr"], &[("miossec", "similar")])),
        ]);
        let catalog = Catalog { cards, proximities: HashMap::new(), vectors: HashMap::new() };
        let dir = std::env::temp_dir().join(format!("forkstify-suggest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("learned")).expect("temp dir");
        let mut learned = Learned::load(&dir);
        for slug in ["experience", "michel-cloup", "zebda", "miossec", "dominique-a", "noir-desir"] {
            learned.like_artist(slug);
        }
        let before = ask(&catalog, &learned, "experience").expect("ask");
        let mut sent: Vec<&String> = before.candidates.iter().collect();
        sent.sort();
        // Michel Cloup and Dominique A are tied; Zebda only by audience
        assert_eq!(sent, vec!["miossec", "noir-desir", "zebda"]);
        assert!(before.prompt.contains("TARGET\n- Expérience [experience] · tags: fr"), "{}", before.prompt);
        assert!(before.prompt.contains("linked from: dominique-a"), "{}", before.prompt);

        learned.decline("experience", "zebda");
        let after = ask(&catalog, &learned, "experience").expect("ask");
        assert!(!after.candidates.contains("zebda"));
        assert_eq!(before.targets[0].fingerprint, after.targets[0].fingerprint, "a refusal is not a change of what was asked from");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The prompt on a real catalog, written out to be read or sent by
    /// hand: `FORKSTIFY_CATALOG=<dir> SUGGEST_TARGET=<slug>
    /// SUGGEST_OUT=<file> bin/test -- --ignored the_prompt_on_a_real_catalog`;
    /// `SUGGEST_TARGET=all` for the orphans.
    #[test]
    #[ignore]
    fn the_prompt_on_a_real_catalog() {
        let dir = std::path::PathBuf::from(std::env::var("FORKSTIFY_CATALOG").expect("FORKSTIFY_CATALOG"));
        let target = std::env::var("SUGGEST_TARGET").expect("SUGGEST_TARGET");
        let mut catalog = Catalog::load(&dir).expect("catalog");
        let learned = Learned::load(&dir);
        learned.weave_into(&mut catalog);
        let ask = if target == "all" {
            ask_orphans(&catalog, &learned, &Store::default(), 10).expect("orphans")
        } else {
            ask(&catalog, &learned, &target).expect("ask")
        };
        println!("asking about {:?}", ask.targets.iter().map(|t| t.slug.as_str()).collect::<Vec<_>>());
        println!("{} candidates, fingerprint {}", ask.candidates.len(), ask.targets[0].fingerprint);
        std::fs::write(std::env::var("SUGGEST_OUT").expect("SUGGEST_OUT"), &ask.prompt).expect("written");
    }

    /// `:suggest` alone (2026-10-07): the orphans are asked about, the
    /// most familiar first, each answered one skipped; the candidates are
    /// the liked artists that are not orphans; an anchor already declined
    /// for an orphan is dropped from its answer.
    #[test]
    fn the_general_run_asks_about_the_orphans() {
        let cards = HashMap::from([
            ("calexico".to_string(), card("Calexico", &["americana"], &[("giant-sand", "member")])),
            ("giant-sand".to_string(), card("Giant Sand", &[], &[])),
            ("cheveu".to_string(), card("Cheveu", &["fr"], &[("sleaford-mods", "audience")])),
            ("sleaford-mods".to_string(), card("Sleaford Mods", &[], &[])),
            ("tindersticks".to_string(), card("Tindersticks", &[], &[("mazzy-star", "similar")])),
            ("mazzy-star".to_string(), card("Mazzy Star", &[], &[])),
        ]);
        let catalog = Catalog { cards, proximities: HashMap::new(), vectors: HashMap::new() };
        let dir = std::env::temp_dir().join(format!("forkstify-orphans-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("learned")).expect("temp dir");
        let mut learned = Learned::load(&dir);
        for slug in ["calexico", "cheveu", "tindersticks", "mazzy-star"] {
            learned.like_artist(slug);
        }
        learned.like_artist("calexico");
        learned.decline("calexico", "mazzy-star");
        let mut store = Store::default();

        let ask = ask_orphans(&catalog, &learned, &store, 10).expect("orphans");
        let asked: Vec<&str> = ask.targets.iter().map(|t| t.slug.as_str()).collect();
        // Calexico and Cheveu tie to nobody liked; Tindersticks and Mazzy
        // Star tie to each other
        assert_eq!(asked, vec!["calexico", "cheveu"], "the most familiar first");
        let mut sent: Vec<&String> = ask.candidates.iter().collect();
        sent.sort();
        assert_eq!(sent, vec!["mazzy-star", "tindersticks"]);
        assert!(ask.prompt.contains("declined: mazzy-star"), "{}", ask.prompt);

        let p = |artist: &str, anchor: &str| Proposal {
            artist: artist.into(),
            anchor: anchor.into(),
            kind: "similar".into(),
            proximity: 3,
            lean: "connection".into(),
            reason: String::new(),
        };
        let kept = keep(&ask, vec![p("calexico", "mazzy-star"), p("calexico", "tindersticks"), p("cheveu", "tindersticks")]);
        assert_eq!(kept.len(), 2, "the declined one is dropped");

        // answered — even with nothing kept — means not asked again
        let only_calexico = Ask { targets: vec![ask.targets[0].clone()], ..ask.clone() };
        store.artists.insert(
            "calexico".into(),
            Entry { fingerprint: only_calexico.targets[0].fingerprint.clone(), asked: String::new(), pending: Vec::new() },
        );
        let next = ask_orphans(&catalog, &learned, &store, 10).expect("Cheveu left");
        assert_eq!(next.targets.iter().map(|t| t.slug.as_str()).collect::<Vec<_>>(), vec!["cheveu"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The card link's closeness is written only when the grid would not
    /// give it.
    #[test]
    fn a_link_carries_its_closeness_only_when_it_differs() {
        let grid = HashMap::from([("scene".to_string(), 3u8), ("influence".to_string(), 2u8)]);
        assert_eq!(written_proximity(&grid, "scene", 3), None);
        assert_eq!(written_proximity(&grid, "influence", 3), Some(3));
    }
}
