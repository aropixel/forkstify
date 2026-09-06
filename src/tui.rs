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
    /// 1b : le volet n'existe qu'au moment du choix.
    pub panel: bool,
    pub pending: Option<String>,
    pub notices: &'a [String],
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

fn stop_line<'a>(stop: &'a Stop, prefix: &'a str, muted: bool) -> Line<'a> {
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
    let [head, axis, notices, prompt] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length((view.notices.len() as u16).min(6)),
        Constraint::Length(1),
    ])
    .areas(area);

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

    // — l'axe : ce qui a sonné, ce qui sonne, ce qui suit
    let mut lines: Vec<Line> = Vec::new();
    let shown = view.past.len().saturating_sub(3);
    for stop in &view.past[shown..] {
        lines.push(stop_line(stop, "  ", true));
    }
    if let Some(stop) = view.current {
        // la sélection est une inversion, comme dans un terminal
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    " {} {} — {} ",
                    if view.paused { "⏸" } else { "▶" },
                    stop.title,
                    stop.artist
                ),
                Style::default().fg(Color::Black).bg(PLAYING).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if !view.queue.is_empty() {
        lines.push(Line::from(Span::styled("à suivre :", Style::default().fg(MUTED))));
        for stop in view.queue {
            lines.push(stop_line(stop, "   ", false));
        }
    } else if view.current.is_some() {
        lines.push(Line::from(Span::styled(
            "(dernier du segment)",
            Style::default().fg(DIM),
        )));
    }
    if let Some(label) = &view.pending {
        lines.push(Line::from(vec![
            Span::styled("→ ", Style::default().fg(BRANCH)),
            Span::styled(label.clone(), Style::default().fg(MUTED)),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), axis);

    // — ce que forkstify vient de dire
    let notice_lines: Vec<Line> = view
        .notices
        .iter()
        .rev()
        .take(notices.height as usize)
        .rev()
        .map(|text| Line::from(Span::styled(text.clone(), Style::default().fg(MUTED))))
        .collect();
    frame.render_widget(Paragraph::new(notice_lines), notices);

    // — l'invite : toujours la dernière ligne
    let comfort_gauge: String = (0..5)
        .map(|i| if i < view.comfort { '█' } else { '░' })
        .collect();
    let prompt_line = Line::from(vec![
        Span::styled(view.prompt.clone(), Style::default().fg(MUTED)),
        Span::raw("  "),
        Span::styled(comfort_gauge, Style::default().fg(VECTOR)),
    ]);
    frame.render_widget(Paragraph::new(prompt_line), prompt);

    // — le volet, posé dessus, et seulement à l'embranchement (1b)
    if view.panel && !view.branches.is_empty() {
        render_panel(frame, area, view);
    }
}

/// Le volet des branches : il arrive, on choisit, il s'en va. Le seul endroit
/// avec le menu du leader où forkstify trace autre chose qu'une règle.
fn render_panel(frame: &mut ratatui::Frame, area: Rect, view: &View) {
    let width = 52.min(area.width.saturating_sub(4));
    let height = (view.branches.len() as u16 * 3 + 4).min(area.height.saturating_sub(2));
    let panel = Rect {
        x: area.x + area.width.saturating_sub(width + 2),
        y: area.y + area.height.saturating_sub(height + 2),
        width,
        height,
    };
    frame.render_widget(Clear, panel);

    let mut lines: Vec<Line> = Vec::new();
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
