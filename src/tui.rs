//! La TUI (décision [0006] : ratatui). Première version, variante **1b** des
//! maquettes : une colonne pleine largeur pour l'axe de lecture, et un volet
//! qui se pose dessus à l'embranchement puis s'en va.
//!
//! Elle n'emprunte à ratatui que le **dessin**. La saisie reste celle de
//! `keys.rs` — termios brut, grammaire sans préfixe — parce qu'elle est
//! déjà éprouvée et que ratatui n'a pas besoin de posséder l'entrée.
//!
//! Les couleurs sont des **rôles**, jamais des hex : forkstify emprunte la
//! palette du terminal, si bien que changer de thème Omarchy le rethème.

use crate::engine::{Branch, Source, Stop};
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
const MUTED: Color = Color::Gray;
const DIM: Color = Color::DarkGray;

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
    pub path: Vec<String>,
    pub seed: &'a str,
    pub segment: usize,
    pub past: &'a [Stop],
    pub current: Option<&'a Stop>,
    pub paused: bool,
    pub queue: &'a [Stop],
    pub branches: &'a [Branch],
    /// Le volet des branches est **toujours là** (Joel, 06/09/2026) : on ne
    /// veut pas attendre l'embranchement pour savoir où l'on peut aller.
    /// Il a donc sa place réservée à droite plutôt que d'être posé sur
    /// l'axe — sinon il masquerait en permanence le bas de la file.
    pub panel: bool,

    pub notices: &'a [String],
    /// La ligne de l'axe sous la sélection — surlignée, mais pas jouée.
    pub selection: Option<usize>,
    /// Un bloc posé sur l'écran, qui ne descend pas dans le journal : le bas
    /// de l'écran ne doit jamais bouger (Joel, 06/09/2026).
    pub overlay: Option<(&'a str, &'a [String])>,
    pub comfort_mode: bool,
    pub comfort: u8,
    pub comfort_word: &'a str,
    pub prompt: String,
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

/// Toutes les parts sont clonées : la ligne ne tient à rien, ce qui permet de
/// la surligner après coup sans traîner d'emprunt.
fn stop_line(stop: &Stop, prefix: &str, muted: bool) -> Line<'static> {
    let title = if muted {
        Style::default().fg(MUTED)
    } else {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    };
    Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(DIM)),
        Span::styled(stop.source.mark().to_string(), Style::default().fg(role_of(stop.source))),
        Span::raw(" "),
        Span::styled(stop.title.clone(), title),
        Span::styled(" — ", Style::default().fg(DIM)),
        Span::styled(stop.artist.clone(), Style::default().fg(CATALOG)),
    ])
}

fn render(frame: &mut ratatui::Frame, view: &View) {
    let area = frame.area();
    // trois zones de hauteur fixe et un corps qui prend le reste : le bas ne
    // bouge jamais, quoi que forkstify dise
    let [head, body, status, prompt] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    // l'axe à gauche, les branches à droite et en bas — leur place est
    // réservée, elles ne recouvrent rien
    let panel_width = 54.min(body.width / 2);
    let (axis, panel_column) = if view.panel && panel_width >= 24 {
        let [left, right] =
            Layout::horizontal([Constraint::Min(24), Constraint::Length(panel_width)]).areas(body);
        (left, Some(right))
    } else {
        (body, None)
    };

    // — l'en-tête : où l'on en est, en une ligne et sa précision
    let mut path: Vec<Span> = Vec::new();
    for (i, name) in view.path.iter().enumerate() {
        if i > 0 {
            path.push(Span::styled(" → ", Style::default().fg(BRANCH)));
        }
        let last = i + 1 == view.path.len();
        path.push(Span::styled(
            name.clone(),
            if last {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            },
        ));
    }
    let head_lines = vec![
        Line::from(path),
        Line::from(Span::styled(
            format!(
                "graine : {} · segment {} · confort {} — {}",
                view.seed, view.segment, view.comfort, view.comfort_word
            ),
            Style::default().fg(DIM),
        )),
    ];
    frame.render_widget(Paragraph::new(head_lines), head);

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
    // tout ce qui a été joué reste à l'écran : c'est la playlist en train de
    // se faire, pas un historique à oublier (Joel, 06/09/2026)
    for stop in view.past {
        if let Some(label) = &stop.head {
            lines.push(Line::from(Span::styled(
                format!(" → {label}"),
                Style::default().fg(DIM),
            )));
        }
        push(stop_line(stop, "  ", true), index, view.selection, &mut lines);
        index += 1;
    }
    let playing_line = lines.len();
    if let Some(stop) = view.current {
        let playing = Line::from(Span::styled(
            format!(
                " {} {} — {} ",
                if view.paused { "⏸" } else { "▶" },
                stop.title,
                stop.artist
            ),
            Style::default().fg(Color::Black).bg(PLAYING).add_modifier(Modifier::BOLD),
        ));
        push(playing, index, view.selection, &mut lines);
        index += 1;
    }
    if !view.queue.is_empty() {
        lines.push(Line::from(Span::styled("à suivre :", Style::default().fg(MUTED))));
        // la file enchaîne plusieurs branches : chacune s'ouvre par son nom,
        // et un filet dit jusqu'où elle va
        let mut in_branch = false;
        for stop in view.queue {
            if let Some(label) = &stop.head {
                in_branch = true;
                lines.push(Line::from(vec![
                    Span::styled(" → ", Style::default().fg(BRANCH)),
                    Span::styled(
                        label.clone(),
                        Style::default().fg(BRANCH).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
            let mut line = stop_line(stop, if in_branch { " │ " } else { "   " }, false);
            if in_branch {
                line.spans[0] = Span::styled(" │ ", Style::default().fg(BRANCH));
            }
            push(line, index, view.selection, &mut lines);
            index += 1;
        }
    } else if view.current.is_some() {
        lines.push(Line::from(Span::styled(
            "(plus rien à suivre — 1-3 pour ajouter une branche)",
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

    // — la dernière chose dite, sur une ligne qui ne grandit pas
    let last = view.notices.iter().rev().find(|line| !line.trim().is_empty());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            last.cloned().unwrap_or_default(),
            Style::default().fg(MUTED),
        ))),
        status,
    );

    // — l'invite : toujours la dernière ligne
    let comfort_gauge: String = (0..5)
        .map(|i| if i < view.comfort { '█' } else { '░' })
        .collect();
    let prompt_line = Line::from(vec![
        Span::styled(view.prompt.clone(), Style::default().fg(MUTED)),
        Span::raw("  "),
        Span::styled(
            comfort_gauge,
            if view.comfort_mode {
                Style::default().fg(Color::Black).bg(VECTOR).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(VECTOR)
            },
        ),
    ]);
    frame.render_widget(Paragraph::new(prompt_line), prompt);

    // — les branches, dans leur colonne : toujours visibles
    if let Some(column) = panel_column {
        render_panel(frame, column, view);
    }

    // — et ce qui se pose par-dessus tout : un bloc demandé (le leader, « ? »)
    if let Some((title, body)) = view.overlay {
        render_block(frame, area, title, body);
    }
}

/// Le volet des branches : il arrive, on choisit, il s'en va. Le seul endroit
/// avec le menu du leader où forkstify trace autre chose qu'une règle.
fn render_panel(frame: &mut ratatui::Frame, column: Rect, view: &View) {
    let wanted = (view.branches.len() as u16 * 3 + 4).max(5);
    let height = wanted.min(column.height);
    // en bas de sa colonne, comme sur la maquette
    let panel = Rect {
        x: column.x,
        y: column.y + column.height.saturating_sub(height),
        width: column.width,
        height,
    };

    let mut lines: Vec<Line> = Vec::new();
    if view.branches.is_empty() {
        lines.push(Line::from(Span::styled(
            " cul-de-sac — « fu » pour revenir",
            Style::default().fg(MUTED),
        )));
    }
    for (i, branch) in view.branches.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}  ", i + 1),
                Style::default().fg(BRANCH).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                branch.label.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));
        if !branch.reason.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("    {}", branch.reason),
                Style::default().fg(MUTED),
            )));
        }
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        " 1-3 prendre · fn1 sans attendre · fr reproposer",
        Style::default().fg(DIM),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRANCH))
        .title(Span::styled(
            " → où va-t-on ? ",
            Style::default().fg(BRANCH).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), panel);
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
}

impl Tui {
    pub fn draw_home(&mut self, view: &HomeView) -> std::io::Result<()> {
        self.terminal.draw(|frame| render_home(frame, view))?;
        Ok(())
    }
}

fn render_home(frame: &mut ratatui::Frame, view: &HomeView) {
    let area = frame.area();
    let [head, whole, prompt] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(area);

    // à gauche ce que forkstify propose, à droite ce qu'il possède
    let column = 46.min(whole.width / 2);
    let (body, collection) = match (&view.collection, column >= 24) {
        (Some(_), true) => {
            let [left, right] =
                Layout::horizontal([Constraint::Min(30), Constraint::Length(column)]).areas(whole);
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
                "↑↓ parcourir · entrée démarrer · s trier",
                Style::default().fg(DIM),
            )),
        ]),
        foot,
    );
}
