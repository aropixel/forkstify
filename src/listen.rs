//! `forkstify listen` — the dry navigator, but it plays. Same engine and
//! menus as `journey`; here the segments actually sound, track by track,
//! and the keys act on the ongoing playback. The engine stays untouched:
//! it produces stops, this module resolves and plays them.
//!
//! Line-based input for now (Enter after each key); real-time single-key
//! and auto-on-silence belong to the TUI step. Music plays in the
//! background (librespot task), the menu is shown while it plays, and a
//! finished segment auto-advances so it never stops.

use crate::catalog::Catalog;
use crate::keys::{self, Cmd, When};
use crate::tui::{Tui, View};

/// What the session says on screen. A log line, not a print: from the
/// TUI there is no stream left to write to — there is an area.
macro_rules! say {
    ($self:ident, $($arg:tt)*) => {
        $self.notice(format!($($arg)*))
    };
}
use crate::engine::Comfort;
use crate::discography::Tail;
use crate::home::{Choice, Home, LastSession, Outcome};
use crate::learned::Learned;
use crate::mediakeys::{self, Control, Shown};
use std::rc::Rc;
use crate::sound::{request_started, track_finished, track_over, track_unavailable, Sound};
use crate::spotify::{Resolved, WebApi};
use std::sync::Arc;
use tokio::sync::Mutex;
use librespot_playback::player::PlayerEvent;
use crate::{state_of, Round};
use librespot_core::SpotifyUri;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[allow(clippy::too_many_arguments)]
pub fn run(
    choice: Option<Choice>,
    learned: Learned,
    tail: Tail,
    comfort: Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
    tui: &mut Tui,
    catalog_dir: &std::path::Path,
    status: Vec<(String, bool)>,
) -> anyhow::Result<Option<crate::setup::Replay>> {
    // current-thread runtime + LocalSet: the MPRIS Player is !Send (RefCell
    // callbacks) and must be driven with spawn_local. librespot's own tasks
    // run fine here (as in spike-play).
    // raw mode for the whole session (0015); the guard puts the terminal
    // back even if we leave through an error or a panic
    let _raw = keys::RawMode::enable();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let local = tokio::task::LocalSet::new();
    local
        .block_on(&rt, async_run(choice, learned, tail, comfort, rx, tui, catalog_dir, status))
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
async fn async_run(
    choice: Option<Choice>,
    learned: Learned,
    tail: Tail,
    comfort: Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
    tui: &mut Tui,
    catalog_dir: &std::path::Path,
    status: Vec<(String, bool)>,
) -> Result<Option<crate::setup::Replay>, Box<dyn std::error::Error>> {
    // the alternate screen belongs to the TUI: the steps are drawn on it,
    // not printed
    tui.clear();
    // loaded here, no longer lent by the caller: the session will make it
    // grow (0016), so it must own it
    let mut catalog = Catalog::load(catalog_dir)?;
    // the listener's own connections join the graph the engine walks; the
    // cards on disk keep to themselves (2026-09-23)
    learned.weave_into(&mut catalog);
    let census = format!(
        "learned: {} artist(s) played, {} with starting familiarity, {} discography(ies) cached",
        learned.known(),
        learned.seeded(),
        tail.known()
    );
    let mut steps = vec![(census, true), ("connecting to spotify…".to_string(), false)];
    let _ = tui.splash(&steps);
    let config = crate::config::Config::load();
    let sound = Sound::connect().await?;
    steps[1] = ("sound: librespot connected".to_string(), true);
    steps.push(("web api authorization — the browser opens if needed…".to_string(), false));
    let _ = tui.splash(&steps);
    let web = WebApi::new(config.playback.prefer_studio).await?;
    steps[2] = ("tracks: web api authorized".to_string(), true);
    let _ = tui.splash(&steps);

    // MPRIS: let the desktop's media keys (⏮ ⏭ ⏯) drive us
    let (ctrl_tx, mut ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<Control>();
    let mpris = match mediakeys::start(ctrl_tx).await {
        Ok(player) => {
            steps.push(("media keys active (mpris)".to_string(), true));
            Some(Rc::new(player))
        }
        Err(e) => {
            steps.push((format!("mpris unavailable ({e}) — media keys inactive"), true));
            None
        }
    };
    let _ = tui.splash(&steps);

    // 0017: the learned commits itself — every ten minutes if it moved, on
    // exit, and on `:sync`; the push happens in the background
    let (sync_tx, mut sync_rx) = tokio::sync::mpsc::unbounded_channel::<Result<String, String>>();
    let (jobs_tx, mut jobs_rx) = tokio::sync::mpsc::unbounded_channel::<Job>();
    let mut autosave = tokio::time::interval(std::time::Duration::from_secs(600));
    autosave.tick().await;

    let mut live = Live {
        catalog,
        catalog_dir: catalog_dir.to_path_buf(),
        learned,
        tail,
        sound,
        web: Arc::new(Mutex::new(web)),
        rng: thread_rng(),
        rounds: Vec::new(),
        past: Vec::new(),
        current: None,
        queue: VecDeque::new(),
        current_request_id: None,
        paused: false,
        comfort,
        notices: std::cell::RefCell::new(Vec::new()),
        tui,
        selection: None,
        comfort_before: None,
        overlay: None,
        overlay_scroll: 0,
        typed: String::new(),
        warm_requested: false,
        web_ok: true,
        wander_requested: None,
        link_pending: None,
        start_requested: None,
        album_requested: None,
        search_requested: None,
        branches: Vec::new(),
        missing: Vec::new(),
        generating: HashSet::new(),
        finder: None,
        size: 3,
        progress: None,
        help_open: false,
        help_auto: false,
        explore: None,
        explore_requested: false,
        sync_tx,
        jobs_tx,
        loading: false,
        dry_advances: 0,
        harvesting: HashMap::new(),
        screen: Screen::Home,
        home: Home::default(),
        status,
        leave: None,
        pending_proposal: None,
        conflicts: Vec::new(),
        catalog_busy: false,
        toast: std::cell::RefCell::new(None),
        pending_seed: None,
        mpris,
        mpris_shown: Shown::default(),
        mpris_sampled: None,
    };
    let mut events = live.sound.events();
    // one tick per second moves the progress bar; it only repaints when
    // something is playing
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));

    // the key reader is the home's: two threads on stdin would steal each
    // other's bytes

    // a given seed starts right away; otherwise the home, with the session
    // underneath ready to play (Joel, 08/09/2026)
    match choice {
        Some(choice) => live.start_journey(choice).await,
        None => live.tui.clear(),
    }
    live.prefetch_next().await;
    live.paint();

    loop {
        tokio::select! {
            event = events.recv() => match event {
                Some(ref ev) => {
                    // remember which track is really current…
                    if let Some(id) = request_started(ev) {
                        live.current_request_id = Some(id);
                    // …and only react to the end of THAT track, not stray
                    // events from one we already skipped past — and never
                    // while paused: losing the output, a bluetooth speaker
                    // walking away, makes librespot stop the stream, which
                    // read as "the track ended" and started the next one on
                    // its own (Joel, 2026-09-23). A paused player has no
                    // business moving on.
                    } else if live.current_request_id.is_some()
                        && track_over(ev) == live.current_request_id
                        && !live.paused
                    {
                        live.on_track_over(track_finished(ev), track_unavailable(ev)).await;
                        live.prefetch_next().await;
                        live.paint();
                    } else if live.follow_needle(ev) {
                        live.paint();
                    }
                }
                None => break,
            },
            _ = tick.tick() => {
                if live.progress.as_ref().is_some_and(|p| p.running) || live.toast_active() {
                    live.paint();
                }
            }
            job = jobs_rx.recv() => {
                if let Some(job) = job {
                    live.on_job(job).await;
                    live.prefetch_next().await;
                    live.paint();
                }
            }
            _ = autosave.tick() => live.autosave(),
            report = sync_rx.recv() => {
                if let Some(report) = report {
                    match report {
                        Ok(word) => say!(live, "✓ {word}"),
                        Err(why) => say!(live, "⏹ {why}"),
                    }
                    live.paint();
                }
            }
            cmd = rx.recv() => match cmd {
                Some(cmd) => {
                    if !live.on_cmd(cmd).await {
                        live.remember();
                        break;
                    }
                    live.prefetch_next().await;
                    live.paint();
                }
                None => break,
            },
            control = ctrl_rx.recv() => match control {
                Some(control) => {
                    live.on_control(control).await;
                    live.prefetch_next().await;
                    live.paint();
                }
                None => {}
            },
        }
    }

    live.sound.stop();
    // what was learned leaves with the session (0017) — said on screen
    // long enough to be read
    let report = match crate::sync::sync(&live.catalog_dir) {
        Ok(word) => (format!("✓ {word}"), true),
        Err(why) => (format!("⏹ learned not pushed — {why} (next launch)"), false),
    };
    let _ = live.tui.splash(&[report]);
    std::thread::sleep(std::time::Duration::from_millis(900));
    Ok(live.leave)
}

/// Past this point of the track, "previous" restarts it instead of going
/// back — three seconds, as players have learned to do.
const RESTART_AFTER_MS: u32 = 3_000;
/// How many branches may auto-advance in a row with nothing reaching the
/// speakers before the music stops and asks for a hand (Joel, 11/09/2026).
const MAX_DRY_ADVANCES: u32 = 4;

/// `A` — how many tracks of an album get promoted at once. Four: an album
/// that carries the plays rarely has more that matter, and beyond that
/// nobody reads back what they just did.
const ALBUM_TOPS: usize = 4;

struct Live<'a> {
    /// **The session owns its catalog** since 09/09/2026: a generated card
    /// must exist for the engine at once, not at the next launch
    /// (`docs/design/on-the-fly-generation.md`). It was lent read-only
    /// until then.
    catalog: Catalog,
    /// Where the cards live: an edit modifies and commits them (0013).
    catalog_dir: std::path::PathBuf,
    learned: Learned,
    /// The long tail, harvested on demand (0012 §1, fourth source).
    tail: Tail,
    sound: Sound,
    /// The web API, shared with the background tasks: a lock serializes
    /// the calls, which is also what Spotify's quota asks for.
    web: Arc<Mutex<WebApi>>,
    rng: ThreadRng,
    rounds: Vec<Round>,
    // playback as a linear timeline: what was played, what plays now, what
    // comes next — so `k` / `j` step back and forth like a normal player.
    past: Vec<crate::engine::Stop>,
    current: Option<crate::engine::Stop>,
    queue: VecDeque<crate::engine::Stop>,
    // the librespot play request currently on air, to filter stale events
    current_request_id: Option<u64>,
    paused: bool,
    /// The comfort dial (0001), from the config, adjustable with `:comfort`.
    comfort: Comfort,
    /// What forkstify just said — cleared at every command, so a block (the
    /// leader, `?`) shows alone and whole.
    notices: std::cell::RefCell<Vec<String>>,
    tui: &'a mut Tui,
    /// The desktop's view of the player (MPRIS), and what it was last told
    /// — pushed from `paint`, like the screen (0021).
    mpris: Option<Rc<mpris_server::Player>>,
    mpris_shown: Shown,
    /// When the needle was last sampled by librespot, as last told: a new
    /// sample is a jump the desktop must hear about (`Seeked`).
    mpris_sampled: Option<std::time::Instant>,
    /// The axis is shown vertically: the arrows move a selection along it,
    /// and nothing changes until confirmed (Joel, 06/09/2026).
    /// Index in the axis: past, then the current track, then the queue.
    selection: Option<usize>,
    /// `c` opens the comfort dial; the previous value is kept so esc can
    /// give it back.
    comfort_before: Option<Comfort>,
    /// What is being typed: a half-done sequence, or a line after `/` or
    /// `:`. It is the only thing that moves at the bottom.
    typed: String,
    /// A block laid over the screen — the leader menu, `?`. It does not go
    /// down into the log: the bottom of the screen must not move.
    overlay: Option<(String, Vec<String>)>,
    /// The first line of the overlay shown — j/k, ↑↓, gg, G scroll it.
    overlay_scroll: usize,
    /// `:warm` asked for a harvest; the command handler is not async, the
    /// loop does it on the next turn.
    warm_requested: bool,
    /// What the Web API last answered. A file on disk cannot say it.
    web_ok: bool,
    /// `:wander [artist]` asked; the command handler is not async either.
    wander_requested: Option<String>,
    /// `ac` chose a target and now asks how close — or reopened a drawn
    /// connection to set it: the question stays until a digit, ⏎, `x` or
    /// esc (Joel, 2026-09-23).
    link_pending: Option<LinkPending>,
    /// Enter on a track of the discography: a new seed, once the modal's
    /// sync handler has returned (Joel, 11/09/2026).
    start_requested: Option<Choice>,
    /// Enter on an album line of the discography: play the whole album as a
    /// new seed, once the modal's sync handler has returned (Joel, 14/09/2026).
    album_requested: Option<(String, String, Vec<String>)>,
    /// `:search <text>` asked for a search; same reason, same turn.
    search_requested: Option<String>,
    branches: Vec<crate::engine::Branch>,
    /// The neighborhood links that point to a missing card (0016):
    /// directions the catalog names but cannot walk yet. They are numbered
    /// after the branches.
    missing: Vec<crate::engine::Missing>,
    /// The cards being generated, so a second `f<n>` does not launch the
    /// same job again — like `harvesting` for the discographies.
    generating: HashSet<String>,
    /// The search modal, when open.
    finder: Option<Finder>,
    size: usize,
    /// Where the needle is in the current track, as librespot last said it
    /// — extrapolated by the clock while it plays (maquette 2b).
    progress: Option<Progress>,
    /// The key helper is open: it follows the pending sequence level by
    /// level, and the key that completes a command closes it.
    help_open: bool,
    /// It opened on its own, because a namespace was typed — not on
    /// space. Then ⌫ closes it instead of going up to the entry level,
    /// which was never asked for (Joel, 23/09/2026).
    help_auto: bool,
    /// `ad` — the artist's discography, laid over the listening. It takes
    /// the keyboard while open: it is a modal, not a screen (Joel's call,
    /// 07/09/2026, maquette 1a).
    explore: Option<crate::explore::Explore>,
    /// Opening may ask for a harvest, and the keyboard is not async: the
    /// loop takes care of it on the next turn, as for `:warm`.
    explore_requested: bool,
    /// Where a background push reports (0017): the loop says the result.
    sync_tx: tokio::sync::mpsc::UnboundedSender<Result<String, String>>,
    /// Where the network jobs report: a title resolved, a discography
    /// harvested. The screen shows first, the loop finishes the gesture
    /// when the answer comes (Joel, 08/09/2026).
    jobs_tx: tokio::sync::mpsc::UnboundedSender<Job>,
    /// The current track is on screen but its address is still being
    /// looked up — nothing sounds yet.
    loading: bool,
    /// Branches auto-chosen in a row without a single track reaching the
    /// speakers. A track that is region-locked ends the instant it starts,
    /// so a run of unplayable tracks would cycle branches forever without a
    /// sound (Joel, 11/09/2026). Reset the moment anything actually plays.
    dry_advances: u32,
    /// Discographies being harvested right now, so a second `ad` or `e<n>`
    /// does not launch the same job twice.
    harvesting: HashMap<String, bool>,
    /// Which screen is up: the home or the session. The session lives on
    /// under the home — the sound, the queue, the branches (Joel,
    /// 08/09/2026).
    screen: Screen,
    home: Home,
    /// The authorizations and the sync, for the home's header.
    status: Vec<(String, bool)>,
    /// The last thing said that deserves a toast, and when: the colored
    /// box at the bottom right, four seconds (Joel, 08/09/2026).
    toast: std::cell::RefCell<Option<(String, std::time::Instant, u64)>>,
    /// A generated artist offered as a new seed but not yet started: enter
    /// accepts, any other key declines (Joel, 11/09/2026).
    pending_seed: Option<(String, String, std::time::Instant)>,
    /// `:setup` / `:library`: the session closes and the setup opens on
    /// the catalog, then home comes back (workstream A, 20/09/2026).
    leave: Option<crate::setup::Replay>,
    /// `Cp` did its work — branch, worktree, commit, push — and waits for
    /// `y` to open the pull request through gh; any other key sends
    /// nothing (workstream B, 20/09/2026).
    pending_proposal: Option<crate::fork::Proposal>,
    /// The cards a `Cu` stopped on: `o` opens the first, `:catalog`
    /// resumes once git has them.
    conflicts: Vec<String>,
    /// A `Cd`, `Cp` or `Cu` is running off the loop: one at a time.
    catalog_busy: bool,
}

/// How long a toast stays — then it fades by itself on the tick.
const TOAST_SECONDS: u64 = 4;
/// The success toast of an offered seed lingers, so there is time to read
/// it and answer (Joel, 11/09/2026).
const SEED_OFFER_SECONDS: u64 = 12;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Home,
    Session,
}

/// What a background job brings back.
enum Job {
    Resolved { title: String, artist: String, result: Resolved },
    Searched { query: String, result: Result<Vec<crate::spotify::SearchHit>, String> },
    /// `quiet`: fetched behind for a proposed branch, nobody asked —
    /// nothing to say, whatever comes back.
    Harvested { slug: String, result: Result<Vec<crate::discography::TailTrack>, String>, quiet: bool },
    Generated { slug: String, after: After, result: Result<crate::generate::Draft, String> },
    /// The card asked for already exists: nothing to generate, but what
    /// was meant for the artist still happens (Joel, 10/09/2026 — enter
    /// on "Kanye West" answered "Ye already has a card" and stopped).
    Existing { slug: String, after: After },
    /// The fresh card's vector (0019) — or why there is none; the card is
    /// adopted either way.
    Vectorized { slug: String, after: After, draft: crate::generate::Draft, vector: Result<Vec<f32>, String> },
    /// The catalog gestures, run off the loop (workstream B).
    Diffed(Result<crate::fork::Diff, String>),
    Proposed(Result<crate::fork::Proposal, String>),
    Updated(Result<crate::fork::Update, String>),
    PullRequest(Result<String, String>),
}

/// What was meant for the artist **once it has a card**. Generation takes
/// a few seconds; the intent is kept with it, or the gesture would get
/// lost on the way.
enum After {
    /// The search: start from the artist, with this track as the opening
    /// if there was one.
    Play { title: Option<String>, uri: Option<String> },
    /// A gap branch: taken like the others, where the key asked for it.
    /// `proximity` is the link's, the branch's weight.
    Branch { when: When, reason: String, proximity: u8 },
    /// `fg<n>` — the card of a gap, and nothing queued: the gap turns into
    /// a branch on show, walked like the others, and the gaps are refreshed
    /// around the fresh card (Joel, 19–20/09/2026).
    Gap { reason: String, proximity: u8 },
    /// The card alone: the track is already in the queue (`ti` on an
    /// off-catalog track), it waits for nothing — it will be attached.
    Card,
    /// `ad` on an artist without a card: the card first, its discography
    /// as soon as it is there (Joel, 10/09/2026).
    Explore,
    /// `:generate <name> <mbid>` while something plays: the card is made
    /// but the journey is **not** started — it would wipe the current
    /// list. Instead a lingering toast offers it (Joel, 11/09/2026).
    Offer,
}

/// The needle: a position sampled at an instant, a duration, and whether
/// time is running. librespot only speaks at starts, pauses and seeks; the
/// seconds in between are ours to count.
struct Progress {
    position_ms: u32,
    duration_ms: u32,
    sampled: std::time::Instant,
    running: bool,
}

impl Progress {
    fn now(&self) -> (u32, u32) {
        let elapsed = if self.running { self.sampled.elapsed().as_millis() as u32 } else { 0 };
        let position = self.position_ms.saturating_add(elapsed);
        let position = if self.duration_ms > 0 { position.min(self.duration_ms) } else { position };
        (position, self.duration_ms)
    }
}

/// What a comfort value means, so the number is never alone on screen.
/// Names credited in a track title — "(feat. X)", "ft. X", "with X" —
/// so `ta` can show them without a network call (Joel, 14/09/2026).
fn featuring(title: &str) -> Option<String> {
    let low = title.to_lowercase();
    for tag in [" feat.", " feat ", " ft.", " ft ", " featuring ", " with "] {
        if let Some(pos) = low.find(tag) {
            let rest = title[pos + tag.len()..].trim();
            let rest = rest.trim_start_matches(['(', '[']).trim_end_matches([')', ']']);
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// The pull request's body for the confirmation: the new cards cut to
/// two lines and a count — the reviewer skims them, so does the author —
/// the edited ones whole, since those are the ones to read.
fn abridged_body(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_new = false;
    let mut shown = 0;
    let mut hidden = 0;
    for line in body.lines() {
        if line.starts_with("## ") {
            if hidden > 0 {
                out.push(format!("… {hidden} more, one line each"));
                hidden = 0;
            }
            in_new = line.starts_with("## New cards");
            shown = 0;
            out.push(line.to_string());
            continue;
        }
        if in_new && line.starts_with("- ") {
            if shown < 2 {
                shown += 1;
                out.push(line.to_string());
            } else {
                hidden += 1;
            }
            continue;
        }
        if in_new && line.is_empty() && hidden > 0 {
            out.push(format!("… {hidden} more, one line each"));
            hidden = 0;
        }
        out.push(line.to_string());
    }
    if hidden > 0 {
        out.push(format!("… {hidden} more, one line each"));
    }
    out
}

pub fn comfort_word(value: u8) -> &'static str {
    match value {
        5 => "cocoon",
        4 => "careful",
        3 => "balanced",
        2 => "curious",
        1 => "adventurous",
        _ => "exploration",
    }
}

/// A `/` search result: a catalog artist to branch from, or a Spotify track
/// to play (with its artist's slug when that artist has a card).
#[derive(Clone)]
enum Hit {
    Artist(String),
    /// `spotify`: the artist's id when the hit came from Spotify — what a
    /// card generated from it is identified by (23/09/2026).
    Track { title: String, artist: String, uri: String, slug: Option<String>, spotify: Option<String> },
}

/// One line of the search modal: what was found, how it shows.
#[derive(Clone)]
struct Found {
    hit: Hit,
    mark: char,
    note: String,
}

/// The search modal (Joel, 08/09/2026, maquette `Recherche.dc.html`) —
/// `:search` alone, or `ti` anchored to a position of the list. The
/// catalogue answers at every keystroke; Spotify answers behind, and its
/// group says "… querying" until it does.
struct Finder {
    /// `Some(at)` = `ti`, insert at index `at` of the queue.
    insert: Option<usize>,
    query: String,
    catalogue: Vec<Found>,
    /// `None` while Spotify is being asked; the `Err` is its excuse.
    spotify: Option<Result<Vec<Found>, String>>,
    /// The query the pending Spotify job was sent for: a late answer to an
    /// older query is dropped, so the list never jumps under the cursor.
    asked: String,
    cursor: usize,
    only_catalogue: bool,
    /// `ac`: this modal draws a connection from that artist (slug, name)
    /// instead of playing the chosen row (Joel, 14/09/2026, as `aL`).
    link_from: Option<(String, String)>,
    /// `ac`: the connections already drawn, on either side, listed first
    /// and marked — enter on one reopens its question (Joel, 23/09/2026).
    linked: Vec<Found>,
    /// Behind each of them, where it is held: (from, to, closeness).
    drawn: Vec<(String, String, u8)>,
}

/// A found row turned into a stop of the axis, and its uri when the search
/// gave one. `None` for an artist row: there is no one track behind it.
fn stop_of(hit: &Hit, catalog: &Catalog) -> Option<(crate::engine::Stop, Option<String>)> {
    match hit {
        Hit::Track { title, artist, uri, slug, .. } => {
            let source = match slug.as_ref().map(|s| &catalog.cards[s]) {
                Some(card) if card.tops.contains(title) => crate::engine::Source::Top,
                Some(_) => crate::engine::Source::Outside,
                None => crate::engine::Source::Offmap,
            };
            Some((
                crate::engine::Stop {
                    slug: slug.clone().unwrap_or_default(),
                    artist: artist.clone(),
                    title: title.clone(),
                    source,
                    head: None,
                    encore: false,
                },
                if uri.is_empty() { None } else { Some(uri.clone()) },
            ))
        }
        Hit::Artist(_) => None,
    }
}

fn found_line(f: &Found, catalogue: bool) -> crate::tui::FinderLine {
    let (title, artist) = match &f.hit {
        Hit::Artist(slug) => (slug.replace('-', " "), String::new()),
        Hit::Track { title, artist, .. } => (title.clone(), artist.clone()),
    };
    crate::tui::FinderLine::Row { catalogue, mark: f.mark, title, artist, note: f.note.clone() }
}

impl Finder {
    /// The links already there that the query keeps: all of them on an
    /// empty query, else those whose slug or name contains it.
    fn linked_rows(&self) -> Vec<&Found> {
        let query = self.query.trim().to_lowercase();
        self.linked
            .iter()
            .filter(|f| {
                query.is_empty()
                    || match &f.hit {
                        Hit::Artist(slug) => slug.contains(&query) || f.note.to_lowercase().contains(&query),
                        _ => false,
                    }
            })
            .collect()
    }

    fn rows(&self) -> Vec<&Found> {
        let mut rows: Vec<&Found> = self.linked_rows();
        rows.extend(self.catalogue.iter());
        if !self.only_catalogue {
            if let Some(Ok(found)) = &self.spotify {
                rows.extend(found.iter());
            }
        }
        rows
    }
}

/// What came of trying to load one stop.
enum Load {
    Playing,
    /// Spotify has no such track: skip it, the walk goes on.
    Missing,
    /// The lookup never went through. The stop comes back untouched so the
    /// caller can put it away instead of losing it.
    Failed(crate::engine::Stop, String),
}

/// What came of trying to move forward.
enum Advance {
    Playing,
    /// Nothing left ahead — time to take a branch.
    Exhausted,
    /// An outage, not an end: the walk holds where it is.
    Blocked(String),
}

impl Live<'_> {
    /// Start a segment: record it, make it the future, play its first track.
    /// `opening` = the seed's own tops (already the first round, don't push).
    async fn start_segment(
        &mut self,
        artists: Vec<String>,
        stops: Vec<crate::engine::Stop>,
        opening: bool,
        keep_queue: bool,
    ) {
        if stops.is_empty() {
            return;
        }
        let tracks = stops.iter().map(|s| s.title.clone()).collect();
        if opening {
            self.rounds[0].tracks = tracks;
        } else {
            self.rounds.push(Round { artists, tracks });
        }
        // `now` keeps what was queued behind the new segment; the plain
        // and `force` forms replace it (0015)
        if keep_queue {
            for stop in stops.into_iter().rev() {
                self.queue.push_front(stop);
            }
        } else {
            self.queue = stops.into();
        }
        if let Advance::Blocked(why) = self.advance().await {
            self.blocked(&why);
            return;
        }
        // branches for this segment are computed once, up front, so `1`-`3`
        // and `p` work anytime; they are only *shown* on the last track
        self.recompute();
        self.render();
    }

    /// Enqueue more of the current artist. `when` says where they land
    /// (0015): at the end of the branch, right after the current track, or
    /// right after it with the rest dropped.
    async fn encore(&mut self, count: usize, when: When) {
        // the encore's artist is the one of the highlighted line, else of
        // what plays (0020) — not the end of the chain of branches, which
        // is no longer what plays since choosing a branch queues it (Joel,
        // 09/09/2026). Off-catalog track: the last known artist, as for
        // the branches.
        let current = self
            .target()
            .map(|stop| stop.slug.clone())
            .filter(|slug| self.catalog.cards.contains_key(slug))
            .unwrap_or_else(|| self.state().1);
        let (_, _, _, _, played) = self.state();
        // sanding is where depth is wanted: if the card's unplayed tops
        // cannot serve the whole request, go and get the tail first (0012 §1)
        let card = &self.catalog.cards[&current];
        let unplayed = card.tops.iter().filter(|t| !played.contains(*t)).count();
        let tail = if unplayed < count {
            match self.harvest(&current, false) {
                Ok(true) => None,
                Ok(false) => Some("its tail is coming — redo e<n> in a moment".to_string()),
                Err(why) => Some(why),
            }
        } else {
            None
        };
        let (_, _, _, _, played) = self.state();
        let mut stops = crate::engine::encore(
            &self.catalog,
            &current,
            &self.learned,
            &self.tail,
            self.comfort,
            &played,
            count,
            &mut self.rng,
        );
        for stop in &mut stops {
            stop.encore = true;
        }
        // the list shows what was added (↻): only speak when the request
        // is not served, and say why
        if stops.len() < count {
            let name = &self.catalog.cards[&current].name;
            let why = match tail {
                Some(why) => why,
                None if self.comfort.value() == 5 => "the tail is closed in cocoon (:comfort)".to_string(),
                None => "its tail is exhausted".to_string(),
            };
            match stops.len() {
                0 => say!(self, "(nothing unplayed left from {name} — {why})"),
                n => say!(self, "(only {n} from {name} — {why})"),
            }
        }
        if stops.is_empty() {
            return;
        }
        self.rounds.push(Round { artists: Vec::new(), tracks: stops.iter().map(|s| s.title.clone()).collect() });
        // `now` lands behind the highlighted line when it is still to
        // come (Joel, 09/09/2026), right after what plays otherwise
        let before = self.past.len() + usize::from(self.current.is_some());
        let at = self
            .selection
            .filter(|index| *index >= before && index - before < self.queue.len())
            .map_or(0, |index| index - before + 1);
        match when {
            When::EndOfBranch => self.queue.extend(stops),
            When::Now | When::NowForce => {
                if when == When::NowForce {
                    self.queue.truncate(at);
                }
                for (offset, stop) in stops.into_iter().enumerate() {
                    self.queue.insert(at + offset, stop);
                }
            }
        }
        self.render();
    }

    /// Start a journey from a seed — from the home, or at launch. It
    /// replaces the one that was playing: sound, list, branches, all of it.
    async fn start_journey(&mut self, choice: Choice) {
        let (seed, opening_track) = match choice {
            Choice::Artist(slug) => (slug, None),
            Choice::Track { slug, title } => (slug, Some(title)),
        };
        if self.current.is_some() {
            self.sound.stop();
        }
        self.current = None;
        self.loading = false;
        self.progress = None;
        self.past.clear();
        self.queue.clear();
        self.branches.clear();
        self.selection = None;
        self.overlay = None;
        self.help_open = false;
        self.explore = None;
        self.notices.borrow_mut().clear();
        self.rounds = vec![Round { artists: vec![seed.clone()], tracks: Vec::new() }];
        // opening: the chosen track first if the seed was one, then the
        // seed's own tops (Joel's call, 05/09: the seed can be both)
        let mut opening = crate::engine::encore(
            &self.catalog,
            &seed,
            &self.learned,
            &self.tail,
            self.comfort,
            &Default::default(),
            self.size,
            &mut self.rng,
        );
        if let Some(title) = opening_track {
            opening.retain(|stop| stop.title != title);
            opening.insert(
                0,
                crate::engine::Stop {
                    slug: seed.clone(),
                    artist: self.catalog.cards[&seed].name.clone(),
                    title,
                    source: crate::engine::Source::Top,
                    head: None,
                    encore: false,
                },
            );
        }
        self.screen = Screen::Session;
        self.tui.clear();
        self.start_segment(vec![seed], opening, true, false).await;
    }

    /// Play a whole album as a new seed (⏎ on an album line of the
    /// discography, Joel 14/09/2026): its tracks open the journey in order,
    /// then the branches fork from the artist, as any seed.
    async fn start_album(&mut self, slug: &str, name: &str, titles: Vec<String>) {
        if self.current.is_some() {
            self.sound.stop();
        }
        self.current = None;
        self.loading = false;
        self.progress = None;
        self.past.clear();
        self.queue.clear();
        self.branches.clear();
        self.selection = None;
        self.overlay = None;
        self.help_open = false;
        self.explore = None;
        self.notices.borrow_mut().clear();
        let card_tops = self.catalog.cards.get(slug).map(|c| c.tops.clone()).unwrap_or_default();
        let opening: Vec<crate::engine::Stop> = titles
            .into_iter()
            .map(|title| crate::engine::Stop {
                slug: slug.to_string(),
                artist: name.to_string(),
                source: if card_tops.contains(&title) {
                    crate::engine::Source::Top
                } else {
                    crate::engine::Source::Tail
                },
                title,
                head: None,
                encore: false,
            })
            .collect();
        self.rounds = vec![Round { artists: vec![slug.to_string()], tracks: Vec::new() }];
        self.screen = Screen::Session;
        self.tui.clear();
        say!(self, "▶ {name} — the whole album");
        self.start_segment(vec![slug.to_string()], opening, true, false).await;
    }

    /// A key at the home: the sound goes on underneath, `p` holds it, the
    /// rest is the home's grammar.
    async fn on_home_cmd(&mut self, cmd: Cmd) -> bool {
        if matches!(cmd, Cmd::PlayPause) {
            self.toggle_pause();
            return true;
        }
        // the key helper, as when listening (Joel, 10/09/2026): space opens
        // it, it follows the sequence, closes at the entry level, on esc,
        // or at the next gesture
        let was_help = self.help_open;
        let keeps_overlay = match &cmd {
            Cmd::Pending(_) | Cmd::Help(_) => true,
            Cmd::Typing(Some(line)) => line.starts_with(':'),
            Cmd::Up | Cmd::Down | Cmd::Auto => !self.help_open,
            _ => false,
        };
        if !keeps_overlay {
            self.overlay = None;
            self.overlay_scroll = 0;
            self.help_open = false;
        }
        match &cmd {
            Cmd::Help(namespace) => {
                if self.help_open && namespace.is_none() {
                    self.overlay = None;
                    self.help_open = false;
                } else {
                    self.help_open = true;
                    self.help_auto = false;
                    self.help(*namespace);
                }
                return true;
            }
            Cmd::Pending(seq) => self.follow_typing(seq),
            Cmd::Typing(Some(line)) => self.follow_line(line),
            // esc closes the help, and nothing else
            Cmd::Escape if was_help => return true,
            // every `:` command works here as when listening (Joel,
            // 23/09/2026): one table of commands, one helper for it
            Cmd::Colon(text) => {
                let text = text.clone();
                return self.run_colon(&text).await;
            }
            _ => {}
        }
        let live = !self.rounds.is_empty();
        // the track namespace and the horizontal axis work here as when
        // listening, on what plays underneath (Joel, 23/09/2026): the
        // collection holds artists and never highlights a track, so the
        // target rule (0020) lands on the playback foot. The branch, the
        // encores and the queue stay on the listening screen, which shows
        // them.
        match &cmd {
            Cmd::Track('i') if !live => {
                say!(self, "(nothing playing — ti inserts into a journey)");
                return true;
            }
            Cmd::Track(key) => {
                let key = *key;
                self.on_track_key(key).await;
                return true;
            }
            Cmd::Next if live => {
                self.next().await;
                return true;
            }
            Cmd::Prev if live => {
                self.back().await;
                return true;
            }
            Cmd::Next | Cmd::Prev => {
                say!(self, "(nothing playing)");
                return true;
            }
            Cmd::Fork { .. }
            | Cmd::ForkGenerate(_)
            | Cmd::Peek
            | Cmd::Reroll
            | Cmd::ForkUndo
            | Cmd::Wander
            | Cmd::Encore { .. }
            | Cmd::MoveDown
            | Cmd::MoveUp => {
                say!(self, "(f, e, J and K are for listening — r goes there)");
                return true;
            }
            _ => {}
        }
        // `cc` opens the dial here as when listening; the keys that follow
        // go to it before the home sees them
        if matches!(cmd, Cmd::ComfortMode) {
            self.comfort_before = Some(self.comfort);
            say!(self, "comfort zone — ↑↓ to adjust, enter confirms, esc cancels");
            return true;
        }
        // an artist measure taken at the home changes the branches of the
        // session playing underneath
        let measured = matches!(cmd, Cmd::Artist(_));
        let before = self.comfort.value();
        let outcome = self.home.on_cmd(cmd, &self.catalog, &mut self.learned, &mut self.comfort, live);
        if self.comfort.value() != before {
            crate::config::remember_comfort(self.comfort.value());
        }
        // what the home has to say goes in a toast, as when listening
        if let Some(said) = self.home.take_said() {
            say!(self, "{said}");
        }
        if measured && live {
            self.recompute();
        }
        match outcome {
            Outcome::Stay => {}
            Outcome::Back => {
                self.screen = Screen::Session;
                self.tui.clear();
            }
            Outcome::Start(choice) => self.start_journey(choice).await,
            Outcome::Resume(saved) => self.restore(saved).await,
            // `ad` on the highlighted line of the collection: the same
            // modal, laid over the home
            Outcome::Explore { slug, name } => self.open_explore_of(&slug, &name).await,
            // the rest of the `a` namespace, on the highlighted line: the
            // same code as when listening, what it says goes up to the
            // home's line
            Outcome::Artist { key, slug, name } => {
                let carded = slug.is_some();
                let stop = crate::engine::Stop {
                    slug: slug.unwrap_or_default(),
                    artist: name.clone(),
                    title: String::new(),
                    source: crate::engine::Source::Outside,
                    head: None,
                    encore: false,
                };
                if key == 'd' && !carded {
                    // 0016: arriving at an artist means making them a card
                    let slug = crate::generate::slugify(&name);
                    self.generate(&slug, Some(&name), None, None, After::Explore);
                } else if key != 'g' && !carded {
                    self.tell(format!("({name} has no card)"));
                } else {
                    // what it says already goes in a toast through `notice`
                    self.artist_action(key, stop);
                }
            }
            // read the same way as when listening: a MBID as the last word
            // counts from the home too (Joel, 09/09/2026)
            Outcome::Generate(asked) => self.generate_asked(&asked),
            Outcome::Quit => return false,
        }
        true
    }

    /// Load a stop as the current track. If the cache knows its address it
    /// plays now; otherwise it is **shown now and looked up behind** — the
    /// screen never waits for Spotify (Joel, 08/09/2026). The answer comes
    /// back through `Job::Resolved`.
    async fn load_stop(&mut self, stop: crate::engine::Stop) -> Load {
        let heading = format!("▶ {} {} — {}", stop.source.mark(), stop.title, stop.artist);
        let artist_id = self.artist_id_of(&stop.slug);
        // the artist's own discography first: harvested by id, it cannot
        // hand over a namesake's track, which the title search can
        let known = match self.tail_uri(&stop.slug, &stop.title) {
            Some(uri) => Some(Resolved::Track(uri)),
            None => self
                .web
                .try_lock()
                .ok()
                .and_then(|web| web.cached(&stop.title, &stop.artist, artist_id.as_deref())),
        };
        match known {
            Some(Resolved::Track(uri)) => match SpotifyUri::from_uri(&uri) {
                Ok(track) => {
                    // nothing to say: the screen's footer already announces
                    // what plays, saying it again made a duplicate (Joel,
                    // 07/09/2026)
                    self.sound.play(track);
                    self.current = Some(stop);
                    self.loading = false;
                    Load::Playing
                }
                Err(_) => {
                    say!(self, "{heading} — unreadable uri, skipping");
                    Load::Missing
                }
            },
            Some(Resolved::Absent) => {
                say!(self, "{heading} — not found on Spotify, skipping");
                Load::Missing
            }
            Some(Resolved::Failed(why)) => {
                self.web_ok = false;
                Load::Failed(stop, why)
            }
            None => {
                // an identified card without its tail yet: the discography
                // is fetched first, and the track resolved from it when it
                // lands (`Job::Harvested`); the title search only if it
                // cannot be — the search took "The Answer" by Boo to The
                // Boo Radleys (Joel, 23/09/2026)
                let harvesting = artist_id.is_some() && matches!(self.harvest(&stop.slug, true), Ok(false));
                if !harvesting {
                    self.spawn_resolve(&stop.title, &stop.artist, artist_id);
                }
                self.current = Some(stop);
                self.loading = true;
                Load::Playing
            }
        }
    }

    /// The Spotify id of the artist behind a stop, when its card carries
    /// one: what the resolution holds the search's hits to (23/09/2026).
    fn artist_id_of(&self, slug: &str) -> Option<String> {
        self.catalog.cards.get(slug).and_then(|card| card.spotify.clone())
    }

    /// Ask Spotify for a track's address, off the loop. The cache inside
    /// `WebApi` remembers the answer; the loop hears of it as a job.
    fn spawn_resolve(&self, title: &str, artist: &str, artist_id: Option<String>) {
        let web = self.web.clone();
        let tx = self.jobs_tx.clone();
        let (title, artist) = (title.to_string(), artist.to_string());
        tokio::task::spawn_local(async move {
            let result = web.lock().await.resolve(&title, &artist, artist_id.as_deref()).await;
            let _ = tx.send(Job::Resolved { title, artist, result });
        });
    }

    /// A job came back: finish the gesture it was part of.
    async fn on_job(&mut self, job: Job) {
        match job {
            Job::Resolved { title, artist, result } => {
                let is_current = self.loading
                    && self.current.as_ref().is_some_and(|s| s.title == title && s.artist == artist);
                if !is_current {
                    // a prefetch, or a track we already skipped: the cache
                    // is warm, that was the point
                    return;
                }
                self.loading = false;
                // the api answered, whatever it answered — only a failure
                // to reach it says otherwise
                self.web_ok = !matches!(result, Resolved::Failed(_));
                let heading = format!("▶ {title} — {artist}");
                match result {
                    Resolved::Track(uri) => match SpotifyUri::from_uri(&uri) {
                        Ok(track) => self.sound.play(track),
                        Err(_) => {
                            say!(self, "{heading} — unreadable uri, skipping");
                            self.current = None;
                            self.next().await;
                        }
                    },
                    Resolved::Absent => {
                        say!(self, "{heading} — not found on Spotify, skipping");
                        self.current = None;
                        self.next().await;
                    }
                    Resolved::Failed(why) => {
                        if let Some(stop) = self.current.take() {
                            self.queue.push_front(stop);
                        }
                        self.blocked(&why);
                    }
                }
            }
            Job::Searched { query, result } => {
                let Some(finder) = self.finder.as_mut() else { return };
                if finder.asked != query {
                    return;
                }
                finder.spotify = Some(result.map(|tracks| {
                    tracks
                        .into_iter()
                        .map(|hit| {
                            let slug = self.catalog.search_names(&hit.artist, 1).into_iter().next();
                            // album, year and length first: that is what
                            // tells the Olympia take from the studio one
                            let mut note: Vec<String> = Vec::new();
                            if !hit.album.is_empty() {
                                note.push(hit.album.clone());
                            }
                            if !hit.year.is_empty() {
                                note.push(hit.year.clone());
                            }
                            if hit.duration_ms > 0 {
                                let secs = hit.duration_ms / 1000;
                                note.push(format!("{}:{:02}", secs / 60, secs % 60));
                            }
                            note.push(if slug.is_some() { "branches next" } else { "⏎ generates the card" }.to_string());
                            let spotify = (!hit.artist_id.is_empty()).then(|| hit.artist_id.clone());
                            Found {
                                hit: Hit::Track { title: hit.title, artist: hit.artist, uri: hit.uri, slug, spotify },
                                mark: '~',
                                note: note.join(" · "),
                            }
                        })
                        .collect()
                }));
            }
            Job::Harvested { slug, result, quiet } => {
                self.harvesting.remove(&slug);
                let name = self.catalog.cards.get(&slug).map(|c| c.name.clone()).unwrap_or(slug.clone());
                if let Ok(tracks) = &result {
                    // the branches on the table keep their tracks: a list
                    // that changes under the eyes is not wanted (Joel,
                    // 11/09/2026) — the tail serves the next draws
                    self.tail.keep(&slug, tracks.clone());
                }
                // a track shown and waiting on this very harvest: from the
                // tail now, or the title search as a last resort
                if self.loading && self.current.as_ref().is_some_and(|s| s.slug == slug) {
                    let (title, artist) = self.current.as_ref().map(|s| (s.title.clone(), s.artist.clone())).unwrap();
                    match self.tail_uri(&slug, &title).and_then(|uri| SpotifyUri::from_uri(&uri).ok()) {
                        Some(track) => {
                            self.loading = false;
                            self.sound.play(track);
                        }
                        None => {
                            let artist_id = self.artist_id_of(&slug);
                            self.spawn_resolve(&title, &artist, artist_id);
                        }
                    }
                }
                match result {
                    Ok(tracks) => {
                        let count = tracks.len();
                        match self.explore.as_mut().filter(|s| s.slug == slug) {
                            Some(screen) => screen.reload(self.tail.of(&slug), &self.learned),
                            None if quiet => {}
                            // 0 tracks is not a success: Spotify answered but
                            // gave nothing — usually the card's spotify id is
                            // off (Joel, 14/09/2026)
                            None if count == 0 => say!(self, "⚑ discography of {name} — 0 tracks: its Spotify id may be wrong in the card"),
                            None => say!(self, "✓ discography of {name} — {count} tracks cached"),
                        }
                    }
                    Err(why) => match self.explore.as_mut().filter(|s| s.slug == slug) {
                        Some(screen) => {
                            screen.loading = false;
                            screen.notice = format!("⏹ discography — {why}");
                        }
                        None if quiet => {}
                        None => say!(self, "⏹ discography of {name} — {why}"),
                    },
                }
            }
            Job::Generated { slug, after, result } => {
                let draft = match result {
                    Ok(draft) => draft,
                    Err(why) => {
                        self.generating.remove(&slug);
                        self.mark_pending();
                        return self.tell(format!(
                            "⏹ {} — {why}",
                            crate::generate::pretty(&slug)
                        ));
                    }
                };
                // the vector, behind (0019): the text is composed now, from
                // the catalog as it is, the model runs off the loop
                let text = match toml::from_str::<crate::catalog::Card>(&draft.toml) {
                    Ok(card) => crate::embed::text_of(&slug, &card, &self.catalog.cards),
                    Err(_) => String::new(),
                };
                if !crate::embed::model_cached() {
                    self.tell(format!("… vector of {} — first run: the model is downloading (241 MB)", draft.name));
                }
                let tx = self.jobs_tx.clone();
                let reported = slug.clone();
                tokio::task::spawn_local(async move {
                    let vector = tokio::task::spawn_blocking(move || {
                        crate::embed::embed(&[text]).map(|mut v| v.remove(0))
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("vectorization was interrupted ({e})")));
                    let _ = tx.send(Job::Vectorized { slug: reported, after, draft, vector });
                });
            }
            Job::Vectorized { slug, after, draft, vector } => {
                self.generating.remove(&slug);
                let (name, tops, links) = (draft.name.clone(), draft.tops, draft.links);
                let caveats = draft.caveats.join(" · ");
                let (vector, no_vector) = match vector {
                    Ok(v) => (Some(v), None),
                    Err(why) => (None, Some(why)),
                };
                let replaced = self.catalog.cards.get(&slug).map(|c| c.name.clone());
                let learned_dropped = match self.adopt(draft, vector) {
                    Ok(dropped) => dropped,
                    Err(why) => {
                        self.mark_pending();
                        return self.tell(format!("⏹ card of {name} — {why}"));
                    }
                };
                // the card now exists for the engine: the link that asked
                // for it is no longer a gap
                self.missing.retain(|m| m.slug != slug);
                let mut done = match replaced {
                    Some(old) => format!("✓ {name} — card regenerated (was {old}): {tops} top(s), {links} link(s)"),
                    None => format!("✓ {name} — generated card: {tops} top(s), {links} link(s)"),
                };
                if !caveats.is_empty() {
                    done.push_str(&format!(" ({caveats})"));
                }
                match no_vector {
                    None => done.push_str(" · vector computed"),
                    Some(why) => done.push_str(&format!(" · no vector ({why}): navigating by the graph")),
                }
                if learned_dropped {
                    done.push_str(" · learned of the old artist dropped");
                }
                self.tell(done);
                // what is already on the axis for this artist — a track
                // inserted off-catalog, maybe playing right now — joins its
                // card: the measures finally have somewhere to write
                self.attach_stops(&slug);
                self.after_card(&slug, &name, after).await;
            }
            Job::Diffed(result) => {
                self.catalog_busy = false;
                match result {
                    Ok(diff) => {
                        let title = format!("C diff — {}", if diff.is_empty() { "nothing beyond the reference".to_string() } else { format!("{} card(s) beyond the reference", diff.new.len() + diff.edited.len()) });
                        self.overlay = Some((title, diff.lines(usize::MAX)));
                    }
                    Err(why) => self.tell(format!("⏹ diff: {why}")),
                }
            }
            Job::Proposed(result) => {
                self.catalog_busy = false;
                match result {
                    Ok(proposal) => {
                        if let Some(url) = &proposal.existing {
                            // one proposal open at a time: the push updated it
                            self.tell(format!("↻ proposal updated — {url}"));
                            return;
                        }
                        if !proposal.through_gh {
                            self.tell(format!("→ gh not found or not logged in: the comparison page opened in the browser — {} cards, {} → {}: read, then click", proposal.count, proposal.head, proposal.upstream));
                            return;
                        }
                        // the question first, in sight — the body was
                        // pushing it under the fold (Joel, 20/09/2026);
                        // the new cards are abridged as on the mockup, Cd
                        // has them all
                        let mut lines = vec![
                            "✓ fetch upstream · worktree proposal, from upstream/main".to_string(),
                            format!("✓ commit {} · push --force", proposal.title),
                            "  the learned goes on committing on main meanwhile".to_string(),
                            String::new(),
                            format!("from   {}  →  {}:main", proposal.head, proposal.upstream),
                            format!("title  {}", proposal.title),
                            String::new(),
                            format!(
                                "↻ open the pull request?  y — any other key sends nothing, the pushed branch stays  (gh logged in{})",
                                proposal.account.as_ref().map(|a| format!(" — {a}")).unwrap_or_default()
                            ),
                            format!("gh pr create --head {}", proposal.head),
                            String::new(),
                        ];
                        lines.extend(abridged_body(&proposal.body).into_iter().map(|l| format!("  {l}")));
                        lines.push(String::new());
                        lines.push("  the reference's action checks the toml, the unique mbids, the slugs and the link targets;".to_string());
                        lines.push("  new cards only: it merges on its own once green — an edited card waits for a reader".to_string());
                        self.overlay = Some(("C propose — the pull request, as it will leave".into(), lines));
                        self.pending_proposal = Some(proposal);
                    }
                    Err(why) => self.tell(format!("⏹ propose: {why}")),
                }
            }
            Job::PullRequest(result) => match result {
                Ok(url) => self.tell(format!("✓ pull request opened — {url}")),
                Err(why) => self.tell(format!("⏹ pull request: {why}")),
            },
            Job::Updated(result) => {
                self.catalog_busy = false;
                match result {
                    Ok(update) if update.conflicts.is_empty() => {
                        self.conflicts.clear();
                        // the session owns its catalog: a card that arrived
                        // from the reference may fill a gap on show
                        if update.cards > 0 {
                            match Catalog::load(&self.catalog_dir) {
                                Ok(mut catalog) => {
                                    self.learned.weave_into(&mut catalog);
                                    self.catalog = catalog;
                                    self.recompute();
                                }
                                Err(why) => self.tell(format!("⏹ catalog not reloaded: {why} — relaunch")),
                            }
                        }
                        self.tell(format!("⇅ {}", update.word));
                    }
                    Ok(update) => {
                        let mut lines = vec!["conflict on cards/ — the merge is stopped, the hand is yours".to_string(), String::new()];
                        for (slug, why) in &update.conflicts {
                            lines.push(format!("  ⊘ {slug} — {why}"));
                        }
                        lines.push(String::new());
                        lines.push("the other cards are merged and wait in the index. nothing lost, nothing overwritten — the learned is not concerned.".to_string());
                        lines.push(String::new());
                        lines.push("  o         open the first card — git add once resolved".to_string());
                        lines.push("  :catalog  resume — the merge completes, the index regenerates".to_string());
                        lines.push("  git merge --abort, by hand, to give it up".to_string());
                        self.conflicts = update.conflicts.iter().map(|(slug, _)| slug.clone()).collect();
                        self.overlay = Some(("C update — stopped on cards".into(), lines));
                    }
                    Err(why) => self.tell(format!("⏹ update: {why}")),
                }
            }
            Job::Existing { slug, after } => {
                let name = self.catalog.cards.get(&slug).map(|c| c.name.clone()).unwrap_or(slug.clone());
                self.after_card(&slug, &name, after).await;
            }
        }
    }

    /// What was meant for the artist, now that the card is there — fresh
    /// from the generator or found in the catalogue.
    async fn after_card(&mut self, slug: &str, name: &str, after: After) {
        match after {
            After::Play { title, uri } => self.play_fresh(slug, title, uri).await,
            After::Branch { when, reason, proximity } => self.branch_to(slug, when, reason, proximity).await,
            After::Gap { reason, proximity } => {
                match self.branch_of(slug, reason, proximity) {
                    Some(branch) => {
                        self.branches.push(branch);
                        let n = self.branches.len();
                        self.tell(format!("✓ {name} — card ready · branch {n}"));
                    }
                    None => self.tell(format!("✓ {name} — card ready, but nothing of it plays yet")),
                }
                self.refresh_gaps(slug);
            }
            After::Card => self.paint(),
            After::Explore => self.open_explore_of(slug, name).await,
            After::Offer => {
                self.pending_seed = Some((slug.to_string(), name.to_string(), std::time::Instant::now()));
                self.linger(
                    format!("✓ {name} — card ready. ⏎ starts from them (replaces what plays); any other key keeps the current list"),
                    SEED_OFFER_SECONDS,
                );
            }
        }
    }

    /// What the header shows of the connections — asked, not read off the
    /// disk. `Status::read()` only ever checked that a credentials file
    /// exists, and a file stays there when the session is long gone (Joel,
    /// 2026-09-25: nothing played all morning under a green tick).
    fn status_now(&self) -> Vec<(String, bool)> {
        let sound = self.sound.alive();
        let mut rows = vec![
            (if sound { "✓ librespot" } else { "⏹ librespot — session lost, q then relaunch" }.to_string(), sound),
            (if self.web_ok { "✓ api web" } else { "⏹ api web — no answer" }.to_string(), self.web_ok),
        ];
        // whatever the caller put after those two — the sync word — is its
        // own business and stays as it was given
        rows.extend(self.status.iter().skip(2).cloned());
        rows
    }

    /// Say something, whatever the screen: **everything is said in a
    /// toast**, under the home as when listening (Joel, 10/09/2026). A
    /// background job may finish on either — generation takes a few
    /// seconds, and the screen may have changed meanwhile.
    fn tell(&mut self, what: String) {
        // under the home as when listening: a toast (Joel, 10/09/2026)
        say!(self, "{what}");
    }

    /// Carry over to the shown gaps what the session is generating, so the
    /// column says it instead of staying mute.
    fn mark_pending(&mut self) {
        for missing in &mut self.missing {
            missing.pending = self.generating.contains(&missing.slug);
        }
    }

    /// Bring a fresh card into the session: the disk, the commit — it is an
    /// edit (0013) — then the catalog **in memory**, without which it would
    /// only exist at the next launch. Over a card that exists, this is a
    /// regeneration: the file is rewritten, the commit says who it was, and
    /// when the artist behind the slug changed, what was learned about the
    /// old one goes in the same commit (Joel, 23/09/2026). Returns whether
    /// it did.
    fn adopt(&mut self, draft: crate::generate::Draft, vector: Option<Vec<f32>>) -> Result<bool, String> {
        let card: crate::catalog::Card = toml::from_str(&draft.toml)
            .map_err(|e| format!("the composed card does not read back ({e})"))?;
        // the text the vector came from, fingerprinted into the index so a
        // later regeneration knows this line is current (2026-09-21)
        let text = crate::embed::text_of(&draft.slug, &card, &self.catalog.cards);
        let (mut edit, identity_changed) = match self.catalog.cards.get(&draft.slug) {
            Some(old) => crate::edit::regenerate_card(
                &self.catalog_dir,
                &draft.slug,
                &draft.name,
                &draft.toml,
                draft.tops,
                draft.links,
                &old.name,
            )?,
            None => (
                crate::edit::create_card(
                    &self.catalog_dir,
                    &draft.slug,
                    &draft.name,
                    &draft.toml,
                    draft.tops,
                    draft.links,
                )?,
                false,
            ),
        };
        let mut learned_dropped = false;
        if identity_changed {
            if let Some(path) = self.learned.forget(&draft.slug) {
                edit.removed.push(path);
                edit.body = Some(format!(
                    "{}\nlearned/ of the old artist dropped.",
                    edit.body.take().unwrap_or_default()
                ));
                learned_dropped = true;
            }
        }
        // the vector goes in the same commit (0019): the index shipped with
        // the fork never lags behind its cards
        if let Some(vector) = &vector {
            edit.also.push(crate::embed::write_vector(&self.catalog_dir, &draft.slug, &text, vector)?);
        }
        crate::edit::commit(&self.catalog_dir, &edit)?;
        self.catalog.cards.insert(draft.slug.clone(), card);
        if let Some(vector) = vector {
            self.catalog.vectors.insert(draft.slug, vector);
        }
        Ok(learned_dropped)
    }

    /// Ask for the card of an artist who has none. Four network calls and
    /// the one-second gap MusicBrainz demands: it runs in the background,
    /// like a discography harvest, and the screen does not wait for it.
    ///
    /// Over an artist who **has** a card, an explicit `mbid` regenerates
    /// it from the sources (Joel, 23/09/2026 — the `boo` card was Boo! of
    /// South Africa); without one, the card stands and what was meant for
    /// the artist happens anyway. `spotify` is the id the caller holds;
    /// when it holds none, the library's, if the artist is in the
    /// collection.
    fn generate(&mut self, slug: &str, hint: Option<&str>, mbid: Option<&str>, spotify: Option<&str>, after: After) {
        let existing = self.catalog.cards.get(slug).map(|c| c.name.clone());
        if let Some(name) = &existing {
            if mbid.is_none() {
                // nothing to generate: what was meant for the artist still
                // happens, by the same path as a fresh card
                self.tell(format!("({name} already has a card)"));
                let _ = self.jobs_tx.send(Job::Existing { slug: slug.to_string(), after });
                return;
            }
        }
        if !self.generating.insert(slug.to_string()) {
            let name = crate::generate::pretty(slug);
            self.tell(format!("(the card of {name} is already underway)"));
            return;
        }
        self.mark_pending();
        let name = hint.map(String::from).unwrap_or_else(|| crate::generate::pretty(slug));
        match &existing {
            Some(old) => self.tell(format!("… card of {name} — regenerating (was {old}), musicbrainz then deezer")),
            None => self.tell(format!("… card of {name} — musicbrainz then deezer, a few seconds")),
        }
        let known = crate::generate::Known::of(&self.catalog);
        let spotify = spotify.map(String::from).or_else(|| self.learned.spotify_of(slug));
        let (asked, hint, mbid) = (slug.to_string(), hint.map(String::from), mbid.map(String::from));
        let reported = asked.clone();
        let tx = self.jobs_tx.clone();
        tokio::task::spawn_local(async move {
            let result = tokio::task::spawn_blocking(move || {
                crate::generate::draft(&asked, hint.as_deref(), mbid.as_deref(), spotify.as_deref(), &known)
            })
            .await
            .unwrap_or_else(|e| Err(format!("generation was interrupted ({e})")));
            let _ = tx.send(Job::Generated { slug: reported, after, result });
        });
    }

    /// `:generate <name> [mbid] [spotify-id]`, from either screen: the name
    /// as typed, then the ids, told apart by their shape — a MusicBrainz id
    /// in place of the search by name, a Spotify id to put in the card and
    /// to hold the search to (Joel, 23/09/2026). An artist proposed as a
    /// gap takes its branch instead of a jump. Over an existing card, the
    /// MBID is what regenerates it: a Spotify id alone is not an identity
    /// (0009).
    fn generate_asked(&mut self, asked: &str) {
        let mut words: Vec<&str> = asked.split_whitespace().collect();
        let (mut mbid, mut spotify) = (None, None);
        for _ in 0..2 {
            match words.last() {
                Some(last) if crate::generate::is_mbid(last) && mbid.is_none() => {
                    mbid = words.pop().map(str::to_lowercase);
                }
                Some(last) if crate::generate::is_spotify_id(last) && spotify.is_none() => {
                    spotify = words.pop().map(String::from);
                }
                _ => break,
            }
        }
        let name = words.join(" ");
        if name.is_empty() {
            self.tell("usage: :generate <artist name> [mbid] [spotify-id]".to_string());
            return;
        }
        let slug = crate::generate::slugify(&name);
        if let Some(card) = self.catalog.cards.get(&slug).filter(|_| mbid.is_none()) {
            self.tell(format!(
                "({} already has a card — :generate {name} <mbid> [spotify-id] regenerates it)",
                card.name
            ));
            return;
        }
        let after = match self.missing.iter().find(|m| m.slug == slug) {
            Some(missing) => After::Branch {
                when: When::EndOfBranch,
                reason: missing.why.clone(),
                proximity: missing.proximity,
            },
            // with an mbid, over a running list, do not overwrite it: make
            // the card and offer the seed instead (Joel, 11/09/2026)
            None if mbid.is_some() && (self.current.is_some() || !self.queue.is_empty()) => After::Offer,
            None => After::Play { title: None, uri: None },
        };
        self.generate(&slug, Some(&name), mbid.as_deref(), spotify.as_deref(), after);
    }

    /// Give the stops of an artist who just got a card their slug: they
    /// were inserted from a Spotify search, off the map, under the name the
    /// card was asked for (Joel, 10/09/2026 — `tl` said "nothing to learn"
    /// on a track that was playing).
    fn attach_stops(&mut self, slug: &str) {
        let Some(card) = self.catalog.cards.get(slug) else { return };
        let tops = card.tops.clone();
        let stops = self.past.iter_mut().chain(self.current.iter_mut()).chain(self.queue.iter_mut());
        for stop in stops {
            if stop.slug.is_empty() && crate::generate::slugify(&stop.artist) == slug {
                stop.slug = slug.to_string();
                stop.source = if tops.contains(&stop.title) {
                    crate::engine::Source::Top
                } else {
                    crate::engine::Source::Outside
                };
            }
        }
    }

    /// A card was just born and we wanted to hear it: from the home the
    /// journey starts there, in session it plays right away — which is what
    /// `:search` already does with a catalog artist.
    async fn play_fresh(&mut self, slug: &str, title: Option<String>, uri: Option<String>) {
        if self.screen == Screen::Home {
            let choice = match title {
                Some(title) => Choice::Track { slug: slug.to_string(), title },
                None => Choice::Artist(slug.to_string()),
            };
            self.start_journey(choice).await;
            return;
        }
        let card = &self.catalog.cards[slug];
        let stop = match title {
            Some(title) => crate::engine::Stop {
                slug: slug.to_string(),
                artist: card.name.clone(),
                title: title.clone(),
                source: if card.tops.contains(&title) {
                    crate::engine::Source::Top
                } else {
                    crate::engine::Source::Outside
                },
                head: None,
                encore: false,
            },
            None => {
                let (_, _, _, _, played) = self.state();
                let mut stops = crate::engine::encore(
                    &self.catalog, slug, &self.learned, &self.tail, self.comfort, &played, 1,
                    &mut self.rng,
                );
                let Some(stop) = stops.pop() else {
                    say!(self, "(nothing to play from {})", card.name);
                    return;
                };
                stop
            }
        };
        let artists = vec![slug.to_string()];
        match uri {
            Some(uri) => self.play_uri(artists, stop, &uri).await,
            None => self.play_stop_now(artists, stop).await,
        }
    }

    /// A gap branch just got its card: from now on it is taken like any
    /// other — **walked** like any other, the fresh card leading and other
    /// artists after it, not an encore of the head (Joel, 20/09/2026).
    async fn branch_to(&mut self, slug: &str, when: When, reason: String, proximity: u8) {
        let Some(branch) = self.branch_of(slug, reason, proximity) else {
            say!(self, "(nothing to play from {})", self.catalog.cards[slug].name);
            return;
        };
        self.take_branch(branch, when).await;
        self.render();
    }

    /// The branch a fresh card leads, from where the playlist stands.
    fn branch_of(&mut self, slug: &str, reason: String, proximity: u8) -> Option<crate::engine::Branch> {
        let (context, _, _, visited, played) = self.state();
        crate::engine::branch_from(
            &self.catalog, &context, slug, reason, proximity as f32, &self.learned, &self.tail,
            self.comfort, &visited, &played, self.size, &mut self.rng,
        )
    }

    /// An outage stops the walk rather than turning every remaining track
    /// into a "not found". Nothing is lost — the queue kept its head.
    fn blocked(&self, why: &str) {
        say!(self, "\n⏹ playback interrupted: {why}.");
        say!(self, "   The track stays at the head of the queue — j to retry.");
        say!(self, "   If it persists: q then relaunch — the Spotify");
        say!(self, "   authorization is asked again by itself if it expired.");
    }

    /// Move to the next track (the current one falls into the past). No
    /// recursion — the caller decides what to do with the outcome (keeps
    /// the async futures sized).
    async fn advance(&mut self) -> Advance {
        if let Some(current) = self.current.take() {
            // a track that never played does not enter the past
            if !self.loading {
                self.past.push(current);
            }
        }
        self.loading = false;
        while let Some(stop) = self.queue.pop_front() {
            match self.load_stop(stop).await {
                Load::Playing => return Advance::Playing,
                Load::Missing => {}
                Load::Failed(stop, why) => {
                    // a fault that has nothing to do with these tracks must
                    // not consume them: put the head back and hold
                    self.queue.push_front(stop);
                    return Advance::Blocked(why);
                }
            }
        }
        Advance::Exhausted
    }

    /// Step back to the previous track, like a player's "previous". The
    /// current track goes back to the front of the queue so `j` returns to it.
    async fn back(&mut self) {
        // like any player: "previous" first restarts the current track,
        // and only goes back to the previous one a second time — or when
        // still at its start (Joel, 08/09/2026)
        let position = self.progress.as_ref().map_or(0, |p| p.now().0);
        if self.current.is_some() && !self.loading && position > RESTART_AFTER_MS {
            self.sound.restart();
            return;
        }
        let interrupted = self.current.take();
        loop {
            match self.past.pop() {
                Some(previous) => match self.load_stop(previous).await {
                    Load::Playing => {
                        if let Some(current) = interrupted {
                            self.queue.push_front(current);
                        }
                        return;
                    }
                    Load::Missing => {}
                    Load::Failed(stop, why) => {
                        // keep the past intact, stay where we were
                        self.past.push(stop);
                        self.current = interrupted;
                        self.blocked(&why);
                        return;
                    }
                },
                None => {
                    self.current = interrupted;
                    say!(self, "(already at the first track)");
                    return;
                }
            }
        }
    }

    /// Move on from the current track: a branch chosen during it starts now
    /// (this is where a deferred choice fires); otherwise the segment plays
    /// its next track, and when it runs out the music auto-advances.
    /// Since choosing a branch **adds** it to the queue, there is no
    /// "pending" branch anymore: everything decided is in the queue.
    async fn next(&mut self) {
        match self.advance().await {
            Advance::Playing => {}
            // nothing ahead and nothing prepared: draw
            Advance::Exhausted => self.auto_advance().await,
            Advance::Blocked(why) => self.blocked(&why),
        }
    }

    async fn on_track_over(&mut self, finished: bool, refused: bool) {
        // read before clearing: what the track had played says whether it
        // ended on its own or was cut short
        let played_ms = self.progress.as_ref().map_or(0, |p| p.now().0);
        self.progress = None;
        // A track that ends without having played is not a skip we asked
        // for, and it used to pass in silence — a whole morning of tracks
        // marching by with nothing said (Joel, 2026-09-25). A stop we
        // caused lands here too, but always with a position behind it.
        if !finished {
            if let Some(stop) = self.current.clone() {
                if refused {
                    say!(self, "\n⏹ {} — {}: Spotify would not play it here, skipped", stop.title, stop.artist);
                } else if played_ms == 0 {
                    say!(self, "\n⏹ {} — {}: nothing came out, skipped", stop.title, stop.artist);
                }
            }
        }
        // a track played through is the only thing that counts as a listen
        if finished {
            if let Some(stop) = self.current.clone() {
                if !stop.slug.is_empty() {
                    self.learned.played(&stop.slug, &stop.title);
                }
            }
        }
        self.next().await;
    }

    /// The journey's state, with what the learned layer excludes folded in.
    /// The engine already excludes by slug (`visited`) and by title
    /// (`played`); a ban rides those channels rather than a new parameter
    /// threaded through every call.
    ///
    /// The playlist as it stands comes first (Joel, 17/09/2026): what `ti`
    /// inserted, what the discography's `e` queued, a line moved with
    /// `J`/`K` — none of it enters a round, and `fr` proposed from the
    /// last round instead of from the end of the list. So the context is
    /// the list's last segment, its artists join the universe, and every
    /// title on the axis — past, current, queue — counts as played. The
    /// rounds remain the fallback (nothing known on the axis) and the
    /// ledger `fu` pops.
    fn state(&self) -> (Vec<String>, String, Vec<String>, HashSet<String>, HashSet<String>) {
        let (mut context, mut current, mut universe, mut visited, mut played) =
            state_of(&self.rounds);
        visited.extend(self.learned.banned_artists().cloned());
        played.extend(self.learned.banned_tracks().cloned());
        let axis = || self.past.iter().chain(self.current.iter()).chain(self.queue.iter());
        played.extend(axis().map(|stop| stop.title.clone()));
        for stop in axis().filter(|stop| self.catalog.cards.contains_key(&stop.slug)) {
            if !universe.contains(&stop.slug) {
                universe.push(stop.slug.clone());
            }
            visited.insert(stop.slug.clone());
        }
        let segment = crate::engine::segment_of(axis(), |slug| self.catalog.cards.contains_key(slug));
        if let Some(last) = segment.last() {
            current = last.clone();
            context = segment;
        }
        (context, current, universe, visited, played)
    }

    /// Start a chosen branch (records it, plays its first track, shows it).
    async fn start_branch(&mut self, branch: crate::engine::Branch, when: When) {
        self.start_segment(branch.artists, branch.stops, false, when == When::Now).await;
    }

    /// Show what plays now and what comes next; on the segment's last track,
    /// show the branches instead of an empty "up next" (Joel, 04/09/2026).
    /// The axis and the panel are drawn from the state: nothing is left to
    /// print here. The method stays as an anchor for the existing calls.
    fn render(&self) {}

    /// Resolve the *next* track's uri ahead of time so the transition is
    /// instant instead of waiting on `/v1/search` when the current track
    /// ends. The next track is a pending branch's first stop if one is
    /// waiting, otherwise the head of the queue; nothing to do at an
    /// undecided last track (we don't guess the auto-pick — later, maybe).
    /// resolve() caches, so this is a no-op once warmed.
    async fn prefetch_next(&mut self) {
        let Some(stop) = self.queue.front() else { return };
        let (slug, title, artist) = (stop.slug.clone(), stop.title.clone(), stop.artist.clone());
        if self.tail_uri(&slug, &title).is_some() {
            return;
        }
        let artist_id = self.artist_id_of(&slug);
        // an identified card: its discography is what the track will be
        // resolved from, so that is what to fetch ahead
        if artist_id.is_some() && !self.tail.has(&slug) {
            let _ = self.harvest(&slug, true);
            return;
        }
        // already known, or lock held by a call in flight: nothing to launch
        let known = self
            .web
            .try_lock()
            .map(|web| web.cached(&title, &artist, artist_id.as_deref()).is_some())
            .unwrap_or(true);
        if !known {
            self.spawn_resolve(&title, &artist, artist_id);
        }
    }

    /// The address of a card's top in the artist's harvested discography,
    /// when the tail is there and names it.
    fn tail_uri(&self, slug: &str, title: &str) -> Option<String> {
        if slug.is_empty() {
            return None;
        }
        crate::discography::find(self.tail.of(slug), title).map(|t| t.uri.clone())
    }

    /// See the branches on demand, wherever we are in the segment (`p`).
    /// A richer "preview then pick ahead" belongs to the future GUI.
    /// `fp` — the panel no longer hides, it is always on the right. The key
    /// stays to say so rather than do nothing.
    fn preview(&self) {
        say!(self, "the branches are always shown, on the right");
    }

    /// Play an exact Spotify uri now (from `/` search), as a fresh segment.
    async fn play_uri(&mut self, round_artists: Vec<String>, stop: crate::engine::Stop, uri: &str) {
        let Ok(track) = SpotifyUri::from_uri(uri) else {
            say!(self, "unreadable uri, not playing");
            return;
        };
        if let Some(current) = self.current.take() {
            self.past.push(current);
        }
        let off_map = round_artists.is_empty();
        self.rounds.push(Round { artists: round_artists, tracks: vec![stop.title.clone()] });
        self.queue.clear();
        say!(self, "\n▶ {} {} — {}", stop.source.mark(), stop.title, stop.artist);
        self.sound.play(track);
        self.current = Some(stop);
        if off_map {
            say!(self, "(off-catalog — the branches will start again from the last known artist)");
        }
        self.recompute();
        self.render();
    }

    /// A media-key / MPRIS control, mapped to the same actions as the keys.
    async fn on_control(&mut self, control: Control) {
        match control {
            Control::Next => self.next().await,
            Control::Previous => {
                self.back().await;
                self.render();
            }
            Control::PlayPause => {
                self.paused = !self.paused;
                if self.paused {
                    self.sound.pause();
                } else {
                    self.sound.resume();
                }
            }
            Control::Stop => {
                self.paused = true;
                self.sound.stop();
                // the Stopped event we just caused must not be read as a
                // track ending (which would advance) — drop the current id
                self.current_request_id = None;
                say!(self, "\n⏹ stopped");
            }
        }
    }

    fn recompute(&mut self) {
        let (context, _, universe, visited, played) = self.state();
        self.branches = crate::engine::propose(
            &self.catalog, &context, &universe, &self.learned, &self.tail, self.comfort, &visited,
            &played, self.size, &mut self.rng,
        );
        // "less often" / "more often" ride on the branches that
        // start with the artist concerned (0014)
        for branch in &mut self.branches {
            if let Some(first) = branch.artists.first() {
                branch.weight *= self.learned.weight(first);
            }
        }
        // the links that lead nowhere: no longer thrown away, they are
        // proposed (0016 — on-the-fly generation)
        self.missing = crate::engine::missing_neighbors(&self.catalog, &context, &visited);
        self.missing.truncate(3);
        self.mark_pending();
        self.harvest_proposed();
    }

    /// The tails of the artists the proposed branches walk through, fetched
    /// behind as soon as the dial opens (Joel, 11/09/2026: at comfort 3 the
    /// tail never came up — a branch never went to get it, only `e<n>`,
    /// `:warm` and `ad` did). One request per artist, cached for good;
    /// a card without a Spotify id is simply skipped.
    fn harvest_proposed(&mut self) {
        if !self.comfort.wants_tail() {
            return;
        }
        let slugs: Vec<String> = self
            .branches
            .iter()
            .flat_map(|branch| branch.artists.iter().cloned())
            .filter(|slug| !self.tail.has(slug))
            .collect();
        for slug in slugs {
            let _ = self.harvest(&slug, true);
        }
    }

    /// Take a branch (by weight) and start playing it. The draw is over the
    /// branches *on show*: proposing three then playing a fourth made
    /// "enter" unreadable (Joel, 05/09/2026).
    async fn auto_advance(&mut self) {
        // a wall of unplayable tracks (region-locked, pulled from Spotify)
        // must not spin the branches forever with nothing on air: after a
        // few dry rounds, stop and hand it back (Joel, 11/09/2026)
        if self.dry_advances >= MAX_DRY_ADVANCES {
            self.dry_advances = 0;
            say!(self, "\n(nothing would play — {MAX_DRY_ADVANCES} branches in a row went silent.");
            say!(self, "   Tracks may be unavailable in your region. f<n> to pick, or q.)");
            return;
        }
        self.dry_advances += 1;
        if self.branches.is_empty() {
            // a gap is not a branch: it needs the network before it can
            // play, and a random draw does not keep anyone waiting (0016)
            match self.missing.len() {
                0 => say!(self, "\n(dead end — u to go back, q to quit)"),
                n => say!(
                    self,
                    "\n(nothing to play right away — {n} card(s) to generate, f1 to f{n})"
                ),
            }
            return;
        }
        let weights: Vec<f32> = self.branches.iter().map(|b| b.weight.max(0.1)).collect();
        let index = WeightedIndex::new(&weights).unwrap().sample(&mut self.rng);
        let branch = self.branches.swap_remove(index);
        self.start_branch(branch, When::EndOfBranch).await;
    }

    /// Choosing a branch **adds it to what is already decided** instead of
    /// replacing it (Joel, 06/09/2026): chain a few choices and the rest of
    /// the evening is set. The next branches are then proposed from the
    /// **end** of the queue, not from what plays — the "chain" of the
    /// mockups.
    async fn choose(&mut self, n: usize, when: When) {
        // gaps are numbered after the branches: choosing one asks for its
        // card, and the branch is taken when it arrives
        if n > self.branches.len() && n <= self.branches.len() + self.missing.len() {
            let missing = &self.missing[n - self.branches.len() - 1];
            let (slug, reason, proximity) = (missing.slug.clone(), missing.why.clone(), missing.proximity);
            self.generate(&slug, None, None, None, After::Branch { when, reason, proximity });
            return;
        }
        if n == 0 || n > self.branches.len() {
            say!(self, "choice not understood");
            return;
        }
        let branch = self.branches.remove(n - 1);
        self.take_branch(branch, when).await;
    }

    /// `fg<n>` — the card of gap n, without taking it: the gap becomes a
    /// branch on show, and the column stays as it was read. `f<n>` still
    /// takes it when the card is there (Joel, 19/09/2026).
    fn generate_gap(&mut self, n: usize) {
        if n >= 1 && n <= self.branches.len() {
            say!(self, "(branch {n} has a card already — f{n} takes it)");
            return;
        }
        if n == 0 || n > self.branches.len() + self.missing.len() {
            say!(self, "choice not understood");
            return;
        }
        let missing = &self.missing[n - self.branches.len() - 1];
        let (slug, reason, proximity) = (missing.slug.clone(), missing.why.clone(), missing.proximity);
        self.generate(&slug, None, None, None, After::Gap { reason, proximity });
    }

    /// The gaps, computed again around the context **and** a fresh card:
    /// its own links to the void show in turn — the catalog growing along
    /// its links, one step further. The next recompute reads the playlist
    /// as it stands, like `fr` (Joel, 20/09/2026).
    fn refresh_gaps(&mut self, fresh: &str) {
        let (mut context, _, _, visited, _) = self.state();
        if !context.iter().any(|slug| slug == fresh) {
            context.push(fresh.to_string());
        }
        self.missing = crate::engine::missing_neighbors(&self.catalog, &context, &visited);
        self.missing.retain(|m| !self.branches.iter().any(|b| b.artists.first() == Some(&m.slug)));
        self.missing.truncate(3);
        self.mark_pending();
    }

    /// Put a branch in the queue, where the key asked for it. Separate from
    /// `choose` because a generated card comes by the same path, several
    /// seconds after the keystroke.
    async fn take_branch(&mut self, branch: crate::engine::Branch, when: When) {
        if self.current.is_none() {
            self.start_branch(branch, when).await;
            return;
        }
        let mut stops = branch.stops;
        if let Some(first) = stops.first_mut() {
            // only the first track carries the name: it is the one that
            // opens the branch on screen
            first.head = Some(crate::engine::Head {
                label: branch.label.clone(),
                reason: branch.reason.clone(),
            });
        }
        // nothing to say: the branch shows in the list with its reason
        // (Joel, 07/09/2026 — the footer never grows)
        self.rounds.push(Round {
            artists: branch.artists,
            tracks: stops.iter().map(|s| s.title.clone()).collect(),
        });
        match when {
            When::EndOfBranch => {
                self.queue.extend(stops);
            }
            When::Now => {
                for stop in stops.into_iter().rev() {
                    self.queue.push_front(stop);
                }
            }
            When::NowForce => {
                self.queue.clear();
                self.queue.extend(stops);
            }
        }
        // the next directions start from where the queue ends
        self.recompute();
    }

    /// `x` — remove the track under the selection from the queue. It can
    /// still be proposed: not a ban, a "not tonight".
    /// `J` / `K` — the highlighted line moves one step down or up the
    /// queue. Only what is up next moves: the past is a story, the current
    /// track is nailed. The branch name travels with its track. It shows
    /// in the numbering, it is not said.
    fn move_selected(&mut self, step: isize) {
        let Some(index) = self.selection else {
            say!(self, "(nothing selected — ↑↓ to choose)");
            return;
        };
        let ahead = self.past.len() + usize::from(self.current.is_some());
        if index < ahead {
            say!(self, "(only what is up next can be moved)");
            return;
        }
        let from = index - ahead;
        let to = from as isize + step;
        if to < 0 || to as usize >= self.queue.len() {
            return;
        }
        self.queue.swap(from, to as usize);
        self.selection = Some(ahead + to as usize);
    }

    fn drop_selected(&mut self) {
        let Some(index) = self.selection else {
            say!(self, "(nothing selected — ↑↓ to choose)");
            return;
        };
        if !self.drop_line(index) {
            say!(self, "(only what is up next can be removed)");
        }
    }

    /// Take line `index` of the axis out of the queue. False when the line
    /// is not queued — the past is a story, the current track is nailed.
    fn drop_line(&mut self, index: usize) -> bool {
        // same axis as `move_selected`: past, then the current track if any
        let before = self.past.len() + usize::from(self.current.is_some());
        if index < before || index - before >= self.queue.len() {
            return false;
        }
        let ahead = index - before;
        let Some(stop) = self.queue.remove(ahead) else { return false };
        // if it was the head of a branch, the next one takes its name
        if let Some(head) = stop.head {
            if let Some(next) = self.queue.get_mut(ahead) {
                if next.head.is_none() {
                    next.head = Some(head);
                }
            }
        }
        self.clamp_selection();
        true
    }

    /// One parsed command (0015). Returns false to quit.
    async fn on_cmd(&mut self, cmd: Cmd) -> bool {
        // the search modal takes the keyboard on both screens: it opens
        // from the home too (Joel, 09/09/2026)
        if self.finder.is_some() {
            return self.on_finder_key(cmd).await;
        }
        // the discography modal too: it has its own table (keys.rs), and
        // opens from the home as from the listening (Joel, 10/09/2026)
        if self.explore.is_some() && self.comfort_before.is_none() {
            let go = self.on_explore_key(cmd);
            if let Some(choice) = self.start_requested.take() {
                self.start_journey(choice).await;
            }
            if let Some((slug, name, titles)) = self.album_requested.take() {
                self.start_album(&slug, &name, titles).await;
            }
            return go;
        }
        // the comfort dial takes over everything else — on the home too:
        // the home used to swallow the arrows (Joel, 11/09/2026)
        if self.comfort_before.is_some() {
            self.notices.borrow_mut().clear();
            return self.on_comfort_key(cmd);
        }
        // an overlay longer than the screen scrolls — j/k, the arrows,
        // gg and G — instead of moving the axis under it (Joel, 20/09/2026)
        if let Some((_, lines)) = self.overlay.as_ref().filter(|_| !self.help_open) {
            let max = crate::tui::overlay_scroll_max(lines.len(), self.tui_height());
            let scrolled = match &cmd {
                Cmd::Down => Some(self.overlay_scroll.saturating_add(1)),
                Cmd::Up => Some(self.overlay_scroll.saturating_sub(1)),
                Cmd::Top => Some(0),
                Cmd::Bottom => Some(max),
                Cmd::Unknown(seq) if seq == "j" => Some(self.overlay_scroll.saturating_add(1)),
                Cmd::Unknown(seq) if seq == "k" => Some(self.overlay_scroll.saturating_sub(1)),
                _ => None,
            };
            if let Some(to) = scrolled {
                self.overlay_scroll = to.min(max);
                return true;
            }
        }
        // a proposal waits for its answer: y opens the pull request,
        // anything else sends nothing (Joel, 19/09/2026)
        if self.pending_proposal.is_some() && !matches!(cmd, Cmd::Pending(_) | Cmd::Typing(_) | Cmd::Help(_)) {
            let proposal = self.pending_proposal.take().expect("pending");
            self.overlay = None;
            if matches!(&cmd, Cmd::Unknown(seq) if seq == "y") {
                self.tell("… gh pr create".to_string());
                let tx = self.jobs_tx.clone();
                tokio::task::spawn_local(async move {
                    let result = tokio::task::spawn_blocking(move || crate::fork::create_pull_request(&proposal))
                        .await
                        .unwrap_or_else(|e| Err(format!("interrupted ({e})")));
                    let _ = tx.send(Job::PullRequest(result));
                });
            } else {
                self.tell("(nothing sent — the pushed branch stays, Cp again to ask once more)".to_string());
            }
            return true;
        }
        // the catalog namespace works on both screens (workstream B)
        if let Cmd::Catalog(key) = cmd {
            self.overlay = None;
            self.help_open = false;
            self.catalog_gesture(key);
            return true;
        }
        if matches!(cmd, Cmd::Open) {
            self.overlay = None;
            self.open_conflict();
            return true;
        }
        // `aL` is waiting on a proximity: here a digit says how close, not
        // which branch to take (Joel, 2026-09-23)
        if let Some(pending) = self.link_pending.as_mut() {
            self.notices.borrow_mut().clear();
            match cmd {
                Cmd::Digit(n) if (1..=5).contains(&n) => self.write_connection(n as u8),
                // a notch at a time, as the comfort dial moves (Joel,
                // 23/09/2026 — "h and l, the neovim way")
                Cmd::Prev | Cmd::Down => pending.proximity = (pending.proximity - 1).max(1),
                Cmd::Next | Cmd::Up => pending.proximity = (pending.proximity + 1).min(5),
                Cmd::Auto => {
                    let proximity = pending.proximity;
                    self.write_connection(proximity);
                }
                Cmd::Remove if pending.drawn => self.undraw_connection(),
                Cmd::Escape | Cmd::Remove => {
                    let drawn = self.link_pending.take().is_some_and(|p| p.drawn);
                    if drawn {
                        say!(self, "(connection left as it was)");
                    } else {
                        say!(self, "(connection dropped, nothing drawn)");
                    }
                }
                _ => say!(self, "(h l move, a digit jumps · ⏎ · x · esc)"),
            }
            return true;
        }
        if self.screen == Screen::Home {
            return self.on_home_cmd(cmd).await;
        }
        self.notices.borrow_mut().clear();
        // a block laid over the screen falls at the next gesture — except
        // the key helper, which follows the pending sequence until it
        // completes
        let keeps_overlay = match &cmd {
            Cmd::Pending(_) | Cmd::Help(_) => true,
            Cmd::Typing(Some(line)) => line.starts_with(':'),
            Cmd::Up | Cmd::Down | Cmd::Auto => !self.help_open,
            _ => false,
        };
        if !keeps_overlay {
            self.overlay = None;
            self.overlay_scroll = 0;
            self.help_open = false;
        }
        // a fresh :generate offered a new seed: ⏎ accepts and starts from
        // them, anything else declines and does its own thing; the offer
        // expires with its toast (Joel, 11/09/2026)
        if let Some((slug, _, at)) = &self.pending_seed {
            let fresh = at.elapsed().as_secs() < SEED_OFFER_SECONDS;
            if fresh && matches!(cmd, Cmd::Auto) {
                let slug = slug.clone();
                self.pending_seed = None;
                self.start_journey(Choice::Artist(slug)).await;
                return true;
            }
            self.pending_seed = None;
        }
        // what is typed is only displayed: no action
        match &cmd {
            Cmd::Pending(seq) => {
                self.typed = seq.clone();
                self.follow_typing(seq);
                return true;
            }
            Cmd::Typing(line) => {
                self.typed = line.clone().unwrap_or_default();
                if let Some(line) = line {
                    self.follow_line(line);
                }
                return true;
            }
            Cmd::Unknown(seq) => {
                self.typed.clear();
                say!(self, "(unknown: {seq})");
                return true;
            }
            _ => self.typed.clear(),
        }
        // while `/` results are on screen, a digit picks one of them rather
        // than a branch; anything else dismisses them
        match cmd {
            // `q` no longer quits: it gives the home back, the listening
            // goes on underneath and `r` brings it back (Joel, 08/09/2026)
            Cmd::Quit => {
                self.remember();
                self.screen = Screen::Home;
                self.tui.clear();
            }
            // enter plays what is selected; without a selection it keeps
            // its usual meaning — "choose for me"
            Cmd::Auto => match self.selection.take() {
                Some(index) => self.play_at(index).await,
                None => self.auto_advance().await,
            },
            Cmd::Up => self.move_selection(-1),
            // both ends of the axis: the start of the evening, or the end
            // of what is decided
            Cmd::Top => self.selection = Some(0),
            Cmd::Bottom => self.selection = Some(self.axis_len().saturating_sub(1)),
            Cmd::Down => self.move_selection(1),
            Cmd::Escape => {
                self.selection = None;
                self.overlay = None;
            }
            Cmd::Remove => self.drop_selected(),
            Cmd::MoveDown => self.move_selected(1),
            Cmd::MoveUp => self.move_selected(-1),
            Cmd::ComfortMode => {
                self.comfort_before = Some(self.comfort);
                say!(self, "comfort zone — ↑↓ to adjust, enter confirms, esc cancels");
            }
            // c<n>: the comfort zone in one go (Joel, 08/09/2026)
            Cmd::Comfort(n) => self.colon(&format!("comfort {n}")),

            // --- f, the branch namespace ---
            Cmd::Digit(n) => self.choose(n, When::EndOfBranch).await,
            Cmd::Fork { branch, when } => self.choose(branch, when).await,
            Cmd::ForkGenerate(n) => self.generate_gap(n),
            Cmd::Peek => self.preview(),
            Cmd::Reroll => {
                self.recompute();
                say!(self, "\n\u{21bb} other branches:");
                self.preview();
            }
            Cmd::ForkUndo => self.fork_undo().await,

            // --- e, encore ---
            Cmd::Encore { count, when } => self.encore(count, when).await,

            // --- navigation ---
            Cmd::Next => self.next().await,
            Cmd::Prev => {
                self.back().await;
                self.render();
            }
            Cmd::PlayPause => self.toggle_pause(),
            Cmd::Help(namespace) => {
                // space opens the help, and closes it at the entry level;
                // esc closes it from anywhere
                if self.help_open && namespace.is_none() {
                    self.overlay = None;
                    self.help_open = false;
                } else {
                    self.help_open = true;
                    self.help_auto = false;
                    self.help(namespace);
                }
            }

            // `/` filters a list — the collection, the discography; there
            // is none here, and searching is `:search` (Joel, 08/09/2026)
            Cmd::Search(query) => say!(self, "(/ filters a list — to search: :search {query})"),

            // --- decided (0015), not wired yet ---
            Cmd::Track(k) => self.on_track_key(k).await,
            Cmd::Artist(k) => self.on_artist_key(k),
            // the reader turns `fw` into the `:wander ` line; a bare Wander
            // can only come from elsewhere — it wanders far
            Cmd::Wander => self.wander("").await,
            // two home keys, with no use once listening
            Cmd::Resume => say!(self, "\n(r is for the home: here, fu backs up one branch)"),
            Cmd::Browse => say!(self, "\n(b is for the home: here, the sound is already on)"),
            // already handled above: they only display
            Cmd::Pending(_) | Cmd::Typing(_) | Cmd::Unknown(_) => {}
            Cmd::Sort => say!(self, "(s sorts the collection, at the home)"),
            // a modal's keys: outside of it, they have no purpose
            Cmd::Enqueue | Cmd::Filter | Cmd::AlbumTop | Cmd::Undo => {
                say!(self, "(ad opens the discography: these keys work there)")
            }
            Cmd::Colon(text) => return self.run_colon(&text).await,
            // routed before the screens split
            Cmd::Open | Cmd::Catalog(_) => {}
        }
        // `ad` and `:discography` ask for the tail before opening: the
        // keyboard is not async, the loop is
        if std::mem::take(&mut self.explore_requested) {
            self.open_explore().await;
        }
        true
    }

    /// A source is fixed when a track is drawn, so liking one afterwards
    /// left the old glyph in the list (Joel, 2026-09-23). Put the right one
    /// back on every line holding this track: played, playing, still to come.
    fn remark(&mut self, slug: &str, title: &str) {
        let marks: Vec<crate::engine::Source> = self
            .past
            .iter()
            .chain(self.current.iter())
            .chain(self.queue.iter())
            .map(|stop| crate::engine::current_source(&self.catalog, &self.tail, &self.learned, stop))
            .collect();
        for (stop, mark) in self
            .past
            .iter_mut()
            .chain(self.current.iter_mut())
            .chain(self.queue.iter_mut())
            .zip(marks)
        {
            if stop.slug == slug && stop.title == title {
                stop.source = mark;
            }
        }
    }

    /// `ta` — track about. What the catalogue and the listening know of it,
    /// plus its album, its featurings and its year when the discography has
    /// them (Joel, 14/09/2026 — replaces `?`, the reason folded in).
    /// and what the listening has learned of them: familiarity (our own
    /// decayed plays, or the seed ranking before we ever played them) and
    /// the weight our own "more / less often" has set.
    fn track_about(&mut self, stop: crate::engine::Stop) {
        let title = format!("{} — {}", stop.title, stop.artist);
        let mut lines = Vec::new();
        // the same glyph the list carries, so `tl` reads here too
        let source = crate::engine::current_source(&self.catalog, &self.tail, &self.learned, &stop);
        lines.push(format!(" mark: {} {}", source.mark(), source.word()));
        if let Some(feat) = featuring(&stop.title) {
            lines.push(format!(" featuring: {feat}"));
        }
        if stop.slug.is_empty() {
            lines.push(" off-catalog: played from Spotify, no card".into());
            self.overlay = Some((title, lines));
            return;
        }
        // album and year from the discography, if it has been harvested
        let key = crate::discography::normalize(&stop.title);
        if let Some(t) = self.tail.of(&stop.slug).iter().find(|t| crate::discography::normalize(&t.title) == key) {
            let mut album = t.album.clone();
            if let Some(year) = t.year() {
                album = format!("{album} ({year})");
            }
            if !album.trim().is_empty() {
                lines.push(format!(" album: {album}"));
            }
        }
        let card = &self.catalog.cards[&stop.slug];
        if !card.tags.is_empty() {
            lines.push(format!(" tags: {}", card.tags.join(", ")));
        }
        lines.push(format!(
            " familiarity {:.0}% · weight {:.2} · {} link(s), {} top(s)",
            self.learned.familiarity01(&stop.slug, &card.name) * 100.0,
            self.learned.weight(&stop.slug),
            card.links.len(),
            card.tops.len()
        ));
        match self.branches.first() {
            Some(branch) => lines.push(format!(" from here: {} ({})", branch.label, branch.reason)),
            None => lines.push(" from here: nothing proposed for now".into()),
        }
        self.overlay = Some((title, lines));
    }

    /// The number of lines on the axis: the past, the current track, the
    /// queue.
    fn axis_len(&self) -> usize {
        self.past.len() + usize::from(self.current.is_some()) + self.queue.len()
    }

    /// Move the selection. It starts on the current track, because that is
    /// where we look from.
    fn move_selection(&mut self, step: isize) {
        let len = self.axis_len();
        if len == 0 {
            return;
        }
        let here = self.selection.unwrap_or(self.past.len());
        let next = (here as isize + step).clamp(0, len as isize - 1) as usize;
        self.selection = Some(next);
    }

    /// Play the selected line. What preceded it in the queue goes to the
    /// past: we jump *to* a track, we do not pull it out of order.
    async fn play_at(&mut self, index: usize) {
        let past_len = self.past.len();
        if index == past_len && self.current.is_some() {
            return;
        }
        let stop = if index < past_len {
            self.past.remove(index)
        } else {
            let ahead = index - past_len - usize::from(self.current.is_some());
            if ahead >= self.queue.len() {
                return;
            }
            for _ in 0..ahead {
                if let Some(skipped) = self.queue.pop_front() {
                    self.past.push(skipped);
                }
            }
            match self.queue.pop_front() {
                Some(stop) => stop,
                None => return,
            }
        };
        if let Some(current) = self.current.take() {
            self.past.push(current);
        }
        if let Load::Failed(stop, why) = self.load_stop(stop).await {
            self.queue.push_front(stop);
            self.blocked(&why);
        }
    }

    /// The `c` mode: the arrows move the gauge, enter confirms, esc gives
    /// the previous value back. Nothing is applied until confirmed.
    fn on_comfort_key(&mut self, cmd: Cmd) -> bool {
        let value = self.comfort.value();
        match cmd {
            Cmd::Up | Cmd::Next => self.comfort = Comfort::new((value + 1).min(5)),
            Cmd::Down | Cmd::Prev => self.comfort = Comfort::new(value.saturating_sub(1)),
            Cmd::Auto => {
                self.comfort_before = None;
                // remembered from one launch to the next (Joel, 14/09/2026)
                crate::config::remember_comfort(value);
                // the dial changes which branches make sense — when there are any
                if !self.rounds.is_empty() {
                    self.recompute();
                }
                say!(self, "comfort zone: {} — {}", value, comfort_word(value));
                return true;
            }
            Cmd::Escape => {
                if let Some(before) = self.comfort_before.take() {
                    self.comfort = before;
                }
                return true;
            }
            Cmd::Quit => return false,
            _ => {}
        }
        let value = self.comfort.value();
        say!(self, "comfort zone — {} — {}   ↑↓ adjust · enter confirm · esc cancel",
             value, comfort_word(value));
        true
    }

    /// One more line in the screen's log.
    fn notice(&self, line: String) {
        let mut notices = self.notices.borrow_mut();
        for part in line.split('\n') {
            notices.push(part.to_string());
        }
        let excess = notices.len().saturating_sub(14);
        notices.drain(..excess);
        // everything said goes in a toast: there is no status line under
        // "up next" anymore (Joel, 08/09/2026)
        let first = line.trim().to_string();
        if !first.is_empty() {
            *self.toast.borrow_mut() = Some((first, std::time::Instant::now(), TOAST_SECONDS));
        }
    }

    /// A toast that lingers longer than the usual four seconds — for an
    /// offer that waits on an answer (Joel, 11/09/2026).
    fn linger(&self, line: String, secs: u64) {
        self.notice(line.clone());
        *self.toast.borrow_mut() = Some((line.trim().to_string(), std::time::Instant::now(), secs));
    }

    /// A toast is on screen, or just faded: a repaint is needed.
    fn toast_active(&self) -> bool {
        self.toast
            .borrow()
            .as_ref()
            .is_some_and(|(_, at, secs)| at.elapsed().as_secs_f32() < *secs as f32 + 1.5)
    }

    /// The toast of the moment: what is loading, sticky while it loads;
    /// otherwise the last thing said, four seconds.
    fn toast(&self) -> Option<crate::tui::Toast> {
        // a question waiting on an answer does not fade: `ac` asks how
        // close, and the toast stays until a digit, ⏎ or esc (Joel,
        // 23/09/2026)
        if let Some(pending) = &self.link_pending {
            let text = pending.question();
            return Some(crate::tui::Toast { tone: crate::tui::tone_of(&text), text, sticky: false });
        }
        if self.loading {
            if let Some(stop) = &self.current {
                return Some(crate::tui::Toast {
                    text: format!("loading — {} — {}", stop.title, stop.artist),
                    tone: crate::tui::LOADING,
                    sticky: true,
                });
            }
        }
        if let Some((slug, _)) = self.harvesting.iter().find(|(_, quiet)| !**quiet) {
            let name = self.catalog.cards.get(slug).map(|c| c.name.as_str()).unwrap_or(slug);
            return Some(crate::tui::Toast {
                text: format!("discography of {name} — loading"),
                tone: crate::tui::LOADING,
                sticky: true,
            });
        }
        self.toast
            .borrow()
            .as_ref()
            .filter(|(_, at, secs)| at.elapsed().as_secs() < *secs)
            .map(|(text, _, _)| crate::tui::Toast {
                text: text.clone(),
                tone: crate::tui::tone_of(text),
                sticky: false,
            })
    }

    /// Tell the desktop what plays (0021). Same discipline as the screen:
    /// derived from the state, pushed only when it changed; the needle at
    /// every call, it is one stored value.
    fn mirror(&mut self) {
        let Some(player) = self.mpris.as_ref() else { return };
        let (position_ms, length_ms) = self.progress.as_ref().map_or((0, 0), Progress::now);
        let mut shown = Shown {
            title: self.current.as_ref().map(|s| s.title.clone()).unwrap_or_default(),
            artist: self.current.as_ref().map(|s| s.artist.clone()).unwrap_or_default(),
            next: self
                .queue
                .front()
                .map(|s| format!("{} — {}", s.title, s.artist))
                .unwrap_or_default(),
            length_ms,
            playing: self.current.as_ref().map(|_| !self.paused && !self.loading),
            serial: self.mpris_shown.serial,
        };
        let same_track = shown.title == self.mpris_shown.title && shown.artist == self.mpris_shown.artist;
        if !same_track {
            shown.serial += 1;
        }
        mediakeys::position(player, position_ms);
        let sampled = self.progress.as_ref().map(|p| p.sampled);
        if shown != self.mpris_shown {
            let track_changed = !same_track
                || shown.next != self.mpris_shown.next
                || shown.length_ms != self.mpris_shown.length_ms;
            mediakeys::publish(player, &shown, track_changed, position_ms);
            self.mpris_shown = shown;
        } else if sampled != self.mpris_sampled {
            mediakeys::seeked(player, position_ms);
        }
        self.mpris_sampled = sampled;
    }

    /// Repaint. Everything goes through here: the TUI only shows the state,
    /// it decides nothing.
    fn paint(&mut self) {
        self.mirror();
        if self.screen == Screen::Home {
            let live = !self.rounds.is_empty();
            // the footer is built field by field: the screen needs `tui`
            // exclusively while the rest is read
            let tracks = self.past.len() + usize::from(self.current.is_some()) + self.queue.len();
            let bar = live.then(|| crate::tui::Bar {
                current: self.current.as_ref(),
                paused: self.paused,
                loading: self.loading,
                progress: self.progress.as_ref().map(Progress::now),
                position: (self.past.len() + 1, tracks),
                next: self.queue.front(),
            });
            // the search modal lays over the home as over the listening —
            // it is the same one, opened from elsewhere
            let finder = self.finder.as_ref().map(|finder| self.finder_view(finder));
            self.home.draw(
                &self.catalog,
                &self.learned,
                &self.tail,
                self.comfort,
                self.comfort_before.is_some(),
                &self.status_now(),
                bar,
                live,
                finder,
                self.explore.as_ref(),
                self.overlay.as_ref().map(|(t, l)| (t.as_str(), l.as_slice(), self.overlay_scroll)),
                self.toast(),
                self.tui,
            );
            return;
        }
        // the queue is a ring buffer: after a `push_front` it wraps, and
        // `as_slices().0` would then show only its first half — a branch
        // just taken vanished, or came back a few tracks later, once the
        // head had turned around (Joel, 09/09/2026). Straighten it first.
        self.queue.make_contiguous();
        // the modal says "▶ playing" on the right line, even when the
        // track changes while it is open
        let playing = self.current.as_ref().map(|stop| stop.title.clone());
        if let Some(screen) = self.explore.as_mut() {
            screen.now_playing(playing.as_deref());
        }
        let path: Vec<String> = state_of(&self.rounds)
            .2
            .iter()
            .filter_map(|slug| self.catalog.cards.get(slug).map(|c| c.name.clone()))
            .collect();
        let seed = self.rounds[0].artists.first().cloned().unwrap_or_default();
        let prompt = if !self.typed.is_empty() {
            self.typed.clone()
        } else {
            format!(
                "[1-{} branch · h/l · p · space = the keys · q]",
                (self.branches.len() + self.missing.len()).max(1)
            )
        };
        // 1a: the branch column is always there, each branch unfolded
        // with its tracks (Joel, 07/09/2026)
        let notes: Vec<String> = self
            .past
            .iter()
            .chain(self.current.iter())
            .chain(self.queue.iter())
            .map(|stop| self.note(stop))
            .collect();
        // the seed block (maquette 2b): everything descends from it
        let seed_card = self.catalog.cards.get(&seed);
        let seed_name = seed_card.map(|c| c.name.clone()).unwrap_or_else(|| seed.clone());
        let seed_facts = seed_card
            .map(|c| {
                format!(
                    "{} link{} · {} top{}",
                    c.links.len(),
                    if c.links.len() > 1 { "s" } else { "" },
                    c.tops.len(),
                    if c.tops.len() > 1 { "s" } else { "" },
                )
            })
            .unwrap_or_default();
        let seed_last = match self.learned.days_since(&seed) {
            Some(days) => format!("last played {}", crate::home::age(Some(days))),
            None => "never played".to_string(),
        };
        let view = View {
            path,
            seed: &seed,
            seed_name: &seed_name,
            seed_facts: &seed_facts,
            seed_last: &seed_last,
            forks: self.rounds.len().saturating_sub(1),
            segment: self.rounds.len(),
            past: &self.past,
            current: self.current.as_ref(),
            paused: self.paused,
            loading: self.loading,
            queue: self.queue.as_slices().0, // whole, made contiguous above
            branches: &self.branches,
            missing: &self.missing,
            panel: true,
            notes: &notes,
            selection: self.selection,
            overlay: self.overlay.as_ref().map(|(t, l)| (t.as_str(), l.as_slice(), self.overlay_scroll)),
            comfort_mode: self.comfort_before.is_some(),
            comfort: self.comfort.value(),
            comfort_word: comfort_word(self.comfort.value()),
            progress: self.progress.as_ref().map(Progress::now),
            toast: self.toast(),
            finder: self.finder.as_ref().map(|f| self.finder_view(f)),
            explore: self.explore.as_ref(),
            prompt,
        };
        let _ = self.tui.draw(&view);
    }

    /// Every ten minutes: commit what listening wrote, push in the
    /// background. Nothing to say when nothing moved; a push that is
    /// refused (the other machine pushed first) waits for the next pull.
    fn autosave(&mut self) {
        if !crate::sync::dirty(&self.catalog_dir) {
            return;
        }
        match crate::sync::commit_learned(&self.catalog_dir) {
            Ok(Some(subject)) => {
                let dir = self.catalog_dir.clone();
                let tx = self.sync_tx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(match crate::sync::push(&dir) {
                        Ok(()) => Ok(format!("learned pushed — {subject}")),
                        Err(why) => Err(format!("push refused — {why} (next pull)")),
                    });
                });
            }
            Ok(None) => {}
            Err(why) => say!(self, "⏹ learned commit — {why}"),
        }
    }

    /// Keep the needle in step with what librespot says: position at every
    /// start, pause and seek, duration at every track change. Returns true
    /// when the screen should follow. Events of a request that is not the
    /// current one are ignored — a skipped track may still speak.
    fn follow_needle(&mut self, event: &PlayerEvent) -> bool {
        use PlayerEvent::*;
        let mine = |id: &u64| Some(*id) == self.current_request_id;
        let sampled = std::time::Instant::now();
        let duration_ms = self.progress.as_ref().map_or(0, |p| p.duration_ms);
        match event {
            Playing { play_request_id, position_ms, .. } if mine(play_request_id) => {
                // a track is truly on air: the auto-advance is not dry
                self.dry_advances = 0;
                self.progress =
                    Some(Progress { position_ms: *position_ms, duration_ms, sampled, running: true });
                true
            }
            Paused { play_request_id, position_ms, .. } if mine(play_request_id) => {
                self.progress =
                    Some(Progress { position_ms: *position_ms, duration_ms, sampled, running: false });
                true
            }
            Seeked { play_request_id, position_ms, .. }
            | PositionCorrection { play_request_id, position_ms, .. }
            | PositionChanged { play_request_id, position_ms, .. }
                if mine(play_request_id) =>
            {
                if let Some(p) = self.progress.as_mut() {
                    p.position_ms = *position_ms;
                    p.sampled = sampled;
                }
                true
            }
            TrackChanged { audio_item } => {
                let running = self.progress.as_ref().is_some_and(|p| p.running);
                let position_ms = self.progress.as_ref().map_or(0, |p| p.now().0);
                self.progress = Some(Progress {
                    position_ms,
                    duration_ms: audio_item.duration_ms,
                    sampled,
                    running,
                });
                true
            }
            _ => false,
        }
    }

    /// What the TUI draws of the search modal.
    fn finder_view(&self, finder: &Finder) -> crate::tui::FinderView {
        let mut lines: Vec<crate::tui::FinderLine> = Vec::new();
        let linked = finder.linked_rows();
        if !linked.is_empty() {
            lines.push(crate::tui::FinderLine::Header {
                catalogue: true,
                text: format!("yours  {} — ← → move the closeness · enter opens it, x there undraws", linked.len()),
            });
            for f in linked {
                lines.push(found_line(f, true));
            }
        }
        let cat = finder.catalogue.len();
        if cat > 0 {
            lines.push(crate::tui::FinderLine::Header {
                catalogue: true,
                text: format!("catalog  {cat} result{}", if cat > 1 { "s" } else { "" }),
            });
            for f in &finder.catalogue {
                lines.push(found_line(f, true));
            }
        }
        let mut spotify_count: Option<usize> = None;
        if !finder.only_catalogue {
            match &finder.spotify {
                None => lines.push(crate::tui::FinderLine::Info("[spotify] … querying".to_string())),
                Some(Err(why)) => lines.push(crate::tui::FinderLine::Info(format!("[spotify] unreachable — {why}"))),
                Some(Ok(found)) if !found.is_empty() => {
                    spotify_count = Some(found.len());
                    lines.push(crate::tui::FinderLine::Header {
                        catalogue: false,
                        text: format!("spotify  {} tracks · off-catalog unless noted", found.len()),
                    });
                    for f in found {
                        lines.push(found_line(f, false));
                    }
                }
                Some(Ok(_)) => {}
            }
        }
        if lines.is_empty() && !finder.query.trim().is_empty() {
            lines.push(crate::tui::FinderLine::Info(format!(
                "(nothing for \"{}\" — neither catalog nor spotify)",
                finder.query.trim()
            )));
        }
        let anchor = finder.insert.map(|at| {
            let before = if at == 0 {
                self.current.as_ref().map(|s| s.title.clone())
            } else {
                self.queue.get(at - 1).map(|s| s.title.clone())
            };
            let after = self.queue.get(at).map(|s| s.title.clone());
            match (before, after) {
                (Some(b), Some(a)) => format!("insertion lands at {} — between {b} and {a}", at + 2),
                (Some(b), None) => format!("insertion lands at {} — after {b}, at the end of the queue", at + 2),
                _ => format!("insertion lands at {}", at + 2),
            }
        });
        crate::tui::FinderView {
            insert: finder.insert.is_some(),
            linking: finder.link_from.as_ref().map(|(_, name)| name.clone()),
            anchor,
            query: finder.query.clone(),
            counts: (cat, spotify_count, finder.spotify.is_none() && !finder.only_catalogue),
            only_catalogue: finder.only_catalogue,
            lines,
            cursor: finder.cursor,
        }
    }

    /// The grey note beside a track (maquette 3a, Joel 07/09/2026): what the
    /// listening knows of it — how often it sounded and when, how often it
    /// was skipped — or that it never did.
    fn note(&self, stop: &crate::engine::Stop) -> String {
        if stop.source == crate::engine::Source::Offmap {
            return "off-catalog".to_string();
        }
        let mut parts: Vec<String> = Vec::new();
        match self.learned.track_stats(&stop.slug, &stop.title) {
            Some((plays, days, skipped)) => {
                let n = plays.round().max(1.0) as u64;
                if plays >= 0.5 {
                    parts.push(format!("{n} play{}", if n > 1 { "s" } else { "" }));
                    parts.push(crate::home::age(days));
                } else {
                    parts.push("never played".to_string());
                }
                if skipped > 0 {
                    parts.push(format!("skipped {skipped}×"));
                }
            }
            None => parts.push("never played".to_string()),
        }
        parts.join(" · ")
    }

    /// The track a gesture works on — the highlighted line if there is
    /// one, what plays otherwise (0020) — or a word saying why there is
    /// none.
    fn under_needle(&self) -> Option<crate::engine::Stop> {
        match self.target() {
            None => {
                say!(self, "\n(nothing playing)");
                None
            }
            Some(stop) if stop.slug.is_empty() => {
                if self.generating.contains(&crate::generate::slugify(&stop.artist)) {
                    say!(self, "\n({} — its card is underway, retry in a moment)", stop.artist);
                } else {
                    say!(self, "\n({} — off-catalog, nothing to learn)", stop.artist);
                }
                None
            }
            Some(stop) => Some(stop),
        }
    }

    /// Whether a gesture aims at what plays (no selection, or the
    /// highlighted line is the current track): that is the only case where
    /// "skip" and "ban" also move the music on.
    fn aims_at_playing(&self) -> bool {
        self.current.is_some() && self.selection.map_or(true, |i| i == self.past.len())
    }

    /// The selection must still point at a line once the axis shrank.
    fn clamp_selection(&mut self) {
        let len = self.axis_len();
        if let Some(index) = self.selection {
            self.selection = (len > 0).then(|| index.min(len - 1));
        }
    }

    /// `t` — the current track. Measures write to `learned/` at once and
    /// without asking (0013); the editions still wait for the layer that
    /// writes cards and commits them.
    /// `ti` — track insert: the search modal, anchored where we are in the
    /// list — before the highlighted line if it is up next, right after
    /// what plays otherwise (Joel, 08/09/2026).
    fn open_insert(&mut self) {
        let ahead = self.past.len() + usize::from(self.current.is_some());
        let at = self
            .selection
            .filter(|index| *index >= ahead && self.screen != Screen::Home)
            .map(|index| (index - ahead).min(self.queue.len()))
            .unwrap_or(0);
        self.open_finder(Some(at), "");
    }

    /// Open the search modal. The reader goes to text mode: from here on
    /// every key is typed, until escape or enter.
    fn open_finder(&mut self, insert: Option<usize>, query: &str) {
        let mut finder = Finder {
            insert,
            query: String::new(),
            catalogue: Vec::new(),
            spotify: Some(Ok(Vec::new())),
            asked: String::new(),
            cursor: 0,
            only_catalogue: false,
            link_from: None,
            linked: Vec::new(),
            drawn: Vec::new(),
        };
        if !query.is_empty() {
            finder.query = query.to_string();
        }
        self.finder = Some(finder);
        self.overlay = None;
        self.help_open = false;
        crate::keys::set_text(true, query);
        if !query.is_empty() {
            self.refind();
        }
    }

    /// `ac` — open the modal to **connect** the given artist to one chosen
    /// by search (Joel, 14/09/2026, as `aL`; 2026-09-23 as `ac`): enter on
    /// a row asks how close, then writes into `learned/`. The old `aL`
    /// linked only to the artist one came from, an implicit target that
    /// confused (Joel: "je ne comprends pas le geste"). Search makes the
    /// target explicit and reaches anyone.
    fn open_connect(&mut self, from_slug: &str, from_name: &str) {
        // the connections already drawn, first — from here (→) and, since
        // the engine follows them both ways, those drawn from the other
        // side (←), which `ac` did not show (Joel, 23/09/2026). Enter on
        // one reopens its question. The card's own links are not listed —
        // they are not this gesture's business, `ae` edits those.
        let name = |slug: &str| self.catalog.cards.get(slug).map(|c| c.name.clone()).unwrap_or_else(|| crate::generate::pretty(slug));
        let mut drawn: Vec<(String, String, u8)> = Vec::new();
        let mut linked: Vec<Found> = Vec::new();
        let all = self.learned.all_connections();
        for (from, to, n) in all.iter().filter(|(from, _, _)| *from == from_slug) {
            drawn.push(((*from).clone(), (*to).clone(), *n));
            linked.push(Found { hit: Hit::Artist((*to).clone()), mark: '✓', note: drawn_note(true, &name(to), *n) });
        }
        for (from, to, n) in all.iter().filter(|(_, to, _)| *to == from_slug) {
            if drawn.iter().any(|(_, other, _)| other == *from) {
                continue;
            }
            drawn.push(((*from).clone(), (*to).clone(), *n));
            linked.push(Found { hit: Hit::Artist((*from).clone()), mark: '✓', note: drawn_note(false, &name(from), *n) });
        }
        self.finder = Some(Finder {
            insert: None,
            query: String::new(),
            catalogue: Vec::new(),
            spotify: Some(Ok(Vec::new())),
            asked: String::new(),
            cursor: 0,
            only_catalogue: false,
            link_from: Some((from_slug.to_string(), from_name.to_string())),
            linked,
            drawn,
        });
        self.overlay = None;
        self.help_open = false;
        crate::keys::set_text(true, "");
    }

    fn close_finder(&mut self) {
        self.finder = None;
        crate::keys::set_text(false, "");
        self.tui.clear();
    }

    /// Recompute the catalogue group now, and ask Spotify behind.
    fn refind(&mut self) {
        let Some(finder) = self.finder.as_mut() else { return };
        let query = finder.query.trim().to_string();
        finder.cursor = 0;
        finder.catalogue.clear();
        if query.is_empty() {
            finder.spotify = Some(Ok(Vec::new()));
            finder.asked.clear();
            return;
        }
        let needle = query.to_lowercase();
        // artists first — a card is where a branch can start
        if finder.insert.is_none() {
            for slug in self.catalog.search_names(&query, 4) {
                let card = &self.catalog.cards[&slug];
                let note = format!(
                    "{} links · {} tops",
                    card.links.len(),
                    card.tops.len()
                );
                finder.catalogue.push(Found { hit: Hit::Artist(slug), mark: '♪', note });
            }
        }
        // then the titles the cards know — tops, and what the ear liked
        let mut titles: Vec<Found> = Vec::new();
        for (slug, card) in &self.catalog.cards {
            let liked = self.learned.liked_tracks(slug);
            let known = card.tops.iter().chain(liked.iter().copied());
            for title in known {
                if !title.to_lowercase().contains(&needle) || titles.iter().any(|f| matches!(&f.hit, Hit::Track { title: t, slug: Some(s), .. } if t == title && s == slug)) {
                    continue;
                }
                let is_liked = liked.contains(&title);
                let note = match self.learned.track_stats(slug, title) {
                    Some((plays, _, _)) if plays >= 0.5 => format!("{} play{}", plays.round() as u64, if plays >= 1.5 { "s" } else { "" }),
                    _ => "never played".to_string(),
                };
                titles.push(Found {
                    hit: Hit::Track {
                        title: title.clone(),
                        artist: card.name.clone(),
                        uri: String::new(),
                        slug: Some(slug.clone()),
                        spotify: None,
                    },
                    mark: if is_liked { '♥' } else { '♪' },
                    note: if is_liked { format!("♥ liked · {note}") } else { note },
                });
            }
        }
        titles.sort_by(|a, b| a.note.cmp(&b.note));
        titles.truncate(8);
        finder.catalogue.extend(titles);
        // Spotify, behind: two letters at least, or it is noise
        if query.chars().count() >= 2 {
            finder.spotify = None;
            finder.asked = query.clone();
            let web = self.web.clone();
            let tx = self.jobs_tx.clone();
            tokio::task::spawn_local(async move {
                let result = web.lock().await.search_tracks(&query, 6).await;
                let _ = tx.send(Job::Searched { query, result });
            });
        } else {
            finder.spotify = Some(Ok(Vec::new()));
            finder.asked.clear();
        }
    }

    /// The search modal's keys: everything is typing, except the arrows,
    /// enter, tab and esc.
    async fn on_finder_key(&mut self, cmd: Cmd) -> bool {
        match cmd {
            Cmd::Typing(line) => {
                if let Some(finder) = self.finder.as_mut() {
                    finder.query = line.unwrap_or_default();
                }
                self.refind();
            }
            Cmd::Up => {
                if let Some(finder) = self.finder.as_mut() {
                    finder.cursor = finder.cursor.saturating_sub(1);
                }
            }
            Cmd::Down => {
                if let Some(finder) = self.finder.as_mut() {
                    let last = finder.rows().len().saturating_sub(1);
                    finder.cursor = (finder.cursor + 1).min(last);
                }
            }
            Cmd::Filter => {
                if let Some(finder) = self.finder.as_mut() {
                    finder.only_catalogue = !finder.only_catalogue;
                    finder.cursor = 0;
                }
            }
            Cmd::Escape => self.close_finder(),
            Cmd::Auto => {
                // In the search opened while listening, enter **adds**: it
                // used to wipe what was still to come and take the sound
                // over at once (Joel, 2026-09-25). The modal stays open, so
                // several tracks can be picked in a row; esc closes it.
                // `ti` still inserts at its anchor, `ac` still picks a
                // target, and the home still starts a journey.
                let adding = self.screen != Screen::Home
                    && self
                        .finder
                        .as_ref()
                        .is_some_and(|f| f.insert.is_none() && f.link_from.is_none());
                if adding {
                    self.queue_highlighted();
                } else {
                    self.take_found().await;
                }
            }
            // ← → on a drawn connection: its closeness moves in the list,
            // written at once (Joel, 23/09/2026)
            Cmd::Next => self.nudge_connection(1),
            Cmd::Prev => self.nudge_connection(-1),
            _ => {}
        }
        true
    }

    /// ← → in the `ac` modal, on a connection already drawn: move its
    /// closeness a notch, in the list and in `learned/`, without going
    /// through the question (Joel, 23/09/2026). Elsewhere in the modal
    /// the arrows do nothing.
    fn nudge_connection(&mut self, delta: i8) {
        let Some(finder) = self.finder.as_mut() else { return };
        let Some((from_slug, _)) = finder.link_from.clone() else { return };
        let other = match finder.linked_rows().get(finder.cursor).map(|f| &f.hit) {
            Some(Hit::Artist(slug)) => slug.clone(),
            _ => return,
        };
        let Some(entry) = finder.drawn.iter_mut().find(|(from, to, _)| *from == other || *to == other) else {
            return;
        };
        let proximity = (entry.2 as i8 + delta).clamp(1, 5) as u8;
        if proximity == entry.2 {
            return;
        }
        entry.2 = proximity;
        let (from, to) = (entry.0.clone(), entry.1.clone());
        let outgoing = from == from_slug;
        let name = |slug: &str| self.catalog.cards.get(slug).map(|c| c.name.clone()).unwrap_or_else(|| crate::generate::pretty(slug));
        let note = drawn_note(outgoing, &name(&other), proximity);
        if let Some(row) = finder.linked.iter_mut().find(|f| matches!(&f.hit, Hit::Artist(s) if *s == other)) {
            row.note = note;
        }
        let (from_name, to_name) = (name(&from), name(&to));
        self.set_connection(&from, &to, proximity);
        say!(self, "✓ {from_name} → {to_name} — closeness {proximity}");
    }

    /// Enter in the search opened while listening: the highlighted row goes
    /// to the **end of the queue**, and the modal stays open so several can
    /// be picked in a row. Nothing is cleared and nothing starts playing —
    /// what is decided stays decided (Joel, 2026-09-25).
    fn queue_highlighted(&mut self) {
        let Some(finder) = self.finder.as_ref() else { return };
        let Some(found) = finder.rows().get(finder.cursor).map(|f| (*f).clone()) else { return };
        self.queue_found(&found);
    }

    /// One found row at the end of the queue. An artist stands for their
    /// best unplayed track, as `ti` reads them; a track by an artist with
    /// no card is queued at once and the card generated behind (0016), so
    /// it plays tonight either way and `tl` has somewhere to write when the
    /// card lands.
    fn queue_found(&mut self, found: &Found) {
        let (stop, generate) = match &found.hit {
            Hit::Artist(slug) => {
                let (_, _, _, _, played) = self.state();
                let mut stops = crate::engine::encore(
                    &self.catalog, slug, &self.learned, &self.tail, self.comfort, &played, 1,
                    &mut self.rng,
                );
                let Some(stop) = stops.pop() else {
                    say!(self, "(nothing unplayed left from {})", self.catalog.cards[slug].name);
                    return;
                };
                (stop, None)
            }
            Hit::Track { artist, slug, spotify, .. } => {
                let Some((stop, _)) = stop_of(&found.hit, &self.catalog) else { return };
                let behind = slug
                    .is_none()
                    .then(|| (crate::generate::slugify(artist), artist.clone(), spotify.clone()));
                (stop, behind)
            }
        };
        let (title, artist) = (stop.title.clone(), stop.artist.clone());
        self.queue.push_back(stop);
        say!(self, "↻ {title} — {artist}: at the end of the queue ({} to come)", self.queue.len());
        if let Some((slug, name, spotify)) = generate {
            self.generate(&slug, Some(&name), None, spotify.as_deref(), After::Card);
        }
    }

    /// Enter in the modal: the row under the cursor — branched, played, or
    /// inserted, according to the door we came in by.
    async fn take_found(&mut self) {
        let Some(finder) = self.finder.as_ref() else { return };
        let Some(found) = finder.rows().get(finder.cursor).map(|f| (*f).clone()) else {
            return;
        };
        let insert = finder.insert;
        let link_from = finder.link_from.clone();
        let drawn = match &found.hit {
            Hit::Artist(other) => finder.drawn.iter().find(|(from, to, _)| from == other || to == other).cloned(),
            _ => None,
        };
        self.close_finder();
        // `ac` on a connection already drawn: the same question, set on
        // its closeness — h/l move it, ⏎ sets, x undraws (Joel,
        // 23/09/2026; enter used to undraw on the spot). Whichever side
        // `ac` was opened on, the connection is edited where it is held.
        if let Some((from_slug, to_slug, proximity)) = drawn {
            let name = |slug: &str| self.catalog.cards.get(slug).map(|c| c.name.clone()).unwrap_or_else(|| crate::generate::pretty(slug));
            let pending = LinkPending {
                from_name: name(&from_slug),
                to_name: name(&to_slug),
                from_slug,
                to_slug,
                proximity,
                drawn: true,
            };
            say!(self, "{}", pending.question());
            self.link_pending = Some(pending);
            return;
        }
        // aL: the chosen row is a target to link to, not a track to play
        if let Some((from_slug, from_name)) = link_from {
            let target = match &found.hit {
                Hit::Artist(slug) => Some((
                    slug.clone(),
                    self.catalog.cards.get(slug).map(|c| c.name.clone())
                        .unwrap_or_else(|| slug.replace('-', " ")),
                )),
                // a catalog track: its artist's card; an off-catalog one:
                // the name slugified — a link to a missing card is a
                // proposal (0016), it need not exist yet to be written
                Hit::Track { slug: Some(slug), artist, .. } => Some((
                    slug.clone(),
                    self.catalog.cards.get(slug).map(|c| c.name.clone())
                        .unwrap_or_else(|| artist.clone()),
                )),
                Hit::Track { artist, .. } => Some((crate::generate::slugify(artist), artist.clone())),
            };
            if let Some((to_slug, to_name)) = target {
                // how close, before writing: a link carries its own
                // proximity, or leaves it to the grid of catalog.toml (0010)
                // 4 is what the grid gives "similar": the middle of "these
                // two go together" without claiming they are the same world
                let pending = LinkPending { from_slug, from_name, to_slug, to_name, proximity: 4, drawn: false };
                say!(self, "{}", pending.question());
                self.link_pending = Some(pending);
            }
            return;
        }
        // from the home, choosing **starts a journey** — that is the home's
        // rule, a digit already does the same there (Joel, 09/09/2026). A
        // track without a card has nothing to branch from: it plays from
        // the listening, not from the home.
        if self.screen == Screen::Home {
            match &found.hit {
                Hit::Artist(slug) => self.start_journey(Choice::Artist(slug.clone())).await,
                Hit::Track { title, slug: Some(slug), .. } => {
                    self.start_journey(Choice::Track { slug: slug.clone(), title: title.clone() })
                        .await
                }
                // off-catalog: bring it in (0016). This is the gesture of
                // 09/09/2026 — "I feel like listening to Jacques Brel".
                Hit::Track { title, artist, uri, spotify, .. } => {
                    let slug = crate::generate::slugify(artist);
                    let after = After::Play {
                        title: Some(title.clone()),
                        uri: Some(uri.clone()),
                    };
                    self.generate(&slug, Some(artist), None, spotify.as_deref(), after);
                }
            }
            return;
        }
        match (insert, &found.hit) {
            // ti: the track enters the queue at the anchor, marked. Off
            // catalog, its card is generated behind (0016) without making
            // it wait: it will be attached when it arrives
            (Some(at), Hit::Track { artist, slug, spotify, .. }) => {
                let Some((mut stop, _)) = stop_of(&found.hit, &self.catalog) else { return };
                stop.head = Some(crate::engine::Head {
                    label: stop.title.clone(),
                    reason: "inserted (ti)".to_string(),
                });
                let at = at.min(self.queue.len());
                say!(self, "→ inserted at {}: {} — {}", at + 2, stop.title, stop.artist);
                self.queue.insert(at, stop);
                if slug.is_none() {
                    let slug = crate::generate::slugify(artist);
                    self.generate(&slug, Some(artist), None, spotify.as_deref(), After::Card);
                }
            }
            // ti on an artist: their best unplayed track
            (Some(at), Hit::Artist(slug)) => {
                let (_, _, _, _, played) = self.state();
                let mut stops = crate::engine::encore(
                    &self.catalog, slug, &self.learned, &self.tail, self.comfort, &played, 1, &mut self.rng,
                );
                let Some(mut stop) = stops.pop() else {
                    say!(self, "(nothing unplayed left from {})", self.catalog.cards[slug].name);
                    return;
                };
                stop.head = Some(crate::engine::Head {
                    label: stop.title.clone(),
                    reason: "inserted (ti)".to_string(),
                });
                let at = at.min(self.queue.len());
                say!(self, "→ inserted at {}: {} — {}", at + 2, stop.title, stop.artist);
                self.queue.insert(at, stop);
            }
            // `:search` while listening adds at the end; it used to clear
            // what was still to come and take the sound over (Joel,
            // 2026-09-25). Enter is routed to `queue_found` before it ever
            // gets here — this arm is what stops any other door leading
            // back to a cleared queue.
            (None, _) => self.queue_found(&found),
        }
    }

    /// Play a catalogue stop now, as a fresh segment — its address is
    /// looked up behind, like any other.
    async fn play_stop_now(&mut self, round_artists: Vec<String>, stop: crate::engine::Stop) {
        if let Some(current) = self.current.take() {
            if !self.loading {
                self.past.push(current);
            }
        }
        self.loading = false;
        self.rounds.push(Round { artists: round_artists, tracks: vec![stop.title.clone()] });
        self.queue.clear();
        let _ = self.load_stop(stop).await;
        self.recompute();
    }

    async fn on_track_key(&mut self, key: char) {
        // `ti` needs no track under the needle: it aims at a slot
        if key == 'i' {
            self.open_insert();
            return;
        }
        // `tg` works off-catalog too, like `ag`: a title and a name are all
        // a search needs (Joel, 2026-09-23)
        if key == 'g' {
            let Some(stop) = self.target() else {
                say!(self, "(nothing playing)");
                return;
            };
            self.google(
                &format!("{} {}", stop.artist, stop.title),
                &format!("{} — {}", stop.title, stop.artist),
            );
            return;
        }
        let Some(stop) = self.under_needle() else { return };
        match key {
            'l' => {
                // toggle: like, or take the like back — no penalty, unlike
                // ts (Joel, 14/09/2026)
                if self.learned.track_liked(&stop.slug, &stop.title) {
                    self.learned.unlike_track(&stop.slug, &stop.title);
                    say!(self, "\n♡ {} — like removed", stop.title);
                } else {
                    self.learned.like_track(&stop.slug, &stop.title);
                    say!(self, "\n♥ {} — more often", stop.title);
                }
                self.remark(&stop.slug, &stop.title);
            }
            'a' => self.track_about(stop),
            's' => {
                self.learned.skip_track(&stop.slug, &stop.title);
                if self.aims_at_playing() {
                    say!(self, "\n↷ {} — less often, skipped", stop.title);
                    self.next().await;
                } else if self.selection.is_some_and(|index| self.drop_line(index)) {
                    // "skipping" a track still to come takes it out of the
                    // queue
                    say!(self, "\n↷ {} — less often, removed from the queue", stop.title);
                } else {
                    say!(self, "\n↷ {} — less often", stop.title);
                }
            }
            'b' => {
                self.learned.ban_track(&stop.slug, &stop.title);
                self.queue.retain(|s| s.title != stop.title);
                self.clamp_selection();
                say!(self, "\n⊘ {} — never again", stop.title);
                if self.aims_at_playing() {
                    self.next().await;
                }
            }
            'm' => match self.learned.mark(&stop.artist, &stop.title) {
                Ok(()) => say!(self, "\n⚑ {} — set aside", stop.title),
                Err(e) => say!(self, "\n(mark not written: {e})"),
            },
            // no `tt` / `tT` when listening: one gesture says the taste,
            // `tl`; tops are fixed in the discography (0018)
            't' | 'T' => say!(self, "(tops are fixed in the discography: ad — here, tl says more often)"),
            'd' => {
                // 0011: a door points to tags, the direction we are going
                // — so the next artist's, else its own
                let towards: Vec<String> = self
                    .queue
                    .iter()
                    .find(|next| next.slug != stop.slug)
                    .and_then(|next| self.catalog.cards.get(&next.slug))
                    .or_else(|| self.catalog.cards.get(&stop.slug))
                    .map(|card| card.tags.iter().take(2).cloned().collect())
                    .unwrap_or_default();
                let done = crate::edit::add_door(
                    &self.catalog_dir,
                    &stop.slug,
                    &stop.artist,
                    &stop.title,
                    &towards,
                );
                self.report(done);
            }
            _ => {}
        }
    }

    /// `a` — the artist under the needle. The three verbs are one scale:
    /// more often, less often, never again.
    fn on_artist_key(&mut self, key: char) {
        // `ad` aims at what is **highlighted**, else at what plays: the
        // selection is visible and plays nothing, and the modal names the
        // artist it opens — the doubt is cleared on screen, not in the
        // fingers
        if key == 'd' {
            self.explore_requested = true;
            return;
        }
        // `ag` works off-catalog too: it only needs a name
        let stop = if key == 'g' {
            let Some(stop) = self.target() else {
                say!(self, "(nothing playing)");
                return;
            };
            stop
        } else {
            let Some(stop) = self.under_needle() else { return };
            stop
        };
        self.artist_action(key, stop);
    }

    /// Draw the connection the modal picked, now that its closeness is
    /// settled. It goes into `learned/`, which forkstify commits on its own
    /// schedule (0017) — no edit, no commit of its own, and `Cp` can never
    /// carry it since that only ever moves `cards/`.
    fn write_connection(&mut self, proximity: u8) {
        let Some(LinkPending { from_slug, from_name, to_slug, to_name, .. }) = self.link_pending.take() else {
            return;
        };
        self.set_connection(&from_slug, &to_slug, proximity);
        say!(self, "✓ {from_name} → {to_name} — yours, closeness {proximity}");
    }

    /// Write a connection at that closeness — into `learned/`, and into
    /// the catalog the engine walks, which follows on the spot rather than
    /// at the next launch.
    fn set_connection(&mut self, from_slug: &str, to_slug: &str, proximity: u8) {
        self.learned.connect(from_slug, to_slug, proximity);
        if let Some(card) = self.catalog.cards.get_mut(from_slug) {
            card.links.retain(|l| !(l.to == to_slug && l.kind == crate::catalog::MINE));
            card.links.push(crate::catalog::Link {
                to: to_slug.to_string(),
                kind: crate::catalog::MINE.to_string(),
                note: None,
                proximity: Some(proximity),
            });
        }
        self.recompute();
    }

    /// `:connections` — every connection drawn with `ac`, by artist, with
    /// its closeness, in a block that scrolls like `Cd` (Joel, 23/09/2026).
    /// Read-only: to set or undraw one, `ac` on either of its artists. It
    /// answers "where did I draw what", which the modal of one artist
    /// cannot.
    fn connections_overlay(&mut self) {
        let name = |slug: &str| self.catalog.cards.get(slug).map(|c| c.name.clone()).unwrap_or_else(|| crate::generate::pretty(slug));
        let mut all: Vec<(String, String, u8)> =
            self.learned.all_connections().into_iter().map(|(from, to, n)| (name(from), name(to), n)).collect();
        all.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()).then(a.1.to_lowercase().cmp(&b.1.to_lowercase())));
        if all.is_empty() {
            self.overlay = Some(("your connections".to_string(), vec![" (none drawn yet — ac draws one)".to_string()]));
            return;
        }
        let artists = all.iter().map(|(from, _, _)| from.clone()).collect::<std::collections::BTreeSet<_>>().len();
        let mut lines = Vec::new();
        let mut last: Option<&str> = None;
        for (from, to, n) in &all {
            if last != Some(from.as_str()) {
                if last.is_some() {
                    lines.push(String::new());
                }
                lines.push(format!(" {from}"));
                last = Some(from.as_str());
            }
            lines.push(format!("   → {to} · closeness {n}"));
        }
        lines.push(String::new());
        lines.push(" ac on either artist sets or undraws one · esc closes".to_string());
        let title = format!("your connections — {} from {} artist{}", all.len(), artists, if artists > 1 { "s" } else { "" });
        self.overlay = Some((title, lines));
    }

    /// `x` on the question of a drawn connection: undraw it (2026-09-23).
    /// Nothing is committed — it lives in `learned/`, which forkstify
    /// commits on its own schedule (0017).
    fn undraw_connection(&mut self) {
        let Some(LinkPending { from_slug, from_name, to_slug, to_name, .. }) = self.link_pending.take() else {
            return;
        };
        self.learned.disconnect(&from_slug, &to_slug);
        if let Some(card) = self.catalog.cards.get_mut(&from_slug) {
            card.links.retain(|l| !(l.to == to_slug && l.kind == crate::catalog::MINE));
        }
        say!(self, "✕ {from_name} → {to_name} — connection undrawn");
        self.recompute();
    }

    /// Search in the default browser: `ag` on an artist, `tg` on a track
    /// with its artist before it (Joel, 2026-09-23). `said` is what the
    /// toast names, which reads the other way round — title then artist.
    fn google(&self, query: &str, said: &str) {
        let url =
            format!("https://www.google.com/search?q={}", crate::spotify::encode(query));
        match std::process::Command::new("xdg-open")
            .arg(&url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(_) => say!(self, "→ {said} — in the browser"),
            Err(e) => say!(self, "⏹ browser not found (xdg-open: {e})"),
        }
    }

    /// One `a` verb on one artist — from the axis, or from the home's
    /// collection (Joel, 10/09/2026: "every artist command from the
    /// home").
    fn artist_action(&mut self, key: char, stop: crate::engine::Stop) {
        // `ag` — google: the targeted artist in the default browser
        // (Joel, 08/09/2026)
        if key == 'g' {
            let artist = stop.artist.clone();
            self.google(&artist, &artist);
            return;
        }
        match key {
            'l' => {
                let weight = self.learned.like_artist(&stop.slug);
                say!(self, "\n↑ {} — more often (weight {weight:.2})", stop.artist);
                self.recompute();
            }
            's' => {
                let weight = self.learned.skip_artist(&stop.slug);
                say!(self, "\n↓ {} — less often (weight {weight:.2})", stop.artist);
                self.recompute();
            }
            'b' => {
                self.learned.ban_artist(&stop.slug);
                let before = self.queue.len();
                self.queue.retain(|s| s.slug != stop.slug);
                self.clamp_selection();
                say!(self, 
                    "\n⊘ {} — never again ({} track(s) removed from the queue)",
                    stop.artist,
                    before - self.queue.len()
                );
                self.recompute();
            }
            'e' => self.edit_card(&stop),
            'c' => {
                // `ac` — a connection of your own (Joel, 2026-09-23). It
                // lives in `learned/`, never in the card: a card is
                // knowledge one may propose, this is one ear's taste. A
                // card link is written by hand now, through `ae`.
                if stop.slug.is_empty() || !self.catalog.cards.contains_key(&stop.slug) {
                    say!(self, "({} has no card — :generate them first, then ac)", stop.artist);
                    return;
                }
                let name = self.catalog.cards[&stop.slug].name.clone();
                say!(self, "connect {name} — pick an artist (esc cancels)");
                self.open_connect(&stop.slug, &name);
            }
            _ => {}
        }
    }

    /// Go and get an artist's discography, once. A partial harvest is kept:
    /// the tail is a reservoir, not an inventory.
    /// Harvest the long tail of one artist, unless it is already known.
    /// Says nothing: the caller decides what the listener needs to hear.
    /// `Ok` carries how many tracks the tail holds.
    /// `Ok(true)` when the tail is already there, `Ok(false)` when the
    /// harvest was launched behind (it reports as `Job::Harvested`),
    /// `Err` when it cannot be launched at all.
    fn harvest(&mut self, slug: &str, quiet: bool) -> Result<bool, String> {
        if self.tail.has(slug) {
            return Ok(true);
        }
        if self.harvesting.contains_key(slug) {
            return Ok(false);
        }
        let card = &self.catalog.cards[slug];
        let Some(spotify_id) = card.spotify.clone() else {
            return Err(format!("{} has no Spotify id in its card", card.name));
        };
        self.harvesting.insert(slug.to_string(), quiet);
        let web = self.web.clone();
        let tx = self.jobs_tx.clone();
        let slug = slug.to_string();
        tokio::task::spawn_local(async move {
            let result = web
                .lock()
                .await
                .discography(&spotify_id)
                .await
                .map_err(|why| format!("unreachable ({why})"));
            let _ = tx.send(Job::Harvested { slug, result, quiet });
        });
        Ok(false)
    }

    // --- the discography modal (`ad`) ---------------------------------------

    /// What a gesture aims at: the **highlighted** line if there is one,
    /// the current track otherwise. The selection plays nothing, it is
    /// visible; so it is in charge when it exists.
    fn target(&self) -> Option<crate::engine::Stop> {
        // at the home the axis is not on screen: a highlight left behind
        // by `q` must not steer a gesture, what plays is the target (Joel,
        // 23/09/2026)
        if let Some(index) = self.selection.filter(|_| self.screen != Screen::Home) {
            let stop =
                self.past.iter().chain(self.current.iter()).chain(self.queue.iter()).nth(index);
            if let Some(stop) = stop {
                return Some(stop.clone());
            }
        }
        self.current.clone()
    }

    /// Open the discography. The tail is usually cached already (`:warm`,
    /// an encore); otherwise it is harvested here, the only async moment
    /// of the whole modal.
    async fn open_explore(&mut self) {
        let Some(stop) = self.target() else {
            say!(self, "(nothing playing)");
            return;
        };
        if stop.slug.is_empty() {
            say!(self, "({} — off-catalog, no card to fix)", stop.artist);
            return;
        }
        self.open_explore_of(&stop.slug, &stop.artist).await;
    }

    /// The discography of one artist with a card — from the axis, or from
    /// the home's collection.
    async fn open_explore_of(&mut self, slug: &str, artist: &str) {
        let stop = crate::engine::Stop {
            slug: slug.to_string(),
            artist: artist.to_string(),
            title: String::new(),
            source: crate::engine::Source::Outside,
            head: None,
            encore: false,
        };
        // a harvest from before the dates cannot make an album: the cache
        // is regenerable and outside the repo, redo it rather than show it
        // wrong
        if self.tail.has(&stop.slug) && !self.tail.dated(&stop.slug) {
            self.tail.forget(&stop.slug);
        }
        // the screen opens on what we have — the tops, the learned — and
        // the discography comes behind, saying so (Joel, 08/09/2026)
        let loading = match self.harvest(&stop.slug, false) {
            Ok(known) => !known,
            Err(why) => {
                say!(self, "⏹ {why}");
                false
            }
        };
        let playing = self.current.as_ref().map(|s| s.title.clone());
        let mut screen = crate::explore::Explore::open(
            &stop.slug,
            &self.catalog.cards[&stop.slug],
            self.tail.of(&stop.slug),
            &self.learned,
            playing.as_deref(),
        );
        if screen.albums.is_empty() && !loading {
            say!(self, "(nothing to show from {} — neither discography nor tops)", stop.artist);
            return;
        }
        if loading {
            screen.loading = true;
            screen.notice = "… discography loading — tops first".to_string();
        }
        crate::keys::set_modal(true);
        self.explore = Some(screen);
    }

    /// The modal's keys (`keys::parse_modal`). It never returns `false`:
    /// forkstify is not quit from a list of tracks.
    fn on_explore_key(&mut self, cmd: Cmd) -> bool {
        // the close guard only holds for the esc that follows
        if !matches!(cmd, Cmd::Escape) {
            if let Some(screen) = self.explore.as_mut() {
                screen.confirm_close = false;
            }
        }
        match cmd {
            Cmd::Track('l') => self.explore_measure(true),
            Cmd::Track('b') => self.explore_measure(false),
            Cmd::Enqueue => self.explore_enqueue(),
            Cmd::Auto => self.explore_enter(),
            Cmd::Escape => self.close_explore(),
            Cmd::Typing(line) => self.typed = line.unwrap_or_default(),
            Cmd::Pending(seq) => self.typed = seq,
            other => {
                let Some(screen) = self.explore.as_mut() else { return true };
                // what is visible is not said: only a gesture with no
                // visible effect leaves a line
                screen.notice.clear();
                match other {
                    Cmd::Up => screen.move_by(-1),
                    Cmd::Down => screen.move_by(1),
                    Cmd::Top => screen.go_top(),
                    Cmd::Bottom => screen.go_bottom(),
                    Cmd::Prev => screen.fold(),
                    Cmd::Next => screen.unfold(),
                    Cmd::Sort => screen.toggle_sort(),
                    Cmd::Filter => screen.cycle_filter(),
                    Cmd::AlbumTop => screen.top_album(ALBUM_TOPS),
                    Cmd::Undo => screen.undo(),
                    Cmd::Search(query) => screen.search(&query),
                    Cmd::Unknown(seq) => screen.notice = format!("({seq} does nothing here)"),
                    _ => {}
                }
            }
        }
        true
    }

    /// `tl` / `tb` in the modal: these are **measures**, they write to
    /// `learned/` at once and without asking (0013) — 0017 will commit them
    /// with the rest. Nothing to do with the batch of tops.
    fn explore_measure(&mut self, like: bool) {
        let Some(mut screen) = self.explore.take() else { return };
        let Some(title) = screen.track().map(|track| track.title.clone()) else {
            screen.notice = "(move onto a track)".into();
            self.explore = Some(screen);
            return;
        };
        if like {
            if self.learned.track_liked(&screen.slug, &title) {
                self.learned.unlike_track(&screen.slug, &title);
                screen.notice = format!("♡ {title} — like removed");
            } else {
                self.learned.like_track(&screen.slug, &title);
                screen.notice = format!("♥ {title} — liked");
            }
            // the same track may be sitting in the queue behind the modal
            let slug = screen.slug.clone();
            self.remark(&slug, &title);
        } else {
            self.learned.ban_track(&screen.slug, &title);
            self.queue.retain(|stop| stop.title != title);
            screen.notice = format!("⊘ {title} — never again");
        }
        screen.refresh(&self.learned);
        self.explore = Some(screen);
    }

    /// `e` — queue the track without closing. An edit only counts for the
    /// engine at the next launch; the queue plays tonight, and that is how
    /// we leave the discography.
    fn explore_enqueue(&mut self) {
        let Some(mut screen) = self.explore.take() else { return };
        let Some((title, source)) = screen.track().map(|track| {
            (
                track.title.clone(),
                if track.is_top() {
                    crate::engine::Source::Top
                } else if track.liked {
                    crate::engine::Source::Liked
                } else {
                    crate::engine::Source::Tail
                },
            )
        }) else {
            screen.notice = "(move onto a track)".into();
            self.explore = Some(screen);
            return;
        };
        self.queue.push_back(crate::engine::Stop {
            slug: screen.slug.clone(),
            artist: screen.name.clone(),
            title: title.clone(),
            source,
            head: None,
            // the encore's ↻ glyph: it is the same gesture, the list says so
            encore: true,
        });
        screen.notice = format!("↻ {title} — queued ({} up next)", self.queue.len());
        self.explore = Some(screen);
    }

    /// ⏎ — write the batch: **one read, one write, one commit**. That is
    /// the point of the pending state (maquette 1a): fix five tops in one
    /// thought, it makes a single commit.
    fn explore_write(&mut self) {
        let Some(mut screen) = self.explore.take() else { return };
        if screen.pending.is_empty() {
            screen.notice = "(nothing to write — a measure is already taken)".into();
            self.explore = Some(screen);
            return;
        }
        let done = crate::edit::set_tops(
            &self.catalog_dir,
            &screen.slug,
            &screen.name,
            &screen.adds(),
            &screen.removes(),
        );
        match done {
            Ok(edit) => {
                let summary = edit.summary.clone();
                screen.notice = match crate::edit::commit(&self.catalog_dir, &edit) {
                    Ok(()) => format!("✓ {summary} — committed (engine sees it next launch)"),
                    Err(why) => format!("✓ {summary} — written, but not committed ({why})"),
                };
                screen.written();
            }
            Err(why) => screen.notice = format!("(nothing done: {why})"),
        }
        self.explore = Some(screen);
    }

    /// ⏎ — write the batch if there is one (one commit, as before), then,
    /// on a track, **start a new seed from it**: the track plays, the
    /// branches fork from its artist (Joel, 11/09/2026). On an album line,
    /// enter only writes.
    fn explore_enter(&mut self) {
        let pending = self.explore.as_ref().is_some_and(|screen| !screen.pending.is_empty());
        if pending {
            self.explore_write();
        }
        let Some(screen) = self.explore.as_mut() else { return };
        if let Some(track) = screen.track() {
            if track.banned {
                screen.notice = format!("(⊘ {} is banned — tl takes it back)", track.title);
            } else {
                let choice = Choice::Track { slug: screen.slug.clone(), title: track.title.clone() };
                crate::keys::set_modal(false);
                self.start_requested = Some(choice);
            }
            return;
        }
        // on an album line: start a seed of the whole album (Joel, 14/09/2026)
        if let Some(album) = screen.album_here() {
            let titles: Vec<String> =
                album.tracks.iter().filter(|t| !t.banned).map(|t| t.title.clone()).collect();
            if titles.is_empty() {
                screen.notice = "(nothing playable in this album)".into();
                return;
            }
            let (slug, name) = (screen.slug.clone(), screen.name.clone());
            crate::keys::set_modal(false);
            self.album_requested = Some((slug, name, titles));
        }
    }

    /// esc — close. With pending edits, the first esc warns: they are not
    /// written, and nothing on screen would say so once the modal is
    /// closed.
    fn close_explore(&mut self) {
        let Some(mut screen) = self.explore.take() else { return };
        if !screen.pending.is_empty() && !screen.confirm_close {
            screen.confirm_close = true;
            screen.notice = format!(
                "⚑ {} unwritten edit(s) — ⏎ to write, esc again to discard them",
                screen.pending.len()
            );
            self.explore = Some(screen);
            return;
        }
        crate::keys::set_modal(false);
        if !screen.pending.is_empty() {
            say!(self, "(discography closed — {} edit(s) discarded)", screen.pending.len());
        }
        self.tui.clear();
    }

    /// `:` commands — 0013 makes every key the shortcut of one. Only
    /// `:size` is served so far: it replaces the old `b<n>`, which the
    /// move to raw mode dropped on the way.
    /// `:catalog` — the state in one line, then what is under it.
    /// The screen's height, for what scrolls: the terminal's, or a
    /// generous guess when it cannot be asked.
    fn tui_height(&self) -> u16 {
        self.tui.height().unwrap_or(40)
    }

    /// `ae` — the card in `$EDITOR`, right here (Joel, 20/09/2026). The
    /// key reader parks, the terminal goes back to the editor, the
    /// alternate screen too; when the editor closes, the card is read
    /// again: readable and changed, it is committed and the engine follows
    /// on the spot; unreadable, it is said and left uncommitted to fix.
    /// The loop blocks meanwhile — the sound goes on in its own thread,
    /// but a track ending waits for the editor to close.
    fn edit_card(&mut self, stop: &crate::engine::Stop) {
        if stop.slug.is_empty() || !self.catalog.cards.contains_key(&stop.slug) {
            say!(self, "({} has no card — :generate them first, then ae)", stop.artist);
            return;
        }
        let path = crate::edit::card_path(&self.catalog_dir, &stop.slug);
        let name = self.catalog.cards[&stop.slug].name.clone();
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .ok()
            .filter(|e| !e.trim().is_empty());
        let Some(editor) = editor else {
            // no editor named: the desktop's, and nothing to wait for
            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
            say!(self, "→ {} — no $EDITOR, opened by the desktop; relaunch to reload it", path.display());
            return;
        };
        let before = std::fs::read_to_string(&path).unwrap_or_default();
        crate::keys::suspend_reader(true);
        std::thread::sleep(std::time::Duration::from_millis(150));
        crate::keys::raw_pause();
        self.tui.suspend();
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{editor} \"$1\""))
            .arg("forkstify")
            .arg(&path)
            .status();
        crate::keys::raw_resume();
        crate::keys::drain_input();
        self.tui.resume();
        crate::keys::suspend_reader(false);
        // painted right here, whatever comes next: the screen must not
        // stay blank until the next gesture
        self.paint();
        if let Err(why) = status {
            say!(self, "⏹ {editor}: {why}");
            return;
        }
        let after = std::fs::read_to_string(&path).unwrap_or_default();
        if after == before {
            say!(self, "({name} — unchanged)");
            return;
        }
        match toml::from_str::<crate::catalog::Card>(&after) {
            Ok(card) => {
                self.catalog.cards.insert(stop.slug.clone(), card);
                let edit = crate::edit::edited_by_hand(&self.catalog_dir, &stop.slug, &name);
                match crate::edit::commit(&self.catalog_dir, &edit) {
                    Ok(()) => say!(self, "✓ {name} — edited by hand, committed"),
                    Err(why) => say!(self, "⏹ {name} — edited, not committed: {why}"),
                }
                self.recompute();
            }
            Err(why) => say!(self, "⏹ {name} — the card does not read: {} — ae again to fix it, nothing committed", why.message()),
        }
    }

    fn catalog_status(&mut self) {
        let status = crate::fork::status(&self.catalog_dir);
        if status.merging || status.pending {
            // a stopped merge: resuming is what :catalog is for then
            match crate::fork::resume(&self.catalog_dir) {
                Ok(update) => {
                    self.conflicts.clear();
                    if let Ok(mut catalog) = Catalog::load(&self.catalog_dir) {
                        self.learned.weave_into(&mut catalog);
                        self.catalog = catalog;
                        self.recompute();
                    }
                    self.tell(format!("⇅ {}", update.word));
                    return;
                }
                Err(why) => self.tell(format!("⊘ {why}")),
            }
        }
        self.overlay = Some(("C — the catalog".into(), status.lines()));
    }

    /// `Cd`, `Cp`, `Cu`: git off the loop, the answer comes back as a job.
    fn catalog_gesture(&mut self, key: char) {
        if self.catalog_busy {
            self.tell("(the catalog is busy — one gesture at a time)".to_string());
            return;
        }
        let dir = self.catalog_dir.clone();
        let tx = self.jobs_tx.clone();
        let what = match key {
            'd' => "… C diff — fetch upstream, comparing to the reference",
            'p' => "… C propose — fetch upstream, worktree proposal, commit, push",
            _ => "… C update — learned committed, fetch upstream, merge, index",
        };
        self.tell(what.to_string());
        self.catalog_busy = true;
        tokio::task::spawn_local(async move {
            let job = tokio::task::spawn_blocking(move || match key {
                'd' => Job::Diffed(crate::fork::diff(&dir)),
                'p' => Job::Proposed(crate::fork::propose(&dir)),
                _ => Job::Updated(crate::fork::update(&dir)),
            })
            .await
            .unwrap_or_else(|e| Job::Diffed(Err(format!("interrupted ({e})"))));
            let _ = tx.send(job);
        });
    }

    /// `o` — the first card a stopped merge waits on, in the desktop's
    /// editor. The terminal editor is out of reach: the key reader holds
    /// stdin (the same limit as `ae`).
    fn open_conflict(&mut self) {
        let Some(slug) = self.conflicts.first().cloned() else {
            self.tell("(nothing to open — o opens a card a merge stopped on)".to_string());
            return;
        };
        let path = crate::edit::card_path(&self.catalog_dir, &slug);
        match std::process::Command::new("xdg-open").arg(&path).spawn() {
            Ok(_) => self.tell(format!("→ {} — git add it once resolved, then :catalog", path.display())),
            Err(why) => self.tell(format!("⏹ xdg-open: {why} — {}", path.display())),
        }
    }

    /// `:catalog shell` — the desktop's terminal, opened in the fork, for
    /// what the gestures do not cover: a `git log`, a card by hand (Joel,
    /// 23/09/2026). `xdg-terminal-exec` — the default-terminal
    /// specification, what Omarchy sets `$TERMINAL` to — takes the
    /// directory; failing that, `$TERMINAL` is launched from it. Its own
    /// process group, so closing the terminal forkstify runs in does not
    /// take the new one down.
    fn catalog_shell(&mut self) {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        let dir = self.catalog_dir.clone();
        let quiet = |mut command: Command| {
            command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0);
            command
        };
        let mut launch = Command::new("xdg-terminal-exec");
        launch.arg(format!("--dir={}", dir.display()));
        let spawned = quiet(launch).spawn().or_else(|why| match std::env::var("TERMINAL") {
            Ok(terminal) => {
                let mut launch = Command::new(terminal);
                launch.current_dir(&dir);
                quiet(launch).spawn()
            }
            Err(_) => Err(why),
        });
        match spawned {
            Ok(_) => self.tell(format!("→ a terminal in {}", dir.display())),
            Err(why) => self.tell(format!("⏹ no terminal ({why}) — the fork is at {}", dir.display())),
        }
    }

    fn colon(&mut self, text: &str) {
        let mut words = text.split_whitespace();
        match (words.next(), words.next()) {
            (Some("size"), Some(n)) => match n.parse::<usize>() {
                Ok(n) if (1..=9).contains(&n) => {
                    self.size = n;
                    say!(self, "Branch size: {n}");
                }
                _ => say!(self, "Size expected between 1 and 9."),
            },
            (Some("size"), None) => say!(self, "Branch size: {}", self.size),
            (Some("comfort"), Some(n)) => match n.parse::<u8>() {
                Ok(n) if n <= 5 => {
                    self.comfort = Comfort::new(n);
                    crate::config::remember_comfort(n);
                    say!(self, "Comfort zone: {n} — {}", comfort_word(n));
                    // the dial changes which branches make sense from here
                    self.recompute();
                    self.preview();
                }
                _ => say!(self, "Comfort expected between 0 (exploration) and 5 (cocoon)."),
            },
            (Some("warm"), _) => self.warm_requested = true,
            // `fw` spelled out: the rest of the line names the artist
            (Some("wander"), _) => {
                self.wander_requested = Some(text.split_whitespace().skip(1).collect::<Vec<_>>().join(" "));
            }
            // `ad` spelled out (0013: every key is the shortcut of a
            // command)
            (Some("discography"), _) => self.explore_requested = true,
            // the personal overlay is computed, not stored
            // :search opens the modal — empty, or prefilled with the text
            (Some("search"), _) => {
                self.search_requested = Some(text.trim().trim_start_matches("search").trim().to_string());
            }
            // :generate — bring in a missing artist, then start from them
            // (0016). The card comes in a few seconds. A last word shaped
            // like a MBID replaces the search by name (Joel, 09/09/2026);
            // and if the artist was proposed as a gap, its branch is
            // taken, not a jump to them.
            (Some("generate"), Some(_)) => {
                self.generate_asked(text.trim().trim_start_matches("generate"));
            }
            (Some("generate"), None) => {
                say!(self, "usage: :generate <artist name> [mbid]")
            }
            // the setup, replayed: the session closes on it and home comes
            // back after (workstream A, `docs/design/before-release.md`)
            (Some("setup"), _) => self.leave = Some(crate::setup::Replay::All),
            (Some("library"), _) => self.leave = Some(crate::setup::Replay::Library),
            (Some("sync"), _) | (Some("push"), _) => match crate::sync::sync(&self.catalog_dir) {
                Ok(word) => say!(self, "✓ {word}"),
                Err(why) => say!(self, "⏹ {why}"),
            },
            // the catalog, as commands (workstream B): the state in one line,
            // the three gestures, and the way out of the local mode
            (Some("catalog"), None) => self.catalog_status(),
            // every connection drawn with `ac`, in one block (Joel,
            // 23/09/2026) — read-only: setting one goes through `ac`
            (Some("connections"), _) => self.connections_overlay(),
            (Some("catalog"), Some("diff")) => self.catalog_gesture('d'),
            (Some("catalog"), Some("propose")) => self.catalog_gesture('p'),
            (Some("catalog"), Some("update")) => self.catalog_gesture('u'),
            (Some("catalog"), Some("shell")) => self.catalog_shell(),
            (Some("catalog"), Some("fork")) => match text.split_whitespace().nth(2) {
                Some(url) => match crate::fork::fork(&self.catalog_dir, url) {
                    Ok(word) => say!(self, "✓ {word}"),
                    Err(why) => say!(self, "⏹ {why}"),
                },
                None => say!(self, "usage: :catalog fork <url of your fork>"),
            },
            (Some("catalog"), Some(other)) => say!(self, "(:catalog {other} — diff, propose, update, shell, or fork <url>)"),
            (Some("comfort"), None) => say!(self, 
                "Comfort zone: {} — {}",
                self.comfort.value(),
                comfort_word(self.comfort.value())
            ),
            (Some(other), _) => self.not_yet(&format!(":{other}"), "this command"),
            (None, _) => {}
        }
    }

    /// Keep where we stopped, so the home screen can offer to resume.
    fn remember(&self) {
        let Some(stop) = &self.current else { return };
        crate::home::remember(&LastSession {
            slug: stop.slug.clone(),
            name: stop.artist.clone(),
            title: stop.title.clone(),
            at: "just now".to_string(),
            // the whole journey, so resume brings it all back (Joel, 14/09/2026)
            past: self.past.clone(),
            current: Some(stop.clone()),
            queue: self.queue.iter().cloned().collect(),
            rounds: self.rounds.clone(),
        });
    }

    /// `r` — resume the saved journey: its history, the track that was
    /// playing, and what was still to come. An old save that predates the
    /// full journey falls back to starting from the last track (Joel,
    /// 14/09/2026).
    async fn restore(&mut self, saved: LastSession) {
        let (Some(current), false) = (saved.current, saved.rounds.is_empty()) else {
            self.start_journey(Choice::Track { slug: saved.slug, title: saved.title }).await;
            return;
        };
        if self.current.is_some() {
            self.sound.stop();
        }
        self.current = None;
        self.loading = false;
        self.progress = None;
        self.selection = None;
        self.overlay = None;
        self.help_open = false;
        self.explore = None;
        self.notices.borrow_mut().clear();
        self.branches.clear();
        self.past = saved.past;
        self.queue = saved.queue.into_iter().collect();
        self.rounds = saved.rounds;
        self.screen = Screen::Session;
        self.tui.clear();
        // replay the track that was on air; the queue and history are back
        let _ = self.load_stop(current).await;
        self.recompute();
        self.render();
    }

    /// Say what an edit did, and commit it. An edit that leaves no
    /// readable trace is not one (0013).
    fn report(&self, done: Result<crate::edit::Edit, String>) {
        match done {
            Ok(edit) => {
                let summary = edit.summary.clone();
                match crate::edit::commit(&self.catalog_dir, &edit) {
                    Ok(()) => say!(self, "✓ {summary} — committed"),
                    Err(why) => say!(self, "✓ {summary} — written, but not committed ({why})"),
                }
                // the catalog in memory does not move: the edit counts at
                // the next launch, and it is better said
                say!(self, "  (the engine will take it into account next launch)");
            }
            Err(why) => say!(self, "(nothing done: {why})"),
        }
    }

    /// `fw` / `:wander [artist]` (feedback no. 6, settled on 2026-09-11):
    /// leave the universe. Bare, the engine draws a head far from the
    /// journey; with a name, the head is that artist of the catalog. The
    /// branch goes at the end of what is decided, as `f<n>` does.
    async fn wander(&mut self, target: &str) {
        if self.rounds.is_empty() {
            say!(self, "(nothing is playing — fw wanders from a journey)");
            return;
        }
        let target = target.trim();
        let slug = if target.is_empty() {
            None
        } else {
            match self.catalog.search_names(target, 1).into_iter().next() {
                Some(slug) => Some(slug),
                None => {
                    say!(self, "(no \"{target}\" in the catalog — :generate {target} brings them in)");
                    return;
                }
            }
        };
        let (context, _, universe, visited, played) = self.state();
        let branch = crate::engine::wander(
            &self.catalog, &context, &universe, slug.as_deref(), &self.learned, &self.tail,
            self.comfort, &visited, &played, self.size, &mut self.rng,
        );
        match branch {
            Some(branch) => {
                say!(self, "→ {} — {}", branch.label, branch.reason);
                self.take_branch(branch, When::EndOfBranch).await;
            }
            None => say!(self, "(nowhere far enough to wander — everything near is played)"),
        }
    }

    /// A gesture the grammar accepts but the code does not serve yet. Saying
    /// so beats a silent no-op: the key is right, the wiring is missing.
    fn not_yet(&self, keys: &str, what: &str) {
        say!(self, "\n{keys} — {what}: decided (0015), not wired yet.");
    }

    /// A `:` line, on either screen (Joel, 23/09/2026). `colon` reads it
    /// and leaves flags; here they are acted on — with the loop, which the
    /// keyboard does not have. False when the session is leaving for the
    /// setup.
    async fn run_colon(&mut self, text: &str) -> bool {
        self.colon(text);
        if self.leave.is_some() {
            return false;
        }
        if let Some(query) = self.search_requested.take() {
            self.open_finder(None, &query);
        }
        if let Some(target) = self.wander_requested.take() {
            self.wander(&target).await;
        }
        if std::mem::take(&mut self.warm_requested) {
            match self.aimed_artist() {
                Some((Some(slug), name)) => {
                    // `:warm` means "go and get it now": forget any cached
                    // harvest first, so an empty one no longer blocks the
                    // retry (Joel, 14/09/2026 — Jarvis Cocker stuck at 0)
                    self.tail.forget(&slug);
                    match self.harvest(&slug, false) {
                        Ok(true) => say!(self, "✓ discography of {name} — {} tracks already cached", self.tail.of(&slug).len()),
                        Ok(false) => say!(self, "… discography of {name} being fetched"),
                        Err(why) => say!(self, "⏹ {why}"),
                    }
                }
                Some((None, name)) => say!(self, "({name} has no card — ad generates it)"),
                None => say!(self, "(nothing highlighted — ↑↓ to choose)"),
            }
        }
        // `ad` spelled out: the modal over the listening, or over the home
        // on the highlighted line — an artist without a card is generated
        // first, as `ad` does there (0016)
        if std::mem::take(&mut self.explore_requested) {
            if self.screen != Screen::Home {
                self.open_explore().await;
            } else {
                match self.aimed_artist() {
                    Some((Some(slug), name)) => self.open_explore_of(&slug, &name).await,
                    Some((None, name)) => {
                        let slug = crate::generate::slugify(&name);
                        self.generate(&slug, Some(&name), None, None, After::Explore);
                    }
                    None => say!(self, "(nothing highlighted — ↑↓ to choose)"),
                }
            }
        }
        true
    }

    /// The artist a `:warm` or `:discography` is aimed at: at the home the
    /// highlighted line of the collection — none when nothing is
    /// highlighted —, when listening the one under the needle: the
    /// highlighted row, otherwise what plays (0020), not the end of the
    /// branch chain, which is no longer what sounds (Joel, 14/09/2026).
    fn aimed_artist(&self) -> Option<(Option<String>, String)> {
        if self.screen == Screen::Home {
            return self.home.highlighted(&self.catalog, &self.learned);
        }
        let slug = self
            .target()
            .map(|stop| stop.slug.clone())
            .filter(|slug| self.catalog.cards.contains_key(slug))
            .unwrap_or_else(|| self.state().1);
        let name = self.catalog.cards[&slug].name.clone();
        Some((Some(slug), name))
    }

    /// The `:` line is a namespace of its own (Joel, 23/09/2026): as soon
    /// as it opens, the helper lists the commands, narrowed to the word
    /// being typed — `:c` leaves catalog and comfort. It closes with the
    /// line, since the line's end is what clears the overlays.
    fn follow_line(&mut self, line: &str) {
        let Some(typed) = line.strip_prefix(':') else { return };
        if !self.help_open {
            self.overlay_scroll = 0;
            self.help_open = true;
            self.help_auto = true;
        }
        self.commands_help(typed);
    }

    /// The commands level of the helper: one table for both screens, every
    /// command works on both (Joel, 23/09/2026).
    fn commands_help(&mut self, typed: &str) {
        const COMMANDS: &[(&str, &str)] = &[
            (":search [text]", "search — the modal: catalog then Spotify, enter takes"),
            (":generate <name> [mbid]", "bring in a missing artist — the id by hand if the name is not enough"),
            (":discography", "the artist's discography by album — shortcut ad"),
            (":connections", "every connection drawn with ac, by artist"),
            (":warm", "fetch the artist's discography now — the long tail"),
            (":wander [artist]", "far away, or to that artist — shortcut fw"),
            (":size <n>", "branch size, 1 to 9"),
            (":comfort <n>", "comfort zone, 5 cocoon → 0 exploration"),
            (":catalog", "the fork's state in one line"),
            (":catalog diff|propose|update", "the three gestures — Cd, Cp, Cu"),
            (":catalog shell", "a terminal in the fork"),
            (":catalog fork <url>", "leave the local mode — rare, no key"),
            (":sync", "commit and push the learned now"),
            (":setup", "replay a step of the setup — catalog, identity, connection, library…"),
            (":library", "harvest the library again — steps 4, 5 and 7 of the setup"),
        ];
        let word = typed.split_whitespace().next().unwrap_or("");
        let mut lines: Vec<String> = COMMANDS
            .iter()
            .filter(|(command, _)| command[1..].starts_with(word))
            .map(|(command, what)| format!("   {command:<30} {what}"))
            .collect();
        if lines.is_empty() {
            lines.push(format!(" (no command starts with {word})"));
        }
        lines.push(String::new());
        lines.push(" enter runs · esc closes".into());
        self.overlay = Some((": — the commands".to_string(), lines));
    }

    /// The key helper follows what is typed (Joel, 07/09/2026): open, it
    /// goes down to the level typed and back up on ⌫. Closed, **a
    /// namespace typed opens its level on its own** — `t` shows `tl`,
    /// `ts`… without space (Joel, 23/09/2026); space is only needed for
    /// the entry level. A helper opened that way closes on ⌫, since the
    /// entry level was never asked for.
    fn follow_typing(&mut self, seq: &str) {
        let level = seq.chars().next();
        if self.help_open {
            if level.is_none() && self.help_auto {
                self.overlay = None;
                self.help_open = false;
            } else {
                self.help(level);
            }
        } else if let Some(namespace) = level.filter(|k| self.has_level(*k)) {
            // it takes the place of whatever block was shown, scrolled or not
            self.overlay_scroll = 0;
            self.help_open = true;
            self.help_auto = true;
            self.help(Some(namespace));
        }
    }

    /// The namespaces the helper has a level for — the ones `help` lists
    /// on their own, not through the entry table. `c` and `g` wait for a
    /// second key too, but they are not namespaces.
    fn has_level(&self, namespace: char) -> bool {
        let home = self.screen == Screen::Home;
        match namespace {
            'f' | 'e' => !home,
            't' | 'a' | 'C' => true,
            _ => false,
        }
    }

    /// Space, the leader: what can I type from here? With a namespace
    /// half-typed, only that namespace — which-key, in a terminal. **One
    /// table for both screens** (Joel, 23/09/2026), and each screen shows
    /// only the rows that mean something on it: a key of the other screen
    /// is left out, not marked — Joel saw the marks and preferred less.
    fn help(&mut self, namespace: Option<char>) {
        use Where::*;
        let home = self.screen == Screen::Home;
        let rows: &[(&str, &str, Where)] = match namespace {
            Some('f') if !home => &[
                ("f<n>", "branch n, at the end of the branch", Both),
                ("fn<n>", "branch n, after the track", Both),
                ("f!<n>", "branch n, after the track, the rest dropped", Both),
                ("fg<n>", "generate — the card of gap n, the branch stays on show", Both),
                ("fp", "peek — preview the branches", Both),
                ("fr", "reroll — propose three others", Both),
                ("fu", "undo — back to the previous branch", Both),
                ("fw", "wander — far away, or `fw <artist>` to their universe", Both),
            ],
            Some('e') if !home => &[
                ("e<n>", "n encores, at the end of the branch", Both),
                ("en<n>", "n encores, after the track", Both),
                ("e!<n>", "n encores, the rest dropped", Both),
            ],
            Some('t') => &[
                ("tl", "like — more often: I like this track", Both),
                ("ts", "skip — less often: not for me (and skips)", Both),
                ("tb", "ban — never again this one", Both),
                ("tm", "mark — set aside", Both),
                ("td", "door — make it a door (card, one commit)", Both),
                ("ta", "about — album, featuring, year, and why this track", Both),
                ("tg", "google — the track and its artist in the browser", Both),
                ("ti", "insert — insert a track here, via search", Both),
                ("ad", "tops are fixed in the discography", Both),
            ],
            Some('C') => &[
                ("Cd", "diff — what this catalog has beyond the reference", Both),
                ("Cp", "propose — offer those cards to the reference, one pull request", Both),
                ("Cu", "update — bring the reference into the fork (a merge)", Both),
                (":catalog", "the state in one line", Both),
                (":catalog shell", "a terminal in the fork", Both),
                (":catalog fork <url>", "out of the local mode — rare, no key", Both),
            ],
            Some('a') => &[
                ("al", "like — this artist, more often", Both),
                ("as", "skip — this artist, less often", Both),
                ("ab", "ban — never again this artist", Both),
                ("ad", "discography — their discography, by album", Both),
                ("ag", "google — the artist in the browser", Both),
                ("ae", "edit — the card in $EDITOR, committed when it changed", Both),
                ("ac", "connect — yours: the ones drawn (enter undraws), then search to draw one", Both),
            ],
            // the entry level: the keys, then the two lines, then the
            // legend (Joel, 23/09/2026)
            _ => &[
                ("1-9", "take a branch · at the home, start on an entry of the blocks", Both),
                ("\u{2191}\u{2193} gg G", "highlight — a row of the axis, an artist of the collection", Both),
                ("enter", "auto — draw among the branches · at the home, start on the highlighted line, else random", Both),
                ("f", "the branch — its keys open on f", Listening),
                ("e", "encore — its keys open on e", Listening),
                ("t", "the track — its keys open on t · at the home, what plays underneath", Both),
                ("a", "the artist — its keys open on a", Both),
                ("h l \u{2190} \u{2192}", "previous / next track", Both),
                ("J K", "move the highlighted line one step", Listening),
                ("p", "pause / play", Both),
                ("c<n>", "comfort zone, 5 cocoon → 0 exploration", Both),
                ("cc", "adjust comfort with the arrows", Both),
                ("x", "remove — the highlighted track out of the queue (not a ban) · at the home, the artist out of the liked", Both),
                ("s", "the order: familiarity → a-z → last played", Home),
                ("v", "the view: liked ⇄ all", Home),
                ("r", "back to listening · else resume the last journey", Home),
                ("C", "the catalog — its keys open on C", Both),
                ("q", "quit · while listening, back to the home", Both),
                ("/text", "filter a list — the collection, the discography", Both),
                (":", "the commands — their list opens on :", Both),
                ("♪♥↳·+~", "top · liked · door · tail · non-top · off-catalog", Both),
            ],
        };
        let title = match namespace {
            Some('f') if !home => "f — the branch",
            Some('e') if !home => "e — encore",
            Some('t') if !home => "t — the track",
            Some('t') => "t — what plays underneath",
            Some('a') => "a — the artist",
            Some('C') => "C — the catalog",
            _ => "the keys",
        };
        let here = if home { Home } else { Listening };
        // a block lays over the screen; it does not go down into the log,
        // whose bottom must never move
        let mut lines: Vec<String> = rows
            .iter()
            .filter(|(_, _, screen)| *screen == Both || *screen == here)
            .map(|(keys, what, _)| format!("   {keys:<10} {what}"))
            .collect();
        // it is a key helper: the key typed here performs the action
        lines.push(String::new());
        lines.push(match namespace {
            Some(_) if self.help_auto => " one key = the action · ⌫ or esc close · space for all the keys".into(),
            Some(_) => " one key = the action · ⌫ back · esc close".into(),
            None => " one key = the action · esc close".into(),
        });
        self.overlay = Some((title.to_string(), lines));
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            self.sound.pause();
        } else {
            self.sound.resume();
        }
    }

    /// Back up one branch: drop the last round and replay the artist we were
    /// on before it. Distinct from `u`, which undoes a *gesture* (0015).
    async fn fork_undo(&mut self) {
        if self.rounds.len() <= 1 {
            say!(self, "\n(already at the seed)");
            return;
        }
        self.rounds.pop();
        // the ledger says where we were before this branch; the list, once
        // the branch is out of it, says what was played
        let current = state_of(&self.rounds).1;
        self.queue.clear();
        let (_, _, _, _, played) = self.state();
        let stops = crate::engine::encore(
            &self.catalog,
            &current,
            &self.learned,
            &self.tail,
            self.comfort,
            &played,
            self.size,
            &mut self.rng,
        );
        self.start_segment(vec![current], stops, false, false).await;
    }
}

/// The note of a drawn connection in the `ac` modal: which way it was
/// drawn, and how close — rewritten in place when ← → move it.
fn drawn_note(outgoing: bool, name: &str, proximity: u8) -> String {
    if outgoing {
        format!("yours → {name} — closeness {proximity}")
    } else {
        format!("yours ← {name} — closeness {proximity}, drawn from there")
    }
}

/// A connection `ac` is asking about: to draw, or already drawn and
/// reopened to set its closeness or undraw it (Joel, 23/09/2026). `from`
/// is the side that holds it in `learned/` — a connection drawn from the
/// other artist is edited there, whichever side `ac` was opened on.
struct LinkPending {
    from_slug: String,
    from_name: String,
    to_slug: String,
    to_name: String,
    /// The closeness on offer: `h`/`l` move it a notch, a digit jumps.
    proximity: u8,
    /// Already in `learned/`: ⏎ sets, `x` undraws, esc leaves it.
    drawn: bool,
}

impl LinkPending {
    /// What `ac` asks — said in the log, and shown as a toast for as long
    /// as it waits. It says which end is which: 5 is the closest.
    fn question(&self) -> String {
        let LinkPending { from_name, to_name, proximity, .. } = self;
        let scale = "h l move · 1 farthest, a distant echo … 5 closest, almost the same universe";
        if self.drawn {
            format!("→ {from_name} → {to_name} — closeness {proximity} · {scale} · ⏎ sets it · x undraws · esc leaves it")
        } else {
            format!("→ {from_name} → {to_name} — how close? {proximity} · {scale} · ⏎ draws it · esc draws nothing")
        }
    }
}

/// Where a row of the key helper means something: one table for both
/// screens, each showing only its own rows (Joel, 23/09/2026).
#[derive(Clone, Copy, PartialEq)]
enum Where {
    Both,
    Listening,
    Home,
}

#[cfg(test)]
mod proposal_tests {
    use super::abridged_body;

    /// The confirmation shows two new cards and counts the rest; the
    /// edited cards stay whole.
    #[test]
    fn the_confirmation_abridges_the_new_cards_only() {
        let body = "## New cards (4) — pipeline output, nothing to read\n\n- a · generated\n- b · generated\n- c · generated\n- d · generated\n\n## Edited cards (2) — please read\n\n- x +1 −0 · tops\n  source: listening\n- y +2 −1 · member\n\nForkstify: proposal 0.1.0\n";
        let lines = abridged_body(body);
        let text = lines.join("\n");
        assert!(text.contains("- a · generated\n- b · generated\n… 2 more, one line each"), "{text}");
        assert!(!text.contains("- c"), "{text}");
        assert!(text.contains("- x +1 −0 · tops\n  source: listening\n- y +2 −1 · member"), "{text}");
        assert!(text.ends_with("Forkstify: proposal 0.1.0"), "{text}");
    }
}
