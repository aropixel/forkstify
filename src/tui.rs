//! The TUI (decision [0006]: ratatui). Variants **1a** then **2b** of the
//! mockups (`Lecture.dc.html`, Joel on 07/09/2026): two permanent panes —
//! the playlist on the left, the branches on the right, each unfolded with
//! its tracks —, the seed as a block above, and a foot that says what is
//! playing and what comes next.
//!
//! It borrows only the **drawing** from ratatui. Input stays the one of
//! `keys.rs` — raw termios, prefix-free grammar — because it is already
//! proven and ratatui has no need to own the input.
//!
//! Colors are **roles**, never hex values: forkstify borrows the terminal
//! palette, so switching the Omarchy theme rethemes it.

use crate::engine::{Branch, Head, Source, Stop};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{Stdout, Write};

// The design system roles, on the terminal's ANSI palette.
const BRANCH: Color = Color::LightMagenta;
const PLAYING: Color = Color::Green;
const CATALOG: Color = Color::Blue;
const VECTOR: Color = Color::Cyan;
const DOOR: Color = Color::LightRed;
const EDIT: Color = Color::Yellow;
const DANGER: Color = Color::Red;
/// What is loading: the color of the vectors, and of network waits.
pub const LOADING: Color = Color::Cyan;
const MUTED: Color = Color::Gray;

/// The share of width given to the left column — the proposals at home,
/// the axis while listening: **same layout on both screens** (Joel,
/// 07/09/2026). 60 leaves the right enough room to show a long name
/// without cutting; 50 makes it more present. A single value to change.
const LEFT_SHARE: u16 = 60;
const DIM: Color = Color::DarkGray;

/// Below this width the right column disappears: one readable column
/// beats two unreadable ones.
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

/// Everything the screen needs to know. The session fills it, the TUI
/// decides nothing: the engine produces, the display shows.
pub struct View<'a> {
    /// The artists walked through — counted in the seed block, no longer
    /// shown in the head: the played list already says it (mockup 2b).
    pub path: Vec<String>,
    pub seed: &'a str,
    pub seed_name: &'a str,
    /// "written card · 41 links · 12 tops"
    pub seed_facts: &'a str,
    /// "last played -3s", or "never played"
    pub seed_last: &'a str,
    /// The forks taken since the seed.
    pub forks: usize,
    pub segment: usize,
    pub past: &'a [Stop],
    pub current: Option<&'a Stop>,
    pub paused: bool,
    /// The current track is shown but not resolved yet: Spotify is looking
    /// up its address behind the screen.
    pub loading: bool,
    pub queue: &'a [Stop],
    pub branches: &'a [Branch],
    /// Links pointing to a missing card (0016): directions the catalog names
    /// but cannot walk yet. They are numbered **after** the branches, and
    /// taking one generates the card instead of playing right away.
    pub missing: &'a [crate::engine::Missing],
    /// The branches pane is **always there** (Joel, 06/09/2026): we don't
    /// want to wait for the fork to know where we can go. So it has its
    /// reserved place on the right rather than being laid over the axis —
    /// otherwise it would permanently hide the bottom of the queue.
    pub panel: bool,

    /// One grey note per track of the axis (past, current, queue), in the
    /// same order: what the plays know about it (mockup 3a).
    pub notes: &'a [String],
    /// The axis line under the selection — highlighted, but not played.
    pub selection: Option<usize>,
    /// A block laid over the screen, which does not go down into the log:
    /// the bottom of the screen must never move (Joel, 06/09/2026).
    /// Title, lines, and how far it is scrolled (j/k, ↑↓ — Joel, 20/09/2026).
    pub overlay: Option<(&'a str, &'a [String], usize)>,
    pub comfort_mode: bool,
    pub comfort: u8,
    pub comfort_word: &'a str,
    /// (position, duration) in milliseconds of the playing track — `None`
    /// as long as librespot has said nothing.
    pub progress: Option<(u32, u32)>,
    /// The box at the bottom right: what is loading, or the last thing
    /// said, in color (Joel, 08/09/2026).
    pub toast: Option<Toast>,
    /// The search modal, when open.
    pub finder: Option<FinderView>,
    /// The discography modal (`ad`), laid over the listening screen: it
    /// takes the body, the head and the foot stay — "playback has not
    /// stopped" (mockup 1a).
    pub explore: Option<&'a crate::explore::Explore>,
    pub prompt: String,
}

/// The playback foot: what is playing and what comes next on one line,
/// then its progress — the bar closes the foot, nothing follows it
/// (mockup 4a, Joel 10/09/2026). The same under the session and under
/// home (Joel, 08/09/2026) — listening goes on when switching screens.
pub struct Bar<'a> {
    pub current: Option<&'a Stop>,
    pub paused: bool,
    pub loading: bool,
    pub progress: Option<(u32, u32)>,
    /// (rank of the current one, total) in the playlist
    pub position: (usize, usize),
    pub next: Option<&'a Stop>,
}

/// The search modal (mockup `Recherche.dc.html`, Joel 08/09/2026): one
/// input line, a rule that cuts and counts, the catalog before Spotify,
/// never mixed.
pub struct FinderView {
    /// `ti` rather than `:search`: the title changes, and the anchor shows.
    pub insert: bool,
    /// `aL`: linking that artist to the chosen row; the title says so.
    pub linking: Option<String>,
    pub anchor: Option<String>,
    pub query: String,
    /// (catalog, spotify, spotify being queried)
    pub counts: (usize, Option<usize>, bool),
    pub only_catalogue: bool,
    pub lines: Vec<FinderLine>,
    /// The index, among the `Row`s only.
    pub cursor: usize,
}

pub enum FinderLine {
    Header { catalogue: bool, text: String },
    Row { catalogue: bool, mark: char, title: String, artist: String, note: String },
    Info(String),
}

/// A toast: a text, its color, and whether it stays while loading.
pub struct Toast {
    pub text: String,
    pub tone: Color,
    pub sticky: bool,
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Alternate screen, hidden cursor. The sequences are written by hand:
    /// ratatui only draws, it does not hold the terminal.
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

    pub fn height(&self) -> Option<u16> {
        self.terminal.size().ok().map(|size| size.height)
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = write!(out, "\x1b[?25h\x1b[?1049l");
        let _ = out.flush();
    }
}

/// A notification line, styled by its nature — read from the glyph that
/// opens it, like the design system's Notice component: ✓ in green, ⏹ and
/// ⊘ in red, ↻ ⚑ in yellow (an edit), → in magenta, a parenthesis in grey,
/// "not wired yet" in dimmed italics. What follows a "—" or a final
/// parenthesis is the detail, dimmed.
/// The color of a message, read from the glyph that opens it — the same
/// for the foot line and for the toast.
pub fn tone_of(text: &str) -> Color {
    let text = text.trim();
    let not_wired = text.contains("not wired yet");
    let first = text.chars().next().unwrap_or(' ');
    match first {
        '✓' | '♥' | '▶' => PLAYING,
        '⏹' | '⊘' => DANGER,
        '↻' | '⚑' => EDIT,
        '→' => BRANCH,
        '…' | '⏳' => LOADING,
        '(' => MUTED,
        _ if not_wired => DIM,
        _ if text.starts_with("failed") || text.contains("not found") || text.contains("unreadable") => DANGER,
        _ => Color::White,
    }
}

fn notice_line(text: &str) -> Line<'static> {
    let text = text.trim();
    if text.is_empty() {
        return Line::from("");
    }
    let not_wired = text.contains("not wired yet");
    let first = text.chars().next().unwrap_or(' ');
    let tone = tone_of(text);
    let style = if not_wired {
        Style::default().fg(DIM).add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(tone)
    };
    // the detail — after "—" or in a final parenthesis — is dimmed
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

/// "m:ss", as a player writes it.
fn clock(ms: u32) -> String {
    let seconds = ms / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// A line with two ends: the left, then the right at the edge. With no
/// cell justification, the gap is computed from the width.
fn justified(left: Vec<Span<'static>>, right: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let count = |spans: &[Span]| spans.iter().map(|s| s.content.chars().count()).sum::<usize>();
    let gap = width.saturating_sub(count(&left) + count(&right)).max(2);
    let mut spans = left;
    spans.push(Span::raw(" ".repeat(gap)));
    spans.extend(right);
    Line::from(spans)
}

/// Cuts a text at `max` characters, with an ellipsis: a list gets cut.
fn fit(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// Where a playlist track stands.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Played,
    Playing { n: usize, paused: bool },
    Ahead { n: usize },
}

/// A track line, on the grid of mockup 3a: the number and the arrow, the
/// track, the reason of the branch it opens in grey — in cyan when it comes
/// from the vectors —, and on the right what the plays know about it. What
/// has played is not numbered and is dimmed; what is playing carries "▶";
/// what comes is counted from it.
/// `marked` puts "▸" in the gutter: the next track, when the listening
/// line is too narrow to name it (mockup 4a′, the fallback of 4b).
fn track_row(stop: &Stop, slot: Slot, marked: bool, opening: Option<&str>, note: &str, width: usize) -> Line<'static> {
    let played = slot == Slot::Played;
    // the number in grey, only the arrow in color (mockup 2b)
    // six cells either way: " 2 →  " or, marked, " 2 ▸→ "
    let gutter = |arrow: &str, tone: Color| -> Vec<Span<'static>> {
        let (mark, tail) = if marked { ("▸", " ") } else { ("", "  ") };
        vec![
            Span::styled(mark.to_string(), Style::default().fg(MUTED)),
            Span::styled(format!("{arrow}{tail}"), Style::default().fg(tone).add_modifier(Modifier::BOLD)),
        ]
    };
    let prefix: Vec<Span> = match slot {
        Slot::Played => vec![Span::raw("      ")],
        Slot::Playing { n, paused } => vec![
            Span::styled(format!("{n:>2} "), Style::default().fg(DIM)),
            Span::styled(
                format!("{} ", if paused { "⏸" } else { "▶" }),
                Style::default().fg(PLAYING).add_modifier(Modifier::BOLD),
            ),
        ],
        // a replay is recognized by its "↻": that is how the gesture is
        // checked, without a notification (Joel, 07/09/2026)
        Slot::Ahead { n } if stop.encore => {
            let mut spans = vec![Span::styled(format!("{n:>2} "), Style::default().fg(DIM))];
            spans.extend(gutter("↻", EDIT));
            spans
        }
        Slot::Ahead { n } => {
            let mut spans = vec![Span::styled(format!("{n:>2} "), Style::default().fg(DIM))];
            spans.extend(gutter("→", BRANCH));
            spans
        }
    };
    let body_text = format!("{} {} — {}", stop.source.mark(), stop.title, stop.artist);
    let body: Vec<Span> = match slot {
        // reversed, as everywhere something is active
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
    // the prefix is 6 cells, the reversed body takes two more
    let body_width = body_text.chars().count() + usize::from(matches!(slot, Slot::Playing { .. })) * 2;
    let used = 6 + body_width + 2 + 2;
    // a note that does not fit is not shown: we don't cut a word
    let note = if used - 2 + note.chars().count() <= width { note } else { "" };
    let room = width.saturating_sub(used + note.chars().count() + 2);
    // a reason reduced to a stump says nothing: under 14 cells, nothing
    let (middle, middle_tone) = match opening {
        Some(reason) if room >= 14 => {
            let tone = if reason.starts_with("close to") { VECTOR } else { MUTED };
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
    // mockup 2b: a head line, the seed block, the body that takes the rest,
    // then the foot and the prompt — the bottom never moves, whatever
    // forkstify says. The foot is two lines since mockup 4a: what plays and
    // what comes next share a line, the bar closes it. Everything said goes
    // in a toast, and the keys follow directly (Joel, 08/09/2026)
    let [head, seed_block, body, now, bar, prompt] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let full = area.width as usize;
    let foot = Bar {
        current: view.current,
        paused: view.paused,
        loading: view.loading,
        progress: view.progress,
        position: (view.past.len() + 1, view.past.len() + usize::from(view.current.is_some()) + view.queue.len()),
        next: view.queue.first(),
    };
    // whether the next track fits on the listening line decides its mark
    // in the list (4a′: when it leaves the line, it falls back to "▸")
    let (now_line, next_on_line) = listening_line(&foot, full);

    // the axis on the left, the branches on the right over the full height
    // (1a), 60/40 like home — their place is reserved, they cover nothing
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

    // — the head, on one line: the command on the left, the state on the right
    let tracks = view.past.len() + usize::from(view.current.is_some()) + view.queue.len();
    let mut right = vec![
        Span::styled(
            format!("segment {} · {} track{} · ", view.segment, tracks, if tracks > 1 { "s" } else { "" }),
            Style::default().fg(DIM),
        ),
        Span::styled("→ ", Style::default().fg(BRANCH)),
        Span::styled(
            match view.queue.len() {
                0 => "fork next".to_string(),
                n => format!("fork in {n}"),
            },
            Style::default().fg(MUTED),
        ),
        Span::styled(" │ ", Style::default().fg(DIM)),
    ];
    right.extend(comfort_spans(view.comfort, view.comfort_word, view.comfort_mode));
    frame.render_widget(
        Paragraph::new(justified(
            vec![
                Span::styled("forkstify", Style::default().fg(MUTED).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" listen {}", view.seed), Style::default().fg(DIM)),
            ],
            right,
            full,
        )),
        head,
    );

    // — the seed, as a block: everything descends from it
    let forks = match view.forks {
        0 => "no fork yet".to_string(),
        1 => "1 fork so far".to_string(),
        n => format!("{n} forks so far"),
    };
    let traversed = view.path.len();
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("── ", Style::default().fg(DIM)),
                Span::styled("seed", Style::default().fg(MUTED)),
                Span::styled(" ──────────────", Style::default().fg(DIM)),
            ]),
            Line::from(vec![
                Span::styled(
                    view.seed_name.to_string(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  [catalog]  ", Style::default().fg(CATALOG)),
                Span::styled(view.seed_facts.to_string(), Style::default().fg(MUTED)),
                Span::styled(format!("  {}", view.seed_last), Style::default().fg(DIM)),
            ]),
            Line::from(Span::styled(
                format!(
                    "{forks} — {traversed} artist{} traversed",
                    if traversed > 1 { "s" } else { "" }
                ),
                Style::default().fg(DIM),
            )),
        ]),
        seed_block,
    );

    // — the axis: what has played, what is playing, what comes next. It is
    // shown vertically, so it is walked vertically.
    let mut lines: Vec<Line> = Vec::new();
    let mut index = 0usize;
    // selecting highlights, it does not play: enter plays
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
    // the playlist on the grid of mockup 3a (Joel, 07/09/2026): what is
    // playing and what comes are numbered and separated by a rule, the
    // reason of each branch reads in grey next to the track that opens it,
    // and what the plays know about the track reads on the right. Everything
    // that has played stays on screen, compact and dimmed: it is the
    // playlist in the making, not a history to forget (Joel, 06/09/2026).
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
    let seed_reason = format!("seed: {}", view.seed);
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
        let marked = !next_on_line && current_at.map_or(false, |c| stop_index == c + 1);
        push(track_row(stop, slot, marked, opening, note, width), index, view.selection, &mut lines);
        index += 1;
    }
    if !all.is_empty() {
        // the horizon: nothing drawn beyond, the rest is in the column
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            fit(
                &format!(
                    "{:>2}    horizon  nothing drawn beyond — 1-3 to add a branch",
                    numbered + 1
                ),
                width,
            ),
            Style::default().fg(DIM),
        )));
    }
    // no line wrapping: a list gets cut, it does not fold — and we follow
    // what is playing rather than the start of the evening
    let height = axis.height as usize;
    let offset = playing_line
        .saturating_sub(height / 2)
        .min(lines.len().saturating_sub(height));
    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), axis);

    // — the foot (mockup 4a), shared with home
    frame.render_widget(Paragraph::new(now_line), now);
    render_progress(frame, bar, &foot);

    // — the prompt: always the last line, with its cursor
    let prompt_line = Line::from(vec![
        Span::styled(view.prompt.clone(), Style::default().fg(MUTED)),
        Span::raw(" "),
        Span::styled(" ", Style::default().bg(VECTOR)),
    ]);
    frame.render_widget(Paragraph::new(prompt_line), prompt);

    // — the branches, in their column: always visible
    if let Some(column) = panel_column {
        render_panel(frame, column, view);
    }

    // — the discography takes the body of the screen, never the foot
    if let Some(screen) = view.explore {
        render_explore(frame, body, screen);
    }

    // — search takes the body, like the discography
    if let Some(finder) = &view.finder {
        render_finder(frame, body, finder);
    }

    // — the toast, at the bottom right of the body, over the column or the
    // modal: what is loading, or what was just said
    if let Some(toast) = &view.toast {
        render_toast(frame, body, toast);
    }

    // — and what sits over everything: a requested block (the leader, "?")
    if let Some((title, body, scroll)) = view.overlay {
        render_block(frame, area, title, body, scroll);
    }
}

/// The search modal: the same light rule as the discography, a single
/// typing line "⟩", the rule that separates the input from the results and
/// carries the count, then the two groups — blue written by a human, cyan
/// guessed.
/// The comfort dial, one place, one look on both screens (Joel,
/// 14/09/2026): the blocks keep their colour, the label stays lit as it did
/// only while editing — the dial reads the same whether or not `cc` is
/// open; `cc` adds "↑↓" to say it is live.
fn comfort_spans(comfort: u8, word: &str, adjusting: bool) -> Vec<Span<'static>> {
    let gauge: String = (0..5).map(|i| if i < comfort { '█' } else { '░' }).collect();
    let lit = Style::default().fg(Color::Black).bg(VECTOR).add_modifier(Modifier::BOLD);
    let mut spans = vec![
        Span::styled(format!("comfort {comfort} "), lit),
        Span::styled(gauge, Style::default().fg(VECTOR)),
        Span::styled(format!(" {word}"), lit),
    ];
    if adjusting {
        spans.push(Span::styled(" ↑↓", Style::default().fg(MUTED)));
    }
    spans
}

fn render_finder(frame: &mut ratatui::Frame, area: Rect, view: &FinderView) {
    frame.render_widget(Clear, area);
    let width = area.width as usize;
    let rule = |text: &str| Span::styled(text.to_string(), Style::default().fg(DIM));
    let mut lines: Vec<Line> = Vec::new();

    // — the title carries the key, as everywhere
    let (heading, key, hint) = match &view.linking {
        Some(name) => (format!("link {name} "), "aL ", "pick an artist to link to"),
        None if view.insert => ("insert a track ".to_string(), "ti ", "tracks only"),
        None => ("search ".to_string(), ":search ", "tracks and artists"),
    };
    lines.push(ruled(
        vec![
            rule("┌─ "),
            Span::styled(heading, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            rule("── "),
            Span::styled(key, Style::default().fg(MUTED)),
            Span::styled(hint, Style::default().fg(DIM)),
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
    // — the only typing area
    let mut input = vec![
        rule("│ "),
        Span::styled("⟩ ", Style::default().fg(BRANCH).add_modifier(Modifier::BOLD)),
        Span::styled(view.query.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default().bg(Color::White)),
    ];
    if view.query.is_empty() {
        input.push(Span::styled("   a track, an artist, or a card slug", Style::default().fg(DIM)));
    }
    lines.push(Line::from(input));
    // — the rule that separates, and counts
    let (cat, spot, asking) = view.counts;
    let mut counts = vec![
        rule("│ "),
        rule(&"─".repeat(width.saturating_sub(40).max(8))),
        Span::styled(format!(" catalog {cat}"), Style::default().fg(CATALOG)),
        Span::styled(" · ", Style::default().fg(DIM)),
    ];
    counts.push(match (view.only_catalogue, asking, spot) {
        (true, _, _) => Span::styled("spotify hidden (tab)".to_string(), Style::default().fg(DIM)),
        (_, true, _) => Span::styled("spotify …".to_string(), Style::default().fg(VECTOR)),
        (_, _, Some(n)) => Span::styled(format!("spotify {n}"), Style::default().fg(VECTOR)),
        _ => Span::styled("spotify 0".to_string(), Style::default().fg(DIM)),
    });
    counts.push(rule(" ──"));
    lines.push(Line::from(counts));

    // — the results, grouped
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
                let source = if *catalogue { "[catalog]" } else { "[spotify]" };
                let tone = if *catalogue { CATALOG } else { VECTOR };
                let text = format!(
                    "{n:>2} {source:<9} {mark} {:<30} {:<22} {}",
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
                        Span::styled(format!("{source:<9} "), Style::default().fg(tone)),
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

    // — the keys, and closing
    lines.push(Line::from(rule("│")));
    lines.push(Line::from(vec![
        rule("│ "),
        Span::styled("enter ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            if view.insert { "insert here   " } else { "branch there, or play the track   " },
            Style::default().fg(MUTED),
        ),
        Span::styled("↑↓ ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("choose   ", Style::default().fg(MUTED)),
        Span::styled("tab ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("catalog only", Style::default().fg(MUTED)),
    ]));
    lines.push(ruled(
        vec![
            rule("└─ "),
            Span::styled("esc ", Style::default().fg(MUTED)),
            Span::styled(
                if view.insert { "closes without inserting — the queue is unchanged" } else { "closes the search — playback hasn't stopped" },
                Style::default().fg(DIM),
            ),
        ],
        width,
    ));
    frame.render_widget(Paragraph::new(lines), area);
}

/// A toast's box: a frame in the message's color, the text in bold inside,
/// laid at the bottom right of the body above the column's keys. Sticky
/// while loading, otherwise four seconds.
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
            if toast.sticky { " ⏳ in progress " } else { " " },
            Style::default().fg(toast.tone).add_modifier(Modifier::BOLD),
        ));
    let text: Vec<Line> = lines
        .into_iter()
        .map(|l| Line::from(Span::styled(format!(" {l}"), Style::default().fg(toast.tone).add_modifier(Modifier::BOLD))))
        .collect();
    frame.render_widget(Paragraph::new(text).block(block), rect);
}

/// The listening line (mockup 4a): one line carries the whole axis of
/// time — what plays on the left, then what comes next, then where it
/// comes from and the clock on the right. Returns whether the next track
/// made it onto the line.
///
/// When the terminal narrows, the right block is untouchable (it carries
/// the time left), everything that gives, gives on the left, in this
/// order (4a′): 1. the next track's artist · 2. the word "then" and the
/// counter, the "│" alone says the cut · 3. the source and the remaining
/// time · 4. the current title, cut with an ellipsis — never before the
/// other three · 5. the next track as a whole, which falls back to "▸" in
/// the list. Two rules: never two cuts on one line, and the next title is
/// never truncated — it is there or it is not.
fn listening_line(view: &Bar, width: usize) -> (Line<'static>, bool) {
    let Some(stop) = view.current else {
        return (Line::from(Span::styled("⏹ nothing playing", Style::default().fg(MUTED))), false);
    };
    let (rank, tracks) = view.position;
    let play_mark = format!("{} ", if view.paused { "⏸" } else { "▶" });
    let loading = if view.loading { " · loading…" } else { "" };
    let count = |spans: &[Span]| spans.iter().map(|s| s.content.chars().count()).sum::<usize>();

    // the right block: the source, then the clock as soon as librespot
    // has said it — `remaining` is the first of the two to give
    let right = |source: bool, remaining: bool| -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        if source {
            spans.push(Span::styled(format!("{} ", stop.source.mark()), Style::default().fg(role_of(stop.source))));
            spans.push(Span::styled(stop.source.word().to_string(), Style::default().fg(MUTED)));
        }
        if let Some((position, duration)) = view.progress {
            if source {
                spans.push(Span::styled(" │ ", Style::default().fg(DIM)));
            }
            spans.push(Span::styled(clock(position), Style::default().fg(MUTED)));
            if duration > 0 {
                let rest = if remaining {
                    format!(" -{}", clock(duration.saturating_sub(position)))
                } else {
                    String::new()
                };
                spans.push(Span::styled(format!(" / {}{rest}", clock(duration)), Style::default().fg(DIM)));
            }
        }
        spans
    };
    // the left: what plays, its counter, then the next after a rule
    let left = |title: &str, counter: bool, next: Option<(bool, bool)>| -> Vec<Span<'static>> {
        let mut spans = vec![
            Span::styled(play_mark.clone(), Style::default().fg(PLAYING)),
            Span::styled(title.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" — ", Style::default().fg(DIM)),
            Span::styled(stop.artist.clone(), Style::default().fg(CATALOG)),
        ];
        if counter {
            spans.push(Span::styled(format!("  ({rank} / {tracks})"), Style::default().fg(DIM)));
        }
        spans.push(Span::styled(loading.to_string(), Style::default().fg(VECTOR)));
        if let (Some(next), Some((word, artist))) = (view.next, next) {
            spans.push(Span::styled(" │ ", Style::default().fg(DIM)));
            if word {
                spans.push(Span::styled("then ", Style::default().fg(DIM)));
            }
            spans.push(Span::styled(format!("{} ", next.source.mark()), Style::default().fg(role_of(next.source))));
            spans.push(Span::styled(next.title.clone(), Style::default().fg(MUTED)));
            if artist {
                spans.push(Span::styled(format!(" — {}", next.artist), Style::default().fg(DIM)));
            }
        }
        spans
    };
    // the steps of the sacrifice, most complete first
    const STEPS: [(bool, bool, bool, bool); 4] = [
        // (next's artist, "then" and the counter, source and remaining, next at all)
        (true, true, true, true),
        (false, true, true, true),
        (false, false, true, true),
        (false, false, false, true),
    ];
    // below this, the current title is no longer readable: the next track
    // leaves the line before the title gets that short
    const TITLE_MIN: usize = 16;
    let title_len = stop.title.chars().count();
    for (next_artist, words, source, _) in STEPS {
        let l = left(&stop.title, words, Some((words, next_artist)));
        let r = right(source, source);
        if count(&l) + 2 + count(&r) <= width {
            return (justified(l, r, width), true);
        }
    }
    // step 4: the title gives, alone, as long as it stays readable
    let r = right(false, false);
    let fixed = count(&left("", false, Some((false, false)))) + 2 + count(&r);
    if width > fixed && width - fixed >= TITLE_MIN.min(title_len) {
        let room = width - fixed;
        return (justified(left(&fit(&stop.title, room), false, Some((false, false))), r, width), true);
    }
    // step 5: the next leaves the line — then the title alone gives, and
    // if even that is not enough, the artist goes rather than a second cut
    let fixed = count(&left("", false, None)) + 2 + count(&r);
    if width > fixed {
        return (justified(left(&fit(&stop.title, width - fixed), false, None), r, width), false);
    }
    let alone = vec![
        Span::styled(play_mark, Style::default().fg(PLAYING)),
        Span::styled(
            fit(&stop.title, width.saturating_sub(2 + 2 + count(&r))),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ];
    (justified(alone, r, width), false)
}

/// The progress, full width, like waybar's media module — the last line
/// of the foot, nothing under it (mockup 4a).
fn render_progress(frame: &mut ratatui::Frame, bar: Rect, view: &Bar) {
    let full = bar.width as usize;
    let bar_line = match view.progress {
        Some((position, duration)) if duration > 0 && view.current.is_some() => {
            let filled = (position as u64 * full as u64 / duration as u64) as usize;
            // one tone for the whole bar, as in the mockup: the rest reads
            // as a dotted trail, not as a dim line (Joel, 10/09/2026)
            Line::from(vec![
                Span::styled("█".repeat(filled.min(full)), Style::default().fg(VECTOR)),
                Span::styled("░".repeat(full.saturating_sub(filled)), Style::default().fg(VECTOR)),
            ])
        }
        _ => Line::from(Span::styled("░".repeat(full), Style::default().fg(DIM))),
    };
    frame.render_widget(Paragraph::new(bar_line), bar);
}

/// Wraps a text into lines of at most `width` characters, on spaces. A
/// reason folds (it is prose), a track gets cut (it is a list): that is why
/// the pane does not leave wrapping to ratatui.
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

/// What the proximity gauge shows under a branch: the link word for the
/// graph, the cosine for the vector space — two natures, two colors, as
/// everywhere else.
fn proximity_of(branch: &Branch) -> (Color, String, Option<String>) {
    let cells: String = (0..5)
        .map(|i| if (i as f32) < branch.weight.round() { '█' } else { '░' })
        .collect();
    if branch.reason.starts_with("close to") {
        // the adventurous branch: the engine put the cosine on the 1–5 scale
        let cosine = branch.weight / 5.0;
        (VECTOR, cells, Some(format!("{cosine:.2}")))
    } else {
        // the graph: the typed link is the first word of the reason
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

/// The branches column (1a): always there, over the full height, each
/// branch unfolded with its tracks — they are already drawn, might as well
/// show them so one chooses knowingly (Joel, 07/09/2026). A rule on the
/// left separates it from the axis; it is the only rule it draws.
fn render_panel(frame: &mut ratatui::Frame, column: Rect, view: &View) {
    // the rule, then one cell of margin: content starts at x + 2
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
        // the gaps are counted apart: they don't play yet
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
            "dead end — fu to go back",
            Style::default().fg(MUTED),
        )));
    }
    for (i, branch) in view.branches.iter().enumerate() {
        // "1  label" — 2 cells of indentation, as the PoC prints it
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
        // the reason, wrapped at 5 cells; when it is only the link word
        // ("stay within the universe"), the gauge does not repeat it
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
        // the tracks, as they will play: we choose what we will hear
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
        // the reason is grey, its word under the gauge too (Joel, 07/09/2026):
        // only the cells tell the nature of the link
        if let Some(word) = word {
            gauge.push(Span::styled(format!(" {word}"), Style::default().fg(MUTED)));
        }
        lines.push(Line::from(gauge));
        lines.push(Line::from(""));
    }

    // the gaps: a catalog link to a card that does not exist yet. They are
    // numbered after the others, in grey, and the empty circle says they
    // must be generated before being walked (0016).
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
        // the same gauge as the branches, in grey: the link's proximity is
        // known, it is the card that is missing
        let cells: String =
            (0..5).map(|i| if i < missing.proximity { '█' } else { '░' }).collect();
        lines.push(Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(cells, Style::default().fg(DIM)),
            Span::styled(
                if missing.pending {
                    format!(" {} · … generation", missing.kind)
                } else {
                    format!(" {} · ○ no card yet", missing.kind)
                },
                Style::default().fg(if missing.pending { BRANCH } else { MUTED }),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // the keys, at the foot of the column — two lines that never move
    let hints = [
        Line::from(vec![
            Span::styled("1-3", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" take  ", Style::default().fg(MUTED)),
            Span::styled("fr", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" reroll", Style::default().fg(MUTED)),
        ]),
        Line::from(vec![
            Span::styled("fn1", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" without waiting for the end", Style::default().fg(DIM)),
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

/// A home line. Home decides **what** to say, the TUI **how** — the same
/// separation as between the engine and the sound.
pub enum Row {
    Rule(String),
    Text(String),
    Dim(String),
    /// A numbered door: numbering runs across the blocks, so choosing a
    /// seed is the gesture that chooses a branch. `artist` is only filled
    /// when the seed is a **track** — the title goes first, the artist
    /// behind, as everywhere else.
    Entry {
        n: usize,
        label: String,
        artist: Option<String>,
        reason: String,
        tracks: Vec<(String, String)>,
    },
    /// A key and what it does; `wired` false shows it dimmed, never as if
    /// it worked.
    Key { key: String, what: String, note: String, wired: bool },
}

pub struct HomeView<'a> {
    /// The name on the left, the authorizations state on the right, on
    /// **a single line** (Joel, 06/09/2026).
    pub status: Vec<(String, bool)>,
    pub census: String,
    pub rows: &'a [Row],
    pub prompt: String,
    pub comfort: u8,
    /// `cc`: the dial is open, the gauge lights up (Joel, 11/09/2026 —
    /// the dial did nothing on home).
    pub comfort_mode: bool,
    pub comfort_word: &'a str,
    /// The right column. It proposes nothing, it lists.
    pub collection: Option<Collection<'a>>,
    /// The playback foot, when a session plays under home.
    pub bar: Option<Bar<'a>>,
    /// The search modal, when open: `:search` also opens from home (Joel,
    /// 09/09/2026).
    pub finder: Option<FinderView>,
    /// The discography, when `ad` opened it from the collection
    /// (Joel, 10/09/2026).
    pub explore: Option<&'a crate::explore::Explore>,
    /// The input help (space), laid over everything.
    /// Title, lines, and how far it is scrolled (j/k, ↑↓ — Joel, 20/09/2026).
    pub overlay: Option<(&'a str, &'a [String], usize)>,
    /// The toast: **every notification shows as a toast**, under home as
    /// while listening (Joel, 10/09/2026) — nothing is said on the bottom
    /// line anymore.
    pub toast: Option<Toast>,
}

impl Tui {
    pub fn draw_home(&mut self, view: &HomeView) -> std::io::Result<()> {
        self.terminal.draw(|frame| render_home(frame, view))?;
        Ok(())
    }
}

fn render_home(frame: &mut ratatui::Frame, view: &HomeView) {
    let area = frame.area();
    // the playback foot takes its two lines when a session plays (4a)
    let foot = if view.bar.is_some() { 2 } else { 0 };
    let [head, whole, foot_area, prompt] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(foot),
        Constraint::Length(1),
    ])
    .areas(area);
    if let Some(bar) = &view.bar {
        let [now, progress] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(foot_area);
        let (line, _) = listening_line(bar, now.width as usize);
        frame.render_widget(Paragraph::new(line), now);
        render_progress(frame, progress, bar);
    }

    // on the left what forkstify proposes, on the right what it owns. The
    // split is proportional (Joel, 06/09/2026): the left carries reasons
    // and tracks, the right a list — hence 60/40 rather than half-half.
    // Below `SPLIT_MIN`, the list disappears.
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

    // the wordmark on the left, the comfort dial on the right — the same
    // place and look as while listening (Joel, 14/09/2026). The status
    // indicators used to sit here; they only earn their place when
    // something is wrong, so they drop to the second line, and only then.
    let comfort = comfort_spans(view.comfort, view.comfort_word, view.comfort_mode);
    let comfort_width: usize = comfort.iter().map(|s| s.content.chars().count()).sum();
    let gap = (head.width as usize)
        .saturating_sub("forkstify".len() + comfort_width)
        .max(2);
    let mut title = vec![
        Span::styled(
            "forkstify",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
    ];
    title.extend(comfort);
    // the second line: the census, preceded by anything degraded (red)
    let mut second = Vec::new();
    for (text, ok) in view.status.iter().filter(|(_, ok)| !ok) {
        second.push(Span::styled(text.clone(), Style::default().fg(Color::Red)));
        let _ = ok;
        second.push(Span::styled("  ", Style::default().fg(DIM)));
    }
    second.push(Span::styled(view.census.clone(), Style::default().fg(DIM)));
    frame.render_widget(
        Paragraph::new(vec![Line::from(title), Line::from(second)]),
        head,
    );

    let mut lines: Vec<Line> = Vec::new();
    for row in view.rows {
        match row {
            // the border runs to the end of the measure: it is what
            // separates the blocks, since there are no cards
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
                // the seed is a track: the title first, the artist behind,
                // as on every forkstify line
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

    // — search takes the whole body of home, collection included: it is
    // what we look at while it is open
    if let Some(finder) = &view.finder {
        render_finder(frame, whole, finder);
    }
    // — the discography, likewise
    if let Some(screen) = view.explore {
        render_explore(frame, whole, screen);
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(view.prompt.clone(), Style::default().fg(MUTED)))),
        prompt,
    );

    // — the toast, at the bottom right of the body, over the collection or
    // the modal: the same rule as while listening, everything is a toast
    if let Some(toast) = &view.toast {
        render_toast(frame, whole, toast);
    }

    // — and what sits over everything: the input help
    if let Some((title, body, scroll)) = view.overlay {
        render_block(frame, area, title, body, scroll);
    }
}

/// A block laid over the screen — the leader menu, "?". It replaces the
/// log that grew downwards: what is long is shown, what is short is said.
/// A line of an overlay. The track-info window (`ta`) reads better with a
/// little colour: its labels are tinted and their values brighten, while a
/// line that is not "label: value" — the help menu's rows — stays muted, as
/// before (Joel, 14/09/2026).
fn info_line(text: &str) -> Line<'static> {
    const LABELS: [&str; 5] = ["featuring", "album", "tags", "from here", "off-catalog"];
    let trimmed = text.trim_start();
    let indent = text[..text.len() - trimmed.len()].to_string();
    for label in LABELS {
        if let Some(rest) = trimmed.strip_prefix(label) {
            if let Some(value) = rest.strip_prefix(':') {
                return Line::from(vec![
                    Span::raw(indent),
                    Span::styled(label.to_string(), Style::default().fg(VECTOR).add_modifier(Modifier::BOLD)),
                    Span::styled(":".to_string(), Style::default().fg(DIM)),
                    Span::styled(value.to_string(), Style::default().fg(Color::Reset)),
                ]);
            }
        }
    }
    // the catalog overlays (Cd, Cp, Cu, :catalog — `Catalogue.dc.html`):
    // a heading, a new card, an edited one, a labelled value, a glyph line
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return Line::from(vec![
            Span::raw(indent),
            Span::styled(rest.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]);
    }
    if let Some(rest) = trimmed.strip_prefix("+ ").or_else(|| trimmed.strip_prefix("~ ")) {
        let (glyph, tone) = if trimmed.starts_with('+') { ("+ ", PLAYING) } else { ("~ ", EDIT) };
        let (name, tail) = match rest.find("  ") {
            Some(cut) => (&rest[..cut], &rest[cut..]),
            None => (rest, ""),
        };
        return Line::from(vec![
            Span::raw(indent),
            Span::styled(glyph.to_string(), Style::default().fg(tone).add_modifier(Modifier::BOLD)),
            Span::styled(name.to_string(), Style::default().fg(Color::White)),
            Span::styled(tail.to_string(), Style::default().fg(DIM)),
        ]);
    }
    if let Some(rest) = trimmed.strip_prefix("- ") {
        let (slug, tail) = match rest.find(" · ").or_else(|| rest.find(" +")) {
            Some(cut) => (&rest[..cut], &rest[cut..]),
            None => (rest, ""),
        };
        return Line::from(vec![
            Span::raw(indent),
            Span::styled("- ".to_string(), Style::default().fg(DIM)),
            Span::styled(slug.to_string(), Style::default().fg(CATALOG)),
            Span::styled(tail.to_string(), Style::default().fg(MUTED)),
        ]);
    }
    const KEYS: [&str; 8] = ["from", "title", "origin", "upstream", "cards", "source:", "gh pr create", "Forkstify:"];
    for key in KEYS {
        if let Some(rest) = trimmed.strip_prefix(key) {
            if rest.starts_with(' ') || rest.is_empty() {
                let tone = if key == "Forkstify:" { DIM } else if key == "gh pr create" { BRANCH } else { Color::White };
                return Line::from(vec![
                    Span::raw(indent),
                    Span::styled(key.to_string(), Style::default().fg(if key == "Forkstify:" { DIM } else { CATALOG })),
                    Span::styled(rest.to_string(), Style::default().fg(tone)),
                ]);
            }
        }
    }
    if trimmed.starts_with(['✓', '⏹', '⊘', '↻', '→', '⇅']) {
        return Line::from(vec![Span::raw(indent), Span::styled(trimmed.to_string(), Style::default().fg(tone_of(trimmed)))]);
    }
    // a hint that opens with a key: "Cd the detail · Cp propose", "o open…"
    for key in ["Cd ", "Cp ", "Cu ", "o ", ":catalog ", "git merge"] {
        if let Some(rest) = trimmed.strip_prefix(key) {
            let mut spans = vec![Span::raw(indent), Span::styled(key.to_string(), Style::default().fg(BRANCH).add_modifier(Modifier::BOLD))];
            for (i, part) in rest.split(" · ").enumerate() {
                if i > 0 {
                    spans.push(Span::styled(" · ".to_string(), Style::default().fg(DIM)));
                }
                match part.split_once(' ').filter(|(k, _)| ["Cd", "Cp", "Cu", "o"].contains(k)) {
                    Some((k, what)) => {
                        spans.push(Span::styled(format!("{k} "), Style::default().fg(BRANCH).add_modifier(Modifier::BOLD)));
                        spans.push(Span::styled(what.to_string(), Style::default().fg(MUTED)));
                    }
                    None => spans.push(Span::styled(part.to_string(), Style::default().fg(MUTED))),
                }
            }
            return Line::from(spans);
        }
    }
    // the metrics line: "familiarity 50% · weight 1.20 · 3 link(s), 2 top(s)"
    if trimmed.starts_with("familiarity") {
        let spans: Vec<Span> = trimmed
            .split(" · ")
            .enumerate()
            .flat_map(|(i, part)| {
                let sep = if i > 0 {
                    vec![Span::styled(" · ".to_string(), Style::default().fg(DIM))]
                } else {
                    vec![Span::raw(indent.clone())]
                };
                sep.into_iter().chain(std::iter::once(Span::styled(
                    part.to_string(),
                    Style::default().fg(EDIT),
                )))
            })
            .collect();
        return Line::from(spans);
    }
    Line::from(Span::styled(text.to_string(), Style::default().fg(MUTED)))
}

/// A block laid over the screen. Longer than the screen, it **scrolls**:
/// `scroll` is the first line shown, and the title says what is above
/// and below — a `Cd` on a fork a month old is a hundred lines (Joel,
/// 20/09/2026).
fn render_block(frame: &mut ratatui::Frame, area: Rect, title: &str, body: &[String], scroll: usize) {
    // as wide as its longest line asks, from 64 columns up to the screen:
    // a diff line — name, kind, tags, links — must not fold in two (Joel,
    // 20/09/2026); the key helper stays narrow
    let longest = body.iter().map(|line| line.chars().count()).max().unwrap_or(0) as u16;
    let width = longest.saturating_add(4).max(64).min(area.width.saturating_sub(4));
    let height = (body.len() as u16 + 2).min(area.height.saturating_sub(2));
    let rect = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(height + 2),
        width,
        height,
    };
    frame.render_widget(Clear, rect);
    let visible = height.saturating_sub(2) as usize;
    let scroll = scroll.min(body.len().saturating_sub(visible));
    let below = body.len().saturating_sub(scroll + visible);
    let mut heading = format!(" {title} ");
    if scroll > 0 || below > 0 {
        heading.push_str(&format!("— ↑ {scroll} · ↓ {below} · j/k "));
    }
    let lines: Vec<Line> = body.iter().map(|text| info_line(text)).collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Span::styled(heading, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }).scroll((scroll as u16, 0)),
        rect,
    );
}

/// How far an overlay of `len` lines may scroll: to the point where its
/// last lines fill the block, never past them.
pub fn overlay_scroll_max(len: usize, screen_height: u16) -> usize {
    let visible = screen_height.saturating_sub(4) as usize;
    len.saturating_sub(visible.max(1))
}

impl Tui {
    /// A waiting screen: what forkstify is doing, while it does it. Nothing
    /// must print outside the TUI — the alternate screen is its own, and a
    /// `println!` leaves leftovers there it cannot erase.
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

    /// Start over from a blank screen. Call it when switching views:
    /// ratatui only redraws what it believes has changed.
    pub fn clear(&mut self) {
        let _ = self.terminal.clear();
    }
}

/// A collection line: the familiarity gauge, the name, what we know about
/// it, and how long since it last played.
#[derive(Clone)]
pub struct CollectionRow {
    pub familiarity: u8,
    /// How many days since it last played — used for sorting, not for
    /// display, which shows `age`.
    pub days: Option<i64>,
    pub name: String,
    /// True if the artist has a card — that is what decides whether it can
    /// be a seed, since the branches come from the card.
    pub carded: bool,
    pub age: String,
    /// An old play stands out: it is a neglected one.
    pub neglected: bool,
}

pub struct Collection<'a> {
    pub rows: &'a [CollectionRow],
    pub total: usize,
    pub carded: usize,
    pub cursor: Option<usize>,
    pub sort: &'a str,
    /// "liked" or "all" — what the list shows
    pub scope: &'a str,
}

/// The right column: it **proposes nothing, it lists**. It is the
/// counterpart of the doors — for whoever wants to choose themselves.
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
                    "the collection",
                    Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} · {} with a card", view.total, view.carded),
                    Style::default().fg(DIM),
                ),
            ]),
            Line::from(vec![
                Span::styled("view: ", Style::default().fg(DIM)),
                Span::styled(view.scope.to_string(), Style::default().fg(CATALOG)),
                Span::styled(" (v) · sorted by ", Style::default().fg(DIM)),
                Span::styled(view.sort.to_string(), Style::default().fg(CATALOG)),
                Span::styled(" (s)", Style::default().fg(DIM)),
            ]),
        ]),
        head,
    );

    // we follow the cursor rather than the start of the list
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
                if rest > 0 { format!("── {rest} more") } else { "──".into() },
                Style::default().fg(DIM),
            )),
            Line::from(Span::styled(
                "↑↓ browse · gg G the ends · enter start · s sort",
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
                reason: "shared members — Siouxsie Sioux and Budgie · shared tags: post-punk, uk"
                    .to_string(),
                artists: vec!["the-creatures".to_string()],
                stops: vec![stop("Right Now", "The Creatures"), stop("Miss the Girl", "The Creatures")],
                weight: 5.0,
            },
            Branch {
                label: "Chelsea Wolfe".to_string(),
                reason: "close to the branch's center (0.78) · shared tags: uk".to_string(),
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
            why: "similar · around Jacques Brel".into(),
            pending,
        }
    }

    /// 0016: a link to a missing card is proposed instead of discarded,
    /// numbered **after** the branches — otherwise the numbers would lie.
    #[test]
    fn gaps_are_numbered_after_the_branches() {
        let current = stop("Ne me quitte pas", "Jacques Brel");
        let rows =
            playlist(100, 38, &branches(), &[], Some(&current), &[], &[missing(false)]);
        let text = rows.join("\n");
        assert!(text.contains("2  Chelsea Wolfe"), "{text}");
        assert!(text.contains("── branches 2 · 1 ○"), "{text}");
        assert!(text.contains("3  Georges Moustaki"), "{text}");
        assert!(text.contains("similar · around Jacques Brel"), "{text}");
        assert!(text.contains("████░ similar · ○ no card yet"), "{text}");
    }

    /// A generation takes a few seconds: the column must say so, otherwise
    /// "f3" looks like it did nothing.
    #[test]
    fn a_gap_being_generated_says_so() {
        let current = stop("Ne me quitte pas", "Jacques Brel");
        let rows = playlist(100, 30, &[], &[], Some(&current), &[], &[missing(true)]);
        let text = rows.join("\n");
        assert!(text.contains("similar · … generation"), "{text}");
        assert!(!text.contains("○ no card yet"), "{text}");
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
            .map(|i| if i == 0 { "1 play · yesterday".to_string() } else { "never played".to_string() })
            .collect();
        let view = View {
            path: vec!["The Cure".to_string(), "Siouxsie and the Banshees".to_string()],
            seed: "the-cure",
            seed_name: "The Cure",
            seed_facts: "written card · 41 links · 12 tops",
            seed_last: "last played -3s",
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
            comfort_word: "balanced",
            progress: current.map(|_| (154_000, 227_000)),
            toast: None,
            finder: None,
            explore: None,
            prompt: "[1-2 branch]".to_string(),
        };
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect()
    }

    /// The discography modal: the albums folded, the cursor's one open, and
    /// the batch waiting for its commit (mockup 1a).
    #[test]
    fn the_discography_folds_albums_and_says_what_is_pending() {
        let card: crate::catalog::Card = toml::from_str(
            "format = 1\nname = \"Cat Power\"\nmbid = \"x\"\ntops = [\"Cross Bones Style\"]\n",
        )
        .expect("card");
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
        // first album open, cursor on its second track; "A" promotes the
        // most played non-top track of the album
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

        assert!(lines[0].contains("discography") && lines[0].contains("Cat Power"));
        // both albums fit, and only the cursor's one is unfolded
        assert!(lines.iter().any(|l| l.contains("▾") && l.contains("Moon Pix")));
        assert!(lines.iter().any(|l| l.contains("▸") && l.contains("The Covers Record")));
        assert!(lines.iter().any(|l| l.contains("Metal Heart") && l.contains("▶ playing")));
        // what is pending is visible, and the commit is announced before
        // pressing
        assert!(lines.iter().any(|l| l.contains("pending") && l.contains("1 edit")));
        // the commit subject reads before pressing
        assert!(lines.iter().any(|l| l.contains("cards/cat-power.toml") && l.contains("+1 −0")));
        assert!(lines.iter().any(|l| l.contains("⏎ write")));
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
        assert!(text.contains("█████ shared members"), "{text}");
        assert!(text.contains("████░ 0.78"), "{text}");
        // the rule runs the whole body height (below the head and the seed
        // block, above the two-line foot and the prompt), the hints sit at
        // its foot
        let body_rows = 5..(30 - 3);
        assert!(body_rows.clone().all(|y| rows[y].contains('│')), "{text}");
        assert!(rows[26].contains("fn1 without waiting for the end"), "{text}");
        assert!(rows[25].contains("1-3 take"), "{text}");
    }

    #[test]
    fn a_bare_reason_is_said_once_by_the_gauge() {
        let mut around = branches();
        around[0].reason = "stay within the journey's universe".to_string();
        around[0].weight = 4.0;
        // 110 columns: the reason (34 cells) must fit on one line of the
        // column, otherwise it folds and reads in two pieces
        let text = screen(110, 30, &around).join("\n");
        assert_eq!(text.matches("stay within the journey's universe").count(), 1, "{text}");
        assert!(text.contains("████░ \n") || text.contains("████░  "), "{text}");
    }

    #[test]
    fn the_playlist_numbers_what_sounds_and_what_comes() {
        let past = [stop("A Forest", "The Cure"), stop("Push", "The Cure")];
        let current = headed(
            stop("Cities in Dust", "Siouxsie and the Banshees"),
            "Siouxsie and the Banshees",
            "family ties — Robert Smith played guitar there in 1983",
        );
        let mut israel = stop("Israel", "Siouxsie and the Banshees");
        israel.encore = true;
        let queue = [
            israel,
            headed(stop("Right Now", "The Creatures"), "The Creatures", "shared members"),
            headed(stop("Alison", "Slowdive"), "Slowdive", "close to the branch's center (0.74)"),
        ];
        let rows = playlist(160, 30, &branches(), &past, Some(&current), &queue, &[]);
        let text = rows.join("\n");
        let axis = |needle: &str| -> String {
            let row = rows.iter().find(|r| r.contains(needle)).unwrap();
            row.chars().take(95).collect::<String>().trim_end().to_string()
        };
        // the past is not numbered, the seed reads next to the first one
        assert!(text.contains("      ♪ A Forest — The Cure  seed: the-cure"), "{text}");
        assert!(axis("A Forest").ends_with("1 play · yesterday"), "{}", axis("A Forest"));
        // what is playing is 1, what comes counts from it
        assert!(text.contains(" 1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees   family ties"), "{text}");
        assert!(axis("Cities in Dust").ends_with("never played"), "{}", axis("Cities in Dust"));
        // a replay carries "↻" instead of the arrow
        assert!(text.contains(" 2 ↻  ♪ Israel — Siouxsie and the Banshees"), "{text}");
        assert!(text.contains(" 3 →  ♪ Right Now — The Creatures  shared members"), "{text}");
        assert!(text.contains(" 4 →  ♪ Alison — Slowdive  close to the branch's center (0.74)"), "{text}");
        assert!(axis("Alison").ends_with("never played"), "{}", axis("Alison"));
        assert!(text.contains(" 5    horizon  nothing drawn beyond"), "{text}");
        // no more rule (Joel, 07/09/2026): the tracks follow each other, only
        // the horizon takes a blank
        assert!(!text.contains('╵') && !text.contains(" │ ♪"), "{text}");
        let horizon = rows.iter().position(|r| r.contains("horizon")).unwrap();
        assert!(rows[horizon - 1].chars().take(95).all(|c| c == ' '), "{text}");
        assert!(rows[horizon - 2].contains("Alison"), "{text}");
        // the head and the seed block (2b)
        assert!(rows[0].starts_with("forkstify listen the-cure"), "{}", rows[0]);
        assert!(rows[0].trim_end().ends_with("segment 2 · 6 tracks · → fork in 3 │ comfort 3 ███░░ balanced"), "{}", rows[0]);
        assert!(rows[2].starts_with("── seed ─"), "{}", rows[2]);
        assert!(rows[3].starts_with("The Cure  [catalog]  written card · 41 links · 12 tops  last played -3s"), "{}", rows[3]);
        assert!(rows[4].starts_with("1 fork so far — 2 artists traversed"), "{}", rows[4]);
        // the foot (4a): what is playing, then what comes next, where it
        // comes from and the clock — one line, the bar closes it
        let now = &rows[rows.len() - 3];
        assert!(
            now.starts_with("▶ Cities in Dust — Siouxsie and the Banshees  (3 / 6) │ then ♪ Israel — Siouxsie and the Banshees"),
            "{now}"
        );
        assert!(now.trim_end().ends_with("♪ top │ 2:34 / 3:47 -1:13"), "{now}");
        // the bar: 154 s out of 227, i.e. 108 full cells out of 160
        let bar = &rows[rows.len() - 2];
        assert_eq!(bar.chars().filter(|c| *c == '█').count(), 108, "{bar}");
        assert_eq!(bar.chars().filter(|c| *c == '░').count(), 52, "{bar}");
        // and the prompt is the last line: nothing between the bar and it
        assert!(rows[rows.len() - 1].starts_with("[1-2 branch]"), "{}", rows[rows.len() - 1]);
        // the next is on the line, so the list does not mark it
        assert!(!text.contains('▸'), "{text}");
    }

    /// The worst case of mockup 4a′: a long title, a long artist, a window
    /// that narrows. The right block never gives; the left gives in order,
    /// one cut at most per line, the next track whole or absent.
    #[test]
    fn the_listening_line_gives_on_the_left_in_order() {
        let current = stop("Storm: Lift Your Skinny Fists Like Antennas to Heaven", "Godspeed You! Black Emperor");
        let next = stop("Idiot Heart", "Sunset Rubdown");
        let bar = Bar {
            current: Some(&current),
            paused: false,
            loading: false,
            progress: Some((108_000, 1_352_000)),
            position: (2, 10),
            next: Some(&next),
        };
        let at = |width: usize| -> (String, bool) {
            let (line, shown) = listening_line(&bar, width);
            (line.spans.iter().map(|s| s.content.to_string()).collect::<String>().trim_end().to_string(), shown)
        };
        // 1. everything fits, nothing gives
        let (line, shown) = at(170);
        assert!(shown);
        assert!(line.starts_with("▶ Storm: Lift Your Skinny Fists Like Antennas to Heaven — Godspeed You! Black Emperor  (2 / 10) │ then ♪ Idiot Heart — Sunset Rubdown"), "{line}");
        assert!(line.ends_with("♪ top │ 1:48 / 22:32 -20:44"), "{line}");
        assert_eq!(line.chars().count(), 170);
        // 2. the next's artist falls, the title stays whole
        let (line, shown) = at(145);
        assert!(shown);
        assert!(line.contains("(2 / 10) │ then ♪ Idiot Heart") && !line.contains("Sunset Rubdown"), "{line}");
        assert!(line.ends_with("♪ top │ 1:48 / 22:32 -20:44"), "{line}");
        // 3. "then" and the counter go, then the source and the remaining
        let (line, shown) = at(138);
        assert!(shown);
        assert!(line.contains("Black Emperor │ ♪ Idiot Heart") && !line.contains("(2 / 10)"), "{line}");
        assert!(line.ends_with("♪ top │ 1:48 / 22:32 -20:44"), "{line}");
        let (line, _) = at(120);
        assert!(line.ends_with("1:48 / 22:32") && !line.contains("top"), "{line}");
        assert!(line.contains("Antennas to Heaven — Godspeed"), "{line}");
        // 4. the current title is cut — one ellipsis, the next still whole
        let (line, shown) = at(96);
        assert!(shown);
        assert!(line.contains("…") && line.contains("│ ♪ Idiot Heart"), "{line}");
        assert_eq!(line.matches('…').count(), 1, "{line}");
        assert_eq!(line.chars().count(), 96);
        // 5. the next leaves the line rather than the title going unreadable
        let (line, shown) = at(72);
        assert!(!shown);
        assert!(!line.contains("Idiot Heart") && line.contains("— Godspeed You! Black Emperor"), "{line}");
        assert_eq!(line.matches('…').count(), 1, "{line}");
        assert!(line.ends_with("1:48 / 22:32"), "{line}");
        assert_eq!(line.chars().count(), 72);
    }

    /// When the next track leaves the listening line, the list says it
    /// with "▸" in the gutter of the track that comes (4a′, the fallback
    /// of 4b).
    #[test]
    fn a_narrow_screen_marks_the_next_in_the_list() {
        let current = stop("Cities in Dust", "Siouxsie and the Banshees");
        let mut israel = stop("Israel", "Siouxsie and the Banshees");
        israel.encore = true;
        let queue = [israel, stop("Right Now", "The Creatures")];
        // wide: the next is named on the listening line, the list is bare
        let rows = playlist(100, 30, &branches(), &[], Some(&current), &queue, &[]);
        let text = rows.join("\n");
        assert!(text.contains(" 2 ↻  ♪ Israel"), "{text}");
        assert!(rows[rows.len() - 3].contains("│ then ♪ Israel"), "{}", rows[rows.len() - 3]);
        // narrow: it leaves the line and the gutter says it
        let rows = playlist(60, 30, &branches(), &[], Some(&current), &queue, &[]);
        let text = rows.join("\n");
        assert!(text.contains(" 2 ▸↻ ♪ Israel"), "{text}");
        assert!(text.contains(" 3 →  ♪ Right Now"), "{text}");
        assert!(!rows[rows.len() - 3].contains("Israel"), "{}", rows[rows.len() - 3]);
    }

    #[test]
    fn a_notice_is_shaped_by_its_nature() {
        let plain = |line: Line| -> String { line.spans.iter().map(|s| s.content.to_string()).collect() };
        let done = notice_line("✓ A Forest promoted to top — committed");
        assert_eq!(done.spans[0].style.fg, Some(PLAYING));
        assert_eq!(done.spans[1].style.fg, Some(DIM));
        assert_eq!(plain(done), "✓ A Forest promoted to top — committed");
        let banned = notice_line("\n⊘ The Fall — never again");
        assert_eq!(banned.spans[0].style.fg, Some(DANGER));
        let hint = notice_line("(no more unplayed tops around The Cure)");
        assert_eq!(hint.spans.len(), 1);
        assert_eq!(hint.spans[0].style.fg, Some(MUTED));
        let later = notice_line("fw — leave: decided (0015), not wired yet.");
        assert!(later.spans[0].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(plain(notice_line("")), "");
    }

    /// Home keeps the playback foot when a session plays underneath (Joel,
    /// 08/09/2026): what is playing and what comes next, then its bar, on
    /// the two lines above the prompt (4a).
    #[test]
    fn the_home_keeps_the_playback_foot() {
        let current = stop("Cities in Dust", "Siouxsie and the Banshees");
        let next = stop("Israel", "Siouxsie and the Banshees");
        let view = HomeView {
            status: vec![("✓ librespot".to_string(), true)],
            census: "local catalog".to_string(),
            rows: &[],
            prompt: "[1-3 to start · r back to listening · q quit]".to_string(),
            comfort: 3,
            comfort_mode: false,
            comfort_word: "balanced",
            collection: None,
            finder: None,
            explore: None,
            overlay: None,
            toast: None,
            bar: Some(Bar {
                current: Some(&current),
                paused: false,
                loading: false,
                progress: Some((60_000, 240_000)),
                position: (2, 5),
                next: Some(&next),
            }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| render_home(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..20).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        assert!(rows[17].starts_with("▶ Cities in Dust — Siouxsie and the Banshees  (2 / 5) │ then ♪ Israel"), "{}", rows[17]);
        assert_eq!(rows[18].chars().filter(|c| *c == '█').count(), 25, "{}", rows[18]);
        // and the keys follow the bar with nothing in between
        assert!(rows[19].starts_with("[1-3 to start · r back to listening"), "{}", rows[19]);
    }

    /// `:search` also opens from home (Joel, 09/09/2026): the modal covers
    /// the body — collection included — and leaves the playback foot and
    /// the prompt.
    #[test]
    fn the_home_says_everything_in_a_toast() {
        // "every notification must show as a toast" (Joel, 10/09/2026):
        // under home too, what is said sits in a box over the body, and the
        // bottom line keeps its keys
        let rows_of_home = [Row::Rule("search".into())];
        let view = HomeView {
            status: vec![("✓ librespot".to_string(), true)],
            census: "local catalog".to_string(),
            rows: &rows_of_home,
            prompt: "[1-3 to start · q]".to_string(),
            comfort: 3,
            comfort_mode: false,
            comfort_word: "balanced",
            collection: None,
            explore: None,
            overlay: None,
            finder: None,
            bar: None,
            toast: Some(Toast { text: "(unknown: zz)".into(), tone: MUTED, sticky: false }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| render_home(frame, &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..20).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        assert!(rows.iter().any(|row| row.contains("(unknown: zz)")), "{rows:?}");
        assert!(rows[19].starts_with("[1-3 to start · q]"), "{}", rows[19]);
    }

    #[test]
    fn the_home_wears_the_finder() {
        let rows_of_home = [Row::Rule("search".into())];
        let view = HomeView {
            status: vec![("✓ librespot".to_string(), true)],
            census: "local catalog".to_string(),
            rows: &rows_of_home,
            prompt: "[1-3 to start · q]".to_string(),
            comfort: 3,
            comfort_mode: false,
            comfort_word: "balanced",
            collection: None,
            explore: None,
            overlay: None,
            toast: None,
            finder: Some(FinderView {
                insert: false,
                linking: None,
                anchor: None,
                query: "siou".to_string(),
                counts: (1, None, true),
                only_catalogue: false,
                lines: vec![FinderLine::Row {
                    catalogue: true,
                    mark: '♪',
                    title: "Cities in Dust".into(),
                    artist: "Siouxsie and the Banshees".into(),
                    note: "3 plays".into(),
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
        // the body belongs to the modal: no more home "search" underneath
        assert!(rows[2].starts_with("┌─ search ── :search"), "{}", rows[2]);
        assert!(rows[3].starts_with("│ ⟩ siou"), "{}", rows[3]);
        assert!(rows.iter().any(|row| row.contains("Cities in Dust")), "{rows:?}");
        assert!(!rows.iter().any(|row| row.contains("── search")), "{rows:?}");
        // the prompt stays, with its comfort gauge
        assert!(rows[19].starts_with("[1-3 to start · q]"), "{}", rows[19]);
    }

    /// The search modal: the input, the rule that counts, the two groups
    /// never mixed, the cursor, and the "ti" anchor.
    #[test]
    fn the_finder_cuts_the_input_from_the_results() {
        let view = FinderView {
            insert: true,
            linking: None,
            anchor: Some("the insertion lands at 4 — between Sea Of Love and Cross Bones Style".to_string()),
            query: "nothing bu".to_string(),
            counts: (2, None, true),
            only_catalogue: false,
            lines: vec![
                FinderLine::Header { catalogue: true, text: "catalog  2 results".to_string() },
                FinderLine::Row { catalogue: true, mark: '♪', title: "Nothing But Time".into(), artist: "Cat Power".into(), note: "3 plays".into() },
                FinderLine::Row { catalogue: true, mark: '♥', title: "Nothing Compares 2 U".into(), artist: "Sinéad O'Connor".into(), note: "♥ liked · 1 play".into() },
                FinderLine::Info("[spotify] … querying".to_string()),
            ],
            cursor: 1,
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 14)).unwrap();
        terminal.draw(|frame| render_finder(frame, frame.area(), &view)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> =
            (0..14).map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect::<String>()).collect();
        assert!(rows[0].starts_with("┌─ insert a track ── ti tracks only ─"), "{}", rows[0]);
        assert!(rows[1].starts_with("│ → the insertion lands at 4"), "{}", rows[1]);
        assert!(rows[2].starts_with("│ ⟩ nothing bu"), "{}", rows[2]);
        assert!(rows[3].contains("catalog 2 · spotify …"), "{}", rows[3]);
        assert!(rows[4].contains("── catalog  2 results"), "{}", rows[4]);
        assert!(rows[5].contains(" 1 [catalog] ♪ Nothing But Time"), "{}", rows[5]);
        assert!(rows[6].contains(" 2 [catalog] ♥ Nothing Compares 2 U"), "{}", rows[6]);
        assert_eq!(buffer[(4, 6)].style().bg, Some(Color::Yellow), "the cursor highlights line 2");
        assert!(rows[7].contains("[spotify] … querying"), "{}", rows[7]);
        assert!(rows[9].contains("enter insert here"), "{}", rows[9]);
        assert!(rows[10].starts_with("└─ esc closes without inserting"), "{}", rows[10]);
    }

    #[test]
    fn a_narrow_terminal_keeps_the_axis_and_drops_the_column() {
        let text = screen(50, 30, &branches()).join("\n");
        assert!(!text.contains("── branches"), "{text}");
        assert!(text.contains("Cities in Dust"), "{text}");
    }
}

// --- the discography modal (`ad`, mockup 1a) --------------------------------

/// The light rule: it is the only box forkstify draws (the leader menu,
/// "?"), and a modal is its third case. A head line closes with a stroke
/// that runs to the edge.
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

/// "14 plays", "—" when it never played.
fn plays_of(plays: f64) -> String {
    if plays < 0.5 {
        "—".to_string()
    } else {
        format!("{plays:.0} play{}", if plays >= 1.5 { "s" } else { "" })
    }
}

fn render_explore(frame: &mut ratatui::Frame, area: Rect, screen: &crate::explore::Explore) {
    use crate::explore::Row;
    frame.render_widget(Clear, area);
    let width = area.width as usize;
    let summary = screen.summary();
    let cursor = Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD);
    let rule = |text: &str| Span::styled(text.to_string(), Style::default().fg(DIM));

    // — the head: what the card and the learned say about the artist
    let mut head = vec![
        ruled(
            vec![
                rule("┌─ "),
                Span::styled(
                    "discography ",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                rule("── "),
                Span::styled(
                    format!("{} ", screen.name),
                    Style::default().fg(CATALOG).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "{} · {} albums · {} tracks{}",
                        if screen.generated { "generated card" } else { "written card" },
                        summary.albums,
                        summary.titles,
                        if screen.loading { " · loading…" } else { "" }
                    ),
                    Style::default().fg(DIM),
                ),
            ],
            width,
        ),
        Line::from(vec![
            rule("│ "),
            Span::styled("the pool draws  ", Style::default().fg(MUTED)),
            Span::styled(
                format!("♪ {} top{}", summary.tops, plural(summary.tops)),
                Style::default().fg(PLAYING),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(
                format!("♥ {} liked", summary.liked),
                Style::default().fg(PLAYING),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(
                format!("⊘ {} banned", summary.banned),
                Style::default().fg(DANGER),
            ),
            Span::styled(" · ", Style::default().fg(DIM)),
            Span::styled(format!("· {} in the tail", summary.tail), Style::default().fg(VECTOR)),
        ]),
    ];
    // the question we came to ask: which album carries the plays
    head.push(Line::from(vec![
        rule("│ "),
        Span::styled("your plays  ", Style::default().fg(MUTED)),
        Span::styled(
            if summary.plays < 0.5 {
                "no play recorded for them".to_string()
            } else {
                format!(
                    "{} album{} carr{} {}% of {:.0} plays",
                    summary.carrying,
                    if summary.carrying > 1 { "s" } else { "" },
                    if summary.carrying > 1 { "y" } else { "ies" },
                    summary.carrying_pct,
                    summary.plays
                )
            },
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!(" — {} album(s) never opened", summary.never),
            Style::default().fg(DIM),
        ),
    ]));
    head.push(Line::from(vec![
        rule("│ "),
        Span::styled(
            format!("order: {} · filter: {}", screen.sort_word(), screen.filter.word()),
            Style::default().fg(DIM),
        ),
        Span::styled(
            if screen.query.is_empty() {
                String::new()
            } else {
                format!(" · \"{}\"", screen.query)
            },
            Style::default().fg(MUTED),
        ),
    ]));

    // — the foot: what is waiting to be written, then the keys
    let mut foot: Vec<Line> = Vec::new();
    if !screen.pending.is_empty() {
        foot.push(ruled(
            vec![
                rule("│ "),
                Span::styled("── pending ", Style::default().fg(EDIT).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(
                        "{} edit{} · one card · one commit",
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
                    if edit.add { "promote to top     " } else { "remove from top    " }.to_string(),
                    Style::default().fg(MUTED),
                ),
                Span::styled(format!("({})", edit.why), Style::default().fg(DIM)),
            ]));
        }
        if screen.pending.len() > 4 {
            foot.push(Line::from(vec![
                rule("│ "),
                rule(&format!("↓ {} more", screen.pending.len() - 4)),
            ]));
        }
        // what the user reads is what git will keep (edit.rs)
        foot.push(Line::from(vec![
            rule("│ "),
            Span::styled(format!("cards/{}.toml ", screen.slug), Style::default().fg(CATALOG)),
            Span::styled(format!("— \"{}\"", screen.commit_line()), Style::default().fg(DIM)),
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
        Span::styled("the album  ", Style::default().fg(MUTED)),
        Span::styled("tl ", Style::default().fg(PLAYING)),
        Span::styled("like  ", Style::default().fg(MUTED)),
        Span::styled("tb ", Style::default().fg(DANGER)),
        Span::styled("ban  ", Style::default().fg(MUTED)),
        Span::styled("e ", Style::default().fg(BRANCH)),
        Span::styled("to the queue  ", Style::default().fg(MUTED)),
        Span::styled("s v / ", Style::default().fg(VECTOR)),
        Span::styled("order, view, filter", Style::default().fg(MUTED)),
    ]));
    foot.push(ruled(
        vec![
            rule("└─ "),
            Span::styled(
                if screen.pending.is_empty() {
                    "esc closes".to_string()
                } else {
                    format!("⏎ write ({} pending, 1 commit)", screen.pending.len())
                },
                Style::default().fg(MUTED),
            ),
            Span::styled(
                " · u undoes the last · esc closes without writing".to_string(),
                Style::default().fg(DIM),
            ),
        ],
        width,
    ));

    // — the body: the albums folded, the cursor's one open
    let rows = screen.rows();
    let room = (area.height as usize).saturating_sub(head.len() + foot.len()).max(1);
    let here = rows.iter().position(|row| screen.at(*row)).unwrap_or(0);
    // the window follows the cursor without sticking it to the edge
    let start = here.saturating_sub(room / 2).min(rows.len().saturating_sub(room));
    let mut lines = head;
    for row in rows.iter().skip(start).take(room) {
        let selected = screen.at(*row);
        lines.push(match *row {
            Row::Album(index) => {
                let album = &screen.albums[index];
                let share = if summary.plays > 0.0 { album.plays() / summary.plays } else { 0.0 };
                let marks = if album.orphan {
                    "to review".to_string()
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
                // ▾ says what is open, not what is highlighted: the cursor
                // may have gone down into the album's tracks
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
                    "▶ playing".to_string()
                } else if track.banned {
                    "banned".to_string()
                } else if track.liked {
                    "♥ liked".to_string()
                } else {
                    String::new()
                };
                let last = match track.days {
                    Some(0) => "today".to_string(),
                    Some(days) => crate::home::age(Some(days)),
                    None => "never".to_string(),
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
                // color says the nature, never the importance: the glyph
                // alone carries it, the rest of the line is text
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
    // the rule runs down to the foot: the bottom of the screen does not
    // move from one album to the next
    while lines.len() + foot.len() < area.height as usize {
        lines.push(Line::from(rule("│")));
    }
    lines.extend(foot);
    frame.render_widget(Paragraph::new(lines), area);
}

// ---------------------------------------------------------------------
// The setup screens (`Installation.dc.html`, chantier A of
// `docs/conception/sortie.md`): one column, the flow, the prompt last —
// the same skeleton as home, with a step counter and its gauge on the
// right of the head.
// ---------------------------------------------------------------------

/// One line of a setup screen.
pub enum SetupRow {
    /// The step's title, bright.
    Title(String),
    Text(String),
    Muted(String),
    Dim(String),
    Rule(String),
    Blank,
    /// A numbered choice: `n  label  — note`, the selected one lit.
    Choice { n: usize, label: String, note: String, selected: bool },
    /// A text field: `label  > value`, the active one with its cursor.
    Field { label: String, value: String, active: bool },
    /// A playlist to tick: `[✓] name  count  owner`.
    Check { on: bool, name: String, count: String, owner: String, cursor: bool },
    /// A progress bar: `label  ████░░  count  note`.
    Bar { label: String, done: usize, total: usize, note: String },
    /// A step of the list (screens 0 and 9): `n ✓ label — note   state`.
    Step { n: usize, done: Option<bool>, label: String, note: String, state: String, cursor: bool },
    /// A key and its value: `key   value`.
    Kv { key: String, value: String },
    /// Names, three per line.
    Names(Vec<String>),
    /// A notice, colored by its glyph like a toast.
    Notice(String),
}

pub struct SetupView<'a> {
    /// "step 1 / 7", or the screen's name.
    pub step: String,
    /// (done, of) cells of the gauge on the right; (0, 0) = none.
    pub gauge: (usize, usize),
    /// The second line of the head: the authorizations, or a subtitle.
    pub status: Vec<(String, bool)>,
    pub rows: &'a [SetupRow],
    pub prompt: String,
    /// What was just said, as a toast at the bottom right.
    pub toast: Option<Toast>,
}

impl Tui {
    pub fn draw_setup(&mut self, view: &SetupView) -> std::io::Result<()> {
        self.terminal.draw(|frame| render_setup(frame, view))?;
        Ok(())
    }
}

fn cells(done: usize, total: usize, width: usize) -> String {
    let filled = if total == 0 { 0 } else { (done * width + total / 2) / total }.min(width);
    (0..width).map(|i| if i < filled { '█' } else { '░' }).collect()
}

pub fn setup_lines(rows: &[SetupRow], width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    for row in rows {
        match row {
            SetupRow::Title(text) => lines.push(Line::from(Span::styled(
                format!("  {text}"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ))),
            SetupRow::Text(text) => lines.push(Line::from(Span::raw(format!("  {text}")))),
            SetupRow::Muted(text) => lines.push(Line::from(Span::styled(format!("  {text}"), Style::default().fg(MUTED)))),
            SetupRow::Dim(text) => lines.push(Line::from(Span::styled(format!("  {text}"), Style::default().fg(DIM)))),
            SetupRow::Rule(text) => {
                let dashes = width.saturating_sub(text.chars().count() + 6);
                lines.push(Line::from(Span::styled(
                    format!("  ── {text} {}", "─".repeat(dashes)),
                    Style::default().fg(DIM),
                )));
            }
            SetupRow::Blank => lines.push(Line::from("")),
            SetupRow::Choice { n, label, note, selected } => {
                let (num, text) = if *selected { (BRANCH, Color::White) } else { (DIM, MUTED) };
                lines.push(Line::from(vec![
                    Span::styled(if *selected { "  ▸ " } else { "    " }.to_string(), Style::default().fg(BRANCH)),
                    Span::styled(format!("{n}  "), Style::default().fg(num).add_modifier(Modifier::BOLD)),
                    Span::styled(label.clone(), Style::default().fg(text)),
                    Span::styled(if note.is_empty() { String::new() } else { format!("  — {note}") }, Style::default().fg(DIM)),
                ]));
            }
            SetupRow::Field { label, value, active } => {
                lines.push(Line::from(vec![
                    Span::styled(format!("    {label:<6}"), Style::default().fg(MUTED)),
                    Span::styled("> ", Style::default().fg(if *active { BRANCH } else { DIM })),
                    Span::styled(value.clone(), Style::default().fg(Color::White)),
                    Span::styled(if *active { "▏" } else { "" }.to_string(), Style::default().fg(BRANCH)),
                ]));
            }
            SetupRow::Check { on, name, count, owner, cursor } => {
                let name_width = width.saturating_sub(30).max(12);
                let shown: String = name.chars().take(name_width).collect();
                lines.push(Line::from(vec![
                    Span::styled(if *cursor { "  ▸ " } else { "    " }.to_string(), Style::default().fg(BRANCH)),
                    Span::styled(if *on { "[✓] " } else { "[ ] " }.to_string(), Style::default().fg(if *on { PLAYING } else { DIM })),
                    Span::styled(
                        format!("{shown:<name_width$}"),
                        Style::default().fg(if *cursor { Color::White } else if *on { MUTED } else { DIM }),
                    ),
                    Span::styled(format!("  {count:>6}"), Style::default().fg(DIM)),
                    Span::styled(format!("  {owner}"), Style::default().fg(DIM)),
                ]));
            }
            SetupRow::Bar { label, done, total, note } => {
                let count = if *total > 0 { format!("{done} / {total}") } else { done.to_string() };
                lines.push(Line::from(vec![
                    Span::styled(format!("    {label:<16}"), Style::default().fg(MUTED)),
                    Span::styled(cells(*done, (*total).max(*done), 14), Style::default().fg(CATALOG)),
                    Span::styled(format!("  {count:>13}"), Style::default().fg(Color::White)),
                    Span::styled(if note.is_empty() { String::new() } else { format!("  {note}") }, Style::default().fg(DIM)),
                ]));
            }
            SetupRow::Step { n, done, label, note, state, cursor } => {
                let (mark, mark_color) = match done {
                    Some(true) => ("✓", PLAYING),
                    Some(false) => ("○", MUTED),
                    None => (" ", DIM),
                };
                let text_color = if *cursor { Color::White } else { MUTED };
                let left: usize = 8 + label.chars().count() + if note.is_empty() { 0 } else { 3 + note.chars().count() };
                let gap = width.saturating_sub(left + state.chars().count() + 4).max(2);
                lines.push(Line::from(vec![
                    Span::styled(if *cursor { "  ▸ " } else { "    " }.to_string(), Style::default().fg(BRANCH)),
                    Span::styled(format!("{n} "), Style::default().fg(DIM)),
                    Span::styled(format!("{mark} "), Style::default().fg(mark_color)),
                    Span::styled(label.clone(), Style::default().fg(text_color)),
                    Span::styled(if note.is_empty() { String::new() } else { format!(" — {note}") }, Style::default().fg(DIM)),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(state.clone(), Style::default().fg(if state == "to do" { EDIT } else { DIM })),
                ]));
            }
            SetupRow::Kv { key, value } => lines.push(Line::from(vec![
                Span::styled(format!("    {key:<10}"), Style::default().fg(DIM)),
                Span::styled(value.clone(), Style::default().fg(MUTED)),
            ])),
            SetupRow::Names(names) => {
                for chunk in names.chunks(3) {
                    let mut spans = vec![Span::raw("    ".to_string())];
                    for name in chunk {
                        spans.push(Span::styled(format!("{:<26}", name.chars().take(25).collect::<String>()), Style::default().fg(MUTED)));
                    }
                    lines.push(Line::from(spans));
                }
            }
            SetupRow::Notice(text) => lines.push(Line::from(Span::styled(format!("  {text}"), Style::default().fg(tone_of(text))))),
        }
    }
    lines
}

fn render_setup(frame: &mut ratatui::Frame, view: &SetupView) {
    let area = frame.area();
    let [head, body, prompt] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(3), Constraint::Length(1)]).areas(area);

    // the head: the wordmark, the step and its gauge on the right
    let right = if view.gauge.1 > 0 {
        format!("{}  {}", view.step, cells(view.gauge.0, view.gauge.1, view.gauge.1))
    } else {
        view.step.clone()
    };
    let gap = (head.width as usize).saturating_sub("forkstify".len() + right.chars().count()).max(2);
    let title = Line::from(vec![
        Span::styled("forkstify", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(gap)),
        Span::styled(right, Style::default().fg(BRANCH)),
    ]);
    let mut second: Vec<Span> = Vec::new();
    for (i, (text, ok)) in view.status.iter().enumerate() {
        if i > 0 {
            second.push(Span::styled(" · ", Style::default().fg(DIM)));
        }
        second.push(Span::styled(text.clone(), Style::default().fg(if *ok { DIM } else { DANGER })));
    }
    frame.render_widget(Paragraph::new(vec![title, Line::from(second)]), head);

    let lines = setup_lines(view.rows, body.width as usize);
    frame.render_widget(Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }), body);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(view.prompt.clone(), Style::default().fg(MUTED)))),
        prompt,
    );
    if let Some(toast) = &view.toast {
        render_toast(frame, body, toast);
    }
}

#[cfg(test)]
mod overlay_tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn draw(body: &[String], scroll: usize) -> String {
        let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
        terminal.draw(|frame| render_block(frame, frame.area(), "C diff", body, scroll)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..buffer.area.height)
            .map(|y| (0..buffer.area.width).map(|x| buffer[(x, y)].symbol().to_string()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A long overlay scrolls: the title says what is above and below,
    /// and the lines shown are the ones asked for.
    #[test]
    fn a_long_overlay_scrolls_and_says_so() {
        let body: Vec<String> = (1..=30).map(|i| format!("line {i}")).collect();
        let top = draw(&body, 0);
        assert!(top.contains("C diff — ↑ 0 · ↓ 22 · j/k"), "{top}");
        assert!(top.contains("line 1 ") && !top.contains("line 9"), "{top}");
        let down = draw(&body, 10);
        assert!(down.contains("↑ 10 · ↓ 12"), "{down}");
        assert!(down.contains("line 11") && !down.contains("line 10 "), "{down}");
        // past the end, it stops where the last lines fill the block
        let end = draw(&body, 100);
        assert!(end.contains("↑ 22 · ↓ 0"), "{end}");
        assert!(end.contains("line 30"), "{end}");
        // a short one says nothing
        let short = draw(&body[..3], 0);
        assert!(short.contains(" C diff ") && !short.contains("j/k"), "{short}");
        // a long line widens the block instead of folding
        let wide = vec![format!("  + {:<24} generated  slowcore, us · 4 similar, 2 links", "Codeine")];
        let text = draw(&wide, 0);
        assert!(text.contains("4 similar, 2 links │"), "{text}");
        assert_eq!(overlay_scroll_max(30, 12), 22);
        assert_eq!(overlay_scroll_max(3, 12), 0);
    }

    /// The lines of a catalog overlay wear their colours: a new card in
    /// green, an edited one in yellow, a heading bright, a key blue.
    #[test]
    fn a_catalog_line_is_coloured_by_its_shape() {
        let colour = |text: &str| -> Vec<(String, Color)> {
            info_line(text).spans.iter().map(|s| (s.content.to_string(), s.style.fg.unwrap_or(Color::Reset))).collect()
        };
        let new = colour("  + Codeine                  generated  slowcore");
        assert_eq!(new[1], ("+ ".to_string(), PLAYING));
        assert_eq!(new[2], ("Codeine".to_string(), Color::White));
        let edited = colour("  ~ The Cure                 +3 −1  tops");
        assert_eq!(edited[1], ("~ ".to_string(), EDIT));
        let heading = colour("  ## Edited cards (3) — please read");
        assert_eq!(heading[1].1, Color::White);
        let key = colour("title  Propose 49 cards");
        assert_eq!(key[1], ("title".to_string(), CATALOG));
        let ask = colour("↻ open the pull request?");
        assert_eq!(ask[1].1, EDIT);
        let hint = colour("  Cd the detail · Cp propose");
        assert_eq!(hint[1], ("Cd ".to_string(), BRANCH));
        assert!(hint.iter().any(|(s, c)| s == "Cp " && *c == BRANCH), "{hint:?}");
        let plain = colour("nothing special here");
        assert_eq!(plain[0].1, MUTED);
    }
}

#[cfg(test)]
mod setup_tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn text_of(rows: &[SetupRow]) -> String {
        let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
        terminal
            .draw(|frame| {
                render_setup(
                    frame,
                    &SetupView {
                        step: "step 5 / 7".into(),
                        gauge: (5, 7),
                        status: vec![("✓ librespot".into(), true), ("⏹ no api web".into(), false)],
                        rows,
                        prompt: "[space tick · j/k · ⏎ harvest]".into(),
                        toast: None,
                    },
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..buffer.area.height)
            .map(|y| (0..buffer.area.width).map(|x| buffer[(x, y)].symbol().to_string()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The setup screen has the skeleton of home — head, flow, prompt —
    /// with the step and its gauge on the right, and every row kind reads.
    #[test]
    fn a_setup_screen_reads_from_top_to_bottom() {
        let rows = vec![
            SetupRow::Title("the playlists".into()),
            SetupRow::Check { on: true, name: "#fipway".into(), count: "412 tracks".into(), owner: "you".into(), cursor: true },
            SetupRow::Check { on: false, name: "pour dormir".into(), count: "140".into(), owner: "you".into(), cursor: false },
            SetupRow::Bar { label: "liked tracks".into(), done: 4812, total: 4812, note: "the main artist only".into() },
            SetupRow::Step { n: 7, done: Some(false), label: "the coverage".into(), note: "9 artists".into(), state: "to do".into(), cursor: false },
            SetupRow::Choice { n: 2, label: "fork it for me".into(), note: "gh detected".into(), selected: true },
            SetupRow::Field { label: "name".into(), value: "Joel".into(), active: true },
        ];
        let text = text_of(&rows);
        assert!(text.contains("step 5 / 7  █████░░"), "{text}");
        assert!(text.contains("✓ librespot · ⏹ no api web"), "{text}");
        assert!(text.contains("the playlists"), "{text}");
        assert!(text.contains("▸ [✓] #fipway"), "{text}");
        assert!(text.contains("[ ] pour dormir"), "{text}");
        assert!(text.contains("liked tracks    ██████████████    4812 / 4812  the main artist only"), "{text}");
        assert!(text.contains("7 ○ the coverage — 9 artists"), "{text}");
        assert!(text.contains("to do"), "{text}");
        assert!(text.contains("▸ 2  fork it for me  — gh detected"), "{text}");
        assert!(text.contains("name  > Joel▏"), "{text}");
        assert!(text.contains("[space tick · j/k · ⏎ harvest]"), "{text}");
    }

    #[test]
    fn the_cells_round_to_the_nearest() {
        assert_eq!(cells(0, 7, 7), "░░░░░░░");
        assert_eq!(cells(1, 7, 7), "█░░░░░░");
        assert_eq!(cells(7, 7, 7), "███████");
        assert_eq!(cells(3, 0, 4), "░░░░");
    }
}
