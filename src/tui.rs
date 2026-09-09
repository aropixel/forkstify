//! La TUI (décision [0006] : ratatui). Variantes **1a** puis **2b** des
//! maquettes (`Lecture.dc.html`, Joel le 07/09/2026) : deux volets
//! permanents — la liste de lecture à gauche, les branches à droite,
//! chacune dépliée avec ses morceaux —, la graine en bloc au-dessus, et un
//! pied qui dit ce qui sonne et ce qui suit.
//!
//! Elle n'emprunte à ratatui que le **dessin**. La saisie reste celle de
//! `keys.rs` — termios brut, grammaire sans préfixe — parce qu'elle est
//! déjà éprouvée et que ratatui n'a pas besoin de posséder l'entrée.
//!
//! Les couleurs sont des **rôles**, jamais des hex : forkstify emprunte la
//! palette du terminal, si bien que changer de thème Omarchy le rethème.

use crate::engine::{Branch, Head, Source, Stop};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{Stdout, Write};

// Les rôles du design system, sur la palette ANSI du terminal.
const BRANCH: Color = Color::LightMagenta;
const PLAYING: Color = Color::Green;
const CATALOG: Color = Color::Blue;
const VECTOR: Color = Color::Cyan;
const DOOR: Color = Color::LightRed;
const EDIT: Color = Color::Yellow;
const DANGER: Color = Color::Red;
/// Ce qui charge : la couleur des vecteurs, celle de l'attente réseau.
pub const LOADING: Color = Color::Cyan;
const MUTED: Color = Color::Gray;

/// La part de largeur donnée à la colonne de gauche — les propositions à
/// l'accueil, l'axe en écoute : **même disposition sur les deux écrans**
/// (Joel, 07/09/2026). 60 laisse à la droite de quoi montrer un nom long
/// sans couper ; 50 la rend plus présente. Une seule valeur à changer.
const LEFT_SHARE: u16 = 60;
const DIM: Color = Color::DarkGray;

/// Sous cette largeur, la colonne de droite s'efface : mieux vaut une
/// colonne lisible que deux illisibles.
const SPLIT_MIN: u16 = 60;

fn role_of(source: Source) -> Color {
    match source {
        Source::Top | Source::Liked => PLAYING,
        Source::Door => DOOR,
        Source::Tail => VECTOR,
        Source::Outside => MUTED,
        Source::Offmap => DIM,
    }
}

/// Tout ce que l'écran a besoin de savoir. La session le remplit, la TUI ne
/// décide de rien : le moteur produit, l'affichage montre.
pub struct View<'a> {
    /// Les artistes traversés — comptés dans le bloc de la graine, plus
    /// affichés en en-tête : la liste jouée le dit déjà (maquette 2b).
    pub path: Vec<String>,
    pub seed: &'a str,
    pub seed_name: &'a str,
    /// « fiche écrite · 41 liens · 12 tops »
    pub seed_facts: &'a str,
    /// « dernière écoute -3s », ou « jamais écouté »
    pub seed_last: &'a str,
    /// Les embranchements pris depuis la graine.
    pub forks: usize,
    pub segment: usize,
    pub past: &'a [Stop],
    pub current: Option<&'a Stop>,
    pub paused: bool,
    /// Le morceau courant est affiché mais pas encore résolu : Spotify
    /// cherche son adresse derrière l'écran.
    pub loading: bool,
    pub queue: &'a [Stop],
    pub branches: &'a [Branch],
    /// Les liens qui pointent vers une fiche absente (0016) : des directions
    /// que le catalogue nomme mais ne sait pas encore marcher. Elles se
    /// numérotent **à la suite** des branches, et les prendre génère la
    /// fiche au lieu de jouer tout de suite.
    pub missing: &'a [crate::engine::Missing],
    /// Le volet des branches est **toujours là** (Joel, 06/09/2026) : on ne
    /// veut pas attendre l'embranchement pour savoir où l'on peut aller.
    /// Il a donc sa place réservée à droite plutôt que d'être posé sur
    /// l'axe — sinon il masquerait en permanence le bas de la file.
    pub panel: bool,

    /// Une note grise par morceau de l'axe (passé, courant, file), dans le
    /// même ordre : ce que l'écoute en sait (maquette 3a).
    pub notes: &'a [String],
    /// La ligne de l'axe sous la sélection — surlignée, mais pas jouée.
    pub selection: Option<usize>,
    /// Un bloc posé sur l'écran, qui ne descend pas dans le journal : le bas
    /// de l'écran ne doit jamais bouger (Joel, 06/09/2026).
    pub overlay: Option<(&'a str, &'a [String])>,
    pub comfort_mode: bool,
    pub comfort: u8,
    pub comfort_word: &'a str,
    /// (position, durée) en millisecondes du morceau qui sonne — `None`
    /// tant que librespot n'a rien dit.
    pub progress: Option<(u32, u32)>,
    /// Le cartouche en bas à droite : ce qui charge, ou la dernière chose
    /// dite, en couleur (Joel, 08/09/2026).
    pub toast: Option<Toast>,
    /// La modale de recherche, quand elle est ouverte.
    pub finder: Option<FinderView>,
    /// La modale de la discographie (`ad`), posée sur l'écoute : elle prend
    /// le corps de l'écran, l'en-tête et le pied restent — « la lecture n'a
    /// pas cessé » (maquette 1a).
    pub explore: Option<&'a crate::explore::Explore>,
    pub prompt: String,
}

/// Le pied de lecture : ce qui sonne, sa progression, ce qui suit, la
/// dernière chose dite. Le même sous la session et sous l'accueil (Joel,
/// 08/09/2026) — l'écoute continue quand on change d'écran.
pub struct Bar<'a> {
    pub current: Option<&'a Stop>,
    pub paused: bool,
    pub loading: bool,
    pub progress: Option<(u32, u32)>,
    /// (rang du courant, total) dans la liste de lecture
    pub position: (usize, usize),
    pub next: Option<&'a Stop>,
    pub ahead: usize,
}

/// La modale de recherche (maquette `Recherche.dc.html`, Joel 08/09/2026) :
/// une ligne de saisie, une règle qui coupe et compte, le catalogue avant
/// Spotify, jamais mêlés.
pub struct FinderView {
    /// `ti` plutôt que `:search` : le titre change, et l'ancre s'affiche.
    pub insert: bool,
    pub anchor: Option<String>,
    pub query: String,
    /// (catalogue, spotify, spotify en cours d'interrogation)
    pub counts: (usize, Option<usize>, bool),
    pub only_catalogue: bool,
    pub lines: Vec<FinderLine>,
    /// L'index, parmi les `Row` seulement.
    pub cursor: usize,
}

pub enum FinderLine {
    Header { catalogue: bool, text: String },
    Row { catalogue: bool, mark: char, title: String, artist: String, note: String },
    Info(String),
}

/// Un toast : un texte, sa couleur, et s'il reste tant que ça charge.
pub struct Toast {
    pub text: String,
    pub tone: Color,
    pub sticky: bool,
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Écran alterné, curseur caché. Les séquences sont écrites à la main :
    /// ratatui ne sert qu'à dessiner, pas à tenir le terminal.
    pub fn enter() -> std::io::Result<Tui> {
        let mut out = std::io::stdout();
        write!(out, "\x1b[?1049h\x1b[?25l")?;
        out.flush()?;
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        Ok(Tui { terminal })
    }

    pub fn draw(&mut self, view: &View) -> std::io::Result<()> {
        self.terminal.draw(|frame| render(frame, view))?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = write!(out, "\x1b[?25h\x1b[?1049l");
        let _ = out.flush();
    }
}

/// La ligne d'une notification, mise en forme par sa nature — lue au
/// glyphe qui l'ouvre, comme le composant Notice du design system : ✓ en
/// vert, ⏹ et ⊘ en rouge, ↻ ⚑ en jaune (une édition), → en magenta, une
/// parenthèse en gris, « pas encore câblé » en italique estompé. Ce qui
/// suit un « — » ou une parenthèse finale est le détail, estompé.
/// La couleur d'un message, lue au glyphe qui l'ouvre — la même pour la
/// ligne du pied et pour le toast.
pub fn tone_of(text: &str) -> Color {
    let text = text.trim();
    let not_wired = text.contains("pas encore c\u{e2}bl\u{e9}");
    let first = text.chars().next().unwrap_or(' ');
    match first {
        '✓' | '♥' | '▶' => PLAYING,
        '⏹' | '⊘' => DANGER,
        '↻' | '⚑' => EDIT,
        '→' => BRANCH,
        '…' | '⏳' => LOADING,
        '(' => MUTED,
        _ if not_wired => DIM,
        _ if text.starts_with("échec") || text.contains("introuvable") || text.contains("illisible") => DANGER,
        _ => Color::White,
    }
}

fn notice_line(text: &str) -> Line<'static> {
    let text = text.trim();
    if text.is_empty() {
        return Line::from("");
    }
    let not_wired = text.contains("pas encore c\u{e2}bl\u{e9}");
    let first = text.chars().next().unwrap_or(' ');
    let tone = tone_of(text);
    let style = if not_wired {
        Style::default().fg(DIM).add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(tone)
    };
    // le détail — après « — » ou dans une parenthèse finale — s'estompe
    let split = if first == '(' {
        None
    } else {
        text.find(" — ").or_else(|| text.rfind(" (").filter(|_| text.ends_with(')')))
    };
    match split {
        Some(at) => Line::from(vec![
            Span::styled(text[..at].to_string(), style),
            Span::styled(text[at..].to_string(), Style::default().fg(DIM)),
        ]),
        None => Line::from(Span::styled(text.to_string(), style)),
    }
}

/// « m:ss », comme un lecteur l'écrit.
fn clock(ms: u32) -> String {
    let seconds = ms / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Une ligne à deux bouts : la gauche, puis la droite au bord. Faute de
/// justification en cellules, l'écart est calculé depuis la largeur.
fn justified(left: Vec<Span<'static>>, right: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let count = |spans: &[Span]| spans.iter().map(|s| s.content.chars().count()).sum::<usize>();
    let gap = width.saturating_sub(count(&left) + count(&right)).max(2);
    let mut spans = left;
    spans.push(Span::raw(" ".repeat(gap)));
    spans.extend(right);
    Line::from(spans)
}

/// Coupe un texte à `max` caractères, avec une ellipse : une liste se coupe.
fn fit(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// Où en est un morceau de la liste de lecture.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Played,
    Playing { n: usize, paused: bool },
    Ahead { n: usize },
}

/// La ligne d'un morceau, sur la grille de la maquette 3a : le numéro et
/// la flèche, le morceau, la raison de la branche qu'il ouvre en gris — en
/// cyan quand elle vient des vecteurs —, et à droite ce que l'écoute en
/// sait. Ce qui a sonné n'est pas numéroté et s'estompe ; ce qui sonne
/// porte « ▶ » ; ce qui vient est compté à partir de lui.
fn track_row(stop: &Stop, slot: Slot, opening: Option<&str>, note: &str, width: usize) -> Line<'static> {
    let played = slot == Slot::Played;
    // le numéro en gris, la flèche seule en couleur (maquette 2b)
    let prefix: Vec<Span> = match slot {
        Slot::Played => vec![Span::raw("      ")],
        Slot::Playing { n, paused } => vec![
            Span::styled(format!("{n:>2} "), Style::default().fg(DIM)),
            Span::styled(
                format!("{} ", if paused { "⏸" } else { "▶" }),
                Style::default().fg(PLAYING).add_modifier(Modifier::BOLD),
            ),
        ],
        // un encore se reconnaît à son « ↻ » : c'est ainsi que le geste se
        // vérifie, sans notification (Joel, 07/09/2026)
        Slot::Ahead { n } if stop.encore => vec![
            Span::styled(format!("{n:>2} "), Style::default().fg(DIM)),
            Span::styled("↻  ", Style::default().fg(EDIT).add_modifier(Modifier::BOLD)),
        ],
        Slot::Ahead { n } => vec![
            Span::styled(format!("{n:>2} "), Style::default().fg(DIM)),
            Span::styled("→  ", Style::default().fg(BRANCH).add_modifier(Modifier::BOLD)),
        ],
    };
    let body_text = format!("{} {} — {}", stop.source.mark(), stop.title, stop.artist);
    let body: Vec<Span> = match slot {
        // inversé, comme partout où quelque chose est actif
        Slot::Playing { .. } => vec![Span::styled(
            format!(" {body_text} "),
            Style::default().fg(Color::Black).bg(PLAYING).add_modifier(Modifier::BOLD),
        )],
        _ => {
            let title = if played {
                Style::default().fg(MUTED)
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };
            vec![
                Span::styled(stop.source.mark().to_string(), Style::default().fg(role_of(stop.source))),
                Span::raw(" "),
                Span::styled(stop.title.clone(), title),
                Span::styled(" — ", Style::default().fg(DIM)),
                Span::styled(
                    stop.artist.clone(),
                    Style::default().fg(if played { MUTED } else { CATALOG }),
                ),
            ]
        }
    };
    // le préfixe fait 6 cellules, le corps inversé en prend deux de plus
    let body_width = body_text.chars().count() + usize::from(matches!(slot, Slot::Playing { .. })) * 2;
    let used = 6 + body_width + 2 + 2;
    // une note qui ne tient pas ne s'affiche pas : on ne coupe pas un mot
    let note = if used - 2 + note.chars().count() <= width { note } else { "" };
    let room = width.saturating_sub(used + note.chars().count() + 2);
    // une raison réduite à un moignon ne dit rien : sous 14 cellules, rien
    let (middle, middle_tone) = match opening {
        Some(reason) if room >= 14 => {
            let tone = if reason.starts_with("proche du centre") { VECTOR } else { MUTED };
            (fit(reason, room), tone)
        }
        _ => (String::new(), MUTED),
    };
    let pad = width
        .saturating_sub(used - 2 + middle.chars().count() + note.chars().count())
        .max(2);
    let mut spans = prefix;
    spans.extend(body);
    spans.push(Span::styled("  ".to_string(), Style::default()));
    spans.push(Span::styled(middle, Style::default().fg(if played { DIM } else { middle_tone })));
    spans.push(Span::styled(" ".repeat(pad), Style::default()));
    spans.push(Span::styled(note.to_string(), Style::default().fg(DIM)));
    Line::from(spans)
}

fn render(frame: &mut ratatui::Frame, view: &View) {
    let area = frame.area();
    // maquette 2b : une ligne d'en-tête, le bloc de la graine, le corps qui
    // prend le reste, puis un pied de trois lignes et l'invite — le bas ne
    // bouge jamais, quoi que forkstify dise
    // plus de ligne de statut sous « à suivre » : tout se dit en toast, et
    // les touches suivent directement (Joel, 08/09/2026)
    let [head, seed_block, body, now, bar, next, prompt] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let full = area.width as usize;

    // l'axe à gauche, les branches à droite sur toute la hauteur (1a), en
    // 60/40 comme l'accueil — leur place est réservée, elles ne recouvrent
    // rien
    let (axis, panel_column) = if view.panel && body.width >= SPLIT_MIN {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(LEFT_SHARE),
            Constraint::Percentage(100 - LEFT_SHARE),
        ])
        .areas(body);
        (left, Some(right))
    } else {
        (body, None)
    };

    // — l'en-tête, sur une ligne : la commande à gauche, l'état à droite
    let tracks = view.past.len() + usize::from(view.current.is_some()) + view.queue.len();
    let gauge: String = (0..5).map(|i| if i < view.comfort { '█' } else { '░' }).collect();
    let comfort_style = if view.comfort_mode {
        Style::default().fg(Color::Black).bg(VECTOR).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(VECTOR)
    };
    frame.render_widget(
        Paragraph::new(justified(
            vec![
                Span::styled("forkstify", Style::default().fg(MUTED).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ecouter {}", view.seed), Style::default().fg(DIM)),
            ],
            vec![
                Span::styled(
                    format!(
                        "segment {} · {} morceau{} · {} à venir · ",
                        view.segment,
                        tracks,
                        if tracks > 1 { "x" } else { "" },
                        view.queue.len()
                    ),
                    Style::default().fg(DIM),
                ),
                Span::styled(format!("confort {} {gauge} {}", view.comfort, view.comfort_word), comfort_style),
            ],
            full,
        )),
        head,
    );

    // — la graine, en bloc : c'est d'elle que tout descend
    let forks = match view.forks {
        0 => "aucun embranchement encore".to_string(),
        1 => "1 embranchement depuis".to_string(),
        n => format!("{n} embranchements depuis"),
    };
    let traversed = view.path.len();
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("── ", Style::default().fg(DIM)),
                Span::styled("graine", Style::default().fg(MUTED)),
                Span::styled(" ──────────────", Style::default().fg(DIM)),
            ]),
            Line::from(vec![
                Span::styled(
                    view.seed_name.to_string(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  [catalogue]  ", Style::default().fg(CATALOG)),
                Span::styled(view.seed_facts.to_string(), Style::default().fg(MUTED)),
                Span::styled(format!("  {}", view.seed_last), Style::default().fg(DIM)),
            ]),
            Line::from(Span::styled(
                format!(
                    "{forks} — {traversed} artiste{} traversé{}",
                    if traversed > 1 { "s" } else { "" },
                    if traversed > 1 { "s" } else { "" }
                ),
                Style::default().fg(DIM),
            )),
        ]),
        seed_block,
    );

    // — l'axe : ce qui a sonné, ce qui sonne, ce qui suit. Il s'affiche
    // verticalement, donc c'est verticalement qu'on s'y déplace.
    let mut lines: Vec<Line> = Vec::new();
    let mut index = 0usize;
    // la sélection surligne, elle ne joue pas : c'est entrée qui joue
    fn push(line: Line<'static>, i: usize, selection: Option<usize>, lines: &mut Vec<Line<'static>>) {
        if selection == Some(i) {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            lines.push(Line::from(Span::styled(
                format!("{text} "),
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(line);
        }
    }
    // la liste de lecture sur la grille de la maquette 3a (Joel, 07/09/2026) :
    // ce qui sonne et ce qui vient sont numérotés et séparés d'un filet, la
    // raison de chaque branche se lit en gris à côté du morceau qui l'ouvre,
    // et ce que l'écoute sait du morceau se lit à droite. Tout ce qui a été
    // joué reste à l'écran, compact et estompé : c'est la playlist en train
    // de se faire, pas un historique à oublier (Joel, 06/09/2026).
    let width = axis.width.saturating_sub(1) as usize;
    let mut playing_line = 0usize;
    let past_len = view.past.len();
    let current_at = view.current.map(|_| past_len);
    let all: Vec<&Stop> = view
        .past
        .iter()
        .chain(view.current)
        .chain(view.queue.iter())
        .collect();
    let seed_reason = format!("graine : {}", view.seed);
    let mut numbered = 0usize;
    for (stop_index, stop) in all.iter().enumerate() {
        let slot = match current_at {
            Some(c) if stop_index == c => {
                numbered += 1;
                Slot::Playing { n: numbered, paused: view.paused }
            }
            Some(c) if stop_index > c => {
                numbered += 1;
                Slot::Ahead { n: numbered }
            }
            _ => Slot::Played,
        };
        if slot != Slot::Played {
            if matches!(slot, Slot::Playing { .. }) {
                playing_line = lines.len();
            }
        }
        let opening = match &stop.head {
            Some(Head { reason, .. }) => Some(reason.as_str()),
            None if stop_index == 0 => Some(seed_reason.as_str()),
            None => None,
        };
        let note = view.notes.get(stop_index).map(String::as_str).unwrap_or("");
        push(track_row(stop, slot, opening, note, width), index, view.selection, &mut lines);
        index += 1;
    }
    if !all.is_empty() {
        // l'horizon : rien de tiré au-delà, la suite est dans la colonne
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            fit(
                &format!(
                    "{:>2}    horizon  rien de tiré au-delà — 1-3 pour ajouter une branche",
                    numbered + 1
                ),
                width,
            ),
            Style::default().fg(DIM),
        )));
    }
    // pas de retour à la ligne : une liste se coupe, elle ne se replie pas —
    // et on suit ce qui joue plutôt que le début de la soirée
    let height = axis.height as usize;
    let offset = playing_line
        .saturating_sub(height / 2)
        .min(lines.len().saturating_sub(height));
    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), axis);

    // — le pied (maquette 2b), partagé avec l'accueil
    render_bar(
        frame,
        [now, bar, next],
        &Bar {
            current: view.current,
            paused: view.paused,
            loading: view.loading,
            progress: view.progress,
            position: (view.past.len() + 1, tracks),
            next: view.queue.first(),
            ahead: view.queue.len(),
        },
    );

    // — l'invite : toujours la dernière ligne, avec son curseur
    let prompt_line = Line::from(vec![
        Span::styled(view.prompt.clone(), Style::default().fg(MUTED)),
        Span::raw(" "),
        Span::styled(" ", Style::default().bg(VECTOR)),
    ]);
    frame.render_widget(Paragraph::new(prompt_line), prompt);

    // — les branches, dans leur colonne : toujours visibles
    if let Some(column) = panel_column {
        render_panel(frame, column, view);
    }

    // — la discographie prend le corps de l'écran, jamais le pied
    if let Some(screen) = view.explore {
        render_explore(frame, body, screen);
    }

    // — la recherche prend le corps, comme la discographie
    if let Some(finder) = &view.finder {
        render_finder(frame, body, finder);
    }

    // — le toast, en bas à droite du corps, par-dessus la colonne ou la
    // modale : ce qui charge, ou ce qui vient d'être dit
    if let Some(toast) = &view.toast {
        render_toast(frame, body, toast);
    }

    // — et ce qui se pose par-dessus tout : un bloc demandé (le leader, « ? »)
    if let Some((title, body)) = view.overlay {
        render_block(frame, area, title, body);
    }
}

/// La modale de recherche : le même filet léger que la discographie, une
/// seule ligne de frappe « ⟩ », la règle qui coupe la saisie des résultats
/// et porte le décompte, puis les deux groupes — bleu écrit par un humain,
/// cyan deviné.
fn render_finder(frame: &mut ratatui::Frame, area: Rect, view: &FinderView) {
    frame.render_widget(Clear, area);
    let width = area.width as usize;
    let rule = |text: &str| Span::styled(text.to_string(), Style::default().fg(DIM));
    let mut lines: Vec<Line> = Vec::new();

    // — le titre porte la touche, comme partout
    lines.push(ruled(
        vec![
            rule("┌─ "),
            Span::styled(
                if view.insert { "insérer un titre " } else { "recherche " },
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            rule("── "),
            Span::styled(if view.insert { "ti " } else { ":search " }, Style::default().fg(MUTED)),
            Span::styled(
                if view.insert { "titres seulement" } else { "titres et artistes" },
                Style::default().fg(DIM),
            ),
        ],
        width,
    ));
    if let Some(anchor) = &view.anchor {
        lines.push(Line::from(vec![
            rule("│ "),
            Span::styled("→ ", Style::default().fg(BRANCH)),
            Span::styled(anchor.clone(), Style::default().fg(MUTED)),
        ]));
    }
    // — la seule zone de frappe
    let mut input = vec![
        rule("│ "),
        Span::styled("⟩ ", Style::default().fg(BRANCH).add_modifier(Modifier::BOLD)),
        Span::styled(view.query.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default().bg(Color::White)),
    ];
    if view.query.is_empty() {
        input.push(Span::styled("   un titre, un artiste, ou un slug de fiche", Style::default().fg(DIM)));
    }
    lines.push(Line::from(input));
    // — la règle qui coupe, et compte
    let (cat, spot, asking) = view.counts;
    let mut counts = vec![
        rule("│ "),
        rule(&"─".repeat(width.saturating_sub(40).max(8))),
        Span::styled(format!(" catalogue {cat}"), Style::default().fg(CATALOG)),
        Span::styled(" · ", Style::default().fg(DIM)),
    ];
    counts.push(match (view.only_catalogue, asking, spot) {
        (true, _, _) => Span::styled("spotify masqué (tab)".to_string(), Style::default().fg(DIM)),
        (_, true, _) => Span::styled("spotify …".to_string(), Style::default().fg(VECTOR)),
        (_, _, Some(n)) => Span::styled(format!("spotify {n}"), Style::default().fg(VECTOR)),
        _ => Span::styled("spotify 0".to_string(), Style::default().fg(DIM)),
    });
    counts.push(rule(" ──"));
    lines.push(Line::from(counts));

    // — les résultats, groupés
    let cursor_style = Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD);
    let mut row_index = 0usize;
    for line in &view.lines {
        match line {
            FinderLine::Header { catalogue, text } => {
                let tone = if *catalogue { CATALOG } else { VECTOR };
                lines.push(Line::from(vec![
                    rule("│ "),
                    Span::styled("── ", Style::default().fg(tone)),
                    Span::styled(text.clone(), Style::default().fg(tone).add_modifier(Modifier::BOLD)),
                ]));
            }
            FinderLine::Info(text) => {
                lines.push(Line::from(vec![rule("│ "), Span::styled(text.clone(), Style::default().fg(DIM))]));
            }
            FinderLine::Row { catalogue, mark, title, artist, note } => {
                let n = row_index + 1;
                let source = if *catalogue { "[catalogue]" } else { "[spotify]" };
                let tone = if *catalogue { CATALOG } else { VECTOR };
                let text = format!(
                    "{n:>2} {source:<11} {mark} {:<30} {:<22} {}",
                    fit(title, 30),
                    fit(artist, 22),
                    note
                );
                if row_index == view.cursor {
                    lines.push(Line::from(vec![rule("│ "), Span::styled(fit(&text, width.saturating_sub(3)), cursor_style)]));
                } else {
                    lines.push(Line::from(vec![
                        rule("│ "),
                        Span::styled(format!("{n:>2} "), Style::default().fg(DIM)),
                        Span::styled(format!("{source:<11} "), Style::default().fg(tone)),
                        Span::styled(format!("{mark} "), Style::default().fg(if *mark == '~' { VECTOR } else { PLAYING })),
                        Span::styled(format!("{:<30} ", fit(title, 30)), Style::default().fg(Color::White)),
                        Span::styled(format!("{:<22} ", fit(artist, 22)), Style::default().fg(if *catalogue { CATALOG } else { MUTED })),
                        Span::styled(note.clone(), Style::default().fg(DIM)),
                    ]));
                }
                row_index += 1;
            }
        }
    }

    // — les touches, et la fermeture
    lines.push(Line::from(rule("│")));
    lines.push(Line::from(vec![
        rule("│ "),
        Span::styled("entrée ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            if view.insert { "insérer ici   " } else { "brancher là, ou jouer le titre   " },
            Style::default().fg(MUTED),
        ),
        Span::styled("↑↓ ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("choisir   ", Style::default().fg(MUTED)),
        Span::styled("tab ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("catalogue seul", Style::default().fg(MUTED)),
    ]));
    lines.push(ruled(
        vec![
            rule("└─ "),
            Span::styled("échap ", Style::default().fg(MUTED)),
            Span::styled(
                if view.insert { "ferme sans insérer — la file est inchangée" } else { "ferme la recherche — la lecture n'a pas cessé" },
                Style::default().fg(DIM),
            ),
        ],
        width,
    ));
    frame.render_widget(Paragraph::new(lines), area);
}

/// Le cartouche d'un toast : un cadre de la couleur du message, le texte
/// en gras dedans, posé en bas à droite du corps au-dessus des touches de
/// la colonne. Collant tant que ça charge, sinon quatre secondes.
fn render_toast(frame: &mut ratatui::Frame, body: Rect, toast: &Toast) {
    let width = 48.min(body.width.saturating_sub(2));
    if width < 12 || body.height < 6 {
        return;
    }
    let lines = wrap_words(&toast.text, width.saturating_sub(4) as usize);
    let height = (lines.len() as u16 + 2).min(body.height.saturating_sub(3));
    let rect = Rect {
        x: body.x + body.width - width - 1,
        y: body.y + body.height.saturating_sub(height + 3),
        width,
        height,
    };
    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(toast.tone))
        .title(Span::styled(
            if toast.sticky { " ⏳ en cours " } else { " " },
            Style::default().fg(toast.tone).add_modifier(Modifier::BOLD),
        ));
    let text: Vec<Line> = lines
        .into_iter()
        .map(|l| Line::from(Span::styled(format!(" {l}"), Style::default().fg(toast.tone).add_modifier(Modifier::BOLD))))
        .collect();
    frame.render_widget(Paragraph::new(text).block(block), rect);
}

/// Le pied de lecture (maquette 2b) : ce qui sonne et sa provenance, sa
/// progression, puis ce qui suit et à combien de morceaux se trouve
/// l'embranchement — ce que la liste ne dit plus quand elle a défilé —, et
/// la dernière chose dite.
fn render_bar(frame: &mut ratatui::Frame, [now, bar, next]: [Rect; 3], view: &Bar) {
    let full = now.width as usize;
    let (rank, tracks) = view.position;
    let now_line = match view.current {
        Some(stop) => justified(
            vec![
                Span::styled(
                    format!("{} ", if view.paused { "⏸" } else { "▶" }),
                    Style::default().fg(PLAYING),
                ),
                Span::styled(
                    stop.title.clone(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" — ", Style::default().fg(DIM)),
                Span::styled(stop.artist.clone(), Style::default().fg(CATALOG)),
                Span::styled(format!("  ({rank} / {tracks})"), Style::default().fg(DIM)),
                Span::styled(
                    if view.loading { " · chargement…" } else { "" }.to_string(),
                    Style::default().fg(VECTOR),
                ),
            ],
            {
                let mut right = vec![
                    Span::styled(
                        format!("{} ", stop.source.mark()),
                        Style::default().fg(role_of(stop.source)),
                    ),
                    Span::styled(stop.source.word().to_string(), Style::default().fg(MUTED)),
                ];
                // les temps, dès que librespot les a dits
                if let Some((position, duration)) = view.progress {
                    right.push(Span::styled(" │ ", Style::default().fg(DIM)));
                    right.push(Span::styled(clock(position), Style::default().fg(MUTED)));
                    if duration > 0 {
                        right.push(Span::styled(
                            format!(" / {} -{}", clock(duration), clock(duration.saturating_sub(position))),
                            Style::default().fg(DIM),
                        ));
                    }
                }
                right
            },
            full,
        ),
        None => Line::from(Span::styled("⏹ rien ne sonne", Style::default().fg(MUTED))),
    };
    frame.render_widget(Paragraph::new(now_line), now);
    // la progression, pleine largeur, comme le module media de waybar
    let bar_line = match view.progress {
        Some((position, duration)) if duration > 0 => {
            let filled = (position as u64 * full as u64 / duration as u64) as usize;
            Line::from(vec![
                Span::styled("█".repeat(filled.min(full)), Style::default().fg(VECTOR)),
                Span::styled("░".repeat(full.saturating_sub(filled)), Style::default().fg(DIM)),
            ])
        }
        _ => Line::from(Span::styled("░".repeat(full), Style::default().fg(DIM))),
    };
    frame.render_widget(Paragraph::new(bar_line), bar);
    let next_line = justified(
        match view.next {
            Some(stop) => vec![
                Span::styled("à suivre  ", Style::default().fg(DIM)),
                Span::styled(stop.title.clone(), Style::default().fg(MUTED)),
                Span::styled(" — ", Style::default().fg(DIM)),
                Span::styled(stop.artist.clone(), Style::default().fg(CATALOG)),
            ],
            None => vec![Span::styled("à suivre  (rien de tiré)", Style::default().fg(DIM))],
        },
        vec![
            Span::styled("→ ", Style::default().fg(BRANCH)),
            Span::styled(
                match view.ahead {
                    0 => "embranchement à la fin du morceau".to_string(),
                    1 => "embranchement dans 1 morceau".to_string(),
                    n => format!("embranchement dans {n} morceaux"),
                },
                Style::default().fg(MUTED),
            ),
        ],
        full,
    );
    frame.render_widget(Paragraph::new(next_line), next);
}

/// Coupe un texte en lignes d'au plus `width` caractères, sur les espaces.
/// Une raison se replie (c'est de la prose), un morceau se coupe (c'est une
/// liste) : c'est pourquoi le volet ne confie pas le repli à ratatui.
fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let width = width.max(8);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let fits = line.chars().count() + 1 + word.chars().count() <= width;
        if line.is_empty() {
            line.push_str(word);
        } else if fits {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Ce que la jauge de proximité affiche sous une branche : le mot du lien
/// pour le graphe, le cosinus pour l'espace vectoriel — deux natures, deux
/// couleurs, comme partout ailleurs.
fn proximity_of(branch: &Branch) -> (Color, String, Option<String>) {
    let cells: String = (0..5)
        .map(|i| if (i as f32) < branch.weight.round() { '█' } else { '░' })
        .collect();
    if branch.reason.starts_with("proche du centre") {
        // la branche aventureuse : le moteur a mis le cosinus sur l'échelle 1–5
        let cosine = branch.weight / 5.0;
        (VECTOR, cells, Some(format!("{cosine:.2}")))
    } else {
        // le graphe : le lien typé est le premier mot de la raison
        let kind = branch
            .reason
            .split(" — ")
            .next()
            .and_then(|head| head.split(" · ").next())
            .unwrap_or("")
            .to_string();
        (CATALOG, cells, Some(kind))
    }
}

/// La colonne des branches (1a) : toujours là, sur toute la hauteur, chaque
/// branche dépliée avec ses morceaux — ils sont déjà tirés, autant les
/// montrer pour qu'on choisisse en connaissance de cause (Joel, 07/09/2026).
/// Un filet à gauche la sépare de l'axe ; c'est la seule règle qu'elle trace.
fn render_panel(frame: &mut ratatui::Frame, column: Rect, view: &View) {
    // le filet, puis une cellule de marge : le contenu commence à x + 2
    let rule: Vec<Line> = (0..column.height)
        .map(|_| Line::from(Span::styled("│", Style::default().fg(DIM))))
        .collect();
    frame.render_widget(Paragraph::new(rule), Rect { width: 1, ..column });
    let inner = Rect {
        x: column.x + 2,
        y: column.y,
        width: column.width.saturating_sub(2),
        height: column.height,
    };
    let width = inner.width as usize;

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("── ", Style::default().fg(BRANCH)),
        Span::styled("branches", Style::default().fg(BRANCH).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}", view.branches.len()), Style::default().fg(DIM)),
        // les creux se comptent à part : ils ne sonnent pas encore
        Span::styled(
            if view.missing.is_empty() {
                String::new()
            } else {
                format!(" · {} ○", view.missing.len())
            },
            Style::default().fg(DIM),
        ),
    ]));
    lines.push(Line::from(""));
    if view.branches.is_empty() && view.missing.is_empty() {
        lines.push(Line::from(Span::styled(
            "cul-de-sac — « fu » pour revenir",
            Style::default().fg(MUTED),
        )));
    }
    for (i, branch) in view.branches.iter().enumerate() {
        // « 1  label » — 2 cellules d'indentation, comme le PoC l'imprime
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}  ", i + 1),
                Style::default().fg(BRANCH).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                branch.label.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));
        let (tone, cells, word) = proximity_of(branch);
        // la raison, repliée à 5 cellules ; quand elle n'est que le mot du
        // lien (« rester dans l'univers »), la jauge ne la redit pas
        let bare = word.as_deref() == Some(branch.reason.as_str());
        let word = if bare { None } else { word };
        if !branch.reason.is_empty() {
            for piece in wrap_words(&branch.reason, width.saturating_sub(5)) {
                lines.push(Line::from(Span::styled(
                    format!("     {piece}"),
                    Style::default().fg(MUTED),
                )));
            }
        }
        // les morceaux, tels qu'ils sonneront : on choisit ce qu'on entendra
        for stop in &branch.stops {
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(
                    stop.source.mark().to_string(),
                    Style::default().fg(role_of(stop.source)),
                ),
                Span::raw(" "),
                Span::styled(stop.title.clone(), Style::default().fg(Color::White)),
                Span::styled(" — ", Style::default().fg(DIM)),
                Span::styled(stop.artist.clone(), Style::default().fg(CATALOG)),
            ]));
        }
        let mut gauge = vec![
            Span::styled("     ", Style::default()),
            Span::styled(cells, Style::default().fg(tone)),
        ];
        // la raison est grise, son mot sous la jauge aussi (Joel, 07/09/2026) :
        // seules les cellules disent la nature du lien
        if let Some(word) = word {
            gauge.push(Span::styled(format!(" {word}"), Style::default().fg(MUTED)));
        }
        lines.push(Line::from(gauge));
        lines.push(Line::from(""));
    }

    // les creux : un lien du catalogue vers une fiche qui n'existe pas
    // encore. Ils se numérotent à la suite, en gris, et le cercle vide dit
    // qu'il faudra les générer avant de les marcher (0016).
    for (i, missing) in view.missing.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}  ", view.branches.len() + i + 1),
                Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
            ),
            Span::styled(missing.name.clone(), Style::default().fg(MUTED)),
        ]));
        for piece in wrap_words(&missing.why, width.saturating_sub(5)) {
            lines.push(Line::from(Span::styled(
                format!("     {piece}"),
                Style::default().fg(DIM),
            )));
        }
        // la même jauge que les branches, en gris : la proximité du lien est
        // connue, c'est la fiche qui manque
        let cells: String =
            (0..5).map(|i| if i < missing.proximity { '█' } else { '░' }).collect();
        lines.push(Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(cells, Style::default().fg(DIM)),
            Span::styled(
                if missing.pending {
                    format!(" {} · … génération", missing.kind)
                } else {
                    format!(" {} · ○ fiche à générer", missing.kind)
                },
                Style::default().fg(if missing.pending { BRANCH } else { MUTED }),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // les touches, au pied de la colonne — deux lignes qui ne bougent pas
    let hints = [
        Line::from(vec![
            Span::styled("1-3", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" prendre  ", Style::default().fg(MUTED)),
            Span::styled("fr", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" reproposer", Style::default().fg(MUTED)),
        ]),
        Line::from(vec![
            Span::styled("fn1", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" sans attendre la fin", Style::default().fg(DIM)),
        ]),
    ];
    let hint_height = hints.len() as u16;
    if inner.height > hint_height + 2 {
        let body = Rect { height: inner.height - hint_height - 1, ..inner };
        frame.render_widget(Paragraph::new(lines), body);
        let foot = Rect {
            y: inner.y + inner.height - hint_height,
            height: hint_height,
            ..inner
        };
        frame.render_widget(Paragraph::new(hints.to_vec()), foot);
    } else {
        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// Une ligne de l'accueil. L'accueil décide **quoi** dire, la TUI **comment**
/// — c'est la même séparation qu'entre le moteur et le son.
pub enum Row {
    Rule(String),
    Text(String),
    Dim(String),
    /// Une porte d'entrée numérotée : la numérotation court à travers les
    /// blocs, si bien que choisir une graine est le geste qui choisit une
    /// branche. `artist` n'est rempli que si la graine est un **morceau** —
    /// le titre passe devant, l'artiste derrière, comme partout ailleurs.
    Entry {
        n: usize,
        label: String,
        artist: Option<String>,
        reason: String,
        tracks: Vec<(String, String)>,
    },
    /// Une touche et ce qu'elle fait ; `wired` faux la montre estompée,
    /// jamais comme si elle marchait.
    Key { key: String, what: String, note: String, wired: bool },
}

pub struct HomeView<'a> {
    /// Le nom à gauche, l'état des autorisations à droite, sur **une seule
    /// ligne** (Joel, 06/09/2026).
    pub status: Vec<(String, bool)>,
    pub census: String,
    pub rows: &'a [Row],
    pub prompt: String,
    pub comfort: u8,
    pub comfort_word: &'a str,
    /// La colonne de droite. Elle ne propose rien, elle liste.
    pub collection: Option<Collection<'a>>,
    /// Le pied de lecture, quand une session joue sous l'accueil.
    pub bar: Option<Bar<'a>>,
    /// La modale de recherche, quand elle est ouverte : `:search` s'ouvre
    /// aussi de l'accueil (Joel, 09/09/2026).
    pub finder: Option<FinderView>,
}

impl Tui {
    pub fn draw_home(&mut self, view: &HomeView) -> std::io::Result<()> {
        self.terminal.draw(|frame| render_home(frame, view))?;
        Ok(())
    }
}

fn render_home(frame: &mut ratatui::Frame, view: &HomeView) {
    let area = frame.area();
    // le pied de lecture prend ses quatre lignes quand une session joue
    let foot = if view.bar.is_some() { 3 } else { 0 };
    let [head, whole, foot_area, prompt] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(foot),
        Constraint::Length(1),
    ])
    .areas(area);
    if let Some(bar) = &view.bar {
        let [now, progress, next] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(foot_area);
        render_bar(frame, [now, progress, next], bar);
    }

    // à gauche ce que forkstify propose, à droite ce qu'il possède. Le
    // partage est proportionnel (Joel, 06/09/2026) : la gauche porte des
    // raisons et des morceaux, la droite une liste — d'où 60/40 plutôt que
    // moitié-moitié. Sous `SPLIT_MIN`, la liste s'efface.
    let (body, collection) = match (&view.collection, whole.width >= SPLIT_MIN) {
        (Some(_), true) => {
            let [left, right] = Layout::horizontal([
                Constraint::Percentage(LEFT_SHARE),
                Constraint::Percentage(100 - LEFT_SHARE),
            ])
            .areas(whole);
            (left, Some(right))
        }
        _ => (whole, None),
    };

    // le mot-marque à gauche, l'état à droite, sur la même ligne : le blanc
    // entre les deux est calculé, faute de justification en cellules
    let status_width: usize = view
        .status
        .iter()
        .map(|(text, _)| text.chars().count())
        .sum::<usize>()
        + view.status.len().saturating_sub(1) * 3;
    let gap = (head.width as usize)
        .saturating_sub("forkstify".len() + status_width)
        .max(2);
    let mut title = vec![
        Span::styled(
            "forkstify",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
    ];
    for (i, (text, ok)) in view.status.iter().enumerate() {
        if i > 0 {
            title.push(Span::styled(" · ", Style::default().fg(DIM)));
        }
        title.push(Span::styled(
            text.clone(),
            Style::default().fg(if *ok { PLAYING } else { Color::Red }),
        ));
    }
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(title),
            Line::from(Span::styled(view.census.clone(), Style::default().fg(DIM))),
        ]),
        head,
    );

    let mut lines: Vec<Line> = Vec::new();
    for row in view.rows {
        match row {
            // le liseré court jusqu'au bout de la mesure : c'est lui qui
            // sépare les blocs, puisqu'il n'y a pas de cartes
            Row::Rule(title) => {
                let width = (body.width as usize).min(66);
                let filled = 3 + title.chars().count() + 1;
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("── ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        title.clone(),
                        Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", "─".repeat(width.saturating_sub(filled))),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
            Row::Text(text) => lines.push(Line::from(Span::styled(
                text.clone(),
                Style::default().fg(Color::Reset),
            ))),
            Row::Dim(text) => {
                lines.push(Line::from(Span::styled(text.clone(), Style::default().fg(DIM))))
            }
            Row::Entry { n, label, artist, reason, tracks } => {
                let mut head = vec![
                    Span::styled(
                        format!("  {n}  "),
                        Style::default().fg(BRANCH).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        label.clone(),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                ];
                // la graine est un morceau : le titre devant, l'artiste
                // derrière, comme sur toutes les lignes de forkstify
                if let Some(name) = artist {
                    head.push(Span::styled(" — ", Style::default().fg(DIM)));
                    head.push(Span::styled(name.clone(), Style::default().fg(CATALOG)));
                }
                lines.push(Line::from(head));
                lines.push(Line::from(Span::styled(
                    format!("     {reason}"),
                    Style::default().fg(MUTED),
                )));
                for (title, name) in tracks {
                    lines.push(Line::from(vec![
                        Span::styled("     ♪ ", Style::default().fg(PLAYING)),
                        Span::styled(title.clone(), Style::default().fg(Color::Reset)),
                        Span::styled(" — ", Style::default().fg(DIM)),
                        Span::styled(name.clone(), Style::default().fg(CATALOG)),
                    ]));
                }
            }
            Row::Key { key, what, note, wired } => {
                let fade = if *wired { BRANCH } else { DIM };
                let mut spans = vec![
                    Span::styled(
                        if *wired { "  " } else { "  · " }.to_string(),
                        Style::default().fg(DIM),
                    ),
                    Span::styled(
                        format!("{key}  "),
                        Style::default().fg(fade).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        what.clone(),
                        Style::default().fg(if *wired { MUTED } else { DIM }),
                    ),
                ];
                if !note.is_empty() {
                    spans.push(Span::styled(
                        format!(" — {note}"),
                        Style::default().fg(DIM),
                    ));
                }
                lines.push(Line::from(spans));
            }
        }
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), body);

    if let (Some(area), Some(list)) = (collection, view.collection.as_ref()) {
        render_collection(frame, area, list);
    }

    // — la recherche prend tout le corps de l'accueil, collection comprise :
    // c'est elle qu'on regarde tant qu'elle est ouverte
    if let Some(finder) = &view.finder {
        render_finder(frame, whole, finder);
    }

    let gauge: String = (0..5).map(|i| if i < view.comfort { '█' } else { '░' }).collect();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(view.prompt.clone(), Style::default().fg(MUTED)),
            Span::raw("  "),
            Span::styled(gauge, Style::default().fg(VECTOR)),
            Span::styled(
                format!(" {} — {}", view.comfort, view.comfort_word),
                Style::default().fg(DIM),
            ),
        ])),
        prompt,
    );
}

/// Un bloc posé sur l'écran — le menu du leader, « ? ». Il remplace le
/// journal qui s'allongeait vers le bas : ce qui est long se montre, ce qui
/// est court se dit.
fn render_block(frame: &mut ratatui::Frame, area: Rect, title: &str, body: &[String]) {
    let width = 64.min(area.width.saturating_sub(4));
    let height = (body.len() as u16 + 2).min(area.height.saturating_sub(2));
    let rect = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(height + 2),
        width,
        height,
    };
    frame.render_widget(Clear, rect);
    let lines: Vec<Line> = body
        .iter()
        .map(|text| Line::from(Span::styled(text.clone(), Style::default().fg(MUTED))))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), rect);
}

impl Tui {
    /// Un écran d'attente : ce que forkstify est en train de faire, pendant
    /// qu'il le fait. Rien ne doit s'imprimer hors de la TUI — l'écran
    /// alterné est à elle, et un `println!` y laisse des restes qu'elle ne
    /// sait pas effacer.
    pub fn splash(&mut self, steps: &[(String, bool)]) -> std::io::Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let mut lines = vec![
                Line::from(Span::styled(
                    "forkstify",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            for (text, done) in steps {
                lines.push(Line::from(vec![
                    Span::styled(
                        if *done { "  ✓ " } else { "  · " }.to_string(),
                        Style::default().fg(if *done { PLAYING } else { DIM }),
                    ),
                    Span::styled(
                        text.clone(),
                        Style::default().fg(if *done { MUTED } else { DIM }),
                    ),
                ]));
            }
            frame.render_widget(Paragraph::new(lines), area);
        })?;
        Ok(())
    }

    /// Repartir d'un écran vide. À appeler quand on change de vue : ratatui
    /// ne redessine que ce qu'il croit avoir changé.
    pub fn clear(&mut self) {
        let _ = self.terminal.clear();
    }
}

/// Une ligne de la collection : la jauge de familiarité, le nom, ce qu'on en
/// sait, et depuis quand il n'a pas sonné.
#[derive(Clone)]
pub struct CollectionRow {
    pub familiarity: u8,
    /// Depuis combien de jours il n'a pas sonné — sert au tri, pas à
    /// l'affichage, qui montre `age`.
    pub days: Option<i64>,
    pub name: String,
    /// Vrai si l'artiste a une fiche — c'est ce qui décide s'il peut servir
    /// de graine, puisque les branches viennent de la fiche.
    pub carded: bool,
    pub age: String,
    /// Une écoute ancienne se signale : c'est un délaissé.
    pub neglected: bool,
}

pub struct Collection<'a> {
    pub rows: &'a [CollectionRow],
    pub total: usize,
    pub carded: usize,
    pub cursor: Option<usize>,
    pub sort: &'a str,
}

/// La colonne de droite : elle **ne propose rien, elle liste**. C'est la
/// contrepartie des portes d'entrée — pour qui veut choisir lui-même.
fn render_collection(frame: &mut ratatui::Frame, area: Rect, view: &Collection) {
    let [head, body, foot] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("── ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "la collection",
                    Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} · {} avec fiche", view.total, view.carded),
                    Style::default().fg(DIM),
                ),
            ]),
            Line::from(vec![
                Span::styled("trié par ", Style::default().fg(DIM)),
                Span::styled(view.sort.to_string(), Style::default().fg(CATALOG)),
                Span::styled("   s pour changer", Style::default().fg(DIM)),
            ]),
        ]),
        head,
    );

    // on suit le curseur plutôt que le début de la liste
    let height = body.height as usize;
    let cursor = view.cursor.unwrap_or(0);
    let offset = cursor.saturating_sub(height / 2).min(view.rows.len().saturating_sub(height));
    let lines: Vec<Line> = view
        .rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, row)| {
            let gauge: String = (0..5)
                .map(|n| if n < row.familiarity { '█' } else { '░' })
                .collect();
            let name_style = if row.carded {
                Style::default().fg(Color::Reset)
            } else {
                Style::default().fg(MUTED)
            };
            let spans = vec![
                Span::styled(gauge, Style::default().fg(CATALOG)),
                Span::raw(" "),
                Span::styled(row.name.clone(), name_style),
                Span::styled(
                    format!("  {}", row.age),
                    Style::default().fg(if row.neglected { DOOR } else { DIM }),
                ),
            ];
            if view.cursor == Some(i) {
                let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
                Line::from(Span::styled(
                    text,
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(spans)
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), body);

    let rest = view.rows.len().saturating_sub(offset + height);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                if rest > 0 { format!("── {rest} de plus") } else { "──".into() },
                Style::default().fg(DIM),
            )),
            Line::from(Span::styled(
                "↑↓ parcourir · gg G les bouts · entrée démarrer · s trier",
                Style::default().fg(DIM),
            )),
        ]),
        foot,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn stop(title: &str, artist: &str) -> Stop {
        Stop {
            slug: artist.to_lowercase().replace(' ', "-"),
            artist: artist.to_string(),
            title: title.to_string(),
            source: Source::Top,
            head: None,
            encore: false,
        }
    }

    fn branches() -> Vec<Branch> {
        vec![
            Branch {
                label: "The Creatures".to_string(),
                reason: "membres en commun — Siouxsie Sioux et Budgie · tags communs : post-punk, uk"
                    .to_string(),
                artists: vec!["the-creatures".to_string()],
                stops: vec![stop("Right Now", "The Creatures"), stop("Miss the Girl", "The Creatures")],
                weight: 5.0,
            },
            Branch {
                label: "Chelsea Wolfe".to_string(),
                reason: "proche du centre de la branche (0.78) · tags communs : uk".to_string(),
                artists: vec!["chelsea-wolfe".to_string()],
                stops: vec![stop("Carrion Flowers", "Chelsea Wolfe")],
                weight: 0.78 * 5.0,
            },
        ]
    }

    fn headed(mut stop: Stop, label: &str, reason: &str) -> Stop {
        stop.head = Some(Head { label: label.to_string(), reason: reason.to_string() });
        stop
    }

    /// Screen rows as plain text, so assertions read like the screen.
    fn screen(width: u16, height: u16, branches: &[Branch]) -> Vec<String> {
        let current = stop("Cities in Dust", "Siouxsie and the Banshees");
        playlist(width, height, branches, &[], Some(&current), &[], &[])
    }

    fn missing(pending: bool) -> crate::engine::Missing {
        crate::engine::Missing {
            slug: "georges-moustaki".into(),
            name: "Georges Moustaki".into(),
            kind: "similar".into(),
            proximity: 4,
            why: "similaires · chez Jacques Brel".into(),
            pending,
        }
    }

    /// 0016 : un lien vers une fiche absente se propose au lieu d'être jeté,
    /// numéroté **à la suite** des branches — sinon les chiffres mentiraient.
    #[test]
    fn les_creux_se_numerotent_apres_les_branches() {
        let current = stop("Ne me quitte pas", "Jacques Brel");
        let rows =
            playlist(100, 38, &branches(), &[], Some(&current), &[], &[missing(false)]);
        let text = rows.join("\n");
        assert!(text.contains("2  Chelsea Wolfe"), "{text}");
        assert!(text.contains("── branches 2 · 1 ○"), "{text}");
        assert!(text.contains("3  Georges Moustaki"), "{text}");
        assert!(text.contains("similaires · chez Jacques Brel"), "{text}");
        assert!(text.contains("████░ similar · ○ fiche à générer"), "{text}");
    }

    /// Une génération dure quelques secondes : la colonne doit le dire, sinon
    /// « f3 » a l'air de n'avoir rien fait.
    #[test]
    fn un_creux_en_cours_de_generation_le_dit() {
        let current = stop("Ne me quitte pas", "Jacques Brel");
        let rows = playlist(100, 30, &[], &[], Some(&current), &[], &[missing(true)]);
        let text = rows.join("\n");
        assert!(text.contains("similar · … génération"), "{text}");
        assert!(!text.contains("○ fiche à générer"), "{text}");
    }

    fn playlist(
        width: u16,
        height: u16,
        branches: &[Branch],
        past: &[Stop],
        current: Option<&Stop>,
        queue: &[Stop],
        missing: &[crate::engine::Missing],
    ) -> Vec<String> {
        let total = past.len() + usize::from(current.is_some()) + queue.len();
        let notes: Vec<String> = (0..total)
            .map(|i| if i == 0 { "1 écoute · hier".to_string() } else { "jamais joué".to_string() })
            .collect();
        let view = View {
            path: vec!["The Cure".to_string(), "Siouxsie and the Banshees".to_string()],
            seed: "the-cure",
            seed_name: "The Cure",
            seed_facts: "fiche écrite · 41 liens · 12 tops",
            seed_last: "dernière écoute -3s",
            forks: 1,
            segment: 2,
            past,
            current,
            paused: false,
            loading: false,
            queue,
            missing,
            branches,
            panel: true,
            notes: &notes,
            selection: None,
            overlay: None,
            comfort_mode: false,
            comfort: 3,
            comfort_word: "équilibré",
            progress: current.map(|_| (154_000, 227_000)),
            toast: None,
            finder: None,
            explore: None,
            prompt: "[1-2 branche]".to_string(),
        };
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect()
    }

    /// La modale de la discographie : les albums pliés, celui du curseur
    /// ouvert, et la fournée qui attend son commit (maquette 1a).
    #[test]
    fn la_discographie_plie_les_albums_et_dit_ce_qui_attend() {
        let card: crate::catalog::Card = toml::from_str(
            "format = 1\nname = \"Cat Power\"\nmbid = \"x\"\ntops = [\"Cross Bones Style\"]\n",
        )
        .expect("fiche");
        let track = |title: &str, album: &str, year: &str, number: u32| {
            crate::discography::TailTrack {
                title: title.into(),
                uri: format!("spotify:track:{title}"),
                album: album.into(),
                released: year.into(),
                number,
                duration_ms: 218_000,
                single: false,
            }
        };
        let tail = vec![
            track("Cross Bones Style", "Moon Pix", "1998", 1),
            track("Metal Heart", "Moon Pix", "1998", 2),
            track("Sea Of Love", "The Covers Record", "2000", 1),
        ];
        let mut learned = crate::learned::Learned::blank();
        learned.played("cat-power", "Metal Heart");
        let mut screen = crate::explore::Explore::open(
            "cat-power",
            &card,
            &tail,
            &learned,
            Some("Metal Heart"),
        );
        // premier album ouvert, curseur sur son deuxième morceau ; « A »
        // promeut le titre le plus écouté de l'album hors tops
        screen.move_by(2);
        screen.top_album(1);
        assert_eq!(screen.pending.len(), 1);

        let (width, height) = (100u16, 18u16);
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render_explore(frame, frame.area(), &screen)).unwrap();
        let buffer = terminal.backend().buffer();
        let lines: Vec<String> = (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect();

        assert!(lines[0].contains("discographie") && lines[0].contains("Cat Power"));
        // les deux albums tiennent, et seul celui du curseur est déplié
        assert!(lines.iter().any(|l| l.contains("▾") && l.contains("Moon Pix")));
        assert!(lines.iter().any(|l| l.contains("▸") && l.contains("The Covers Record")));
        assert!(lines.iter().any(|l| l.contains("Metal Heart") && l.contains("▶ sonne")));
        // ce qui attend se voit, et le commit s'annonce avant d'appuyer
        assert!(lines.iter().any(|l| l.contains("en attente") && l.contains("1 édition")));
        // le sujet du commit se lit avant d'appuyer
        assert!(lines.iter().any(|l| l.contains("cards/cat-power.toml") && l.contains("+1 −0")));
        assert!(lines.iter().any(|l| l.contains("⏎ écrire")));
    }

    #[test]
    fn the_column_unfolds_every_branch_with_its_tracks() {
        let rows = screen(100, 30, &branches());
        let text = rows.join("\n");
        assert!(text.contains("── branches 2"), "{text}");
        assert!(text.contains("1  The Creatures"), "{text}");
        assert!(text.contains("♪ Right Now — The Creatures"), "{text}");
        assert!(text.contains("♪ Miss the Girl — The Creatures"), "{text}");
        assert!(text.contains("♪ Carrion Flowers — Chelsea Wolfe"), "{text}");
        // the graph says its link, the vector space its cosine
        assert!(text.contains("█████ membres en commun"), "{text}");
        assert!(text.contains("████░ 0.78"), "{text}");
        // the rule runs the whole body height (below the head and the seed
        // block, above the four-line foot), the hints sit at its foot
        let body_rows = 5..(30 - 4);
        assert!(body_rows.clone().all(|y| rows[y].contains('│')), "{text}");
        assert!(rows[25].contains("fn1 sans attendre la fin"), "{text}");
        assert!(rows[24].contains("1-3 prendre"), "{text}");
    }

    #[test]
    fn a_bare_reason_is_said_once_by_the_gauge() {
        let mut around = branches();
        around[0].reason = "rester dans l'univers du parcours".to_string();
        around[0].weight = 4.0;
        let text = screen(100, 30, &around).join("\n");
        assert_eq!(text.matches("rester dans l'univers du parcours").count(), 1, "{text}");
        assert!(text.contains("████░ \n") || text.contains("████░  "), "{text}");
    }

    #[test]
    fn the_playlist_numbers_what_sounds_and_what_comes() {
        let past = [stop("A Forest", "The Cure"), stop("Push", "The Cure")];
        let current = headed(
            stop("Cities in Dust", "Siouxsie and the Banshees"),
            "Siouxsie and the Banshees",
            "liens familiaux — Robert Smith y a joué de la guitare en 1983",
        );
        let mut israel = stop("Israel", "Siouxsie and the Banshees");
        israel.encore = true;
        let queue = [
            israel,
            headed(stop("Right Now", "The Creatures"), "The Creatures", "membres en commun"),
            headed(stop("Alison", "Slowdive"), "Slowdive", "proche du centre de la branche (0.74)"),
        ];
        let rows = playlist(160, 30, &branches(), &past, Some(&current), &queue, &[]);
        let text = rows.join("\n");
        let axis = |needle: &str| -> String {
            let row = rows.iter().find(|r| r.contains(needle)).unwrap();
            row.chars().take(95).collect::<String>().trim_end().to_string()
        };
        // le passé n'est pas numéroté, la graine se lit à côté du premier
        assert!(text.contains("      ♪ A Forest — The Cure  graine : the-cure"), "{text}");
        assert!(axis("A Forest").ends_with("1 écoute · hier"), "{}", axis("A Forest"));
        // ce qui sonne est le 1, ce qui vient compte à partir de lui
        assert!(text.contains(" 1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees   liens familiaux"), "{text}");
        assert!(axis("Cities in Dust").ends_with("jamais joué"), "{}", axis("Cities in Dust"));
        // un encore porte « ↻ » à la place de la flèche
        assert!(text.contains(" 2 ↻  ♪ Israel — Siouxsie and the Banshees"), "{text}");
        assert!(text.contains(" 3 →  ♪ Right Now — The Creatures  membres en commun"), "{text}");
        assert!(text.contains(" 4 →  ♪ Alison — Slowdive  proche du centre de la branche (0.74)"), "{text}");
        assert!(axis("Alison").ends_with("jamais joué"), "{}", axis("Alison"));
        assert!(text.contains(" 5    horizon  rien de tiré au-delà"), "{text}");
        // plus de filet (Joel, 07/09/2026) : les morceaux se suivent, seul
        // l'horizon prend un blanc
        assert!(!text.contains('╵') && !text.contains(" │ ♪"), "{text}");
        let horizon = rows.iter().position(|r| r.contains("horizon")).unwrap();
        assert!(rows[horizon - 1].chars().take(95).all(|c| c == ' '), "{text}");
        assert!(rows[horizon - 2].contains("Alison"), "{text}");
        // l'en-tête et le bloc de la graine (2b)
        assert!(rows[0].starts_with("forkstify ecouter the-cure"), "{}", rows[0]);
        assert!(rows[0].trim_end().ends_with("segment 2 · 6 morceaux · 3 à venir · confort 3 ███░░ équilibré"), "{}", rows[0]);
        assert!(rows[2].starts_with("── graine ─"), "{}", rows[2]);
        assert!(rows[3].starts_with("The Cure  [catalogue]  fiche écrite · 41 liens · 12 tops  dernière écoute -3s"), "{}", rows[3]);
        assert!(rows[4].starts_with("1 embranchement depuis — 2 artistes traversés"), "{}", rows[4]);
        // le pied : ce qui sonne et sa provenance, ce qui suit et l'embranchement
        let now = &rows[rows.len() - 4];
        assert!(now.starts_with("▶ Cities in Dust — Siouxsie and the Banshees  (3 / 6)"), "{now}");
        assert!(now.trim_end().ends_with("♪ top │ 2:34 / 3:47 -1:13"), "{now}");
        // la barre : 154 s sur 227, soit 108 cellules pleines sur 160
        let bar = &rows[rows.len() - 3];
        assert_eq!(bar.chars().filter(|c| *c == '█').count(), 108, "{bar}");
        assert_eq!(bar.chars().filter(|c| *c == '░').count(), 52, "{bar}");
        let next = &rows[rows.len() - 2];
        assert!(next.starts_with("à suivre  Israel — Siouxsie and the Banshees"), "{next}");
        assert!(next.trim_end().ends_with("→ embranchement dans 3 morceaux"), "{next}");
    }

    #[test]
    fn a_notice_is_shaped_by_its_nature() {
        let plain = |line: Line| -> String { line.spans.iter().map(|s| s.content.to_string()).collect() };
        let done = notice_line("✓ A Forest promu top — commité");
        assert_eq!(done.spans[0].style.fg, Some(PLAYING));
        assert_eq!(done.spans[1].style.fg, Some(DIM));
        assert_eq!(plain(done), "✓ A Forest promu top — commité");
        let banned = notice_line("\n⊘ The Fall — plus jamais");
        assert_eq!(banned.spans[0].style.fg, Some(DANGER));
        let hint = notice_line("(plus de tops non joués chez The Cure)");
        assert_eq!(hint.spans.len(), 1);
        assert_eq!(hint.spans[0].style.fg, Some(MUTED));
        let later = notice_line("« fw » — partir : décidé (0015), pas encore câblé.");
        assert!(later.spans[0].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(plain(notice_line("")), "");
    }

    /// L'accueil garde le pied de lecture quand une session joue en dessous
    /// (Joel, 08/09/2026) : ce qui sonne, sa barre, ce qui suit, sur les
    /// quatre lignes au-dessus de l'invite.
    #[test]
    fn the_home_keeps_the_playback_foot() {
        let current = stop("Cities in Dust", "Siouxsie and the Banshees");
        let next = stop("Israel", "Siouxsie and the Banshees");
        let view = HomeView {
            status: vec![("✓ librespot".to_string(), true)],
            census: "catalogue local".to_string(),
            rows: &[],
            prompt: "[1-3 pour démarrer · r retour à l'écoute · q quitter]".to_string(),
            comfort: 3,
            comfort_word: "équilibré",
            collection: None,
            finder: None,
            bar: Some(Bar {
                current: Some(&current),
                paused: false,
                loading: false,
                progress: Some((60_000, 240_000)),
                position: (2, 5),
                next: Some(&next),
                ahead: 3,
            }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| render_home(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..20).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        assert!(rows[16].starts_with("▶ Cities in Dust — Siouxsie and the Banshees  (2 / 5)"), "{}", rows[16]);
        assert_eq!(rows[17].chars().filter(|c| *c == '█').count(), 25, "{}", rows[17]);
        assert!(rows[18].starts_with("à suivre  Israel — Siouxsie and the Banshees"), "{}", rows[18]);
        assert!(rows[18].trim_end().ends_with("→ embranchement dans 3 morceaux"), "{}", rows[18]);
        // et les touches suivent « à suivre » sans rien entre les deux
        assert!(rows[19].starts_with("[1-3 pour démarrer · r retour à l'écoute"), "{}", rows[19]);
    }

    /// `:search` s'ouvre aussi de l'accueil (Joel, 09/09/2026) : la modale
    /// couvre le corps — la collection comprise — et laisse le pied de
    /// lecture et l'invite.
    #[test]
    fn the_home_wears_the_finder() {
        let rows_of_home = [Row::Rule("chercher".into())];
        let view = HomeView {
            status: vec![("✓ librespot".to_string(), true)],
            census: "catalogue local".to_string(),
            rows: &rows_of_home,
            prompt: "[1-3 pour démarrer · q]".to_string(),
            comfort: 3,
            comfort_word: "équilibré",
            collection: None,
            finder: Some(FinderView {
                insert: false,
                anchor: None,
                query: "siou".to_string(),
                counts: (1, None, true),
                only_catalogue: false,
                lines: vec![FinderLine::Row {
                    catalogue: true,
                    mark: '♪',
                    title: "Cities in Dust".into(),
                    artist: "Siouxsie and the Banshees".into(),
                    note: "3 écoutes".into(),
                }],
                cursor: 0,
            }),
            bar: None,
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| render_home(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..20).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        // le corps est à la modale : plus de « chercher » de l'accueil dessous
        assert!(rows[2].starts_with("┌─ recherche ── :search"), "{}", rows[2]);
        assert!(rows[3].starts_with("│ ⟩ siou"), "{}", rows[3]);
        assert!(rows.iter().any(|row| row.contains("Cities in Dust")), "{rows:?}");
        assert!(!rows.iter().any(|row| row.contains("── chercher")), "{rows:?}");
        // l'invite reste, elle, avec sa jauge de confort
        assert!(rows[19].starts_with("[1-3 pour démarrer · q]"), "{}", rows[19]);
    }

    /// La modale de recherche : la saisie, la règle qui compte, les deux
    /// groupes jamais mêlés, le curseur, et l'ancre de « ti ».
    #[test]
    fn the_finder_cuts_the_input_from_the_results() {
        let view = FinderView {
            insert: true,
            anchor: Some("l'insertion tombe en 4 — entre Sea Of Love et Cross Bones Style".to_string()),
            query: "nothing bu".to_string(),
            counts: (2, None, true),
            only_catalogue: false,
            lines: vec![
                FinderLine::Header { catalogue: true, text: "catalogue  2 résultats".to_string() },
                FinderLine::Row { catalogue: true, mark: '♪', title: "Nothing But Time".into(), artist: "Cat Power".into(), note: "3 écoutes".into() },
                FinderLine::Row { catalogue: true, mark: '♥', title: "Nothing Compares 2 U".into(), artist: "Sinéad O'Connor".into(), note: "♥ aimé · 1 écoute".into() },
                FinderLine::Info("[spotify] … interrogation".to_string()),
            ],
            cursor: 1,
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 14)).unwrap();
        terminal.draw(|frame| render_finder(frame, frame.area(), &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..14).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        assert!(rows[0].starts_with("┌─ insérer un titre ── ti titres seulement ─"), "{}", rows[0]);
        assert!(rows[1].starts_with("│ → l'insertion tombe en 4"), "{}", rows[1]);
        assert!(rows[2].starts_with("│ ⟩ nothing bu"), "{}", rows[2]);
        assert!(rows[3].contains("catalogue 2 · spotify …"), "{}", rows[3]);
        assert!(rows[4].contains("── catalogue  2 résultats"), "{}", rows[4]);
        assert!(rows[5].contains(" 1 [catalogue] ♪ Nothing But Time"), "{}", rows[5]);
        assert!(rows[6].contains(" 2 [catalogue] ♥ Nothing Compares 2 U"), "{}", rows[6]);
        assert_eq!(buffer[(4, 6)].style().bg, Some(Color::Yellow), "le curseur surligne la ligne 2");
        assert!(rows[7].contains("[spotify] … interrogation"), "{}", rows[7]);
        assert!(rows[9].contains("entrée insérer ici"), "{}", rows[9]);
        assert!(rows[10].starts_with("└─ échap ferme sans insérer"), "{}", rows[10]);
    }

    #[test]
    fn a_narrow_terminal_keeps_the_axis_and_drops_the_column() {
        let text = screen(50, 30, &branches()).join("\n");
        assert!(!text.contains("── branches"), "{text}");
        assert!(text.contains("Cities in Dust"), "{text}");
    }
}

// --- la modale de la discographie (`ad`, maquette 1a) -----------------------

/// Le filet léger : c'est la seule boîte que forkstify dessine (le menu du
/// leader, « ? »), et une modale en est le troisième cas. Une ligne d'en-tête
/// se ferme par un trait qui court jusqu'au bord.
fn ruled(mut spans: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let used: usize = spans.iter().map(|span| span.content.chars().count()).sum();
    if width > used + 1 {
        spans.push(Span::styled(
            format!(" {}", "─".repeat(width - used - 1)),
            Style::default().fg(DIM),
        ));
    }
    Line::from(spans)
}

fn plural(n: usize) -> &'static str {
    if n > 1 {
        "s"
    } else {
        ""
    }
}

fn bar(share: f64, width: usize) -> String {
    let full = (share * width as f64).round().clamp(0.0, width as f64) as usize;
    format!("{}{}", "█".repeat(full), "░".repeat(width - full))
}

fn clock_ms(ms: u32) -> String {
    if ms == 0 {
        return String::new();
    }
    let seconds = ms / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// « 14 écoutes », « — » quand il n'a jamais sonné.
fn plays_of(plays: f64) -> String {
    if plays < 0.5 {
        "—".to_string()
    } else {
        format!("{plays:.0} écoute{}", if plays >= 1.5 { "s" } else { "" })
    }
}

fn render_explore(frame: &mut ratatui::Frame, area: Rect, screen: &crate::explore::Explore) {
    use crate::explore::Row;
    frame.render_widget(Clear, area);
    let width = area.width as usize;
    let summary = screen.summary();
    let cursor = Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD);
    let rule = |text: &str| Span::styled(text.to_string(), Style::default().fg(DIM));

    // — l'en-tête : ce que la fiche et l'appris disent de l'artiste
    let mut head = vec![
        ruled(
            vec![
                rule("┌─ "),
                Span::styled(
                    "discographie ",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                rule("── "),
                Span::styled(
                    format!("{} ", screen.name),
                    Style::default().fg(CATALOG).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "{} · {} albums · {} titres{}",
                        if screen.generated { "fiche générée" } else { "fiche écrite" },
                        summary.albums,
                        summary.titles,
                        if screen.loading { " · chargement…" } else { "" }
                    ),
                    Style::default().fg(DIM),
                ),
            ],
            width,
        ),
        Line::from(vec![
            rule("│ "),
            Span::styled("le réservoir en tire  ", Style::default().fg(MUTED)),
            Span::styled(
                format!("♪ {} top{}", summary.tops, plural(summary.tops)),
                Style::default().fg(PLAYING),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(
                format!("♥ {} aimé{}", summary.liked, plural(summary.liked)),
                Style::default().fg(PLAYING),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(
                format!("⊘ {} banni{}", summary.banned, plural(summary.banned)),
                Style::default().fg(DANGER),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(format!("· {} en traîne", summary.tail), Style::default().fg(VECTOR)),
        ]),
    ];
    // la question qu'on vient poser : quel album porte les écoutes
    head.push(Line::from(vec![
        rule("│ "),
        Span::styled("tes écoutes  ", Style::default().fg(MUTED)),
        Span::styled(
            if summary.plays < 0.5 {
                "aucune écoute enregistrée chez lui".to_string()
            } else {
                format!(
                    "{} album{} porte{} {} % des {:.0} écoutes",
                    summary.carrying,
                    if summary.carrying > 1 { "s" } else { "" },
                    if summary.carrying > 1 { "nt" } else { "" },
                    summary.carrying_pct,
                    summary.plays
                )
            },
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!(" — {} album(s) jamais ouvert(s)", summary.never),
            Style::default().fg(DIM),
        ),
    ]));
    head.push(Line::from(vec![
        rule("│ "),
        Span::styled(
            format!("ordre : {} · filtre : {}", screen.sort_word(), screen.filter.word()),
            Style::default().fg(DIM),
        ),
        Span::styled(
            if screen.query.is_empty() {
                String::new()
            } else {
                format!(" · « {} »", screen.query)
            },
            Style::default().fg(MUTED),
        ),
    ]));

    // — le pied : ce qui attend d'être écrit, puis les touches
    let mut foot: Vec<Line> = Vec::new();
    if !screen.pending.is_empty() {
        foot.push(ruled(
            vec![
                rule("│ "),
                Span::styled("── en attente ", Style::default().fg(EDIT).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(
                        "{} édition{} · une fiche · un commit",
                        screen.pending.len(),
                        plural(screen.pending.len())
                    ),
                    Style::default().fg(DIM),
                ),
            ],
            width,
        ));
        for edit in screen.pending.iter().take(4) {
            foot.push(Line::from(vec![
                rule("│ "),
                Span::styled(
                    format!("{} ", if edit.add { "♪+" } else { "♪−" }),
                    Style::default().fg(EDIT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<32}", fit(&edit.title, 32)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    if edit.add { "promouvoir en top  " } else { "retirer du top     " }.to_string(),
                    Style::default().fg(MUTED),
                ),
                Span::styled(format!("({})", edit.why), Style::default().fg(DIM)),
            ]));
        }
        if screen.pending.len() > 4 {
            foot.push(Line::from(vec![
                rule("│ "),
                rule(&format!("↓ {} de plus", screen.pending.len() - 4)),
            ]));
        }
        // ce que l'utilisateur lit est ce que git retiendra (edit.rs)
        foot.push(Line::from(vec![
            rule("│ "),
            Span::styled(format!("cards/{}.toml ", screen.slug), Style::default().fg(CATALOG)),
            Span::styled(format!("— « {} »", screen.commit_line()), Style::default().fg(DIM)),
        ]));
    }
    if !screen.notice.is_empty() {
        let mut line = notice_line(&screen.notice).spans;
        line.insert(0, rule("│ "));
        foot.push(Line::from(line));
    }
    foot.push(Line::from(vec![
        rule("│ "),
        Span::styled("A ", Style::default().fg(EDIT).add_modifier(Modifier::BOLD)),
        Span::styled("l'album  ", Style::default().fg(MUTED)),
        Span::styled("tl ", Style::default().fg(PLAYING)),
        Span::styled("aimer  ", Style::default().fg(MUTED)),
        Span::styled("tb ", Style::default().fg(DANGER)),
        Span::styled("bannir  ", Style::default().fg(MUTED)),
        Span::styled("e ", Style::default().fg(BRANCH)),
        Span::styled("à la file  ", Style::default().fg(MUTED)),
        Span::styled("s v / ", Style::default().fg(VECTOR)),
        Span::styled("ordre, vue, filtre", Style::default().fg(MUTED)),
    ]));
    foot.push(ruled(
        vec![
            rule("└─ "),
            Span::styled(
                if screen.pending.is_empty() {
                    "échap ferme".to_string()
                } else {
                    format!("⏎ écrire ({} en attente, 1 commit)", screen.pending.len())
                },
                Style::default().fg(MUTED),
            ),
            Span::styled(
                " · u annule la dernière · échap ferme sans écrire".to_string(),
                Style::default().fg(DIM),
            ),
        ],
        width,
    ));

    // — le corps : les albums pliés, celui du curseur ouvert
    let rows = screen.rows();
    let room = (area.height as usize).saturating_sub(head.len() + foot.len()).max(1);
    let here = rows.iter().position(|row| screen.at(*row)).unwrap_or(0);
    // la fenêtre suit le curseur sans le coller au bord
    let start = here.saturating_sub(room / 2).min(rows.len().saturating_sub(room));
    let mut lines = head;
    for row in rows.iter().skip(start).take(room) {
        let selected = screen.at(*row);
        lines.push(match *row {
            Row::Album(index) => {
                let album = &screen.albums[index];
                let share = if summary.plays > 0.0 { album.plays() / summary.plays } else { 0.0 };
                let marks = if album.orphan {
                    "à relire".to_string()
                } else {
                    let mut said = Vec::new();
                    if album.tops() > 0 {
                        said.push(format!("♪ {}", album.tops()));
                    }
                    if album.liked() > 0 {
                        said.push(format!("♥ {}", album.liked()));
                    }
                    if album.banned() > 0 {
                        said.push(format!("⊘ {}", album.banned()));
                    }
                    said.join(" · ")
                };
                // ▾ dit ce qui est ouvert, pas ce qui est surligné : le
                // curseur peut être descendu dans les morceaux de l'album
                let open = screen.cursor.album == index && !screen.folded;
                let text = format!(
                    "{} {:<5}{:<34}{:>3}  {:<16}{:>6}  {}",
                    if open { "▾" } else { "▸" },
                    album.year.map(|y| y.to_string()).unwrap_or_default(),
                    fit(&album.title, 33),
                    album.tracks.len(),
                    fit(&marks, 16),
                    if album.plays() < 0.5 { "—".into() } else { format!("{:.0}", album.plays()) },
                    bar(share, 10),
                );
                if selected {
                    Line::from(vec![rule("│ "), Span::styled(text, cursor)])
                } else {
                    Line::from(vec![rule("│ "), Span::styled(text, Style::default().fg(MUTED))])
                }
            }
            Row::Track(album, track) => {
                let track = &screen.albums[album].tracks[track];
                let glyph = if track.banned {
                    ('⊘', DANGER)
                } else if track.is_top() {
                    ('♪', PLAYING)
                } else if track.liked {
                    ('♥', PLAYING)
                } else {
                    ('·', VECTOR)
                };
                let state = if screen.is_playing(track) {
                    "▶ sonne".to_string()
                } else if track.banned {
                    "banni".to_string()
                } else if track.liked {
                    "♥ aimé".to_string()
                } else {
                    String::new()
                };
                let last = match track.days {
                    Some(0) => "aujourd'hui".to_string(),
                    Some(days) => crate::home::age(Some(days)),
                    None => "jamais".to_string(),
                };
                let before = format!(
                    "   {:>3} ",
                    if track.number > 0 { track.number.to_string() } else { String::new() }
                );
                let after = format!(
                    " {:<32}{:<6}{:<10}{:>11}  {}",
                    fit(&track.title, 31),
                    clock_ms(track.duration_ms),
                    state,
                    plays_of(track.plays),
                    last,
                );
                // la couleur dit la nature, jamais l'importance : le glyphe
                // seul la porte, le reste de la ligne est du texte
                let (text_style, glyph_style) = if selected {
                    (cursor, cursor)
                } else {
                    (
                        Style::default().fg(if track.banned { DIM } else { MUTED }),
                        Style::default().fg(glyph.1),
                    )
                };
                Line::from(vec![
                    rule("│ "),
                    Span::styled(before, text_style),
                    Span::styled(glyph.0.to_string(), glyph_style),
                    Span::styled(after, text_style),
                ])
            }
        });
    }
    // le filet descend jusqu'au pied : le bas de l'écran ne bouge pas d'un
    // album à l'autre
    while lines.len() + foot.len() < area.height as usize {
        lines.push(Line::from(rule("│")));
    }
    lines.extend(foot);
    frame.render_widget(Paragraph::new(lines), area);
}
