//! forkstify — branch music player, PoC.
//!
//!   forkstify parcours <graine> [catalogue]   navigate dry (no sound)
//!   forkstify ecouter  <graine> [catalogue]   navigate and play (Spotify)
//!   forkstify check    <artiste> [catalogue]  a card's neighbors in space
//!
//! `parcours` prints segments without playing — fast, for iterating on the
//! engine. `ecouter` plays them through the embedded librespot device.

mod catalog;
mod config;
mod edit;
mod engine;
mod generate;
mod explore;
mod home;
mod import;
mod keys;
mod discography;
mod learned;
mod listen;
mod mediakeys;
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

/// Le catalogue actif : l'argument s'il y en a un, sinon le réglage, sinon
/// l'emplacement par défaut. Un réglage plutôt qu'un chemin en dur, parce
/// qu'on peut avoir importé plusieurs catalogues et basculer
/// ([0004](docs/decisions/0004-deux-depots-catalogue-ciblable.md)).
fn catalog_path(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return PathBuf::from(path);
    }
    let configured = config::Config::load().catalogue.path;
    if !configured.trim().is_empty() {
        return PathBuf::from(configured.trim());
    }
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Work/forkstify-catalog")
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
            eprintln!("« {text} » : aucune fiche ne correspond.");
            None
        }
        several => {
            eprintln!("« {text} » est ambigu :");
            for slug in several {
                eprintln!("  {slug}");
            }
            None
        }
    }
}

/// One chosen branch: the artists it walked (empty for an encore) and the
/// tracks it played. `u` pops a whole round.
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
    println!("\n── depuis {} ──", catalog.cards[current].name);
    if branches.is_empty() {
        println!("Cul-de-sac : plus aucune branche. « u » pour revenir, « q » pour quitter.");
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

        print!("\n[1-{}, entrée = auto, e/<n>e = encore, b<n> = taille des branches, u = retour, q = quitter] > ", branches.len().max(1));
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
                    println!("Déjà à la graine.");
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
                            println!("Taille des branches : {n}");
                        }
                        _ => println!("Taille incomprise : {text} (b1 à b9)"),
                    }
                } else if let Some(number) = text.strip_suffix('e') {
                    // `e` / `<n>e` — :encore. In the dry PoC there is no
                    // playing track, so it targets the segment's last artist.
                    let count = if number.is_empty() { size } else { number.parse().unwrap_or(0) };
                    if count == 0 || count > 9 {
                        println!("Encore incompris : {text} (e, 2e … 9e)");
                    } else {
                        let stops =
                            engine::encore(catalog, &current, learned, tail, comfort, &played, count, &mut rng);
                        if stops.is_empty() {
                            println!("(plus de tops non joués chez {})", catalog.cards[&current].name);
                        } else {
                            println!("Encore {} :", catalog.cards[&current].name);
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
                        _ => println!("Choix incompris : {text}"),
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
    println!("\nParcours : {}", path.join(" → "));
}

/// `forkstify check`: a card's neighbors in the vector space, to judge its
/// effects rather than its text.
fn check(catalog: &Catalog, slug: &str) {
    let none = HashSet::new();
    let linked: HashSet<String> = engine::graph_neighbors(catalog, slug, &none)
        .into_iter()
        .map(|(neighbor, ..)| neighbor)
        .collect();
    println!("{} est proche de :", catalog.cards[slug].name);
    for (neighbor, score) in engine::vector_neighbors(catalog, slug, &none).iter().take(10) {
        let mark = if linked.contains(neighbor) { "lié" } else { "   " };
        println!("  {score:.3}  {mark}  {}", catalog.cards[neighbor].name);
    }
}

/// `forkstify` sans rien : l'accueil, puis une session, puis l'accueil de
/// nouveau. L'écran se rend **avant** toute connexion — catalogue et appris
/// sont locaux — et le réseau n'entre en jeu qu'au moment de jouer.
fn accueil(path: Option<&String>) -> anyhow::Result<()> {
    let dir = catalog_path(path);
    // 0017 : ce que l'autre machine a appris arrive avant qu'on lise quoi
    // que ce soit — et ce qu'on a appris ici part d'abord
    sync::ensure_merge_driver(&dir);
    let synced = sync::pull(&dir);
    let catalog = Catalog::load(&dir)?;
    let _raw = keys::RawMode::enable();
    let mut tui = tui::Tui::enter()?;
    let mut rx = home::reader();
    let comfort = engine::Comfort::new(config::Config::load().journey.comfort);

    let last_path: Vec<String> = loop {
        tui.clear();
        let status = home::Status::read();
        if !status.connected() {
            let rows = home::disconnected_rows(&status, &catalog);
            let _ = tui.draw_home(&tui::HomeView {
                status: vec![
                    (
                        if status.librespot { "✓ librespot" } else { "⏹ aucun son" }.into(),
                        status.librespot,
                    ),
                    (
                        if status.web { "✓ api web" } else { "⏹ aucun titre résolu" }.into(),
                        status.web,
                    ),
                    (
                        match &synced {
                            Ok(word) => format!("⇅ {word}"),
                            Err(_) => "⇅ hors ligne".to_string(),
                        },
                        synced.is_ok(),
                    ),
                ],
                census: String::new(),
                rows: &rows,
                prompt: "[en attente sur le réseau local (mdns) · q]".into(),
                comfort: comfort.value(),
                comfort_word: listen::comfort_word(comfort.value()),
                collection: None,
                bar: None,
                finder: None,
            });
            if !status.librespot {
                // l'application s'annonce elle-même : brancher le téléphone
                // fait partie du produit, ce n'est plus une mise en route
                if home::ask_phone().is_err() {
                    return Ok(());
                }
                continue;
            }
            // le jeton web se redemande tout seul à l'ouverture de la session
        }

        // connecté : l'accueil est désormais un écran de la session, qui
        // tient le son, l'API et l'appris jusqu'à ce qu'on quitte pour de
        // bon (Joel, 08/09/2026)
        let learned = learned::Learned::load(&dir);
        let tail = discography::Tail::load();
        let status = vec![
            ("✓ librespot".to_string(), true),
            ("✓ api web".to_string(), true),
            (
                match &synced {
                    Ok(word) => format!("⇅ {word}"),
                    Err(_) => "⇅ hors ligne".to_string(),
                },
                synced.is_ok(),
            ),
        ];
        break listen::run(None, learned, tail, comfort, &mut rx, &mut tui, &dir, status)?;
    };

    // l'écran alterné rendu, on laisse le parcours derrière soi
    drop(tui);
    if !last_path.is_empty() {
        println!("\nParcours : {}", last_path.join(" → "));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // sans argument, forkstify ouvre son accueil — les sous-commandes sont
    // vouées à disparaître (Joel, 05/09/2026)
    let (command, target, path) = match args.as_slice() {
        [] => return accueil(None),
        // git's merge driver for learned/ (0017): base, ours, theirs
        [command, base, ours, theirs] if command == "merge-learned" => {
            return match sync::merge_learned(Path::new(base), Path::new(ours), Path::new(theirs)) {
                Ok(()) => Ok(()),
                Err(why) => {
                    eprintln!("merge-learned : {why}");
                    std::process::exit(1);
                }
            };
        }
        [command, target, rest @ ..] => (command.as_str(), target, rest.first()),
        _ => {
            eprintln!("usage : forkstify [parcours|ecouter|check <graine>] [catalogue]\n        forkstify import <url d'un catalogue> [catalogue]\n        forkstify merge-learned <base> <ours> <theirs>   (pilote de fusion git)");
            std::process::exit(2);
        }
    };

    // l'import ne charge pas le catalogue : il le modifie
    if command == "import" {
        return import::run(&catalog_path(path), target).map_err(|e| anyhow::anyhow!(e));
    }

    let catalog = Catalog::load(&catalog_path(path))?;
    let learned = learned::Learned::load(&catalog_path(path));
    let Some(slug) = resolve(&catalog, target) else {
        std::process::exit(1);
    };

    match command {
        "parcours" => journey(
            &catalog,
            &slug,
            &learned,
            &discography::Tail::load(),
            engine::Comfort::new(config::Config::load().journey.comfort),
        ),
        "ecouter" => {
            sync::ensure_merge_driver(&catalog_path(path));
            match sync::pull(&catalog_path(path)) {
                Ok(word) => println!("⇅ {word}"),
                Err(why) => println!("⇅ hors ligne — {why}"),
            }
            let _raw = keys::RawMode::enable();
            let mut rx = home::reader();
            let mut tui = tui::Tui::enter()?;
            let path = listen::run(
                Some(home::Choice::Artist(slug)),
                learned,
                discography::Tail::load(),
                engine::Comfort::new(config::Config::load().journey.comfort),
                &mut rx,
                &mut tui,
                &catalog_path(path),
                vec![("✓ librespot".to_string(), true), ("✓ api web".to_string(), true)],
            )?;
            drop(tui);
            println!("\nParcours : {}", path.join(" → "));
        }
        "check" => check(&catalog, &slug),
        other => {
            eprintln!("commande inconnue : {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}
