//! forkstify — dry-navigation PoC: a seed, three readable branches with
//! their reasons, a keyboard choice, segments printed without playing
//! them. Success criterion: journeys that read as coherent
//! (docs/conception/forme-de-l-application.md).
//!
//!   forkstify parcours <graine> [chemin-du-catalogue]
//!   forkstify check <artiste> [chemin-du-catalogue]

mod catalog;
mod engine;

use catalog::Catalog;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

fn catalog_path(arg: Option<&String>) -> PathBuf {
    match arg {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Work/forkstify-catalog"),
    }
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

/// One chosen branch: the artists it walked (empty when sanding) and the
/// tracks it played. `u` pops a whole round.
struct Round {
    artists: Vec<String>,
    tracks: Vec<String>,
}

fn journey(catalog: &Catalog, seed: &str) {
    let mut rng = thread_rng();
    let mut rounds = vec![Round { artists: vec![seed.to_string()], tracks: Vec::new() }];
    let mut size = 3usize;

    loop {
        let current = rounds
            .iter()
            .rev()
            .find_map(|round| round.artists.last())
            .unwrap()
            .clone();
        let visited: HashSet<String> =
            rounds.iter().flat_map(|round| round.artists.iter().cloned()).collect();
        let played: HashSet<String> =
            rounds.iter().flat_map(|round| round.tracks.iter().cloned()).collect();

        println!("\n── depuis {} ──", catalog.cards[&current].name);
        let branches = engine::propose(catalog, &current, &visited, &played, size, &mut rng);
        if branches.is_empty() {
            println!("Cul-de-sac : plus aucune branche. « u » pour revenir, « q » pour quitter.");
        }
        for (i, branch) in branches.iter().enumerate() {
            println!("\n  {}  {}", i + 1, branch.label);
            if !branch.reason.is_empty() {
                println!("     {}", branch.reason);
            }
            for stop in &branch.stops {
                println!("     ♪ {} — {}", stop.title, stop.artist);
            }
        }

        print!("\n[1-3, entrée = auto, b<n> = taille des branches, u = retour, q = quitter] > ");
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

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, target, path) = match args.as_slice() {
        [command, target, rest @ ..] => (command.as_str(), target, rest.first()),
        _ => {
            eprintln!("usage : forkstify parcours <graine> [catalogue]\n        forkstify check <artiste> [catalogue]");
            std::process::exit(2);
        }
    };

    let catalog = Catalog::load(&catalog_path(path))?;
    let Some(slug) = resolve(&catalog, target) else {
        std::process::exit(1);
    };

    match command {
        "parcours" => journey(&catalog, &slug),
        "check" => check(&catalog, &slug),
        other => {
            eprintln!("commande inconnue : {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}
