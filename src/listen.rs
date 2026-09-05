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
use crate::mediakeys::{self, Control};
use crate::sound::{request_started, track_over, Sound};
use crate::spotify::{Resolved, WebApi};
use crate::{show_branches, state_of, Round};
use librespot_core::SpotifyUri;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::VecDeque;

pub fn run(catalog: &Catalog, seed: &str) -> anyhow::Result<()> {
    // current-thread runtime + LocalSet: the MPRIS Player is !Send (RefCell
    // callbacks) and must be driven with spawn_local. librespot's own tasks
    // run fine here (as in spike-play).
    // raw mode for the whole session (0015); the guard puts the terminal
    // back even if we leave through an error or a panic
    let _raw = keys::RawMode::enable();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let local = tokio::task::LocalSet::new();
    local
        .block_on(&rt, async_run(catalog, seed))
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

async fn async_run(catalog: &Catalog, seed: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        sound,
        web,
        rng: thread_rng(),
        rounds: vec![Round { artists: vec![seed.to_string()], tracks: Vec::new() }],
        past: Vec::new(),
        current: None,
        queue: VecDeque::new(),
        current_request_id: None,
        paused: false,
        branches: Vec::new(),
        pending_branch: None,
        pending: Vec::new(),
        size: 3,
    };
    let mut events = live.sound.events();

    // keys on a blocking thread → async channel, already parsed
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Cmd>();
    keys::spawn_reader(tx);

    // opening: play the seed's own tops, then show the first branches
    let opening = crate::engine::encore(catalog, seed, &Default::default(), live.size, &mut live.rng);
    live.start_segment(vec![seed.to_string()], opening, true, false).await;
    live.prefetch_next().await;
    live.prompt();

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
                    live.on_track_over().await;
                    live.prefetch_next().await;
                    live.prompt();
                }
                Some(_) => {}
                None => break,
            },
            cmd = rx.recv() => match cmd {
                Some(cmd) => {
                    if !live.on_cmd(cmd).await {
                        break;
                    }
                    live.prefetch_next().await;
                    live.prompt();
                }
                None => break,
            },
            control = ctrl_rx.recv() => match control {
                Some(control) => {
                    live.on_control(control).await;
                    live.prefetch_next().await;
                    live.prompt();
                }
                None => {}
            },
        }
    }

    live.sound.stop();
    let path: Vec<&str> = live
        .rounds
        .iter()
        .flat_map(|round| round.artists.iter())
        .map(|slug| catalog.cards[slug].name.as_str())
        .collect();
    println!("\nParcours : {}", path.join(" → "));
    Ok(())
}

struct Live<'a> {
    catalog: &'a Catalog,
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
    branches: Vec<crate::engine::Branch>,
    // a chosen branch and *when* it should take over (0015): at the end of
    // the branch, or right after the current track
    pending_branch: Option<(crate::engine::Branch, When)>,
    // results of the last `/` search, awaiting a numeric pick
    pending: Vec<Hit>,
    size: usize,
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
        let (_, current, _, _, played) = state_of(&self.rounds);
        let stops = crate::engine::encore(self.catalog, &current, &played, count, &mut self.rng);
        if stops.is_empty() {
            println!("(plus de tops non joués chez {})", self.catalog.cards[&current].name);
            return;
        }
        let where_ = match when {
            When::EndOfBranch => "en fin de branche",
            When::Now => "tout de suite",
            When::NowForce => "tout de suite, le reste retiré",
        };
        println!("↻ encore {} ({} morceaux, {where_})", self.catalog.cards[&current].name, stops.len());
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
        print!("\n▶ {} — {} … ", stop.title, stop.artist);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match self.web.resolve(&stop.title, &stop.artist).await {
            Resolved::Track(uri) => match SpotifyUri::from_uri(&uri) {
                Ok(track) => {
                    println!("({uri})");
                    self.sound.play(track);
                    self.current = Some(stop);
                    Load::Playing
                }
                Err(_) => {
                    println!("uri illisible, on saute");
                    Load::Missing
                }
            },
            Resolved::Absent => {
                println!("introuvable sur Spotify, on saute");
                Load::Missing
            }
            Resolved::Failed(why) => {
                println!("échec — {why}");
                Load::Failed(stop, why)
            }
        }
    }

    /// An outage stops the walk rather than turning every remaining track
    /// into a « introuvable ». Nothing is lost — the queue kept its head.
    fn blocked(&self, why: &str) {
        println!("\n⏹ lecture interrompue : {why}.");
        println!("   Le morceau reste en tête de file — « j » pour réessayer.");
        println!("   Si ça persiste : « q » puis relancer — l'autorisation");
        println!("   Spotify sera redemandée d'elle-même si elle a expiré.");
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
                    println!("(déjà au premier morceau)");
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

    async fn on_track_over(&mut self) {
        self.next().await;
    }

    /// Start a chosen branch (records it, plays its first track, shows it).
    async fn start_branch(&mut self, branch: crate::engine::Branch, when: When) {
        println!("\n→ {}", branch.label);
        self.start_segment(branch.artists, branch.stops, false, when == When::Now).await;
    }

    /// Show what plays now and what comes next; on the segment's last track,
    /// show the branches instead of an empty « à suivre » (Joel, 04/09/2026).
    fn render(&self) {
        if self.queue.is_empty() {
            let (_, current, ..) = state_of(&self.rounds);
            println!("\n(dernier du segment — les branches :)");
            show_branches(self.catalog, &current, &self.branches);
        } else {
            println!("\nà suivre :");
            for stop in &self.queue {
                println!("   • {} — {}", stop.title, stop.artist);
            }
        }
    }

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
    fn preview(&self) {
        let (_, current, ..) = state_of(&self.rounds);
        show_branches(self.catalog, &current, &self.branches);
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
            Err(why) => println!("(Spotify injoignable — {why} ; catalogue seul)"),
        }

        if hits.is_empty() {
            println!("(rien pour « {query} »)");
            return;
        }
        println!("\nRésultats pour « {query} » :");
        for (i, hit) in hits.iter().enumerate() {
            match hit {
                Hit::Artist(slug) => {
                    println!("  {}  [catalogue] {}", i + 1, self.catalog.cards[slug].name)
                }
                Hit::Track { title, artist, slug, .. } => {
                    let mark = if slug.is_some() { "↳ branche ensuite" } else { "hors catalogue" };
                    println!("  {}  [spotify]   {title} — {artist} ({mark})", i + 1);
                }
            }
        }
        self.pending = hits;
    }

    /// Pick a `/` result by number.
    async fn pick_search(&mut self, n: usize) {
        if n == 0 || n > self.pending.len() {
            println!("Résultat incompris.");
            self.pending.clear();
            return;
        }
        let hit = self.pending.remove(n - 1);
        self.pending.clear();
        match hit {
            Hit::Artist(slug) => {
                let (_, _, _, _, played) = state_of(&self.rounds);
                let stops = crate::engine::encore(self.catalog, &slug, &played, self.size, &mut self.rng);
                println!("→ {}", self.catalog.cards[&slug].name);
                self.start_segment(vec![slug], stops, false, false).await;
            }
            Hit::Track { title, artist, uri, slug } => {
                // a one-track "segment": play it now, branch from its artist
                // if we know it, otherwise it's off-map (no branches from here)
                let round_artists = slug.iter().cloned().collect();
                let stop = crate::engine::Stop {
                    slug: slug.unwrap_or_default(),
                    artist,
                    title,
                };
                self.play_uri(round_artists, stop, &uri).await;
            }
        }
    }

    /// Play an exact Spotify uri now (from `/` search), as a fresh segment.
    async fn play_uri(&mut self, round_artists: Vec<String>, stop: crate::engine::Stop, uri: &str) {
        let Ok(track) = SpotifyUri::from_uri(uri) else {
            println!("uri illisible, on ne joue pas");
            return;
        };
        if let Some(current) = self.current.take() {
            self.past.push(current);
        }
        let off_map = round_artists.is_empty();
        self.rounds.push(Round { artists: round_artists, tracks: vec![stop.title.clone()] });
        self.queue.clear();
        println!("\n▶ {} — {}", stop.title, stop.artist);
        self.sound.play(track);
        self.current = Some(stop);
        if off_map {
            println!("(hors catalogue — les branches repartiront du dernier artiste connu)");
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
                    println!("\n⏸ pause");
                } else {
                    self.sound.resume();
                    println!("\n▶ reprise");
                }
            }
            Control::Stop => {
                self.paused = true;
                self.sound.stop();
                // the Stopped event we just caused must not be read as a
                // track ending (which would advance) — drop the current id
                self.current_request_id = None;
                println!("\n⏹ arrêt");
            }
        }
    }

    fn recompute(&mut self) {
        let (context, _, universe, visited, played) = state_of(&self.rounds);
        self.branches = crate::engine::propose(
            self.catalog, &context, &universe, &visited, &played, self.size, &mut self.rng,
        );
    }

    fn prompt(&self) {
        if self.pending.is_empty() {
            println!(
                "\n[1-{} branche · h/l · p · espace = les touches · q]",
                self.branches.len().max(1)
            );
        } else {
            println!("\n[1-{} pour jouer un résultat, autre touche = annuler]", self.pending.len());
        }
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }

    /// Take a branch (by weight) and start playing it. The draw is over the
    /// branches *on show*: proposing three then playing a fourth made
    /// « entrée » unreadable (Joel, 05/09/2026).
    async fn auto_advance(&mut self) {
        if self.branches.is_empty() {
            println!("\n(cul-de-sac — « u » pour revenir, « q » pour quitter)");
            return;
        }
        let weights: Vec<f32> = self.branches.iter().map(|b| b.weight.max(0.1)).collect();
        let index = WeightedIndex::new(&weights).unwrap().sample(&mut self.rng);
        let branch = self.branches.swap_remove(index);
        self.start_branch(branch, When::EndOfBranch).await;
    }

    async fn choose(&mut self, n: usize, when: When) {
        if n == 0 || n > self.branches.len() {
            println!("Choix incompris.");
            return;
        }
        let branch = self.branches.remove(n - 1);
        // nothing playing: no reason to park it
        if self.current.is_none() {
            self.start_branch(branch, when).await;
            return;
        }
        match when {
            When::EndOfBranch => println!(
                "→ {} (à la fin de la branche — {} morceau(x) d'abord)",
                branch.label,
                self.queue.len()
            ),
            When::Now => println!("→ {} (à la fin du morceau)", branch.label),
            When::NowForce => {
                println!("→ {} (à la fin du morceau, le reste retiré)", branch.label);
                self.queue.clear();
            }
        }
        self.pending_branch = Some((branch, when));
        self.render();
    }

    /// One parsed command (0015). Returns false to quit.
    async fn on_cmd(&mut self, cmd: Cmd) -> bool {
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
                println!("\n\u{21bb} autres branches :");
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
            Cmd::Track(k) => self.not_yet(&format!("t{k}"), "affinage du morceau"),
            Cmd::Artist(k) => self.not_yet(&format!("a{k}"), "affinage de l'artiste"),
            Cmd::Wander => self.not_yet("fw", "partir hors de l'univers courant"),
            Cmd::Undo => self.not_yet("u", "annuler le dernier geste"),
            Cmd::Repeat => self.not_yet(".", "r\u{e9}p\u{e9}ter le dernier geste"),
            Cmd::Why => self.not_yet("?", "expliquer le morceau ou la branche"),
            Cmd::Queue => self.not_yet("Q", "mode file d'attente"),
            Cmd::Colon(text) => self.not_yet(&format!(":{text}"), "commandes \u{ab} : \u{bb}"),
        }
        true
    }

    /// A gesture the grammar accepts but the code does not serve yet. Saying
    /// so beats a silent no-op: the key is right, the wiring is missing.
    fn not_yet(&self, keys: &str, what: &str) {
        println!("\n\u{ab} {keys} \u{bb} \u{2014} {what} : d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}.");
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
                ("tl", "like \u{2014} aimer le morceau", false),
                ("ts", "skip \u{2014} pas celui-l\u{e0}, pas maintenant", false),
                ("tb", "ban \u{2014} plus jamais celui-l\u{e0}", false),
                ("tm", "mark \u{2014} mettre de c\u{f4}t\u{e9}", false),
                ("tt", "top \u{2014} promouvoir en top", false),
                ("tT", "untop \u{2014} retirer des tops", false),
                ("td", "door \u{2014} en faire une door", false),
            ],
            Some('a') => &[
                ("al", "like \u{2014} cet artiste, plus souvent", false),
                ("as", "skip \u{2014} cet artiste, moins souvent", false),
                ("ab", "ban \u{2014} plus jamais cet artiste", false),
                ("ae", "edit \u{2014} ouvrir la fiche", false),
                ("aL", "link \u{2014} lier \u{e0} un autre artiste", false),
            ],
            _ => &[
                ("1-9", "prendre une branche", true),
                ("f\u{2026}", "la branche \u{2014} espace pour le d\u{e9}tail", true),
                ("e\u{2026}", "encore \u{2014} espace pour le d\u{e9}tail", true),
                ("t\u{2026}", "le morceau \u{2014} espace pour le d\u{e9}tail", false),
                ("a\u{2026}", "l'artiste \u{2014} espace pour le d\u{e9}tail", false),
                ("entr\u{e9}e", "auto \u{2014} tirer parmi les branches", true),
                ("h l \u{2190} \u{2192}", "morceau pr\u{e9}c\u{e9}dent / suivant", true),
                ("p", "pause / lecture", true),
                ("/texte", "chercher", true),
                ("u", "annuler le dernier geste", false),
                (".", "r\u{e9}p\u{e9}ter le dernier geste", false),
                ("?", "pourquoi ce morceau", false),
                ("Q", "mode file d'attente", false),
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
        println!("\n\u{250c}\u{2500} {title} \u{2500}");
        for (keys, what, wired) in rows {
            let mark = if *wired { " " } else { "\u{b7}" };
            println!("\u{2502} {mark} {keys:<8} {what}");
        }
        if rows.iter().any(|(_, _, wired)| !wired) {
            println!("\u{2514}\u{2500} \u{b7} = d\u{e9}cid\u{e9} (0015), pas encore c\u{e2}bl\u{e9}");
        } else {
            println!("\u{2514}\u{2500}");
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            self.sound.pause();
            println!("\n\u{23f8} pause");
        } else {
            self.sound.resume();
            println!("\n\u{25b6} reprise");
        }
    }

    /// Back up one branch: drop the last round and replay the artist we were
    /// on before it. Distinct from `u`, which undoes a *gesture* (0015).
    async fn fork_undo(&mut self) {
        if self.rounds.len() <= 1 {
            println!("\n(d\u{e9}j\u{e0} \u{e0} la graine)");
            return;
        }
        self.rounds.pop();
        let (_, current, _, _, played) = state_of(&self.rounds);
        let stops = crate::engine::encore(self.catalog, &current, &played, self.size, &mut self.rng);
        self.queue.clear();
        self.start_segment(vec![current], stops, false, false).await;
    }
}
