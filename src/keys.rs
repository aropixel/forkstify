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

use std::io::{Read, Write};
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
    PlayPause,
    Undo,
    Repeat,
    Why,
    Queue,
    Quit,
    Search(String),
    Colon(String),
}

pub enum Parse {
    Done(Cmd),
    /// A valid start: wait for more.
    Pending,
    Unknown,
}

const TRACK_KEYS: [char; 7] = ['l', 's', 'b', 'm', 't', 'T', 'd'];
const ARTIST_KEYS: [char; 5] = ['l', 's', 'b', 'e', 'L'];

/// Match the pending buffer against the grammar of 0015.
pub fn parse(buf: &str) -> Parse {
    let c: Vec<char> = buf.chars().collect();
    let digit = |ch: char| ch.to_digit(10).filter(|n| *n > 0).map(|n| n as usize);

    match c.as_slice() {
        [] => Parse::Pending,

        // a lone digit: branch, or search result — the caller decides
        [d] if digit(*d).is_some() => Parse::Done(Cmd::Digit(digit(*d).unwrap())),

        // --- f, the branch namespace ---
        ['f'] | ['f', 'n'] | ['f', '!'] => Parse::Pending,
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

        // --- the bare keyboard ---
        ['h'] => Parse::Done(Cmd::Prev),
        ['l'] => Parse::Done(Cmd::Next),
        [' '] => Parse::Done(Cmd::PlayPause),
        ['u'] => Parse::Done(Cmd::Undo),
        ['.'] => Parse::Done(Cmd::Repeat),
        ['?'] => Parse::Done(Cmd::Why),
        ['Q'] => Parse::Done(Cmd::Queue),
        ['q'] => Parse::Done(Cmd::Quit),
        ['\r'] | ['\n'] => Parse::Done(Cmd::Auto),

        _ => Parse::Unknown,
    }
}

/// Put the terminal in raw mode and put it back when dropped — including on
/// a panic, which is why this is a guard and not a pair of calls.
pub struct RawMode(libc::termios);

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
            let mut raw = original;
            // no canonical line buffering, no echo: keys arrive one by one
            raw.c_lflag &= !(libc::ICANON | libc::ECHO);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) != 0 {
                return None;
            }
            Some(RawMode(original))
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

/// Read keys on a blocking thread and send commands. Owns the pending
/// buffer and the line mode, so the async side only ever sees a `Cmd`.
pub fn spawn_reader(tx: UnboundedSender<Cmd>) {
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut byte = [0u8; 1];
        let mut pending = String::new();

        while stdin.read_exact(&mut byte).is_ok() {
            let key = byte[0] as char;

            // arrows arrive as ESC [ C / ESC [ D
            if byte[0] == 0x1b {
                let mut rest = [0u8; 2];
                if stdin.read_exact(&mut rest).is_ok() {
                    let cmd = match rest {
                        [b'[', b'C'] => Some(Cmd::Next),
                        [b'[', b'D'] => Some(Cmd::Prev),
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        clear_pending(&mut pending);
                        if tx.send(cmd).is_err() {
                            return;
                        }
                        continue;
                    }
                }
                clear_pending(&mut pending);
                continue;
            }

            // `/` and `:` open a line: a query is typed, not chorded
            if pending.is_empty() && (key == '/' || key == ':') {
                if let Some(text) = read_line(&mut stdin, key) {
                    let cmd = if key == '/' { Cmd::Search(text) } else { Cmd::Colon(text) };
                    if tx.send(cmd).is_err() {
                        return;
                    }
                }
                continue;
            }

            pending.push(key);
            match parse(&pending) {
                Parse::Done(cmd) => {
                    clear_pending(&mut pending);
                    if tx.send(cmd).is_err() {
                        return;
                    }
                }
                Parse::Pending => {
                    // show what we are waiting on, the way vim shows a
                    // half-typed command
                    print!("{key}");
                    std::io::stdout().flush().ok();
                }
                Parse::Unknown => {
                    let shown = pending.clone();
                    clear_pending(&mut pending);
                    println!("\r\x1b[K(inconnu : {shown})");
                    std::io::stdout().flush().ok();
                }
            }
        }
    });
}

/// Erase the half-typed command from the line before printing anything else.
fn clear_pending(pending: &mut String) {
    if !pending.is_empty() {
        print!("\r\x1b[K");
        std::io::stdout().flush().ok();
    }
    pending.clear();
}

/// Read a line in raw mode: echo, backspace, Enter to send, Esc to cancel.
fn read_line(stdin: &mut std::io::Stdin, prefix: char) -> Option<String> {
    let mut text = String::new();
    print!("\n{prefix}");
    std::io::stdout().flush().ok();
    let mut byte = [0u8; 1];
    while stdin.read_exact(&mut byte).is_ok() {
        match byte[0] {
            b'\r' | b'\n' => {
                println!();
                return Some(text);
            }
            0x1b => {
                println!("\r\x1b[K(annulé)");
                return None;
            }
            0x7f | 0x08 => {
                if text.pop().is_some() {
                    print!("\x08 \x08");
                    std::io::stdout().flush().ok();
                }
            }
            b if b.is_ascii_graphic() || b == b' ' => {
                text.push(b as char);
                print!("{}", b as char);
                std::io::stdout().flush().ok();
            }
            _ => {}
        }
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
            "0123456789fenatlsbmTdLpruwhq Q.?!\r".chars().collect();
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
                    "« {a} » est un préfixe de « {b} » : il ne pourrait jamais se déclencher"
                );
            }
        }
    }

    #[test]
    fn les_trois_variantes() {
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

    #[test]
    fn namespaces_et_clavier_nu() {
        assert_eq!(parse("tl").done(), Some(Cmd::Track('l')));
        assert_eq!(parse("al").done(), Some(Cmd::Artist('l')));
        assert_eq!(parse("aL").done(), Some(Cmd::Artist('L')));
        assert_eq!(parse("h").done(), Some(Cmd::Prev));
        assert_eq!(parse("l").done(), Some(Cmd::Next));
        assert_eq!(parse("fu").done(), Some(Cmd::ForkUndo));
        assert!(matches!(parse("t"), Parse::Pending));
        assert!(matches!(parse("tz"), Parse::Unknown));
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
