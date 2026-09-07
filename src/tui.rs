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
    pub queue: &'a [Stop],
    pub branches: &'a [Branch],
    /// Le volet des branches est **toujours là** (Joel, 06/09/2026) : on ne
    /// veut pas attendre l'embranchement pour savoir où l'on peut aller.
    /// Il a donc sa place réservée à droite plutôt que d'être posé sur
    /// l'axe — sinon il masquerait en permanence le bas de la file.
    pub panel: bool,

    pub notices: &'a [String],
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
    let [head, seed_block, body, now, bar, next, status, prompt] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(1),
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

    // — le pied (maquette 2b) : ce qui sonne et sa provenance, puis ce qui
    // suit et à combien de morceaux se trouve l'embranchement — ce que la
    // liste ne dit plus quand elle a défilé
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
                Span::styled(format!("  ({} / {tracks})", view.past.len() + 1), Style::default().fg(DIM)),
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
        match view.queue.first() {
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
                match view.queue.len() {
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

    // — la dernière chose dite, sur une ligne qui ne grandit pas
    let last = view.notices.iter().rev().find(|line| !line.trim().is_empty());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            last.cloned().unwrap_or_default(),
            Style::default().fg(MUTED),
        ))),
        status,
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

    // — et ce qui se pose par-dessus tout : un bloc demandé (le leader, « ? »)
    if let Some((title, body)) = view.overlay {
        render_block(frame, area, title, body);
    }
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
    ]));
    lines.push(Line::from(""));
    if view.branches.is_empty() {
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
        playlist(width, height, branches, &[], Some(&current), &[])
    }

    fn playlist(
        width: u16,
        height: u16,
        branches: &[Branch],
        past: &[Stop],
        current: Option<&Stop>,
        queue: &[Stop],
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
            queue,
            branches,
            panel: true,
            notices: &[],
            notes: &notes,
            selection: None,
            overlay: None,
            comfort_mode: false,
            comfort: 3,
            comfort_word: "équilibré",
            progress: current.map(|_| (154_000, 227_000)),
            prompt: "[1-2 branche]".to_string(),
        };
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect()
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
        let body_rows = 5..(30 - 5);
        assert!(body_rows.clone().all(|y| rows[y].contains('│')), "{text}");
        assert!(rows[24].contains("fn1 sans attendre la fin"), "{text}");
        assert!(rows[23].contains("1-3 prendre"), "{text}");
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
        let queue = [
            stop("Israel", "Siouxsie and the Banshees"),
            headed(stop("Right Now", "The Creatures"), "The Creatures", "membres en commun"),
            headed(stop("Alison", "Slowdive"), "Slowdive", "proche du centre de la branche (0.74)"),
        ];
        let rows = playlist(160, 30, &branches(), &past, Some(&current), &queue);
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
        assert!(text.contains(" 2 →  ♪ Israel — Siouxsie and the Banshees"), "{text}");
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
        let now = &rows[rows.len() - 5];
        assert!(now.starts_with("▶ Cities in Dust — Siouxsie and the Banshees  (3 / 6)"), "{now}");
        assert!(now.trim_end().ends_with("♪ top │ 2:34 / 3:47 -1:13"), "{now}");
        // la barre : 154 s sur 227, soit 108 cellules pleines sur 160
        let bar = &rows[rows.len() - 4];
        assert_eq!(bar.chars().filter(|c| *c == '█').count(), 108, "{bar}");
        assert_eq!(bar.chars().filter(|c| *c == '░').count(), 52, "{bar}");
        let next = &rows[rows.len() - 3];
        assert!(next.starts_with("à suivre  Israel — Siouxsie and the Banshees"), "{next}");
        assert!(next.trim_end().ends_with("→ embranchement dans 3 morceaux"), "{next}");
    }

    #[test]
    fn a_narrow_terminal_keeps_the_axis_and_drops_the_column() {
        let text = screen(50, 30, &branches()).join("\n");
        assert!(!text.contains("── branches"), "{text}");
        assert!(text.contains("Cities in Dust"), "{text}");
    }
}
