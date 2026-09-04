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
use crate::mediakeys::{self, Control};
use crate::sound::{request_started, track_over, Sound};
use crate::spotify::WebApi;
use crate::{show_branches, state_of, Round};
use librespot_core::SpotifyUri;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::VecDeque;
use std::io::BufRead;

pub fn run(catalog: &Catalog, seed: &str) -> anyhow::Result<()> {
    // current-thread runtime + LocalSet: the MPRIS Player is !Send (RefCell
    // callbacks) and must be driven with spawn_local. librespot's own tasks
    // run fine here (as in spike-play).
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

    // stdin on a blocking thread → async channel
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    // opening: play the seed's own tops, then show the first branches
    let opening = crate::engine::encore(catalog, seed, &Default::default(), live.size, &mut live.rng);
    live.start_segment(vec![seed.to_string()], opening, true).await;
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
            line = rx.recv() => match line {
                Some(line) => {
                    if !live.on_input(line.trim()).await {
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
    // a chosen branch waiting for the current track to end (Joel wants the
    // song to finish before the branch starts); `j` forces it now
    pending_branch: Option<crate::engine::Branch>,
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

impl Live<'_> {
    /// Start a segment: record it, make it the future, play its first track.
    /// `opening` = the seed's own tops (already the first round, don't push).
    async fn start_segment(&mut self, artists: Vec<String>, stops: Vec<crate::engine::Stop>, opening: bool) {
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
        // the chosen segment replaces whatever was still ahead
        self.queue = stops.into();
        self.advance().await;
        // branches for this segment are computed once, up front, so `1`-`3`
        // and `p` work anytime; they are only *shown* on the last track
        self.recompute();
        self.render();
    }

    /// Enqueue more of the current artist right after the current track.
    async fn encore(&mut self, count: usize) {
        let (_, current, _, _, played) = state_of(&self.rounds);
        let stops = crate::engine::encore(self.catalog, &current, &played, count, &mut self.rng);
        if stops.is_empty() {
            println!("(plus de tops non joués chez {})", self.catalog.cards[&current].name);
            return;
        }
        println!("↻ encore {} ({} morceaux)", self.catalog.cards[&current].name, stops.len());
        self.rounds.push(Round { artists: Vec::new(), tracks: stops.iter().map(|s| s.title.clone()).collect() });
        // insert at the front so they play next, without cutting the current track
        for stop in stops.into_iter().rev() {
            self.queue.push_front(stop);
        }
        self.render();
    }

    /// Resolve a stop and load it as the current track. Returns false when
    /// Spotify has no playable match (the caller skips it).
    async fn load_stop(&mut self, stop: crate::engine::Stop) -> bool {
        print!("\n▶ {} — {} … ", stop.title, stop.artist);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match self.web.resolve(&stop.title, &stop.artist).await {
            Some(uri) => match SpotifyUri::from_uri(&uri) {
                Ok(track) => {
                    println!("({uri})");
                    self.sound.play(track);
                    self.current = Some(stop);
                    true
                }
                Err(_) => {
                    println!("uri illisible, on saute");
                    false
                }
            },
            None => {
                println!("introuvable sur Spotify, on saute");
                false
            }
        }
    }

    /// Move to the next track (the current one falls into the past). Returns
    /// false when nothing is left ahead — no recursion, the caller decides
    /// whether to auto-advance (keeps the async futures sized).
    async fn advance(&mut self) -> bool {
        if let Some(current) = self.current.take() {
            self.past.push(current);
        }
        while let Some(stop) = self.queue.pop_front() {
            if self.load_stop(stop).await {
                return true;
            }
        }
        false
    }

    /// Step back to the previous track, like a player's « précédent ». The
    /// current track goes back to the front of the queue so `j` returns to it.
    async fn back(&mut self) {
        let interrupted = self.current.take();
        loop {
            match self.past.pop() {
                Some(previous) => {
                    if self.load_stop(previous).await {
                        if let Some(current) = interrupted {
                            self.queue.push_front(current);
                        }
                        return;
                    }
                }
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
        if let Some(branch) = self.pending_branch.take() {
            self.start_branch(branch).await;
        } else if self.advance().await {
            self.render();
        } else {
            self.auto_advance().await;
        }
    }

    async fn on_track_over(&mut self) {
        self.next().await;
    }

    /// Start a chosen branch now (records it, plays its first track, shows it).
    async fn start_branch(&mut self, branch: crate::engine::Branch) {
        println!("\n→ {}", branch.label);
        self.start_segment(branch.artists, branch.stops, false).await;
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
            .and_then(|branch| branch.stops.first())
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
        for (title, artist, uri) in self.web.search_tracks(query, 5).await {
            let slug = self.catalog.search_names(&artist, 1).into_iter().next();
            hits.push(Hit::Track { title, artist, uri, slug });
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
                self.start_segment(vec![slug], stops, false).await;
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
            print!("\n[1-{}, entrée/auto, j/k = suiv./préc., /texte = chercher, p = branches, e/<n>e = encore, b<n> = taille, u, q] > ",
                self.branches.len().max(1));
        } else {
            print!("\n[1-{} pour jouer un résultat, autre = annuler] > ", self.pending.len());
        }
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }

    /// Take a branch (by weight) and start playing it.
    async fn auto_advance(&mut self) {
        self.recompute();
        if self.branches.is_empty() {
            println!("\n(cul-de-sac — « u » pour revenir, « q » pour quitter)");
            return;
        }
        let weights: Vec<f32> = self.branches.iter().map(|b| b.weight.max(0.1)).collect();
        let index = WeightedIndex::new(&weights).unwrap().sample(&mut self.rng);
        let branch = self.branches.swap_remove(index);
        println!("\n→ {}", branch.label);
        self.start_segment(branch.artists, branch.stops, false).await;
    }

    async fn choose(&mut self, n: usize) {
        if n == 0 || n > self.branches.len() {
            println!("Choix incompris.");
            return;
        }
        let branch = self.branches.remove(n - 1);
        // let the current track finish, then start the branch; if nothing is
        // playing, start it right away
        if self.current.is_some() {
            println!("→ {} (à la fin du morceau — « j » pour tout de suite)", branch.label);
            self.pending_branch = Some(branch);
        } else {
            self.start_branch(branch).await;
        }
    }

    /// Handle one input line; returns false to quit.
    async fn on_input(&mut self, text: &str) -> bool {
        // `/query` starts a search; while results are pending, a number picks
        // one of them rather than a branch
        if let Some(query) = text.strip_prefix('/') {
            self.search(query.trim()).await;
            return true;
        }
        if !self.pending.is_empty() {
            if let Ok(n) = text.parse::<usize>() {
                self.pick_search(n).await;
                return true;
            }
            self.pending.clear(); // any other key cancels the search
        }
        match text {
            "q" => return false,
            "" => self.auto_advance().await,
            // player-style track navigation (vim: j down/next, k up/previous)
            "j" => self.next().await,
            "k" => {
                self.back().await;
                self.render();
            }
            "p" => self.preview(),
            "u" => {
                if self.rounds.len() > 1 {
                    self.rounds.pop();
                    let (_, current, _, _, played) = state_of(&self.rounds);
                    let stops = crate::engine::encore(self.catalog, &current, &played, self.size, &mut self.rng);
                    self.queue.clear();
                    self.start_segment(vec![current], stops, false).await;
                } else {
                    println!("Déjà à la graine.");
                }
            }
            _ if text.starts_with('b') => match text[1..].parse::<usize>() {
                Ok(n) if (1..=9).contains(&n) => {
                    self.size = n;
                    println!("Taille des branches : {n}");
                }
                _ => println!("Taille incomprise (b1 à b9)."),
            },
            _ if text.ends_with('e') => {
                let number = &text[..text.len() - 1];
                let count = if number.is_empty() { self.size } else { number.parse().unwrap_or(0) };
                if count == 0 || count > 9 {
                    println!("Encore incompris (e, 2e … 9e).");
                } else {
                    self.encore(count).await;
                }
            }
            _ => match text.parse::<usize>() {
                Ok(n) => self.choose(n).await,
                Err(_) => println!("Commande incomprise : {text}"),
            },
        }
        true
    }
}
