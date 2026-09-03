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

/// A segment: a few tops, drawn without replacement over the session
/// (decision 0012: repetition must never be endured).
fn draw_segment(card: &catalog::Card, played: &mut HashSet<String>, rng: &mut impl Rng) -> Vec<String> {
    let remaining: Vec<&String> = card.tops.iter().filter(|t| !played.contains(*t)).collect();
    let drawn: Vec<String> = remaining
        .choose_multiple(rng, 3)
        .map(|t| (*t).clone())
        .collect();
    played.extend(drawn.iter().cloned());
    drawn
}

fn journey(catalog: &Catalog, seed: &str) {
    let mut rng = thread_rng();
    let mut history: Vec<String> = vec![seed.to_string()];
    let mut played: HashSet<String> = HashSet::new();

    loop {
        let current = history.last().unwrap().clone();
        let card = &catalog.cards[&current];
        let visited: HashSet<String> = history.iter().cloned().collect();

        println!("\n── {} ──", card.name);
        let titles = draw_segment(card, &mut played, &mut rng);
        if titles.is_empty() {
            println!("   (pas de tops : le moteur piochera dans l'usage — à venir)");
        }
        for title in &titles {
            println!("   ♪ {title}");
        }

        let branches = engine::propose(catalog, &current, &visited);
        if branches.is_empty() {
            println!("\nCul-de-sac : plus aucune branche. « u » pour revenir, « q » pour quitter.");
        } else {
            println!();
            for (i, branch) in branches.iter().enumerate() {
                println!("  {}  {}\n     {}", i + 1, branch.name, branch.reason);
            }
        }

        print!("\n[1-3, entrée = auto, u = retour, q = quitter] > ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
            break; // end of input
        }
        match line.trim() {
            "q" => break,
            "u" => {
                if history.len() > 1 {
                    history.pop();
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
                let picked = &branches[draw.sample(&mut rng)];
                println!("→ {} ({})", picked.name, picked.reason);
                history.push(picked.slug.clone());
            }
            text => match text.parse::<usize>() {
                Ok(n) if n >= 1 && n <= branches.len() => {
                    history.push(branches[n - 1].slug.clone());
                }
                _ => println!("Choix incompris : {text}"),
            },
        }
    }

    println!("\nParcours : {}", history
        .iter()
        .map(|slug| catalog.cards[slug].name.as_str())
        .collect::<Vec<_>>()
        .join(" → "));
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
