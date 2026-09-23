//! Raw-mode keyboard (0015): one key at a time, no Enter.
//!
//! Keys accumulate in a pending buffer that is matched against the grammar
//! after every keystroke. The grammar is **prefix-free** — no complete
//! command is the start of a longer one — so a sequence fires the instant
//! it is unambiguous, with no timeout and no lookahead. That property is
//! what decides where the count and the modifiers sit:
//!
//! * `3` alone takes branch 3, so `f3` may stay pending;
//! * a modifier therefore comes *before* the count (`fn3`, `f!3`), since
//!   `f3n` would make `f3` both complete and a prefix.
//!
//! `/` and `:` leave raw mode for a line, which is where a query belongs.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

/// When a chosen branch or an encore should start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum When {
    /// Once the current branch has played out (the default).
    EndOfBranch,
    /// Right after the current track, keeping what was queued.
    Now,
    /// Right after the current track, dropping what was queued.
    NowForce,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// A bare digit: a branch, or a search result when one is pending —
    /// the caller knows which, the parser does not.
    Digit(usize),
    Fork { branch: usize, when: When },
    /// `fg<n>` — fork generate: the card of gap n, and nothing queued —
    /// the gap turns into a branch on show (Joel, 19/09/2026).
    ForkGenerate(usize),
    Peek,
    Reroll,
    Wander,
    ForkUndo,
    Auto,
    Encore { count: usize, when: When },
    Track(char),
    Artist(char),
    Prev,
    Next,
    /// The axis is shown vertically: the vertical arrows move a
    /// **selection**, they play nothing. Enter is what plays
    /// (Joel, 06/09/2026).
    Up,
    Down,
    /// Esc — cancels the selection, closes a pane, leaves a mode.
    Escape,
    /// `cc` — comfort: opens the setting, arrows move it, enter validates.
    ComfortMode,
    /// `c<n>` — the comfort zone, in one go (Joel, 08/09/2026).
    Comfort(u8),
    /// `s` — sort: changes the collection order, on home.
    Sort,
    /// `gg` and `G` — both ends of a list, as in vim. `g` alone is nothing:
    /// it waits for its second (Joel, 06/09/2026).
    Top,
    Bottom,
    /// `J` / `K` — move the highlighted line one step in the queue, right
    /// away (Joel, 08/09/2026). The opposite key undoes.
    MoveDown,
    MoveUp,
    PlayPause,
    /// Space, the leader: open the key helper. Carries the namespace that
    /// was half-typed, so `f` then space lists only the branch keys — and
    /// the sequence stays pending, so the next key completes it. Which-key,
    /// in a terminal: the helper is an input aid, not a poster (Joel,
    /// 07/09/2026). Escape closes it, backspace steps back one key.
    Help(Option<char>),
    Undo,
    /// `x` — take the thing under the cursor out of the list it is in.
    Remove,
    /// `r` — resume: resume the last journey (home).
    Resume,
    /// `b` — browse: dry run, no sound (not-connected screens).
    Browse,
    Quit,
    Search(String),
    Colon(String),
    /// The half-typed sequence, or empty when it closes. The reader **no
    /// longer prints anything**: the screen belongs to the TUI, and a byte
    /// written behind its back leaves remains it cannot erase (noticed by
    /// Joel on 06/09/2026).
    Pending(String),
    /// A line being typed (`/` or `:`), prefix included.
    Typing(Option<String>),
    /// A sequence that means nothing.
    Unknown(String),

    // — the keys of a modal, which has its own table —
    /// `e` — queue the track under the cursor, without closing: an edit
    /// only plays on the next launch, the queue plays tonight.
    Enqueue,
    /// `v` — the view: cycle the provenance filter.
    Filter,
    /// `A` — promote the whole album, at the grain of the problem.
    AlbumTop,
    /// `o` — open: the setup's "open the browser", "generate"; while
    /// listening, the first card a stopped merge left to resolve.
    Open,
    /// `C<k>` — the catalog namespace, upper case (Joel, 19/09/2026):
    /// `Cd` diff, `Cp` propose, `Cu` update. The first namespace in a
    /// capital: the rare, heavy gesture, as `A` promotes a whole album.
    Catalog(char),
}

/// A modal takes the keyboard and gives it **its** table — `keybindings.md`
/// has planned for it from the start ("a separate mode, with its own
/// table"). The key reader lives in a thread and does not know the screen
/// state: so it is an atomic, set on opening and given back on closing. The
/// pending sequence is forgotten on the switch, otherwise a `t` typed on
/// one side would complete on the other.
static MODAL: AtomicBool = AtomicBool::new(false);
/// Text mode (Joel, 08/09/2026): the search modal owns the keyboard — every
/// printable key is typed, arrows move, enter takes, escape closes, tab
/// toggles the scope. No grammar, or "cros" would fire c, r, o, s.
static TEXT: AtomicBool = AtomicBool::new(false);

/// What the text-mode line contains on opening — `:search bowie` opens the
/// modal already filled, and the next keystroke must **continue** that
/// word, not erase it (Joel, 09/09/2026).
static TEXT_LINE: Mutex<String> = Mutex::new(String::new());
/// Bumped on every opening: two fields opened back to back (the setup's
/// name then mail) must not share a line — the reader reloads it when the
/// generation moved, not only when the mode flipped (seen 20/09/2026).
static TEXT_GENERATION: AtomicUsize = AtomicUsize::new(0);

pub fn set_text(on: bool, start: &str) {
    if let Ok(mut line) = TEXT_LINE.lock() {
        line.clear();
        line.push_str(start);
    }
    TEXT.store(on, Ordering::Relaxed);
    TEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
}

fn text_generation() -> usize {
    TEXT_GENERATION.load(Ordering::Relaxed)
}

fn text() -> bool {
    TEXT.load(Ordering::Relaxed)
}

pub fn set_modal(on: bool) {
    MODAL.store(on, Ordering::Relaxed);
}

/// The setup screens (workstream A of `docs/design/before-release.md`) have
/// their own table too: digits pick, `j`/`k` move, `o` opens or launches.
static SETUP: AtomicBool = AtomicBool::new(false);

pub fn set_setup(on: bool) {
    SETUP.store(on, Ordering::Relaxed);
}

fn setup() -> bool {
    SETUP.load(Ordering::Relaxed)
}

fn modal() -> bool {
    MODAL.load(Ordering::Relaxed)
}

pub enum Parse {
    Done(Cmd),
    /// A valid start: wait for more.
    Pending,
    Unknown,
}

// `t` and `T` stay parseable for the discography screen (`ad`), where the
// tops are corrected; the listening session itself refuses them (0018)
const TRACK_KEYS: [char; 10] = ['l', 's', 'b', 'm', 't', 'T', 'd', 'i', 'a', 'g'];
const ARTIST_KEYS: [char; 7] = ['l', 's', 'b', 'e', 'c', 'd', 'g'];
/// `Cd` diff, `Cp` propose, `Cu` update.
const CATALOG_KEYS: [char; 3] = ['d', 'p', 'u'];
/// In the discography modal, `t` only serves what makes sense on a list
/// line: the two edits and the two measures.
// no more `tt` / `tT` here either (Joel, 08/09/2026): the modal likes,
// bans, queues — and `A` promotes an album in one go
const MODAL_TRACK_KEYS: [char; 2] = ['l', 'b'];

/// Match the pending buffer against the grammar of 0015.
pub fn parse(buf: &str) -> Parse {
    let c: Vec<char> = buf.chars().collect();
    let digit = |ch: char| ch.to_digit(10).filter(|n| *n > 0).map(|n| n as usize);

    match c.as_slice() {
        [] => Parse::Pending,

        // a lone digit: branch, or search result — the caller decides
        [d] if digit(*d).is_some() => Parse::Done(Cmd::Digit(digit(*d).unwrap())),

        // --- f, the branch namespace ---
        ['f'] | ['f', 'n'] | ['f', '!'] | ['f', 'g'] => Parse::Pending,
        ['f', d] if digit(*d).is_some() => Parse::Done(Cmd::Fork {
            branch: digit(*d).unwrap(),
            when: When::EndOfBranch,
        }),
        ['f', 'n', d] if digit(*d).is_some() => Parse::Done(Cmd::Fork {
            branch: digit(*d).unwrap(),
            when: When::Now,
        }),
        ['f', '!', d] if digit(*d).is_some() => Parse::Done(Cmd::Fork {
            branch: digit(*d).unwrap(),
            when: When::NowForce,
        }),
        ['f', 'g', d] if digit(*d).is_some() => Parse::Done(Cmd::ForkGenerate(digit(*d).unwrap())),
        ['f', 'p'] => Parse::Done(Cmd::Peek),
        ['f', 'r'] => Parse::Done(Cmd::Reroll),
        ['f', 'w'] => Parse::Done(Cmd::Wander),
        ['f', 'u'] => Parse::Done(Cmd::ForkUndo),

        // --- e, encore (same shape as f, count mandatory) ---
        ['e'] | ['e', 'n'] | ['e', '!'] => Parse::Pending,
        ['e', d] if digit(*d).is_some() => Parse::Done(Cmd::Encore {
            count: digit(*d).unwrap(),
            when: When::EndOfBranch,
        }),
        ['e', 'n', d] if digit(*d).is_some() => Parse::Done(Cmd::Encore {
            count: digit(*d).unwrap(),
            when: When::Now,
        }),
        ['e', '!', d] if digit(*d).is_some() => Parse::Done(Cmd::Encore {
            count: digit(*d).unwrap(),
            when: When::NowForce,
        }),

        // --- t and a, the affinage namespaces ---
        ['t'] | ['a'] => Parse::Pending,
        ['t', k] if TRACK_KEYS.contains(k) => Parse::Done(Cmd::Track(*k)),
        ['a', k] if ARTIST_KEYS.contains(k) => Parse::Done(Cmd::Artist(*k)),

        // --- C, the catalog (workstream B, `docs/design/before-release.md`) ---
        ['C'] => Parse::Pending,
        ['C', k] if CATALOG_KEYS.contains(k) => Parse::Done(Cmd::Catalog(*k)),

        // --- the bare keyboard ---
        ['o'] => Parse::Done(Cmd::Open),
        ['h'] => Parse::Done(Cmd::Prev),
        ['l'] => Parse::Done(Cmd::Next),
        ['p'] => Parse::Done(Cmd::PlayPause),
        ['r'] => Parse::Done(Cmd::Resume),
        ['b'] => Parse::Done(Cmd::Browse),
        // c has been a namespace since 08/09/2026: c<n> sets, cc opens the
        // gauge — a lone c can no longer be complete without breaking the
        // grammar
        ['c'] => Parse::Pending,
        ['c', 'c'] => Parse::Done(Cmd::ComfortMode),
        ['c', d] if ('0'..='5').contains(d) => Parse::Done(Cmd::Comfort(*d as u8 - b'0')),
        ['s'] => Parse::Done(Cmd::Sort),
        // the collection view, liked ⇄ all — the same word as in the
        // discography (Joel, 09/09/2026)
        ['v'] => Parse::Done(Cmd::Filter),
        ['g'] => Parse::Pending,
        ['g', 'g'] => Parse::Done(Cmd::Top),
        ['G'] => Parse::Done(Cmd::Bottom),
        ['J'] => Parse::Done(Cmd::MoveDown),
        ['K'] => Parse::Done(Cmd::MoveUp),
        // `x` — take this out of where it is: a track out of the queue, an
        // artist out of the liked (Joel, 2026-09-23). It was `tx`, but
        // removing is not a verb of the track alone.
        ['x'] => Parse::Done(Cmd::Remove),
        ['q'] => Parse::Done(Cmd::Quit),
        ['\r'] | ['\n'] => Parse::Done(Cmd::Auto),

        _ => Parse::Unknown,
    }
}

/// The table of the setup screens (`Installation.dc.html`): prefix-free,
/// vertical like the modal, with the digits back — a step's choices are
/// numbered — and `o` to open (the browser, the generation).
pub fn parse_setup(buf: &str) -> Parse {
    let c: Vec<char> = buf.chars().collect();
    match c.as_slice() {
        [] => Parse::Pending,
        [d] if d.is_ascii_digit() => Parse::Done(Cmd::Digit(d.to_digit(10).unwrap() as usize)),
        ['j'] => Parse::Done(Cmd::Down),
        ['k'] => Parse::Done(Cmd::Up),
        ['h'] => Parse::Done(Cmd::Prev),
        ['l'] => Parse::Done(Cmd::Next),
        ['g'] => Parse::Pending,
        ['g', 'g'] => Parse::Done(Cmd::Top),
        ['G'] => Parse::Done(Cmd::Bottom),
        ['o'] => Parse::Done(Cmd::Open),
        ['q'] => Parse::Done(Cmd::Quit),
        ['\r'] | ['\n'] => Parse::Done(Cmd::Auto),
        _ => Parse::Unknown,
    }
}

/// The table of the discography modal (mockup 1a). It is **prefix-free**
/// like the other, and borrows from vim what the listening grammar leaves
/// free: `j`/`k` go down and up, `h`/`l` fold and unfold — a vertical axis,
/// this time.
pub fn parse_modal(buf: &str) -> Parse {
    let c: Vec<char> = buf.chars().collect();
    match c.as_slice() {
        [] => Parse::Pending,

        ['j'] => Parse::Done(Cmd::Down),
        ['k'] => Parse::Done(Cmd::Up),
        ['h'] => Parse::Done(Cmd::Prev),
        ['l'] => Parse::Done(Cmd::Next),
        ['g'] => Parse::Pending,
        ['g', 'g'] => Parse::Done(Cmd::Top),
        ['G'] => Parse::Done(Cmd::Bottom),

        ['t'] => Parse::Pending,
        ['t', k] if MODAL_TRACK_KEYS.contains(k) => Parse::Done(Cmd::Track(*k)),

        ['e'] => Parse::Done(Cmd::Enqueue),
        ['s'] => Parse::Done(Cmd::Sort),
        ['v'] => Parse::Done(Cmd::Filter),
        ['A'] => Parse::Done(Cmd::AlbumTop),
        ['u'] => Parse::Done(Cmd::Undo),
        ['\r'] | ['\n'] => Parse::Done(Cmd::Auto),

        _ => Parse::Unknown,
    }
}

/// Put the terminal in raw mode and put it back when dropped — including on
/// a panic, which is why this is a guard and not a pair of calls.
pub struct RawMode(libc::termios);

/// The terminal as it was before raw mode, kept for `raw_pause`: `ae`
/// hands the terminal to `$EDITOR` and takes it back (Joel, 20/09/2026).
struct Original(libc::termios);
// termios is plain data
unsafe impl Send for Original {}
static ORIGINAL: Mutex<Option<Original>> = Mutex::new(None);

fn set_raw(original: &libc::termios) -> bool {
    let mut raw = *original;
    // no canonical line buffering, no echo: keys arrive one by one
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) == 0 }
}

impl RawMode {
    pub fn enable() -> Option<RawMode> {
        unsafe {
            if libc::isatty(libc::STDIN_FILENO) != 1 {
                return None;
            }
            let mut original: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut original) != 0 {
                return None;
            }
            if !set_raw(&original) {
                return None;
            }
            if let Ok(mut kept) = ORIGINAL.lock() {
                *kept = Some(Original(original));
            }
            Some(RawMode(original))
        }
    }
}

/// Give the terminal back as it was — for an editor — and take it again.
pub fn raw_pause() {
    if let Ok(kept) = ORIGINAL.lock() {
        if let Some(original) = kept.as_ref() {
            unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &original.0);
            }
        }
    }
}

pub fn raw_resume() {
    if let Ok(kept) = ORIGINAL.lock() {
        if let Some(original) = kept.as_ref() {
            set_raw(&original.0);
        }
    }
}

/// The reader parks while an editor has the terminal: it polls before it
/// reads, so no keystroke meant for the editor is swallowed.
static SUSPENDED: AtomicBool = AtomicBool::new(false);

pub fn suspend_reader(on: bool) {
    SUSPENDED.store(on, Ordering::Relaxed);
}

/// Throw away what the terminal answered the editor — nvim asks it who it
/// is on the way out (`ESC [ c`), and the reply lands on stdin after the
/// editor is gone: read as keys, "62;1;4c" would take branches 6, 2, 1, 4.
/// Called while the reader is parked, before it is released.
pub fn drain_input() {
    while input_pending(80) {
        if raw_byte().is_none() {
            break;
        }
    }
}

fn suspended() -> bool {
    SUSPENDED.load(Ordering::Relaxed)
}

/// One byte, once the reader is allowed to read: while suspended it
/// waits, and it only reads what `poll` says is there.
fn next_byte() -> Option<u8> {
    loop {
        if suspended() {
            std::thread::sleep(std::time::Duration::from_millis(40));
            continue;
        }
        if input_pending(100) {
            if suspended() {
                continue;
            }
            return raw_byte();
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.0);
        }
    }
}

/// One byte from the terminal, straight from the descriptor: std's stdin
/// buffers, and a buffered "ESC [ A" would hide its tail from `poll` —
/// the escape would then look alone, and "[" "A" would be typed.
fn raw_byte() -> Option<u8> {
    let mut byte = 0u8;
    let got = unsafe { libc::read(libc::STDIN_FILENO, &mut byte as *mut u8 as *mut libc::c_void, 1) };
    (got == 1).then_some(byte)
}

/// Is there input waiting, within `ms` milliseconds?
fn input_pending(ms: i32) -> bool {
    let mut fd = libc::pollfd { fd: libc::STDIN_FILENO, events: libc::POLLIN, revents: 0 };
    unsafe { libc::poll(&mut fd, 1, ms) > 0 }
}

/// After an ESC: the tail of an escape sequence ("[ A" for ↑, or "O A"
/// in application mode) if it is already there, `None` for the Escape key
/// alone. The reader used to wait for two more bytes whatever happened, so
/// a lone escape needed two more keystrokes to pass — "I often have to
/// press several times" (Joel, 08/09/2026). A terminal delivers a
/// sequence in one go; twenty milliseconds is plenty to tell them apart.
fn escape_sequence() -> Option<[u8; 2]> {
    if !input_pending(20) {
        return None;
    }
    let first = raw_byte()?;
    if first != b'[' && first != b'O' {
        // alt+key, or two escapes in a row: an escape, and the byte is spent
        return None;
    }
    if !input_pending(20) {
        return None;
    }
    let second = raw_byte()?;
    Some([first, second])
}

/// Read keys on a blocking thread and send commands. Owns the pending
/// buffer and the line mode, so the async side only ever sees a `Cmd`.
pub fn spawn_reader(tx: UnboundedSender<Cmd>) {
    std::thread::spawn(move || {
        let mut pending = String::new();
        let mut was_modal = modal();
        let mut was_setup = setup();
        // the text-mode line: it lives here, the screen only sees its state
        let mut line = String::new();
        let mut was_text = text();
        let mut was_generation = text_generation();

        while let Some(b) = next_byte() {
            let byte = [b];
            let key = b as char;

            if text() != was_text || text_generation() != was_generation {
                was_text = text();
                was_generation = text_generation();
                line = match (was_text, TEXT_LINE.lock()) {
                    (true, Ok(start)) => start.clone(),
                    _ => String::new(),
                };
                clear_pending(&mut pending, &tx);
            }
            if was_text {
                let cmd = match byte[0] {
                    0x1b => match escape_sequence() {
                        Some([_, b'A']) => Some(Cmd::Up),
                        Some([_, b'B']) => Some(Cmd::Down),
                        Some([_, b'C']) | Some([_, b'D']) => None,
                        _ => Some(Cmd::Escape),
                    },
                    b'\r' | b'\n' => Some(Cmd::Auto),
                    b'\t' => Some(Cmd::Filter),
                    0x7f | 0x08 => {
                        line.pop();
                        Some(Cmd::Typing(Some(line.clone())))
                    }
                    b if b.is_ascii_graphic() || b == b' ' || b >= 0x80 => {
                        line.push(b as char);
                        Some(Cmd::Typing(Some(line.clone())))
                    }
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    if tx.send(cmd).is_err() {
                        return;
                    }
                }
                continue;
            }

            // the screen switched tables under our feet: the pending
            // sequence belonged to the other one
            if modal() != was_modal {
                was_modal = !was_modal;
                clear_pending(&mut pending, &tx);
            }
            if setup() != was_setup {
                was_setup = !was_setup;
                clear_pending(&mut pending, &tx);
            }

            // arrows arrive as ESC [ C / ESC [ D — and a lone ESC is escape
            if byte[0] == 0x1b {
                if let Some(rest) = escape_sequence() {
                    let cmd = match rest {
                        [_, b'C'] => Some(Cmd::Next),
                        [_, b'D'] => Some(Cmd::Prev),
                        [_, b'A'] => Some(Cmd::Up),
                        [_, b'B'] => Some(Cmd::Down),
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        clear_pending(&mut pending, &tx);
                        if tx.send(cmd).is_err() {
                            return;
                        }
                        continue;
                    }
                }
                clear_pending(&mut pending, &tx);
                if tx.send(Cmd::Escape).is_err() {
                    return;
                }
                continue;
            }

            // space is the leader, not a key of the grammar: it opens the
            // helper on what can be typed, here or inside the pending
            // namespace — and leaves the sequence pending, so the next key
            // completes it from inside the helper
            if key == ' ' {
                let namespace = pending.chars().next();
                if tx.send(Cmd::Help(namespace)).is_err() {
                    return;
                }
                continue;
            }

            // backspace steps back one key of the sequence — inside the
            // helper, that is going up one level
            if byte[0] == 0x7f || byte[0] == 0x08 {
                pending.pop();
                if tx.send(Cmd::Pending(pending.clone())).is_err() {
                    return;
                }
                continue;
            }

            // `/` and `:` open a line: a query is typed, not chorded
            if pending.is_empty() && (key == '/' || key == ':') {
                match read_line(key, "", &tx) {
                    Some(text) => {
                        let cmd = if key == '/' { Cmd::Search(text) } else { Cmd::Colon(text) };
                        if tx.send(Cmd::Typing(None)).is_err() || tx.send(cmd).is_err() {
                            return;
                        }
                    }
                    None => {
                        if tx.send(Cmd::Typing(None)).is_err() {
                            return;
                        }
                    }
                }
                continue;
            }

            // control bytes are not keys: a NUL or a ^C coming from the pty
            // must not become "(unknown)"
            if byte[0] < 0x20 && byte[0] != b'\r' && byte[0] != b'\n' {
                continue;
            }

            pending.push(key);
            let outcome = if was_setup {
                parse_setup(&pending)
            } else if was_modal {
                parse_modal(&pending)
            } else {
                parse(&pending)
            };
            match outcome {
                // `fw` takes a name: it opens the `:wander ` line, already
                // filled — enter alone wanders far, a name wanders there
                // (Joel, 11/09/2026)
                Parse::Done(Cmd::Wander) => {
                    pending.clear();
                    if tx.send(Cmd::Pending(String::new())).is_err() {
                        return;
                    }
                    let cmd = read_line(':', "wander ", &tx).map(Cmd::Colon);
                    if tx.send(Cmd::Typing(None)).is_err() {
                        return;
                    }
                    if let Some(cmd) = cmd {
                        if tx.send(cmd).is_err() {
                            return;
                        }
                    }
                    continue;
                }
                Parse::Done(cmd) => {
                    let was_pending = pending.chars().count() > 1;
                    pending.clear();
                    if was_pending && tx.send(Cmd::Pending(String::new())).is_err() {
                        return;
                    }
                    if tx.send(cmd).is_err() {
                        return;
                    }
                }
                Parse::Pending => {
                    // show what is expected, as vim shows a half-typed
                    // command — but the TUI is what displays it
                    if tx.send(Cmd::Pending(pending.clone())).is_err() {
                        return;
                    }
                }
                Parse::Unknown => {
                    let shown = pending.clone();
                    pending.clear();
                    if tx.send(Cmd::Unknown(shown)).is_err() {
                        return;
                    }
                }
            }
        }
    });
}

/// Forget the pending sequence, and tell the screen.
fn clear_pending(pending: &mut String, tx: &UnboundedSender<Cmd>) {
    if !pending.is_empty() {
        let _ = tx.send(Cmd::Pending(String::new()));
    }
    pending.clear();
}

/// Read a line: keystrokes go up to the screen instead of being written on
/// it. Enter sends, esc cancels.
fn read_line(prefix: char, start: &str, tx: &UnboundedSender<Cmd>) -> Option<String> {
    let mut text = start.to_string();
    let _ = tx.send(Cmd::Typing(Some(format!("{prefix}{text}"))));
    while let Some(b) = raw_byte() {
        match b {
            b'\r' | b'\n' => return Some(text),
            // an arrow inside a line does nothing, but its bytes must not
            // fall back into the grammar
            0x1b => {
                if escape_sequence().is_none() {
                    return None;
                }
                continue;
            }
            0x7f | 0x08 => {
                text.pop();
            }
            b if b.is_ascii_graphic() || b == b' ' => text.push(b as char),
            _ => {}
        }
        let _ = tx.send(Cmd::Typing(Some(format!("{prefix}{text}"))));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the whole design rests on: no complete command may be
    /// the start of a longer one, or it could never fire without a timeout.
    #[test]
    fn grammar_is_prefix_free() {
        let alphabet: Vec<char> =
            "0123456789fenatlsbcgGmTdLpruwhqQCo.?!\r".chars().collect();
        let mut complete: Vec<String> = Vec::new();
        // every sequence up to 3 keys
        let mut queue: Vec<String> = vec![String::new()];
        for _ in 0..3 {
            let mut next = Vec::new();
            for base in &queue {
                for ch in &alphabet {
                    let candidate = format!("{base}{ch}");
                    match parse(&candidate) {
                        Parse::Done(_) => complete.push(candidate),
                        Parse::Pending => next.push(candidate),
                        Parse::Unknown => {}
                    }
                }
            }
            queue = next;
        }
        for a in &complete {
            for b in &complete {
                assert!(
                    a == b || !b.starts_with(a.as_str()),
                    "\"{a}\" is a prefix of \"{b}\": it could never fire"
                );
            }
        }
    }

    #[test]
    fn the_three_variants() {
        assert_eq!(parse("3").done(), Some(Cmd::Digit(3)));
        assert_eq!(
            parse("f3").done(),
            Some(Cmd::Fork { branch: 3, when: When::EndOfBranch })
        );
        assert_eq!(
            parse("fn3").done(),
            Some(Cmd::Fork { branch: 3, when: When::Now })
        );
        assert_eq!(
            parse("f!3").done(),
            Some(Cmd::Fork { branch: 3, when: When::NowForce })
        );
        assert!(matches!(parse("fg"), Parse::Pending));
        assert_eq!(parse("fg2").done(), Some(Cmd::ForkGenerate(2)));
        assert_eq!(
            parse("e2").done(),
            Some(Cmd::Encore { count: 2, when: When::EndOfBranch })
        );
        assert_eq!(
            parse("en2").done(),
            Some(Cmd::Encore { count: 2, when: When::Now })
        );
        assert_eq!(
            parse("e!2").done(),
            Some(Cmd::Encore { count: 2, when: When::NowForce })
        );
    }

    /// The setup table: digits pick (0 included, for the comfort), `o`
    /// opens, and it stays prefix-free.
    #[test]
    fn the_setup_table() {
        assert_eq!(parse_setup("3").done(), Some(Cmd::Digit(3)));
        assert_eq!(parse_setup("0").done(), Some(Cmd::Digit(0)));
        assert_eq!(parse_setup("j").done(), Some(Cmd::Down));
        assert_eq!(parse_setup("o").done(), Some(Cmd::Open));
        assert_eq!(parse_setup("q").done(), Some(Cmd::Quit));
        assert!(matches!(parse_setup("g"), Parse::Pending));
        assert_eq!(parse_setup("gg").done(), Some(Cmd::Top));
        assert!(matches!(parse_setup("f"), Parse::Unknown));
    }

    #[test]
    fn namespaces_and_bare_keys() {
        assert_eq!(parse("tl").done(), Some(Cmd::Track('l')));
        assert_eq!(parse("al").done(), Some(Cmd::Artist('l')));
        assert_eq!(parse("ac").done(), Some(Cmd::Artist('c')));
        assert_eq!(parse("ag").done(), Some(Cmd::Artist('g')));
        assert!(matches!(parse("C"), Parse::Pending));
        assert_eq!(parse("Cd").done(), Some(Cmd::Catalog('d')));
        assert_eq!(parse("Cp").done(), Some(Cmd::Catalog('p')));
        assert_eq!(parse("Cu").done(), Some(Cmd::Catalog('u')));
        assert!(matches!(parse("Cx"), Parse::Unknown));
        assert_eq!(parse("o").done(), Some(Cmd::Open));
        assert_eq!(parse("ti").done(), Some(Cmd::Track('i')));
        assert_eq!(parse("h").done(), Some(Cmd::Prev));
        assert_eq!(parse("p").done(), Some(Cmd::PlayPause));
        assert_eq!(parse("r").done(), Some(Cmd::Resume));
        assert_eq!(parse("b").done(), Some(Cmd::Browse));
        assert!(matches!(parse("c"), Parse::Pending));
        assert_eq!(parse("cc").done(), Some(Cmd::ComfortMode));
        assert_eq!(parse("c3").done(), Some(Cmd::Comfort(3)));
        assert_eq!(parse("c0").done(), Some(Cmd::Comfort(0)));
        assert!(matches!(parse("c7"), Parse::Unknown));
        assert_eq!(parse("s").done(), Some(Cmd::Sort));
        assert_eq!(parse("gg").done(), Some(Cmd::Top));
        assert_eq!(parse("G").done(), Some(Cmd::Bottom));
        assert_eq!(parse("J").done(), Some(Cmd::MoveDown));
        assert_eq!(parse("K").done(), Some(Cmd::MoveUp));
        assert!(matches!(parse("g"), Parse::Pending));
        assert_eq!(parse("l").done(), Some(Cmd::Next));
        assert_eq!(parse("fu").done(), Some(Cmd::ForkUndo));
        assert!(matches!(parse("t"), Parse::Pending));
        assert!(matches!(parse("tz"), Parse::Unknown));
        // 0018: `tt` still parses — the discography uses it — but the
        // session is what refuses it when listening
        assert_eq!(parse("tt").done(), Some(Cmd::Track('t')));
        assert!(matches!(parse("e"), Parse::Pending));
        assert!(matches!(parse("e0"), Parse::Unknown));
    }

    impl Parse {
        fn done(self) -> Option<Cmd> {
            match self {
                Parse::Done(cmd) => Some(cmd),
                _ => None,
            }
        }
    }
}
