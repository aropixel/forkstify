//! `forkstify ecouter` — the dry navigator, but it plays. Same engine and
//! menus as `parcours`; here the segments actually sound, track by track,
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

/// Ce que la session dit à l'écran. Une ligne de journal, pas un print :
/// depuis la TUI, il n'y a plus de flux où écrire — il y a une zone.
macro_rules! say {
    ($self:ident, $($arg:tt)*) => {
        $self.notice(format!($($arg)*))
    };
}
use crate::engine::Comfort;
use crate::discography::Tail;
use crate::home::{Choice, LastSession};
use crate::learned::Learned;
use crate::mediakeys::{self, Control};
use crate::sound::{request_started, track_finished, track_over, Sound};
use crate::spotify::{Resolved, WebApi};
use librespot_playback::player::PlayerEvent;
use crate::{state_of, Round};
use librespot_core::SpotifyUri;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};

pub fn run(
    catalog: &Catalog,
    choice: Choice,
    learned: Learned,
    tail: Tail,
    comfort: Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
    tui: &mut Tui,
    catalog_dir: &std::path::Path,
) -> anyhow::Result<Vec<String>> {
    // current-thread runtime + LocalSet: the MPRIS Player is !Send (RefCell
    // callbacks) and must be driven with spawn_local. librespot's own tasks
    // run fine here (as in spike-play).
    // raw mode for the whole session (0015); the guard puts the terminal
    // back even if we leave through an error or a panic
    let _raw = keys::RawMode::enable();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let local = tokio::task::LocalSet::new();
    local
        .block_on(&rt, async_run(catalog, choice, learned, tail, comfort, rx, tui, catalog_dir))
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

async fn async_run(
    catalog: &Catalog,
    choice: Choice,
    learned: Learned,
    tail: Tail,
    comfort: Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
    tui: &mut Tui,
    catalog_dir: &std::path::Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let (seed, opening_track) = match &choice {
        Choice::Artist(slug) => (slug.clone(), None),
        Choice::Track { slug, title } => (slug.clone(), Some(title.clone())),
    };
    let seed = seed.as_str();
    // l'écran alterné appartient à la TUI : les étapes s'y dessinent, elles
    // ne s'impriment pas
    tui.clear();
    let census = format!(
        "appris : {} artiste(s) écouté(s), {} de familiarité de départ, {} discographie(s) en cache",
        learned.known(),
        learned.seeded(),
        tail.known()
    );
    let mut steps = vec![(census, true), ("connexion à spotify…".to_string(), false)];
    let _ = tui.splash(&steps);
    let config = crate::config::Config::load();
    let sound = Sound::connect().await?;
    steps[1] = ("le son : librespot connecté".to_string(), true);
    steps.push(("autorisation de l'api web — le navigateur s'ouvre si besoin…".to_string(), false));
    let _ = tui.splash(&steps);
    let web = WebApi::new(config.playback.prefer_studio).await?;
    steps[2] = ("les titres : api web autorisée".to_string(), true);
    let _ = tui.splash(&steps);

    // MPRIS: let the desktop's media keys (⏮ ⏭ ⏯) drive us
    let (ctrl_tx, mut ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<Control>();
    let _mpris = match mediakeys::start(ctrl_tx).await {
        Ok(player) => {
            steps.push(("touches multimédia actives (mpris)".to_string(), true));
            Some(player)
        }
        Err(e) => {
            steps.push((format!("mpris indisponible ({e}) — touches multimédia inactives"), true));
            None
        }
    };
    let _ = tui.splash(&steps);

    let mut live = Live {
        catalog,
        catalog_dir: catalog_dir.to_path_buf(),
        learned,
        tail,
        sound,
        web,
        rng: thread_rng(),
        rounds: vec![Round { artists: vec![seed.to_string()], tracks: Vec::new() }],
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
        typed: String::new(),
        warm_requested: false,
        branches: Vec::new(),
        pending: Vec::new(),
        size: 3,
        progress: None,
        help_open: false,
    };
    let mut events = live.sound.events();
    // un tic par seconde fait avancer la barre de progression ; il ne
    // redessine que si quelque chose sonne
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));

    // le lecteur de touches est celui de l'accueil : deux threads sur stdin
    // se voleraient les octets

    // opening: the chosen track first if the seed was one, then the seed's
    // own tops (arbitrage du 05/09 : la graine peut être les deux)
    let mut opening = crate::engine::encore(
        catalog,
        seed,
        &live.learned,
        &live.tail,
        live.comfort,
        &Default::default(),
        live.size,
        &mut live.rng,
    );
    if let Some(title) = opening_track {
        opening.retain(|stop| stop.title != title);
        opening.insert(
            0,
            crate::engine::Stop {
                slug: seed.to_string(),
                artist: catalog.cards[seed].name.clone(),
                title,
                source: crate::engine::Source::Top,
                head: None,
                encore: false,
            },
        );
    }
    live.start_segment(vec![seed.to_string()], opening, true, false).await;
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
                    // events from one we already skipped past
                    } else if live.current_request_id.is_some()
                        && track_over(ev) == live.current_request_id
                    {
                        live.progress = None;
                        live.on_track_over(track_finished(ev)).await;
                        live.prefetch_next().await;
                        live.paint();
                    } else if live.follow_needle(ev) {
                        live.paint();
                    }
                }
                None => break,
            },
            _ = tick.tick() => {
                if live.progress.as_ref().is_some_and(|p| p.running) {
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
    let path: Vec<String> = live
        .rounds
        .iter()
        .flat_map(|round| round.artists.iter())
        .map(|slug| catalog.cards[slug].name.clone())
        .collect();
    // l'écran alterné appartient à l'application entière : le parcours
    // remonte à l'accueil au lieu de s'imprimer sur un écran qui disparaît
    Ok(path)
}

struct Live<'a> {
    catalog: &'a Catalog,
    /// Où vivent les fiches : une édition les modifie et les commite (0013).
    catalog_dir: std::path::PathBuf,
    learned: Learned,
    /// The long tail, harvested on demand (0012 §1, fourth source).
    tail: Tail,
    sound: Sound,
    web: WebApi,
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
    /// Ce que forkstify vient de dire — vidé à chaque commande, pour qu'un
    /// bloc (le leader, « ? ») s'affiche seul et en entier.
    notices: std::cell::RefCell<Vec<String>>,
    tui: &'a mut Tui,
    /// L'axe s'affiche verticalement : les flèches y déplacent une sélection,
    /// et rien ne change tant qu'on n'a pas validé (Joel, 06/09/2026).
    /// Index dans l'axe : passé, puis le courant, puis la file.
    selection: Option<usize>,
    /// « c » ouvre le réglage du confort ; on garde la valeur d'avant pour
    /// qu'échap la rende.
    comfort_before: Option<Comfort>,
    /// Ce qui est en train d'être tapé : une séquence à moitié faite, ou une
    /// ligne après « / » ou « : ». C'est la seule chose qui bouge en bas.
    typed: String,
    /// Un bloc posé sur l'écran — le menu du leader, « ? ». Il ne descend pas
    /// dans le journal : le bas de l'écran ne doit pas bouger.
    overlay: Option<(String, Vec<String>)>,
    /// `:warm` asked for a harvest; the command handler is not async, the
    /// loop does it on the next turn.
    warm_requested: bool,
    branches: Vec<crate::engine::Branch>,
    // results of the last `/` search, awaiting a numeric pick
    pending: Vec<Hit>,
    size: usize,
    /// Where the needle is in the current track, as librespot last said it
    /// — extrapolated by the clock while it plays (maquette 2b).
    progress: Option<Progress>,
    /// The key helper is open: it follows the pending sequence level by
    /// level, and the key that completes a command closes it.
    help_open: bool,
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
pub fn comfort_word(value: u8) -> &'static str {
    match value {
        5 => "cocon",
        4 => "prudent",
        3 => "équilibré",
        2 => "curieux",
        1 => "aventureux",
        _ => "exploration",
    }
}

/// A `/` search result: a catalog artist to branch from, or a Spotify track
/// to play (with its artist's slug when that artist has a card).
enum Hit {
    Artist(String),
    Track { title: String, artist: String, uri: String, slug: Option<String> },
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
        // « now » keeps what was queued behind the new segment; the plain
        // and « force » forms replace it (0015)
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
        let (_, current, _, _, played) = self.state();
        // sanding is where depth is wanted: if the card's unplayed tops
        // cannot serve the whole request, go and get the tail first (0012 §1)
        let card = &self.catalog.cards[&current];
        let unplayed = card.tops.iter().filter(|t| !played.contains(*t)).count();
        let tail = if unplayed < count { self.harvest(&current).await.err() } else { None };
        let (_, current, _, _, played) = self.state();
        let mut stops = crate::engine::encore(
            self.catalog,
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
        // la liste montre ce qui a été ajouté (« ↻ ») : on ne parle que si
        // la demande n'est pas servie, et on dit pourquoi
        if stops.len() < count {
            let name = &self.catalog.cards[&current].name;
            let why = match tail {
                Some(why) => why,
                None if self.comfort.value() == 5 => "la traîne est fermée au cocon (:comfort)".to_string(),
                None => "sa traîne est épuisée".to_string(),
            };
            match stops.len() {
                0 => say!(self, "(plus rien de non joué chez {name} — {why})"),
                n => say!(self, "({n} seulement chez {name} — {why})"),
            }
        }
        if stops.is_empty() {
            return;
        }
        self.rounds.push(Round { artists: Vec::new(), tracks: stops.iter().map(|s| s.title.clone()).collect() });
        if when == When::NowForce {
            self.queue.clear();
        }
        match when {
            When::EndOfBranch => self.queue.extend(stops),
            _ => {
                for stop in stops.into_iter().rev() {
                    self.queue.push_front(stop);
                }
            }
        }
        self.render();
    }

    /// Resolve a stop and load it as the current track.
    async fn load_stop(&mut self, stop: crate::engine::Stop) -> Load {
        let heading = format!("▶ {} {} — {}", stop.source.mark(), stop.title, stop.artist);
        match self.web.resolve(&stop.title, &stop.artist).await {
            Resolved::Track(uri) => match SpotifyUri::from_uri(&uri) {
                Ok(track) => {
                    // rien à dire : le pied de l'écran annonce déjà ce qui
                    // sonne, le redire en faisait un doublon (Joel, 07/09/2026)
                    self.sound.play(track);
                    self.current = Some(stop);
                    Load::Playing
                }
                Err(_) => {
                    say!(self, "{heading} — uri illisible, on saute");
                    Load::Missing
                }
            },
            Resolved::Absent => {
                say!(self, "{heading} — introuvable sur Spotify, on saute");
                Load::Missing
            }
            Resolved::Failed(why) => {
                say!(self, "{heading} — échec : {why}");
                Load::Failed(stop, why)
            }
        }
    }

    /// An outage stops the walk rather than turning every remaining track
    /// into a « introuvable ». Nothing is lost — the queue kept its head.
    fn blocked(&self, why: &str) {
        say!(self, "\n⏹ lecture interrompue : {why}.");
        say!(self, "   Le morceau reste en tête de file — « j » pour réessayer.");
        say!(self, "   Si ça persiste : « q » puis relancer — l'autorisation");
        say!(self, "   Spotify sera redemandée d'elle-même si elle a expiré.");
    }

    /// Move to the next track (the current one falls into the past). No
    /// recursion — the caller decides what to do with the outcome (keeps
    /// the async futures sized).
    async fn advance(&mut self) -> Advance {
        if let Some(current) = self.current.take() {
            self.past.push(current);
        }
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

    /// Step back to the previous track, like a player's « précédent ». The
    /// current track goes back to the front of the queue so `j` returns to it.
    async fn back(&mut self) {
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
                    say!(self, "(déjà au premier morceau)");
                    return;
                }
            }
        }
    }

    /// Move on from the current track: a branch chosen during it starts now
    /// (this is where a deferred choice fires); otherwise the segment plays
    /// its next track, and when it runs out the music auto-advances.
    /// Depuis que choisir une branche l'**ajoute** à la file, il n'y a plus
    /// de branche « en attente » : tout ce qui est décidé est dans la file.
    async fn next(&mut self) {
        match self.advance().await {
            Advance::Playing => {}
            // rien devant et rien de préparé : on tire
            Advance::Exhausted => self.auto_advance().await,
            Advance::Blocked(why) => self.blocked(&why),
        }
    }

    async fn on_track_over(&mut self, finished: bool) {
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
    fn state(&self) -> (Vec<String>, String, Vec<String>, HashSet<String>, HashSet<String>) {
        let (context, current, universe, mut visited, mut played) = state_of(&self.rounds);
        visited.extend(self.learned.banned_artists().cloned());
        played.extend(self.learned.banned_tracks().cloned());
        (context, current, universe, visited, played)
    }

    /// Start a chosen branch (records it, plays its first track, shows it).
    async fn start_branch(&mut self, branch: crate::engine::Branch, when: When) {
        self.start_segment(branch.artists, branch.stops, false, when == When::Now).await;
    }

    /// Show what plays now and what comes next; on the segment's last track,
    /// show the branches instead of an empty « à suivre » (Joel, 04/09/2026).
    /// L'axe et le volet se dessinent depuis l'état : il n'y a plus rien à
    /// imprimer ici. La méthode reste comme point d'accroche des appels
    /// existants.
    fn render(&self) {}

    /// Resolve the *next* track's uri ahead of time so the transition is
    /// instant instead of waiting on `/v1/search` when the current track
    /// ends. The next track is a pending branch's first stop if one is
    /// waiting, otherwise the head of the queue; nothing to do at an
    /// undecided last track (we don't guess the auto-pick — later, maybe).
    /// resolve() caches, so this is a no-op once warmed.
    async fn prefetch_next(&mut self) {
        let next = self.queue.front();
        if let Some(stop) = next {
            let (title, artist) = (stop.title.clone(), stop.artist.clone());
            let _ = self.web.resolve(&title, &artist).await;
        }
    }

    /// See the branches on demand, wherever we are in the segment (`p`).
    /// A richer « preview then pick ahead » belongs to the future GUI.
    /// `fp` — le volet ne se cache plus, il est toujours à droite. La touche
    /// reste pour le dire plutôt que de ne rien faire.
    fn preview(&self) {
        say!(self, "les branches sont affichées en permanence, à droite");
    }

    /// `/` search: catalog artists first (branch-native), then Spotify tracks
    /// (play anything). Results wait in `self.pending` for a numeric pick.
    async fn search(&mut self, query: &str) {
        let mut hits: Vec<Hit> = self
            .catalog
            .search_names(query, 5)
            .into_iter()
            .map(Hit::Artist)
            .collect();
        match self.web.search_tracks(query, 5).await {
            Ok(tracks) => {
                for (title, artist, uri) in tracks {
                    let slug = self.catalog.search_names(&artist, 1).into_iter().next();
                    hits.push(Hit::Track { title, artist, uri, slug });
                }
            }
            Err(why) => say!(self, "(Spotify injoignable — {why} ; catalogue seul)"),
        }

        if hits.is_empty() {
            say!(self, "(rien pour « {query} »)");
            return;
        }
        say!(self, "\nRésultats pour « {query} » :");
        for (i, hit) in hits.iter().enumerate() {
            match hit {
                Hit::Artist(slug) => {
                    say!(self, "  {}  [catalogue] {}", i + 1, self.catalog.cards[slug].name)
                }
                Hit::Track { title, artist, slug, .. } => {
                    // prose, not a glyph: « ↳ » belongs to the door in the
                    // provenance marks, and one glyph must carry one meaning
                    let mark = if slug.is_some() { "branche ensuite" } else { "hors catalogue" };
                    say!(self, "  {}  [spotify]   {title} — {artist} ({mark})", i + 1);
                }
            }
        }
        self.pending = hits;
    }

    /// Pick a `/` result by number.
    async fn pick_search(&mut self, n: usize) {
        if n == 0 || n > self.pending.len() {
            say!(self, "Résultat incompris.");
            self.pending.clear();
            return;
        }
        let hit = self.pending.remove(n - 1);
        self.pending.clear();
        match hit {
            Hit::Artist(slug) => {
                let (_, _, _, _, played) = self.state();
                let stops = crate::engine::encore(
                    self.catalog,
                    &slug,
                    &self.learned,
                    &self.tail,
                    self.comfort,
                    &played,
                    self.size,
                    &mut self.rng,
                );
                say!(self, "→ {}", self.catalog.cards[&slug].name);
                self.start_segment(vec![slug], stops, false, false).await;
            }
            Hit::Track { title, artist, uri, slug } => {
                // a one-track "segment": play it now, branch from its artist
                // if we know it, otherwise it's off-map (no branches from here)
                let round_artists: Vec<String> = slug.iter().cloned().collect();
                // a searched track may or may not already be one of the
                // artist's tops — that is exactly what `tt` would change
                let source = match slug.as_ref().map(|s| &self.catalog.cards[s]) {
                    Some(card) if card.tops.contains(&title) => crate::engine::Source::Top,
                    Some(_) => crate::engine::Source::Outside,
                    None => crate::engine::Source::Offmap,
                };
                let stop = crate::engine::Stop {
                    slug: slug.unwrap_or_default(),
                    artist,
                    title,
                    source,
                    head: None,
                    encore: false,
                };
                self.play_uri(round_artists, stop, &uri).await;
            }
        }
    }

    /// Play an exact Spotify uri now (from `/` search), as a fresh segment.
    async fn play_uri(&mut self, round_artists: Vec<String>, stop: crate::engine::Stop, uri: &str) {
        let Ok(track) = SpotifyUri::from_uri(uri) else {
            say!(self, "uri illisible, on ne joue pas");
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
            say!(self, "(hors catalogue — les branches repartiront du dernier artiste connu)");
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
                say!(self, "\n⏹ arrêt");
            }
        }
    }

    fn recompute(&mut self) {
        let (context, _, universe, visited, played) = self.state();
        self.branches = crate::engine::propose(
            self.catalog, &context, &universe, &self.learned, &self.tail, self.comfort, &visited,
            &played, self.size, &mut self.rng,
        );
        // « moins souvent » / « plus souvent » ride on the branches that
        // start with the artist concerned (0014)
        for branch in &mut self.branches {
            if let Some(first) = branch.artists.first() {
                branch.weight *= self.learned.weight(first);
            }
        }
    }


    /// Take a branch (by weight) and start playing it. The draw is over the
    /// branches *on show*: proposing three then playing a fourth made
    /// « entrée » unreadable (Joel, 05/09/2026).
    async fn auto_advance(&mut self) {
        if self.branches.is_empty() {
            say!(self, "\n(cul-de-sac — « u » pour revenir, « q » pour quitter)");
            return;
        }
        let weights: Vec<f32> = self.branches.iter().map(|b| b.weight.max(0.1)).collect();
        let index = WeightedIndex::new(&weights).unwrap().sample(&mut self.rng);
        let branch = self.branches.swap_remove(index);
        self.start_branch(branch, When::EndOfBranch).await;
    }

    /// Choisir une branche **l'ajoute à ce qui est déjà décidé** au lieu de
    /// le remplacer (Joel, 06/09/2026) : on enchaîne quelques choix et la
    /// suite de la soirée est faite. Les branches suivantes se proposent
    /// alors depuis le **bout** de la file, pas depuis ce qui sonne — c'est
    /// la « chaîne » des maquettes.
    async fn choose(&mut self, n: usize, when: When) {
        if n == 0 || n > self.branches.len() {
            say!(self, "choix incompris");
            return;
        }
        let branch = self.branches.remove(n - 1);
        if self.current.is_none() {
            self.start_branch(branch, when).await;
            return;
        }
        let mut stops = branch.stops;
        if let Some(first) = stops.first_mut() {
            // seul le premier morceau porte le nom : c'est lui qui ouvre la
            // branche à l'écran
            first.head = Some(crate::engine::Head {
                label: branch.label.clone(),
                reason: branch.reason.clone(),
            });
        }
        // rien à dire : la branche apparaît dans la liste avec sa raison
        // (Joel, 07/09/2026 — le pied ne grandit jamais)
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
        // les prochaines directions partent de là où la file s'arrête
        self.recompute();
    }

    /// `x` — retirer de la file le morceau sous la sélection. Il reste
    /// proposable : ce n'est pas un ban, c'est un « pas dans cette soirée ».
    fn drop_selected(&mut self) {
        let Some(index) = self.selection else {
            say!(self, "(rien de sélectionné — ↑↓ pour choisir)");
            return;
        };
        let ahead = index as isize - self.past.len() as isize - 1;
        if ahead < 0 || ahead as usize >= self.queue.len() {
            say!(self, "(on ne retire que ce qui est à suivre)");
            return;
        }
        let ahead = ahead as usize;
        let Some(stop) = self.queue.remove(ahead) else { return };
        // si c'était la tête d'une branche, la suivante en prend le nom
        if let Some(head) = stop.head {
            if let Some(next) = self.queue.get_mut(ahead) {
                if next.head.is_none() {
                    next.head = Some(head);
                }
            }
        }
        if self.selection.map(|i| i >= self.axis_len()).unwrap_or(false) {
            self.selection = Some(self.axis_len().saturating_sub(1));
        }
    }

    /// One parsed command (0015). Returns false to quit.
    async fn on_cmd(&mut self, cmd: Cmd) -> bool {
        self.notices.borrow_mut().clear();
        // le réglage du confort prend la main sur tout le reste
        if self.comfort_before.is_some() {
            return self.on_comfort_key(cmd);
        }
        // un bloc posé sur l'écran tombe au geste suivant — sauf l'aide à la
        // saisie, qui suit la séquence en cours jusqu'à ce qu'elle aboutisse
        let keeps_overlay = match &cmd {
            Cmd::Pending(_) | Cmd::Help(_) => true,
            Cmd::Up | Cmd::Down | Cmd::Auto => !self.help_open,
            _ => false,
        };
        if !keeps_overlay {
            self.overlay = None;
            self.help_open = false;
        }
        // ce qui se tape ne fait que s'afficher : aucune action
        match &cmd {
            Cmd::Pending(seq) => {
                self.typed = seq.clone();
                if self.help_open {
                    // l'aide suit la frappe : « e » ouvre le niveau de e,
                    // ⌫ remonte (Joel, 07/09/2026)
                    self.help(seq.chars().next());
                }
                return true;
            }
            Cmd::Typing(line) => {
                self.typed = line.clone().unwrap_or_default();
                return true;
            }
            Cmd::Unknown(seq) => {
                self.typed.clear();
                say!(self, "(inconnu : {seq})");
                return true;
            }
            _ => self.typed.clear(),
        }
        // while `/` results are on screen, a digit picks one of them rather
        // than a branch; anything else dismisses them
        if !self.pending.is_empty() {
            match cmd {
                Cmd::Digit(n) => {
                    self.pick_search(n).await;
                    return true;
                }
                _ => self.pending.clear(),
            }
        }

        match cmd {
            Cmd::Quit => return false,
            // entrée joue ce qui est sélectionné ; sans sélection, elle garde
            // son sens de toujours — « choisis pour moi »
            Cmd::Auto => match self.selection.take() {
                Some(index) => self.play_at(index).await,
                None => self.auto_advance().await,
            },
            Cmd::Up => self.move_selection(-1),
            // les deux bouts de l'axe : le début de la soirée, ou le bout de
            // ce qui est décidé
            Cmd::Top => self.selection = Some(0),
            Cmd::Bottom => self.selection = Some(self.axis_len().saturating_sub(1)),
            Cmd::Down => self.move_selection(1),
            Cmd::Escape => {
                self.selection = None;
                self.overlay = None;
            }
            Cmd::Track('x') => self.drop_selected(),
            Cmd::ComfortMode => {
                self.comfort_before = Some(self.comfort);
                say!(self, "zone de confort — ↑↓ pour régler, entrée valide, échap annule");
            }

            // --- f, the branch namespace ---
            Cmd::Digit(n) => self.choose(n, When::EndOfBranch).await,
            Cmd::Fork { branch, when } => self.choose(branch, when).await,
            Cmd::Peek => self.preview(),
            Cmd::Reroll => {
                self.recompute();
                say!(self, "\n\u{21bb} autres branches :");
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
                // espace ouvre l'aide, et la referme au niveau d'entrée ;
                // échap la ferme de partout
                if self.help_open && namespace.is_none() {
                    self.overlay = None;
                    self.help_open = false;
                } else {
                    self.help_open = true;
                    self.help(namespace);
                }
            }

            Cmd::Search(query) => self.search(query.trim()).await,

            // --- decided (0015), not wired yet ---
            Cmd::Track(k) => self.on_track_key(k).await,
            Cmd::Artist(k) => self.on_artist_key(k),
            Cmd::Wander => self.not_yet("fw", "partir hors de l'univers courant"),
            Cmd::Undo => self.not_yet("u", "annuler le dernier geste"),
            Cmd::Repeat => self.not_yet(".", "r\u{e9}p\u{e9}ter le dernier geste"),
            Cmd::Why => self.why(),
            Cmd::Queue => self.not_yet("Q", "mode file d'attente"),
            // deux touches de l'accueil, sans emploi une fois qu'on écoute
            Cmd::Resume => say!(self, "\n(« r » sert à l'accueil : ici, « fu » remonte d'une branche)"),
            Cmd::Browse => say!(self, "\n(« b » sert à l'accueil : ici, le son est déjà là)"),
            // déjà traités plus haut : ils ne font qu'afficher
            Cmd::Pending(_) | Cmd::Typing(_) | Cmd::Unknown(_) => {}
            Cmd::Sort => say!(self, "(« s » trie la collection, à l'accueil)"),
            Cmd::Colon(text) => {
                self.colon(&text);
                if std::mem::take(&mut self.warm_requested) {
                    let (_, current, ..) = self.state();
                    match self.harvest(&current).await {
                        Ok(count) => say!(self, "✓ discographie de {} — {count} titres en cache", self.catalog.cards[&current].name),
                        Err(why) => say!(self, "⏹ {why}"),
                    }
                }
            }
        }
        true
    }

    /// `?` — why this track. Says what the catalogue knows of the artist
    /// and what the listening has learned of them: familiarity (our own
    /// decayed plays, or the seed ranking before we ever played them) and
    /// the weight our own « plus / moins souvent » has set.
    fn why(&mut self) {
        let Some(stop) = self.current.clone() else {
            say!(self, "(rien en cours)");
            return;
        };
        let title = format!("{} — {}", stop.title, stop.artist);
        if stop.slug.is_empty() {
            self.overlay = Some((
                title,
                vec![" hors catalogue : joué depuis Spotify, sans fiche".into()],
            ));
            return;
        }
        let card = &self.catalog.cards[&stop.slug];
        let mut lines = Vec::new();
        if !card.tags.is_empty() {
            lines.push(format!(" tags : {}", card.tags.join(", ")));
        }
        lines.push(format!(
            " familiarité {:.0} % · poids {:.2} · {} lien(s), {} top(s)",
            self.learned.familiarity01(&stop.slug, &card.name) * 100.0,
            self.learned.weight(&stop.slug),
            card.links.len(),
            card.tops.len()
        ));
        match self.branches.first() {
            Some(branch) => lines.push(format!(" d'ici : {} ({})", branch.label, branch.reason)),
            None => lines.push(" d'ici : rien de proposé pour l'instant".into()),
        }
        self.overlay = Some((title, lines));
    }

    /// Le nombre de lignes de l'axe : le passé, le morceau en cours, la file.
    fn axis_len(&self) -> usize {
        self.past.len() + usize::from(self.current.is_some()) + self.queue.len()
    }

    /// Déplacer la sélection. Elle démarre sur le morceau en cours, parce que
    /// c'est de là qu'on regarde.
    fn move_selection(&mut self, step: isize) {
        let len = self.axis_len();
        if len == 0 {
            return;
        }
        let here = self.selection.unwrap_or(self.past.len());
        let next = (here as isize + step).clamp(0, len as isize - 1) as usize;
        self.selection = Some(next);
    }

    /// Jouer la ligne sélectionnée. Ce qui la précédait dans la file passe au
    /// passé : on saute *vers* un morceau, on ne le sort pas de l'ordre.
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

    /// Le mode « c » : les flèches bougent la jauge, entrée valide, échap rend
    /// la valeur d'avant. Rien n'est appliqué tant qu'on n'a pas validé.
    fn on_comfort_key(&mut self, cmd: Cmd) -> bool {
        let value = self.comfort.value();
        match cmd {
            Cmd::Up | Cmd::Next => self.comfort = Comfort::new((value + 1).min(5)),
            Cmd::Down | Cmd::Prev => self.comfort = Comfort::new(value.saturating_sub(1)),
            Cmd::Auto => {
                self.comfort_before = None;
                self.recompute();
                say!(self, "zone de confort : {} — {}", value, comfort_word(value));
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
        say!(self, "zone de confort — {} — {}   ↑↓ régler · entrée valider · échap annuler",
             value, comfort_word(value));
        true
    }

    /// Une ligne de plus dans le journal de l'écran.
    fn notice(&self, line: String) {
        let mut notices = self.notices.borrow_mut();
        for part in line.split('\n') {
            notices.push(part.to_string());
        }
        let excess = notices.len().saturating_sub(14);
        notices.drain(..excess);
    }

    /// Redessine. Tout passe par là : la TUI ne montre que l'état, elle ne
    /// décide de rien.
    fn paint(&mut self) {
        let path: Vec<String> = state_of(&self.rounds)
            .2
            .iter()
            .filter_map(|slug| self.catalog.cards.get(slug).map(|c| c.name.clone()))
            .collect();
        let seed = self.rounds[0].artists.first().cloned().unwrap_or_default();
        let prompt = if !self.typed.is_empty() {
            self.typed.clone()
        } else if self.pending.is_empty() {
            format!(
                "[1-{} branche · h/l · p · espace = les touches · q]",
                self.branches.len().max(1)
            )
        } else {
            format!("[1-{} pour jouer un résultat, autre touche = annuler]", self.pending.len())
        };
        // 1a : la colonne des branches est toujours là, chaque branche
        // dépliée avec ses morceaux (Joel, 07/09/2026)
        let notes: Vec<String> = self
            .past
            .iter()
            .chain(self.current.iter())
            .chain(self.queue.iter())
            .map(|stop| self.note(stop))
            .collect();
        // le bloc de la graine (maquette 2b) : d'elle tout descend
        let seed_card = self.catalog.cards.get(&seed);
        let seed_name = seed_card.map(|c| c.name.clone()).unwrap_or_else(|| seed.clone());
        let seed_facts = seed_card
            .map(|c| {
                format!(
                    "{} · {} lien{} · {} top{}",
                    if c.generated { "fiche générée" } else { "fiche écrite" },
                    c.links.len(),
                    if c.links.len() > 1 { "s" } else { "" },
                    c.tops.len(),
                    if c.tops.len() > 1 { "s" } else { "" },
                )
            })
            .unwrap_or_default();
        let seed_last = match self.learned.days_since(&seed) {
            Some(days) => format!("dernière écoute {}", crate::home::age(Some(days))),
            None => "jamais écouté".to_string(),
        };
        let notices = self.notices.borrow();
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
            queue: self.queue.as_slices().0,
            branches: &self.branches,
            panel: true,
            notices: &notices,
            notes: &notes,
            selection: self.selection,
            overlay: self.overlay.as_ref().map(|(t, l)| (t.as_str(), l.as_slice())),
            comfort_mode: self.comfort_before.is_some(),
            comfort: self.comfort.value(),
            comfort_word: comfort_word(self.comfort.value()),
            progress: self.progress.as_ref().map(Progress::now),
            prompt,
        };
        let _ = self.tui.draw(&view);
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

    /// The grey note beside a track (maquette 3a, Joel 07/09/2026): what the
    /// listening knows of it — how often it sounded and when, how often it
    /// was skipped — or that it never did.
    fn note(&self, stop: &crate::engine::Stop) -> String {
        if stop.source == crate::engine::Source::Offmap {
            return "hors catalogue".to_string();
        }
        let mut parts: Vec<String> = Vec::new();
        match self.learned.track_stats(&stop.slug, &stop.title) {
            Some((plays, days, skipped)) => {
                let n = plays.round().max(1.0) as u64;
                if plays >= 0.5 {
                    parts.push(format!("{n} écoute{}", if n > 1 { "s" } else { "" }));
                    parts.push(crate::home::age(days));
                } else {
                    parts.push("jamais joué".to_string());
                }
                if skipped > 0 {
                    parts.push(format!("passé {skipped}×"));
                }
            }
            None => parts.push("jamais joué".to_string()),
        }
        parts.join(" · ")
    }

    /// The track under the needle, or a word saying why there is none.
    fn under_needle(&self) -> Option<crate::engine::Stop> {
        match &self.current {
            None => {
                say!(self, "\n(rien en cours)");
                None
            }
            Some(stop) if stop.slug.is_empty() => {
                say!(self, "\n({} — hors catalogue, rien à apprendre)", stop.artist);
                None
            }
            Some(stop) => Some(stop.clone()),
        }
    }

    /// `t` — the current track. Measures write to `learned/` at once and
    /// without asking (0013); the editions still wait for the layer that
    /// writes cards and commits them.
    async fn on_track_key(&mut self, key: char) {
        let Some(stop) = self.under_needle() else { return };
        match key {
            'l' => {
                self.learned.like_track(&stop.slug, &stop.title);
                say!(self, "\n♥ {} — aimé", stop.title);
            }
            's' => {
                self.learned.skip_track(&stop.slug, &stop.title);
                say!(self, "\n↷ {} — passé, noté", stop.title);
                self.next().await;
            }
            'b' => {
                self.learned.ban_track(&stop.slug, &stop.title);
                self.queue.retain(|s| s.title != stop.title);
                say!(self, "\n⊘ {} — plus jamais", stop.title);
                self.next().await;
            }
            'm' => match self.learned.mark(&stop.artist, &stop.title) {
                Ok(()) => say!(self, "\n⚑ {} — mis de côté", stop.title),
                Err(e) => say!(self, "\n(récolte non écrite : {e})"),
            },
            't' => {
                let done = crate::edit::add_top(
                    &self.catalog_dir,
                    &stop.slug,
                    &stop.artist,
                    &stop.title,
                );
                self.report(done);
            }
            'T' => {
                let done = crate::edit::remove_top(
                    &self.catalog_dir,
                    &stop.slug,
                    &stop.artist,
                    &stop.title,
                );
                self.report(done);
            }
            'd' => {
                // 0011 : une door pointe vers des tags, la direction où l'on
                // va — donc ceux de l'artiste suivant, sinon les siens
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
        let Some(stop) = self.under_needle() else { return };
        match key {
            'l' => {
                let weight = self.learned.like_artist(&stop.slug);
                say!(self, "\n↑ {} — plus souvent (poids {weight:.2})", stop.artist);
                self.recompute();
            }
            's' => {
                let weight = self.learned.skip_artist(&stop.slug);
                say!(self, "\n↓ {} — moins souvent (poids {weight:.2})", stop.artist);
                self.recompute();
            }
            'b' => {
                self.learned.ban_artist(&stop.slug);
                let before = self.queue.len();
                self.queue.retain(|s| s.slug != stop.slug);
                say!(self, 
                    "\n⊘ {} — plus jamais ({} morceau(x) retiré(s) de la file)",
                    stop.artist,
                    before - self.queue.len()
                );
                self.recompute();
            }
            'e' => {
                // $EDITOR demanderait de rendre l'entrée au terminal, or le
                // lecteur de touches tient stdin en permanence — il lui
                // volerait ses frappes. Ça attend une saisie interrogée
                // plutôt qu'un fil bloqué.
                let path = crate::edit::card_path(&self.catalog_dir, &stop.slug);
                say!(self, "fiche : {} (l'ouvrir ici attend la refonte de la saisie)", path.display());
            }
            'L' => {
                // lier à l'artiste d'où l'on vient : c'est le lien qu'on a
                // sous les yeux au moment où l'on veut l'écrire
                let from = self
                    .rounds
                    .iter()
                    .rev()
                    .flat_map(|round| round.artists.iter())
                    .find(|slug| **slug != stop.slug)
                    .cloned();
                match from.and_then(|slug| {
                    self.catalog.cards.get(&slug).map(|card| (slug.clone(), card.name.clone()))
                }) {
                    Some((to_slug, to_name)) => {
                        let done = crate::edit::add_link(
                            &self.catalog_dir,
                            &stop.slug,
                            &stop.artist,
                            &to_slug,
                            &to_name,
                            "similar",
                        );
                        self.report(done);
                    }
                    None => say!(self, "(aucun artiste d'où venir — le lien attend un second)"),
                }
            }
            _ => {}
        }
    }

    /// Go and get an artist's discography, once. A partial harvest is kept:
    /// the tail is a reservoir, not an inventory.
    /// Harvest the long tail of one artist, unless it is already known.
    /// Says nothing: the caller decides what the listener needs to hear.
    /// `Ok` carries how many tracks the tail holds.
    async fn harvest(&mut self, slug: &str) -> Result<usize, String> {
        if self.tail.has(slug) {
            return Ok(self.tail.of(slug).len());
        }
        let card = &self.catalog.cards[slug];
        let Some(spotify_id) = card.spotify.clone() else {
            return Err(format!("{} n'a pas d'identifiant Spotify dans sa fiche", card.name));
        };
        match self.web.discography(&spotify_id).await {
            Ok(tracks) => {
                let count = tracks.len();
                self.tail.keep(slug, tracks);
                Ok(count)
            }
            Err(why) => Err(format!("discographie injoignable ({why})")),
        }
    }

    /// `:` commands — 0013 makes every key the shortcut of one. Only
    /// `:size` is served so far: it replaces the old `b<n>`, which the
    /// move to raw mode dropped on the way.
    fn colon(&mut self, text: &str) {
        let mut words = text.split_whitespace();
        match (words.next(), words.next()) {
            (Some("size"), Some(n)) => match n.parse::<usize>() {
                Ok(n) if (1..=9).contains(&n) => {
                    self.size = n;
                    say!(self, "Taille des branches : {n}");
                }
                _ => say!(self, "Taille attendue entre 1 et 9."),
            },
            (Some("size"), None) => say!(self, "Taille des branches : {}", self.size),
            (Some("comfort"), Some(n)) => match n.parse::<u8>() {
                Ok(n) if n <= 5 => {
                    self.comfort = Comfort::new(n);
                    say!(self, "Zone de confort : {n} — {}", comfort_word(n));
                    // the dial changes which branches make sense from here
                    self.recompute();
                    self.preview();
                }
                _ => say!(self, "Confort attendu entre 0 (cocon) et 5 (exploration)."),
            },
            (Some("warm"), _) => self.warm_requested = true,
            // la surcouche personnelle se calcule, elle ne se stocke pas
            (Some("mine"), _) => match crate::edit::mine(&self.catalog_dir) {
                Ok(lines) => self.overlay = Some(("ce qui est à moi".into(), lines)),
                Err(why) => say!(self, "(impossible de comparer à l'amont : {why})"),
            },
            (Some("comfort"), None) => say!(self, 
                "Zone de confort : {} — {}",
                self.comfort.value(),
                comfort_word(self.comfort.value())
            ),
            (Some(other), _) => self.not_yet(&format!(":{other}"), "cette commande"),
            (None, _) => {}
        }
    }

    /// Keep where we stopped, so the home screen can offer to resume.
    fn remember(&self) {
        let Some(stop) = &self.current else { return };
        if stop.slug.is_empty() {
            return;
        }
        crate::home::remember(&LastSession {
            slug: stop.slug.clone(),
            name: stop.artist.clone(),
            title: stop.title.clone(),
            at: "à l'instant".to_string(),
        });
    }

    /// Dire ce qu'une édition a fait, et la commiter. Une édition qui ne
    /// laisse pas de trace relisible n'en est pas une (0013).
    fn report(&self, done: Result<crate::edit::Edit, String>) {
        match done {
            Ok(edit) => {
                let summary = edit.summary.clone();
                match crate::edit::commit(&self.catalog_dir, &edit) {
                    Ok(()) => say!(self, "✓ {summary} — commité"),
                    Err(why) => say!(self, "✓ {summary} — écrit, mais pas commité ({why})"),
                }
                // le catalogue en mémoire ne bouge pas : l'édition compte au
                // prochain lancement, et il vaut mieux le dire
                say!(self, "  (le moteur en tiendra compte au prochain lancement)");
            }
            Err(why) => say!(self, "(rien fait : {why})"),
        }
    }

    /// A gesture the grammar accepts but the code does not serve yet. Saying
    /// so beats a silent no-op: the key is right, the wiring is missing.
    fn not_yet(&self, keys: &str, what: &str) {
        say!(self, "\n\u{ab} {keys} \u{bb} \u{2014} {what} : d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}.");
    }

    /// Space, the leader: what can I type from here? With a namespace
    /// half-typed, only that namespace — which-key, in a terminal. Each
    /// line says whether the gesture is wired, so the menu never promises
    /// what the code does not do.
    fn help(&mut self, namespace: Option<char>) {
        let rows: &[(&str, &str, bool)] = match namespace {
            Some('f') => &[
                ("f<n>", "branche n, en fin de branche", true),
                ("fn<n>", "branche n, apr\u{e8}s le morceau", true),
                ("f!<n>", "branche n, apr\u{e8}s le morceau, le reste retir\u{e9}", true),
                ("fp", "peek \u{2014} pr\u{e9}voir les branches", true),
                ("fr", "reroll \u{2014} en reproposer trois autres", true),
                ("fu", "undo \u{2014} revenir \u{e0} la branche pr\u{e9}c\u{e9}dente", true),
                ("fw", "wander \u{2014} partir hors de l'univers", false),
            ],
            Some('e') => &[
                ("e<n>", "n encores, en fin de branche", true),
                ("en<n>", "n encores, apr\u{e8}s le morceau", true),
                ("e!<n>", "n encores, le reste retir\u{e9}", true),
            ],
            Some('t') => &[
                ("tl", "like \u{2014} aimer le morceau", true),
                ("ts", "skip \u{2014} pas celui-l\u{e0}, pas maintenant", true),
                ("tb", "ban \u{2014} plus jamais celui-l\u{e0}", true),
                ("tm", "mark \u{2014} mettre de c\u{f4}t\u{e9}", true),
                ("tt", "top \u{2014} promouvoir en top", false),
                ("tT", "untop \u{2014} retirer des tops", false),
                ("td", "door \u{2014} en faire une door", false),
            ],
            Some('a') => &[
                ("al", "like \u{2014} cet artiste, plus souvent", true),
                ("as", "skip \u{2014} cet artiste, moins souvent", true),
                ("ab", "ban \u{2014} plus jamais cet artiste", true),
                ("ae", "edit \u{2014} ouvrir la fiche", false),
                ("aL", "link \u{2014} lier \u{e0} un autre artiste", false),
            ],
            _ => &[
                ("1-9", "prendre une branche", true),
                ("f", "la branche \u{2014} tape f pour ses touches", true),
                ("e", "encore \u{2014} tape e pour ses touches", true),
                ("t", "le morceau \u{2014} tape t pour ses touches", true),
                ("a", "l'artiste \u{2014} tape a pour ses touches", true),
                ("entr\u{e9}e", "auto \u{2014} tirer parmi les branches", true),
                ("h l \u{2190} \u{2192}", "morceau pr\u{e9}c\u{e9}dent / suivant", true),
                ("p", "pause / lecture", true),
                ("/texte", "chercher", true),
                ("u", "annuler le dernier geste", false),
                (".", "r\u{e9}p\u{e9}ter le dernier geste", false),
                ("?", "pourquoi ce morceau", true),
                ("Q", "mode file d'attente", false),
                (":size <n>", "taille des branches", true),
                (":comfort <n>", "zone de confort, 5 cocon → 0 exploration", true),
                (":warm", "récolter la discographie de l'artiste en cours", true),
                (":mine", "ce que ce catalogue a de plus que l'amont", true),
                ("♪♥↳·+~", "top · aimé · door · traîne · hors tops · hors catalogue", true),
                ("q", "quitter", true),
            ],
        };
        let title = match namespace {
            Some('f') => "f \u{2014} la branche",
            Some('e') => "e \u{2014} encore",
            Some('t') => "t \u{2014} le morceau",
            Some('a') => "a \u{2014} l'artiste",
            _ => "les touches",
        };
        // un bloc se pose sur l'écran ; il ne descend pas dans le journal,
        // dont le bas ne doit jamais bouger
        let mut lines: Vec<String> = rows
            .iter()
            .map(|(keys, what, wired)| {
                let mark = if *wired { " " } else { "\u{b7}" };
                format!(" {mark} {keys:<10} {what}")
            })
            .collect();
        if rows.iter().any(|(_, _, wired)| !wired) {
            lines.push(" \u{b7} = d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}".into());
        }
        // c'est une aide à la saisie : la touche tapée ici fait l'action
        lines.push(String::new());
        lines.push(match namespace {
            Some(_) => " une touche = l'action \u{b7} \u{232b} retour \u{b7} \u{e9}chap fermer".into(),
            None => " une touche = l'action \u{b7} \u{e9}chap fermer".into(),
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
            say!(self, "\n(d\u{e9}j\u{e0} \u{e0} la graine)");
            return;
        }
        self.rounds.pop();
        let (_, current, _, _, played) = self.state();
        let stops = crate::engine::encore(
            self.catalog,
            &current,
            &self.learned,
            &self.tail,
            self.comfort,
            &played,
            self.size,
            &mut self.rng,
        );
        self.queue.clear();
        self.start_segment(vec![current], stops, false, false).await;
    }
}
