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

/// « il y a n jours » en deux caractères, comme une TUI le dit. Sert à la
/// collection de l'accueil et aux notes de la liste de lecture.
pub(crate) fn age(days: Option<i64>) -> String {
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
fn collection(
    catalog: &Catalog,
    learned: &Learned,
    sort: Sort,
    filter: &str,
) -> Vec<(Option<String>, CollectionRow)> {
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
        if !filter.is_empty() {
        let needle = filter.to_lowercase();
        rows.retain(|(_, row)| row.name.to_lowercase().contains(&needle));
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
        key: ":search".into(),
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
        key: ":search".into(),
        what: "un artiste ou un morceau".into(),
        note: "catalogue et spotify".into(),
        wired: true,
    });
    rows.push(Row::Key {
        key: "/".into(),
        what: "filtrer la collection".into(),
        note: "à droite ; échap efface".into(),
        wired: true,
    });
    rows.push(Row::Dim(
        "     entrée démarre un parcours sur ce qui est choisi : l'artiste,\n     ou le morceau puis les branches de son artiste.".into(),
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
/// Ce que l'accueil répond à une touche.
pub enum Outcome {
    Stay,
    /// Démarrer un parcours — il remplace celui qui joue, s'il y en a un.
    Start(Choice),
    /// Revenir à l'écran de la session en cours, sans rien changer.
    Back,
    /// Ouvrir la modale de recherche, vide ou déjà remplie — elle vit sur
    /// l'écran de la session, qui la tient pour les deux écrans (Joel,
    /// 09/09/2026).
    Find(String),
    /// `:generate <nom>` — faire entrer un artiste absent du catalogue, puis
    /// partir de chez lui ([0016]).
    Generate(String),
    Quit,
}

/// L'accueil est un **écran de la session**, pas une boucle à part (Joel,
/// 08/09/2026) : on y revient de l'écoute par `q`, l'écoute continue en
/// dessous, et `r` ou échap ramènent à l'écran de session. Cet état est ce
/// qui bouge à l'accueil entre deux touches.
pub struct Home {
    /// ce qui est en train d'être tapé : la seule chose qui bouge en bas
    typed: String,
    said: String,
    sort: Sort,
    /// le curseur de la collection : tant qu'il n'existe pas, entrée garde
    /// son sens de toujours — « choisis pour moi »
    cursor: Option<usize>,
    /// `/texte` filtre la collection (Joel, 08/09/2026) ; échap l'efface.
    filter: String,
}

impl Default for Home {
    fn default() -> Self {
        Home {
            typed: String::new(),
            said: String::new(),
            sort: Sort::Familiarity,
            cursor: None,
            filter: String::new(),
        }
    }
}

impl Home {
    /// Ce que l'accueil dit sous la liste, quand la réponse vient d'un
    /// geste fait ailleurs — la modale de recherche, par exemple.
    pub fn say(&mut self, text: String) {
        self.said = text;
    }

    /// Dessine l'accueil. `bar` est le pied de lecture, quand une session
    /// joue en dessous ; `live` dit s'il y a une session où retourner.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        catalog: &Catalog,
        learned: &Learned,
        tail: &Tail,
        comfort: Comfort,
        status: &[(String, bool)],
        bar: Option<crate::tui::Bar<'_>>,
        live: bool,
        finder: Option<crate::tui::FinderView>,
        tui: &mut Tui,
    ) {
        let listing = collection(catalog, learned, self.sort, &self.filter);
        let shelf: Vec<CollectionRow> = listing.iter().map(|(_, row)| row.clone()).collect();
        let carded = listing.iter().filter(|(slug, _)| slug.is_some()).count();
        let blocks = entries(catalog, learned, comfort);
        let (rows, count) = rows_of(learned, &blocks);
        let _ = tui.draw_home(&HomeView {
            status: status.to_vec(),
            census: format!(
                "catalogue local — {} fiches · {} artistes classés · {} discographie(s) en cache",
                catalog.cards.len(),
                learned.seeded(),
                tail.known()
            ),
            rows: &rows,
            prompt: if !self.typed.is_empty() {
                self.typed.clone()
            } else if !self.said.is_empty() {
                self.said.clone()
            } else if !self.filter.is_empty() {
                format!(
                    "[filtre « {} » — {} artiste(s) · ↑↓ entrée · échap efface · :search <texte> cherche]",
                    self.filter,
                    listing.len()
                )
            } else if live {
                format!(
                    "[1-{count} pour démarrer · r retour à l'écoute · /filtre · :search · entrée au hasard · c<n> · q quitter]"
                )
            } else {
                format!(
                    "[1-{count} pour démarrer · r reprendre · /filtre · :search · entrée au hasard · c<n> · q]"
                )
            },
            comfort: comfort.value(),
            comfort_word: crate::listen::comfort_word(comfort.value()),
            collection: Some(Collection {
                rows: &shelf,
                total: listing.len(),
                carded,
                cursor: self.cursor,
                sort: self.sort.label(),
            }),
            bar,
            finder,
        });
    }

    /// Une touche à l'accueil. Les listes se recalculent à chaque touche :
    /// elles sont petites, et c'est ce qui garantit qu'on choisit dans ce
    /// qui est affiché.
    pub fn on_cmd(
        &mut self,
        cmd: Cmd,
        catalog: &Catalog,
        learned: &Learned,
        comfort: &mut Comfort,
        live: bool,
    ) -> Outcome {
        match &cmd {
            Cmd::Pending(seq) => {
                self.typed = seq.clone();
                return Outcome::Stay;
            }
            Cmd::Typing(line) => {
                self.typed = line.clone().unwrap_or_default();
                return Outcome::Stay;
            }
            Cmd::Unknown(seq) => {
                self.typed.clear();
                self.said = format!("(inconnu : {seq})");
                return Outcome::Stay;
            }
            _ => {
                self.typed.clear();
                self.said.clear();
            }
        }
        let listing = collection(catalog, learned, self.sort, &self.filter);
        let blocks = entries(catalog, learned, *comfort);
        let flat: Vec<&Entry> = blocks.iter().flat_map(|(_, b)| b.iter()).collect();
        let pick = |entry: &Entry| match &entry.choice {
            Choice::Artist(slug) => Choice::Artist(slug.clone()),
            Choice::Track { slug, title } => Choice::Track { slug: slug.clone(), title: title.clone() },
        };
        match cmd {
            Cmd::Quit => Outcome::Quit,
            Cmd::Digit(n) => match flat.get(n - 1) {
                Some(entry) => Outcome::Start(pick(entry)),
                None => {
                    self.said = format!("(pas d'entrée {n})");
                    Outcome::Stay
                }
            },
            // « r » ramène à l'écoute en cours ; sans écoute, il reprend la
            // dernière session
            Cmd::Resume if live => Outcome::Back,
            Cmd::Resume => match recall() {
                Some(last) => Outcome::Start(Choice::Track { slug: last.slug, title: last.title }),
                None => {
                    self.said = "(aucun parcours à reprendre)".into();
                    Outcome::Stay
                }
            },
            // le curseur de la collection prend le pas : entrée démarre ce
            // qui est sous lui, sinon elle garde son sens de toujours
            Cmd::Auto if self.cursor.is_some() => {
                let index = self.cursor.unwrap();
                match listing.get(index) {
                    Some((Some(slug), _)) => Outcome::Start(Choice::Artist(slug.clone())),
                    Some((None, row)) => {
                        self.said = format!(
                            "{} n'a pas de fiche : rien d'où brancher (le catalogue grandit avec l'usage)",
                            row.name
                        );
                        Outcome::Stay
                    }
                    None => Outcome::Stay,
                }
            }
            Cmd::Auto => match flat.first() {
                Some(entry) => Outcome::Start(pick(entry)),
                None => Outcome::Stay,
            },
            // « / » filtre la collection ; chercher, c'est « :search »
            Cmd::Search(query) => {
                self.filter = query.trim().to_string();
                self.cursor = if self.filter.is_empty() { None } else { Some(0) };
                Outcome::Stay
            }
            Cmd::Comfort(n) => {
                *comfort = Comfort::new(n);
                self.said = format!("confort {n} — {}", crate::listen::comfort_word(n));
                Outcome::Stay
            }
            Cmd::Colon(text) => {
                let mut words = text.split_whitespace();
                match (words.next(), words.next()) {
                    (Some("comfort"), Some(v)) => {
                        if let Ok(v) = v.parse::<u8>() {
                            if v <= 5 {
                                *comfort = Comfort::new(v);
                            }
                        }
                        Outcome::Stay
                    }
                    // la même modale qu'en écoute, ouverte depuis l'accueil :
                    // « :search » seul l'ouvre vide, « :search <texte> » la
                    // remplit (Joel, 09/09/2026)
                    (Some("search"), _) => {
                        Outcome::Find(text.trim().trim_start_matches("search").trim().to_string())
                    }
                    (Some("generate"), Some(_)) => Outcome::Generate(
                        text.trim().trim_start_matches("generate").trim().to_string(),
                    ),
                    _ => Outcome::Stay,
                }
            }
            Cmd::Up => {
                let here = self.cursor.unwrap_or(0);
                self.cursor = Some(here.saturating_sub(1));
                Outcome::Stay
            }
            Cmd::Down => {
                let here = self.cursor.map_or(0, |i| i + 1);
                self.cursor = Some(here.min(listing.len().saturating_sub(1)));
                Outcome::Stay
            }
            // les deux bouts, comme dans vim
            Cmd::Top => {
                self.cursor = Some(0);
                Outcome::Stay
            }
            Cmd::Bottom => {
                self.cursor = Some(listing.len().saturating_sub(1));
                Outcome::Stay
            }
            // échap efface le filtre, puis rend le curseur ; sans l'un ni
            // l'autre, il rend l'écran de session
            Cmd::Escape if !self.filter.is_empty() => {
                self.filter.clear();
                self.cursor = None;
                Outcome::Stay
            }
            Cmd::Escape if self.cursor.is_some() => {
                self.cursor = None;
                Outcome::Stay
            }
            Cmd::Escape if live => Outcome::Back,
            Cmd::Sort => {
                self.sort = self.sort.next();
                Outcome::Stay
            }
            _ => Outcome::Stay,
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
