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
) -> anyhow::Result<()> {
    // current-thread runtime + LocalSet: the MPRIS Player is !Send (RefCell
    // callbacks) and must be driven with spawn_local. librespot's own tasks
    // run fine here (as in spike-play).
    // raw mode for the whole session (0015); the guard puts the terminal
    // back even if we leave through an error or a panic
    let _raw = keys::RawMode::enable();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let local = tokio::task::LocalSet::new();
    local
        .block_on(&rt, async_run(catalog, choice, learned, tail, comfort, rx))
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

async fn async_run(
    catalog: &Catalog,
    choice: Choice,
    learned: Learned,
    tail: Tail,
    comfort: Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (seed, opening_track) = match &choice {
        Choice::Artist(slug) => (slug.clone(), None),
        Choice::Track { slug, title } => (slug.clone(), Some(title.clone())),
    };
    let seed = seed.as_str();
    println!(
        "Appris : {} artiste(s) écouté(s), {} de familiarité de départ, {} discographie(s) en cache.",
        learned.known(),
        learned.seeded(),
        tail.known()
    );
    println!("Connexion à Spotify…");
    let config = crate::config::Config::load();
    let sound = Sound::connect().await?;
    let web = WebApi::new(config.playback.prefer_studio).await?;

    // MPRIS: let the desktop's media keys (⏮ ⏭ ⏯) drive us
    let (ctrl_tx, mut ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<Control>();
    let _mpris = match mediakeys::start(ctrl_tx).await {
        Ok(player) => {
            println!("✓ Prêt. Le son sort de forkstify (touches multimédia actives via MPRIS).");
            Some(player)
        }
        Err(e) => {
            println!("✓ Prêt (MPRIS indisponible : {e} — touches multimédia inactives).");
            None
        }
    };

    let mut live = Live {
        catalog,
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
        tui: Tui::enter()?,
        force_panel: std::cell::Cell::new(false),
        warm_requested: false,
        branches: Vec::new(),
        pending_branch: None,
        pending: Vec::new(),
        size: 3,
    };
    let mut events = live.sound.events();

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
            },
        );
    }
    live.start_segment(vec![seed.to_string()], opening, true, false).await;
    live.prefetch_next().await;
    live.paint();

    loop {
        tokio::select! {
            event = events.recv() => match event {
                // remember which track is really current…
                Some(ref ev) if request_started(ev).is_some() => {
                    live.current_request_id = request_started(ev);
                }
                // …and only react to the end of THAT track, not stray events
                // from one we already skipped past
                Some(ref ev) if track_over(ev) == live.current_request_id
                    && live.current_request_id.is_some() =>
                {
                    live.on_track_over(track_finished(ev)).await;
                    live.prefetch_next().await;
                    live.paint();
                }
                Some(_) => {}
                None => break,
            },
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
    // la TUI tient l'écran alterné : on la rend avant d'écrire le parcours,
    // sinon il s'afficherait sur un écran qui va disparaître
    drop(live);
    println!("\nParcours : {}", path.join(" → "));
    Ok(())
}

struct Live<'a> {
    catalog: &'a Catalog,
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
    tui: Tui,
    /// « fp » a demandé le volet avant la fin du segment.
    force_panel: std::cell::Cell<bool>,
    /// `:warm` asked for a harvest; the command handler is not async, the
    /// loop does it on the next turn.
    warm_requested: bool,
    branches: Vec<crate::engine::Branch>,
    // a chosen branch and *when* it should take over (0015): at the end of
    // the branch, or right after the current track
    pending_branch: Option<(crate::engine::Branch, When)>,
    // results of the last `/` search, awaiting a numeric pick
    pending: Vec<Hit>,
    size: usize,
}

/// What a comfort value means, so the number is never alone on screen.
pub fn comfort_word(value: u8) -> &'static str {
    match value {
        0 => "cocon",
        1 => "prudent",
        2 => "équilibré",
        3 => "curieux",
        4 => "aventureux",
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
        // a new segment supersedes any branch that was waiting to start
        self.pending_branch = None;
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
        // sanding is where depth is wanted: if the card cannot serve the
        // whole request, go and get the tail first (0012 §1)
        if self.catalog.cards[&current].tops.len() < count + played.len().min(3) {
            self.harvest(&current).await;
        }
        let (_, current, _, _, played) = self.state();
        let stops = crate::engine::encore(
            self.catalog,
            &current,
            &self.learned,
            &self.tail,
            self.comfort,
            &played,
            count,
            &mut self.rng,
        );
        if stops.is_empty() {
            say!(self, "(plus de tops non joués chez {})", self.catalog.cards[&current].name);
            return;
        }
        let where_ = match when {
            When::EndOfBranch => "en fin de branche",
            When::Now => "tout de suite",
            When::NowForce => "tout de suite, le reste retiré",
        };
        say!(self, "↻ encore {} ({} morceaux, {where_})", self.catalog.cards[&current].name, stops.len());
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
                    say!(self, "{heading}");
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
    async fn next(&mut self) {
        // a branch asked for « now » takes over as soon as the track ends
        if matches!(self.pending_branch, Some((_, When::Now)) | Some((_, When::NowForce))) {
            let (branch, when) = self.pending_branch.take().unwrap();
            self.start_branch(branch, when).await;
            return;
        }
        match self.advance().await {
            Advance::Playing => self.render(),
            Advance::Exhausted => {
                // the branch ran out: one parked for its end takes over,
                // otherwise we draw one
                match self.pending_branch.take() {
                    Some((branch, when)) => self.start_branch(branch, when).await,
                    None => self.auto_advance().await,
                }
            }
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
        say!(self, "\n→ {}", branch.label);
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
        let next = self
            .pending_branch
            .as_ref()
            .and_then(|(branch, _)| branch.stops.first())
            .or_else(|| self.queue.front());
        if let Some(stop) = next {
            let (title, artist) = (stop.title.clone(), stop.artist.clone());
            let _ = self.web.resolve(&title, &artist).await;
        }
    }

    /// See the branches on demand, wherever we are in the segment (`p`).
    /// A richer « preview then pick ahead » belongs to the future GUI.
    /// `fp` — faire venir le volet sans attendre le dernier morceau (1b).
    fn preview(&self) {
        self.force_panel.set(true);
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
                    say!(self, "\n⏸ pause");
                } else {
                    self.sound.resume();
                    say!(self, "\n▶ reprise");
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

    async fn choose(&mut self, n: usize, when: When) {
        if n == 0 || n > self.branches.len() {
            say!(self, "Choix incompris.");
            return;
        }
        let branch = self.branches.remove(n - 1);
        // nothing playing: no reason to park it
        if self.current.is_none() {
            self.start_branch(branch, when).await;
            return;
        }
        match when {
            When::EndOfBranch => say!(self, 
                "→ {} (à la fin de la branche — {} morceau(x) d'abord)",
                branch.label,
                self.queue.len()
            ),
            When::Now => say!(self, "→ {} (à la fin du morceau)", branch.label),
            When::NowForce => {
                say!(self, "→ {} (à la fin du morceau, le reste retiré)", branch.label);
                self.queue.clear();
            }
        }
        self.pending_branch = Some((branch, when));
        self.render();
    }

    /// One parsed command (0015). Returns false to quit.
    async fn on_cmd(&mut self, cmd: Cmd) -> bool {
        self.notices.borrow_mut().clear();
        self.force_panel.set(false);
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
            Cmd::Auto => self.auto_advance().await,

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
            Cmd::Help(namespace) => self.help(namespace),

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
            Cmd::Colon(text) => {
                self.colon(&text);
                if std::mem::take(&mut self.warm_requested) {
                    let (_, current, ..) = self.state();
                    self.harvest(&current).await;
                }
            }
        }
        true
    }

    /// `?` — why this track. Says what the catalogue knows of the artist
    /// and what the listening has learned of them: familiarity (our own
    /// decayed plays, or the seed ranking before we ever played them) and
    /// the weight our own « plus / moins souvent » has set.
    fn why(&self) {
        let Some(stop) = self.current.as_ref() else {
            say!(self, "\n(rien en cours)");
            return;
        };
        say!(self, "\n┌─ {} — {}", stop.title, stop.artist);
        if stop.slug.is_empty() {
            say!(self, "└─ hors catalogue : joué depuis Spotify, sans fiche");
            return;
        }
        let card = &self.catalog.cards[&stop.slug];
        if !card.tags.is_empty() {
            say!(self, "│  tags : {}", card.tags.join(", "));
        }
        say!(self, 
            "│  familiarité {:.0}% · poids {:.2} · {} lien(s), {} top(s)",
            self.learned.familiarity01(&stop.slug, &card.name) * 100.0,
            self.learned.weight(&stop.slug),
            card.links.len(),
            card.tops.len()
        );
        match self.branches.first() {
            Some(branch) => say!(self, "└─ d'ici : {} ({})", branch.label, branch.reason),
            None => say!(self, "└─"),
        }
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
        let prompt = if self.pending.is_empty() {
            format!(
                "[1-{} branche · h/l · p · espace = les touches · q]",
                self.branches.len().max(1)
            )
        } else {
            format!("[1-{} pour jouer un résultat, autre touche = annuler]", self.pending.len())
        };
        // 1b : le volet n'est là qu'au moment du choix — dernier morceau du
        // segment, ou « fp » demandé
        let notices = self.notices.borrow();
        let view = View {
            path,
            seed: &seed,
            segment: self.rounds.len(),
            past: &self.past,
            current: self.current.as_ref(),
            paused: self.paused,
            queue: self.queue.as_slices().0,
            branches: &self.branches,
            panel: self.force_panel.get() || (self.queue.is_empty() && self.current.is_some()),
            pending: self.pending_branch.as_ref().map(|(b, _)| b.label.clone()),
            notices: &notices,
            comfort: self.comfort.value(),
            comfort_word: comfort_word(self.comfort.value()),
            prompt,
        };
        let _ = self.tui.draw(&view);
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
            't' => self.not_yet("tt", "promouvoir en top (édition de fiche)"),
            'T' => self.not_yet("tT", "retirer des tops (édition de fiche)"),
            'd' => self.not_yet("td", "en faire une door (édition de fiche)"),
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
            'e' => self.not_yet("ae", "ouvrir la fiche dans $EDITOR"),
            'L' => self.not_yet("aL", "lier à un autre artiste"),
            _ => {}
        }
    }

    /// Go and get an artist's discography, once. A partial harvest is kept:
    /// the tail is a reservoir, not an inventory.
    async fn harvest(&mut self, slug: &str) {
        if self.tail.has(slug) {
            return;
        }
        let card = &self.catalog.cards[slug];
        let Some(spotify_id) = card.spotify.clone() else {
            say!(self, "\n({} n'a pas d'identifiant Spotify dans sa fiche)", card.name);
            return;
        };
        say!(self, "\n… discographie de {} ", card.name);
        match self.web.discography(&spotify_id).await {
            Ok(tracks) => {
                let count = tracks.len();
                self.tail.keep(slug, tracks);
                say!(self, "→ {count} titres en cache");
            }
            Err(why) => say!(self, "→ échec ({why})"),
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

    /// A gesture the grammar accepts but the code does not serve yet. Saying
    /// so beats a silent no-op: the key is right, the wiring is missing.
    fn not_yet(&self, keys: &str, what: &str) {
        say!(self, "\n\u{ab} {keys} \u{bb} \u{2014} {what} : d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}.");
    }

    /// Space, the leader: what can I type from here? With a namespace
    /// half-typed, only that namespace — which-key, in a terminal. Each
    /// line says whether the gesture is wired, so the menu never promises
    /// what the code does not do.
    fn help(&self, namespace: Option<char>) {
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
                ("f\u{2026}", "la branche \u{2014} espace pour le d\u{e9}tail", true),
                ("e\u{2026}", "encore \u{2014} espace pour le d\u{e9}tail", true),
                ("t\u{2026}", "le morceau \u{2014} espace pour le d\u{e9}tail", true),
                ("a\u{2026}", "l'artiste \u{2014} espace pour le d\u{e9}tail", true),
                ("entr\u{e9}e", "auto \u{2014} tirer parmi les branches", true),
                ("h l \u{2190} \u{2192}", "morceau pr\u{e9}c\u{e9}dent / suivant", true),
                ("p", "pause / lecture", true),
                ("/texte", "chercher", true),
                ("u", "annuler le dernier geste", false),
                (".", "r\u{e9}p\u{e9}ter le dernier geste", false),
                ("?", "pourquoi ce morceau", true),
                ("Q", "mode file d'attente", false),
                (":size <n>", "taille des branches", true),
                (":comfort <n>", "zone de confort, 0 cocon → 5 exploration", true),
                (":warm", "récolter la discographie de l'artiste en cours", true),
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
        say!(self, "\n\u{250c}\u{2500} {title} \u{2500}");
        for (keys, what, wired) in rows {
            let mark = if *wired { " " } else { "\u{b7}" };
            say!(self, "\u{2502} {mark} {keys:<8} {what}");
        }
        if rows.iter().any(|(_, _, wired)| !wired) {
            say!(self, "\u{2514}\u{2500} \u{b7} = d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}");
        } else {
            say!(self, "\u{2514}\u{2500}");
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            self.sound.pause();
            say!(self, "\n\u{23f8} pause");
        } else {
            self.sound.resume();
            say!(self, "\n\u{25b6} reprise");
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
