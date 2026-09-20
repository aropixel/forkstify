//! forkstify — branch music player, PoC.
//!
//!   forkstify journey <seed> [catalog]     navigate dry (no sound)
//!   forkstify listen  <seed> [catalog]     navigate and play (Spotify)
//!   forkstify check   <artist> [catalog]   a card's neighbors in space
//!
//! `journey` prints segments without playing — fast, for iterating on the
//! engine. `listen` plays them through the embedded librespot device.

mod catalog;
mod config;
mod edit;
mod embed;
mod engine;
mod generate;
mod explore;
mod fork;
mod home;
mod import;
mod keys;
mod discography;
mod learned;
mod library;
mod listen;
mod mediakeys;
mod setup;
mod sound;
mod sync;
mod tui;
mod spotify;

use catalog::Catalog;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The active catalog: the argument if any, else the setting, else the
/// default location. A setting rather than a hard-coded path, because one
/// may have imported several catalogs and switch between them
/// ([0004](docs/decisions/0004-deux-depots-catalogue-ciblable.md)).
fn catalog_path(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return PathBuf::from(path);
    }
    let configured = config::Config::load().catalog.path;
    if !configured.trim().is_empty() {
        return PathBuf::from(configured.trim());
    }
    // nothing configured: the xdg place the setup clones into — or the
    // clone of before the setup existed (`~/Work/forkstify-catalog`),
    // written into the config once so it stays the one
    let default = config::default_catalog_dir();
    if default.join(".git").exists() {
        return default;
    }
    let legacy = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Work/forkstify-catalog");
    if legacy.join("cards").exists() {
        let _ = config::set_catalog_path(&legacy);
        return legacy;
    }
    default
}

/// The seed: an exact slug, otherwise a search through card names.
fn resolve(catalog: &Catalog, text: &str) -> Option<String> {
    if catalog.cards.contains_key(text) {
        return Some(text.to_string());
    }
    let lower = text.to_lowercase();
    let matches: Vec<&String> = catalog
        .cards
        .iter()
        .filter(|(_, card)| card.name.to_lowercase().contains(&lower))
        .map(|(slug, _)| slug)
        .collect();
    match matches.as_slice() {
        [only] => Some((*only).clone()),
        [] => {
            eprintln!("\"{text}\": no card matches.");
            None
        }
        several => {
            eprintln!("\"{text}\" is ambiguous:");
            for slug in several {
                eprintln!("  {slug}");
            }
            None
        }
    }
}

/// One chosen branch: the artists it walked (empty for an encore) and the
/// tracks it played. `u` pops a whole round.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Round {
    pub artists: Vec<String>,
    pub tracks: Vec<String>,
}

/// The engine state derived from the journey so far: the context (last
/// non-empty branch), the current artist, the universe/visited artists and
/// the played tracks. Shared by the dry navigator and the live listener.
pub fn state_of(rounds: &[Round]) -> (Vec<String>, String, Vec<String>, HashSet<String>, HashSet<String>) {
    let context: Vec<String> = rounds
        .iter()
        .rev()
        .find(|round| !round.artists.is_empty())
        .unwrap()
        .artists
        .clone();
    let current = context.last().unwrap().clone();
    let mut universe: Vec<String> = Vec::new();
    for slug in rounds.iter().flat_map(|round| round.artists.iter()) {
        if !universe.contains(slug) {
            universe.push(slug.clone());
        }
    }
    let visited: HashSet<String> = universe.iter().cloned().collect();
    let played: HashSet<String> =
        rounds.iter().flat_map(|round| round.tracks.iter().cloned()).collect();
    (context, current, universe, visited, played)
}

/// Print the branch menu (shared by both modes).
pub fn show_branches(catalog: &Catalog, current: &str, branches: &[engine::Branch]) {
    println!("\n── from {} ──", catalog.cards[current].name);
    if branches.is_empty() {
        println!("Dead end: no branch left. u to go back, q to quit.");
    }
    for (i, branch) in branches.iter().enumerate() {
        println!("\n  {}  {}", i + 1, branch.label);
        if !branch.reason.is_empty() {
            println!("     {}", branch.reason);
        }
        for stop in &branch.stops {
            println!("     {} {} — {}", stop.source.mark(), stop.title, stop.artist);
        }
    }
}

fn journey(
    catalog: &Catalog,
    seed: &str,
    learned: &learned::Learned,
    tail: &discography::Tail,
    comfort: engine::Comfort,
) {
    let mut rng = thread_rng();
    let mut rounds = vec![Round { artists: vec![seed.to_string()], tracks: Vec::new() }];
    let mut size = 3usize;

    loop {
        let (context, current, universe, visited, played) = state_of(&rounds);
        let branches =
            engine::propose(
                catalog, &context, &universe, learned, tail, comfort, &visited, &played, size,
                &mut rng,
            );
        show_branches(catalog, &current, &branches);

        print!("\n[1-{}, enter = auto, e/<n>e = encore, b<n> = branch size, u = back, q = quit] > ", branches.len().max(1));
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
            break; // end of input
        }
        let apply = |branch: &engine::Branch, rounds: &mut Vec<Round>| {
            println!("→ {}", branch.label);
            rounds.push(Round {
                artists: branch.artists.clone(),
                tracks: branch.stops.iter().map(|s| s.title.clone()).collect(),
            });
        };
        match line.trim() {
            "q" => break,
            "u" => {
                if rounds.len() > 1 {
                    rounds.pop();
                } else {
                    println!("Already at the seed.");
                }
            }
            "" => {
                // the machine picks, weighted by proximity or cosine
                if branches.is_empty() {
                    continue;
                }
                let weights: Vec<f32> = branches.iter().map(|b| b.weight.max(0.1)).collect();
                let draw = WeightedIndex::new(&weights).unwrap();
                apply(&branches[draw.sample(&mut rng)], &mut rounds);
            }
            text => {
                if let Some(number) = text.strip_prefix('b') {
                    match number.parse::<usize>() {
                        Ok(n) if (1..=9).contains(&n) => {
                            size = n;
                            println!("Branch size: {n}");
                        }
                        _ => println!("Size not understood: {text} (b1 to b9)"),
                    }
                } else if let Some(number) = text.strip_suffix('e') {
                    // `e` / `<n>e` — :encore. In the dry PoC there is no
                    // playing track, so it targets the segment's last artist.
                    let count = if number.is_empty() { size } else { number.parse().unwrap_or(0) };
                    if count == 0 || count > 9 {
                        println!("Encore not understood: {text} (e, 2e … 9e)");
                    } else {
                        let stops =
                            engine::encore(catalog, &current, learned, tail, comfort, &played, count, &mut rng);
                        if stops.is_empty() {
                            println!("(no unplayed tops left at {})", catalog.cards[&current].name);
                        } else {
                            println!("Encore {}:", catalog.cards[&current].name);
                            for stop in &stops {
                                println!("   {} {}", stop.source.mark(), stop.title);
                            }
                            rounds.push(Round {
                                artists: Vec::new(),
                                tracks: stops.iter().map(|s| s.title.clone()).collect(),
                            });
                        }
                    }
                } else {
                    match text.parse::<usize>() {
                        Ok(n) if n >= 1 && n <= branches.len() => {
                            apply(&branches[n - 1], &mut rounds);
                        }
                        _ => println!("Choice not understood: {text}"),
                    }
                }
            }
        }
    }

    let path: Vec<&str> = rounds
        .iter()
        .flat_map(|round| round.artists.iter())
        .map(|slug| catalog.cards[slug].name.as_str())
        .collect();
    println!("\nJourney: {}", path.join(" → "));
}

/// `forkstify check`: a card's neighbors in the vector space, to judge its
/// effects rather than its text.
fn check(catalog: &Catalog, slug: &str) {
    let none = HashSet::new();
    let linked: HashSet<String> = engine::graph_neighbors(catalog, slug, &none)
        .into_iter()
        .map(|(neighbor, ..)| neighbor)
        .collect();
    println!("{} is close to:", catalog.cards[slug].name);
    for (neighbor, score) in engine::vector_neighbors(catalog, slug, &none).iter().take(10) {
        let mark = if linked.contains(neighbor) { "lnk" } else { "   " };
        println!("  {score:.3}  {mark}  {}", catalog.cards[neighbor].name);
    }
}

/// Bare `forkstify`: home, then a session, then home again. The screen
/// renders **before** any connection — catalog and learned are local — and
/// the network only comes in when playing.
fn accueil(path: Option<&String>) -> anyhow::Result<()> {
    let mut dir = catalog_path(path);
    let _raw = keys::RawMode::enable();
    let mut tui = tui::Tui::enter()?;
    let mut rx = home::reader();
    // the first launch is simply forkstify without a readable catalog:
    // the setup opens instead of failing (chantier A, 20/09/2026); and
    // `:setup` / `:library` bring it back from the session
    let mut replay: Option<setup::Replay> = None;
    loop {
    if replay.is_some() || Catalog::load(&dir).is_err() {
        let known = Catalog::load(&dir).is_ok().then(|| dir.clone());
        match setup::run(&mut rx, &mut tui, replay.take(), known)? {
            setup::Outcome::Ready(ready) => dir = ready,
            setup::Outcome::Quit => break,
        }
    }
    // 0017: what the other machine learned arrives before we read anything
    // — and what was learned here leaves first
    sync::ensure_merge_driver(&dir);
    let synced = sync::pull(&dir);
    let catalog = Catalog::load(&dir)?;
    let comfort = engine::Comfort::new(config::comfort_at_start());

    loop {
        tui.clear();
        let status = home::Status::read();
        if !status.connected() {
            let rows = home::disconnected_rows(&status, &catalog);
            let _ = tui.draw_home(&tui::HomeView {
                status: vec![
                    (
                        if status.librespot { "✓ librespot" } else { "⏹ no sound" }.into(),
                        status.librespot,
                    ),
                    (
                        if status.web { "✓ api web" } else { "⏹ no track resolved" }.into(),
                        status.web,
                    ),
                    (
                        match &synced {
                            Ok(word) => format!("⇅ {word}"),
                            Err(_) => "⇅ offline".to_string(),
                        },
                        synced.is_ok(),
                    ),
                ],
                census: String::new(),
                rows: &rows,
                prompt: "[waiting on the local network (mdns) · q]".into(),
                comfort: comfort.value(),
            comfort_mode: false,
                comfort_word: listen::comfort_word(comfort.value()),
                collection: None,
                bar: None,
                finder: None,
                explore: None,
                overlay: None,
                toast: None,
            });
            if !status.librespot {
                // the app announces itself: connecting the phone is part of
                // the product, not a setup step anymore
                if home::ask_phone().is_err() {
                    return Ok(());
                }
                continue;
            }
            // the web token is requested again on its own when the session opens
        }

        // connected: home is now a screen of the session, which holds the
        // sound, the API and the learned until we quit for good
        // (Joel, 08/09/2026)
        let learned = learned::Learned::load(&dir);
        let tail = discography::Tail::load();
        let status = vec![
            ("✓ librespot".to_string(), true),
            ("✓ api web".to_string(), true),
            (
                match &synced {
                    Ok(word) => format!("⇅ {word}"),
                    Err(_) => "⇅ offline".to_string(),
                },
                synced.is_ok(),
            ),
        ];
        replay = listen::run(None, learned, tail, comfort, &mut rx, &mut tui, &dir, status)?;
        break;
    }
    if replay.is_none() {
        break;
    }
    }
    // the alternate screen is handed back; nothing is printed on the way out
    drop(tui);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // with no argument, forkstify opens its home — the subcommands are
    // bound to disappear (Joel, 05/09/2026)
    let (command, target, path) = match args.as_slice() {
        [] => return accueil(None),
        // git's merge driver for learned/ (0017): base, ours, theirs
        [command, base, ours, theirs] if command == "merge-learned" => {
            return match sync::merge_learned(Path::new(base), Path::new(ours), Path::new(theirs)) {
                Ok(()) => Ok(()),
                Err(why) => {
                    eprintln!("merge-learned: {why}");
                    std::process::exit(1);
                }
            };
        }
        // the whole index, from the cards (0019) — `--texts` shows what gets
        // vectorized instead, to compare with another vectorizer
        [command, rest @ ..] if command == "vectors" => {
            let path = rest.iter().find(|a| !a.starts_with("--"));
            let catalog = Catalog::load(&catalog_path(path))?;
            if rest.iter().any(|a| a == "--texts") {
                let mut slugs: Vec<&String> = catalog.cards.keys().collect();
                slugs.sort();
                for slug in slugs {
                    println!("--- {slug}\n{}\n", embed::text_of(slug, &catalog.cards[slug], &catalog.cards));
                }
                return Ok(());
            }
            return match embed::regenerate(&catalog_path(path), &catalog.cards) {
                Ok(n) => {
                    println!("✓ {n} vectors written to vectors/vectors.jsonl");
                    Ok(())
                }
                Err(why) => {
                    eprintln!("vectors: {why}");
                    std::process::exit(1);
                }
            };
        }
        [command, target, rest @ ..] => (command.as_str(), target, rest.first()),
        _ => {
            eprintln!("usage: forkstify [journey|listen|check <seed>] [catalog]\n       forkstify import <catalog url> [catalog]\n       forkstify vectors [catalog] [--texts]\n       forkstify merge-learned <base> <ours> <theirs>   (git merge driver)");
            std::process::exit(2);
        }
    };

    // import does not load the catalog: it modifies it
    if command == "import" {
        return import::run(&catalog_path(path), target).map_err(|e| anyhow::anyhow!(e));
    }

    let catalog = Catalog::load(&catalog_path(path))?;
    let learned = learned::Learned::load(&catalog_path(path));
    let Some(slug) = resolve(&catalog, target) else {
        std::process::exit(1);
    };

    match command {
        "journey" => journey(
            &catalog,
            &slug,
            &learned,
            &discography::Tail::load(),
            engine::Comfort::new(config::comfort_at_start()),
        ),
        "listen" => {
            sync::ensure_merge_driver(&catalog_path(path));
            match sync::pull(&catalog_path(path)) {
                Ok(word) => println!("⇅ {word}"),
                Err(why) => println!("⇅ offline — {why}"),
            }
            let _raw = keys::RawMode::enable();
            let mut rx = home::reader();
            let mut tui = tui::Tui::enter()?;
            let _ = listen::run(
                Some(home::Choice::Artist(slug)),
                learned,
                discography::Tail::load(),
                engine::Comfort::new(config::comfort_at_start()),
                &mut rx,
                &mut tui,
                &catalog_path(path),
                vec![("✓ librespot".to_string(), true), ("✓ api web".to_string(), true)],
            )?;
            drop(tui);
        }
        "check" => check(&catalog, &slug),
        other => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}
