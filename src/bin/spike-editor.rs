//! Spike: hand the terminal to $EDITOR from under ratatui, take it back,
//! redraw. Run in a pty; the bytes after the editor's exit say whether
//! the redraw is emitted.
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1).expect("a file");
    let mut out = std::io::stdout();
    write!(out, "\x1b[?1049h\x1b[?25l")?;
    out.flush()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.draw(|f| f.render_widget(Paragraph::new("BEFORE-EDITOR"), f.area()))?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    write!(out, "\x1b[?25h\x1b[?1049l")?;
    out.flush()?;
    let status = std::process::Command::new("sh").arg("-c").arg("nvim \"$1\"").arg("spike").arg(&path).status()?;
    write!(out, "\x1b[?1049h\x1b[?25l")?;
    out.flush()?;
    terminal.clear()?;
    terminal.draw(|f| f.render_widget(Paragraph::new(format!("AFTER-EDITOR-REDRAWN {status}")), f.area()))?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    write!(out, "\x1b[?25h\x1b[?1049l")?;
    out.flush()?;
    Ok(())
}
