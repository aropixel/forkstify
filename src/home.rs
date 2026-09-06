//! L'écran d'accueil — ce que `forkstify` montre quand on le lance sans rien.
//!
//! Deux états, comme les maquettes le tranchent : un écran **non connecté**
//! tant qu'il manque une autorisation, et l'**accueil** ensuite. L'accueil ne
//! se dégrade pas, il n'existe qu'une fois les autorisations en place.
//!
//! Il se rend **avant toute connexion** : le catalogue et l'appris sont
//! locaux, donc l'écran s'affiche tout de suite et le réseau n'entre en jeu
//! qu'au moment de jouer.
//!
//! Chaque bloc porte sa raison en une ligne (règle de marque : toute décision
//! automatique s'explique). La numérotation court **à travers** les blocs, si
//! bien que choisir une graine est le même geste que choisir une branche.

use crate::catalog::Catalog;
use crate::discography::Tail;
use crate::engine::Comfort;
use crate::keys::{self, Cmd};
use crate::learned::Learned;
use crate::tui::{Collection, CollectionRow, HomeView, Row, Tui};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Ce qu'on a choisi de démarrer. La graine peut être les deux (arbitrage de
/// Joel, 05/09/2026) : un artiste démarre un segment sur lui, un morceau se
/// joue puis branche depuis son artiste.
pub enum Choice {
    Artist(String),
    Track { slug: String, title: String },
}

/// Le dernier parcours, pour « reprendre ». Vit dans le cache, pas dans le
/// catalogue : c'est de la session, pas de la connaissance.
#[derive(Serialize, Deserialize, Clone)]
pub struct LastSession {
    pub slug: String,
    pub name: String,
    pub title: String,
    pub at: String,
}

fn last_path() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache"));
    base.join("forkstify").join("last.json")
}

pub fn remember(last: &LastSession) {
    let path = last_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(last) {
        let _ = std::fs::write(path, text);
    }
}

fn recall() -> Option<LastSession> {
    serde_json::from_str(&std::fs::read_to_string(last_path()).ok()?).ok()
}

/// Comment la collection est triée. Trois lectures d'une même liste : ce
/// qu'on connaît le mieux, l'ordre alphabétique, et ce qu'on n'a pas joué
/// depuis longtemps.
#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Familiarity,
    Alphabetical,
    LastPlayed,
}

impl Sort {
    fn next(self) -> Sort {
        match self {
            Sort::Familiarity => Sort::Alphabetical,
            Sort::Alphabetical => Sort::LastPlayed,
            Sort::LastPlayed => Sort::Familiarity,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Sort::Familiarity => "familiarité",
            Sort::Alphabetical => "a-z",
            Sort::LastPlayed => "dernière écoute",
        }
    }
}

/// « il y a n jours » en deux caractères, comme une TUI le dit.
fn age(days: Option<i64>) -> String {
    match days {
        None => "jamais".into(),
        Some(0) => "auj.".into(),
        Some(1) => "hier".into(),
        Some(d) if d < 14 => format!("-{d}j"),
        Some(d) if d < 60 => format!("-{}s", d / 7),
        Some(d) if d < 365 => format!("-{}m", d / 30),
        Some(d) => format!("-{}a", d / 365),
    }
}

/// La collection entière : le catalogue **et** le classement, réunis. Un
/// artiste sans fiche y figure, mais il ne peut pas servir de graine — les
/// branches viennent de la fiche, et le dire vaut mieux que le cacher.
fn collection(catalog: &Catalog, learned: &Learned, sort: Sort) -> Vec<(Option<String>, CollectionRow)> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut rows: Vec<(Option<String>, CollectionRow)> = Vec::new();

    for (slug, card) in &catalog.cards {
        seen.insert(card.name.to_lowercase());
        let days = learned.days_since(slug);
        rows.push((
            Some(slug.clone()),
            CollectionRow {
                familiarity: (learned.familiarity01(slug, &card.name) * 5.0).round() as u8,
                days,
                name: card.name.clone(),
                carded: true,
                age: age(days),
                neglected: days.is_some_and(|d| d >= 180),
            },
        ));
    }
    for name in learned.ranked_names() {
        if seen.contains(&name.to_lowercase()) {
            continue;
        }
        rows.push((
            None,
            CollectionRow {
                familiarity: (learned.familiarity01("", name) * 5.0).round() as u8,
                days: None,
                name: name.clone(),
                carded: false,
                age: "jamais".into(),
                neglected: false,
            },
        ));
    }

    match sort {
        Sort::Familiarity => rows.sort_by(|a, b| {
            b.1.familiarity.cmp(&a.1.familiarity).then_with(|| a.1.name.cmp(&b.1.name))
        }),
        Sort::Alphabetical => rows.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())),
        // le plus récemment joué d'abord ; ceux qui n'ont jamais sonné
        // ferment la marche, puisqu'ils n'ont pas de dernière écoute
        Sort::LastPlayed => rows.sort_by(|a, b| {
            match (a.1.days, b.1.days) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then_with(|| a.1.name.cmp(&b.1.name))
        }),
    }
    rows
}

/// Ce qui est autorisé, lu sur disque sans rien ouvrir.
pub struct Status {
    pub librespot: bool,
    pub web: bool,
}

impl Status {
    pub fn read() -> Status {
        Status { librespot: crate::sound::has_credentials(), web: crate::spotify::has_refresh() }
    }
    pub fn connected(&self) -> bool {
        self.librespot && self.web
    }
}

/// Une porte d'entrée : une graine, et la phrase qui dit pourquoi elle est là.
struct Entry {
    choice: Choice,
    label: String,
    /// Rempli seulement quand la graine est un **morceau** : le titre passe
    /// devant, l'artiste derrière.
    artist: Option<String>,
    reason: String,
    preview: Vec<(String, String)>,
}


/// L'écran non connecté. Deux situations qui ne se ressemblent pas : jamais
/// autorisé, où il faut expliquer les deux gestes ; autorisation perdue, qui
/// est un passage et non un mur.
pub fn disconnected_rows(status: &Status, catalog: &Catalog) -> Vec<Row> {
    let mut rows = Vec::new();
    if !status.librespot && !status.web {
        rows.push(Row::Rule("premier lancement".into()));
        rows.push(Row::Text(
            "aucune autorisation encore donnée. il en faut deux, elles sont indépendantes.".into(),
        ));
        rows.push(Row::Text(String::new()));
        rows.push(Row::Text("  1 librespot — le son".into()));
        rows.push(Row::Dim(
            "     les identifiants arrivent du téléphone par zeroconf — rien à taper ici.".into(),
        ));
        rows.push(Row::Text(format!(
            "     ouvrez spotify sur le téléphone, « appareils disponibles »,\n     puis choisissez « {} » dans la liste.",
            crate::sound::DEVICE_NAME
        )));
        rows.push(Row::Text(String::new()));
        rows.push(Row::Text("  2 l'api web — les titres".into()));
        rows.push(Row::Dim(
            "     une autorisation oauth dans le navigateur (client id ncspot, cinq scopes).".into(),
        ));
    } else {
        rows.push(Row::Rule("autorisation incomplète".into()));
        if !status.librespot {
            rows.push(Row::Text("il manque les identifiants du téléphone.".into()));
            rows.push(Row::Text(format!(
                "sur spotify : « appareils disponibles », puis « {} ».",
                crate::sound::DEVICE_NAME
            )));
        }
        if !status.web {
            rows.push(Row::Text(
                "le jeton de l'api web a expiré — forkstify le redemandera d'elle-même.".into(),
            ));
            rows.push(Row::Dim(
                "le son n'est pas coupé ; seuls les titres ne se résolvent plus.".into(),
            ));
        }
    }
    rows.push(Row::Rule("en attendant".into()));
    rows.push(Row::Dim(format!(
        "le catalogue est local : {} fiches, leurs vecteurs et l'appris se lisent hors connexion.\ncet écran n'est pas un cul-de-sac.",
        catalog.cards.len()
    )));
    rows.push(Row::Key {
        key: "b".into(),
        what: "parcourir à sec".into(),
        note: "les branches s'affichent, rien ne sonne".into(),
        wired: false,
    });
    rows.push(Row::Key {
        key: "/".into(),
        what: "chercher une fiche au catalogue seul".into(),
        note: String::new(),
        wired: false,
    });
    rows
}

/// Les portes de l'accueil, dans l'ordre que le confort décide.
fn entries(catalog: &Catalog, learned: &Learned, comfort: Comfort) -> Vec<(String, Vec<Entry>)> {
    let mut familiar: Vec<(&String, f32)> = catalog
        .cards
        .iter()
        .filter(|(slug, _)| !learned.artist_is_banned(slug))
        .map(|(slug, card)| (slug, learned.familiarity01(slug, &card.name)))
        .collect();
    familiar.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut habitues: Vec<Entry> = familiar
        .iter()
        .filter(|(_, f)| *f > 0.0)
        .take(2)
        .map(|(slug, f)| {
            let card = &catalog.cards[*slug];
            Entry {
                choice: Choice::Artist((*slug).clone()),
                label: card.name.clone(),
                artist: None,
                reason: format!("familiarité {:.0} %", f * 100.0),
                preview: card
                    .tops
                    .iter()
                    .take(2)
                    .map(|top| (top.clone(), card.name.clone()))
                    .collect(),
            }
        })
        .collect();

    // une graine qui est un morceau, pas un artiste — un titre aimé s'il y en
    // a, sinon un top de l'artiste le plus familier
    let track = learned.liked_anywhere().into_iter().next().or_else(|| {
        familiar.first().and_then(|(slug, _)| {
            catalog.cards[*slug].tops.first().map(|t| ((*slug).clone(), t.clone()))
        })
    });
    if let Some((slug, title)) = track {
        if let Some(card) = catalog.cards.get(&slug) {
            habitues.push(Entry {
                choice: Choice::Track { slug: slug.clone(), title: title.clone() },
                label: title.clone(),
                artist: Some(card.name.clone()),
                reason: format!(
                    "un morceau, pas un artiste : il se joue, puis les branches partent de {}",
                    card.name
                ),
                preview: Vec::new(),
            });
        }
    }

    // les délaissés n'existent qu'avec de l'usage ; le premier jour, ce sont
    // les fiches que rien n'a jamais touchées
    let neglected = learned.neglected(3.0, 90);
    let (second_title, second) = if neglected.is_empty() {
        let jamais: Vec<Entry> = familiar
            .iter()
            .rev()
            .filter(|(_, f)| *f == 0.0)
            .take(2)
            .map(|(slug, _)| {
                let card = &catalog.cards[*slug];
                Entry {
                    choice: Choice::Artist((*slug).clone()),
                    label: card.name.clone(),
                    artist: None,
                    reason: "au catalogue, jamais écouté".to_string(),
                    preview: card
                        .tops
                        .iter()
                        .take(1)
                        .map(|top| (top.clone(), card.name.clone()))
                        .collect(),
                }
            })
            .collect();
        ("jamais écoutés", jamais)
    } else {
        let delaisses: Vec<Entry> = neglected
            .iter()
            .filter_map(|(slug, months)| {
                let card = catalog.cards.get(slug)?;
                Some(Entry {
                    choice: Choice::Artist(slug.clone()),
                    label: card.name.clone(),
                    artist: None,
                    reason: format!("dernière écoute il y a {months} mois"),
                    preview: card
                        .tops
                        .iter()
                        .take(1)
                        .map(|top| (top.clone(), card.name.clone()))
                        .collect(),
                })
            })
            .take(2)
            .collect();
        ("délaissés", delaisses)
    };

    let habitues_bloc = (
        "vos habitués".to_string(),
        habitues,
    );
    let second_bloc = (second_title.to_string(), second);

    // 0012 §4 : c'est le confort qui décide, il n'y a pas d'autre réglage
    if comfort.value() >= 4 {
        vec![second_bloc, habitues_bloc]
    } else {
        vec![habitues_bloc, second_bloc]
    }
}

fn rows_of(
    learned: &Learned,
    blocks: &[(String, Vec<Entry>)],
) -> (Vec<Row>, usize) {
    let mut rows = Vec::new();
    if let Some(last) = recall() {
        rows.push(Row::Rule("reprendre".into()));
        // le titre devant, l'artiste derrière — partout pareil
        rows.push(Row::Key {
            key: "r".into(),
            what: format!("{} — {}", last.title, last.name),
            note: format!("interrompu {}", last.at),
            wired: true,
        });
    }

    rows.push(Row::Rule("chercher".into()));
    rows.push(Row::Key {
        key: "/".into(),
        what: "un artiste ou un morceau".into(),
        note: "catalogue et spotify".into(),
        wired: true,
    });
    rows.push(Row::Dim(
        "     un artiste démarre un segment sur lui ; un morceau se joue,\n     puis branche depuis son artiste s'il a une fiche.".into(),
    ));

    let mut n = 0;
    for (title, block) in blocks {
        rows.push(Row::Rule(title.clone()));
        if title == "délaissés" {
            rows.push(Row::Dim(
                "     une familiarité qui fut haute et a décru — un rappel, pas une découverte."
                    .into(),
            ));
        }
        for entry in block {
            n += 1;
            rows.push(Row::Entry {
                n,
                label: entry.label.clone(),
                artist: entry.artist.clone(),
                reason: entry.reason.clone(),
                tracks: entry.preview.clone(),
            });
        }
    }
    let _ = learned;

    rows.push(Row::Rule("au hasard".into()));
    rows.push(Row::Key {
        key: "entrée".into(),
        what: "tirage pondéré par la zone de confort".into(),
        note: "la porte qui ne demande pas de choisir".into(),
        wired: true,
    });
    (rows, n)
}

/// L'accueil. Rend, lit une touche, et dit ce qu'il faut démarrer.
pub fn run(
    catalog: &Catalog,
    learned: &Learned,
    tail: &Tail,
    comfort: &mut Comfort,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Cmd>,
    tui: &mut Tui,
) -> Option<Choice> {
    // ce qui est en train d'être tapé : la seule chose qui bouge en bas
    let mut typed = String::new();
    let mut said = String::new();
    let mut sort = Sort::Familiarity;
    // le curseur de la collection : tant qu'il n'existe pas, entrée garde son
    // sens de toujours — « choisis pour moi »
    let mut cursor: Option<usize> = None;
    loop {
        let listing = collection(catalog, learned, sort);
        // le Vec doit vivre aussi longtemps que la vue qui l'emprunte
        let shelf: Vec<CollectionRow> = listing.iter().map(|(_, row)| row.clone()).collect();
        let carded = listing.iter().filter(|(slug, _)| slug.is_some()).count();
        let blocks = entries(catalog, learned, *comfort);
        let flat: Vec<&Entry> = blocks.iter().flat_map(|(_, b)| b.iter()).collect();
        let (rows, count) = rows_of(learned, &blocks);
        let _ = tui.draw_home(&HomeView {
            status: vec![("✓ librespot".into(), true), ("✓ api web".into(), true)],
            census: format!(
                "catalogue local — {} fiches · {} artistes classés · {} discographie(s) en cache",
                catalog.cards.len(),
                learned.seeded(),
                tail.known()
            ),
            rows: &rows,
            prompt: if !typed.is_empty() {
                typed.clone()
            } else if !said.is_empty() {
                said.clone()
            } else {
                format!(
                    "[1-{count} pour démarrer · r reprendre · /texte · entrée au hasard · :comfort · q]"
                )
            },
            comfort: comfort.value(),
            comfort_word: crate::listen::comfort_word(comfort.value()),
            collection: Some(Collection {
                rows: &shelf,
                total: listing.len(),
                carded,
                cursor,
                sort: sort.label(),
            }),
        });

        let cmd = rx.blocking_recv()?;
        match &cmd {
            Cmd::Pending(seq) => {
                typed = seq.clone();
                continue;
            }
            Cmd::Typing(line) => {
                typed = line.clone().unwrap_or_default();
                continue;
            }
            Cmd::Unknown(seq) => {
                typed.clear();
                said = format!("(inconnu : {seq})");
                continue;
            }
            _ => {
                typed.clear();
                said.clear();
            }
        }
        match cmd {
            Cmd::Quit => return None,
            Cmd::Digit(n) => {
                if let Some(entry) = flat.get(n - 1) {
                    return Some(match &entry.choice {
                        Choice::Artist(slug) => Choice::Artist(slug.clone()),
                        Choice::Track { slug, title } => {
                            Choice::Track { slug: slug.clone(), title: title.clone() }
                        }
                    });
                }
                said = format!("(pas d'entrée {n})");
            }
            Cmd::Resume => match recall() {
                Some(last) => {
                    return Some(Choice::Track { slug: last.slug, title: last.title })
                }
                None => said = "(aucun parcours à reprendre)".into(),
            },
            // entrée veut dire « choisis pour moi » partout ailleurs : elle
            // garde ce sens ici, et « au hasard » ne coûte pas de touche neuve
            // le curseur de la collection prend le pas : entrée démarre ce
            // qui est sous lui, sinon elle garde son sens de toujours
            Cmd::Auto if cursor.is_some() => {
                let index = cursor.unwrap();
                match listing.get(index) {
                    Some((Some(slug), _)) => return Some(Choice::Artist(slug.clone())),
                    Some((None, row)) => {
                        said = format!(
                            "{} n'a pas de fiche : rien d'où brancher (le catalogue grandit avec l'usage)",
                            row.name
                        )
                    }
                    None => {}
                }
            }
            Cmd::Auto => {
                if let Some(entry) = flat.first() {
                    return Some(match &entry.choice {
                        Choice::Artist(slug) => Choice::Artist(slug.clone()),
                        Choice::Track { slug, title } => {
                            Choice::Track { slug: slug.clone(), title: title.clone() }
                        }
                    });
                }
            }
            Cmd::Search(query) => match crate::resolve(catalog, query.trim()) {
                Some(slug) => return Some(Choice::Artist(slug)),
                None => {}
            },
            Cmd::Colon(text) => {
                let mut words = text.split_whitespace();
                match (words.next(), words.next()) {
                    (Some("comfort"), Some(v)) => match v.parse::<u8>() {
                        Ok(v) if v <= 5 => *comfort = Comfort::new(v),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Cmd::Up => {
                let here = cursor.unwrap_or(0);
                cursor = Some(here.saturating_sub(1));
            }
            Cmd::Down => {
                let here = cursor.map_or(0, |i| i + 1);
                cursor = Some(here.min(listing.len().saturating_sub(1)));
            }
            Cmd::Escape => cursor = None,
            Cmd::Sort => sort = sort.next(),
            Cmd::Help(_) => {}
            _ => {}
        }
    }
}

/// La boucle de découverte, montrée pendant que l'écran non connecté attend.
pub fn ask_phone() -> Result<String, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    rt.block_on(crate::sound::discover())
}

pub fn reader() -> tokio::sync::mpsc::UnboundedReceiver<Cmd> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Cmd>();
    keys::spawn_reader(tx);
    rx
}
