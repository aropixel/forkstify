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

use std::io::Read;
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
    /// L'axe s'affiche verticalement : les flèches verticales y déplacent une
    /// **sélection**, elles ne jouent rien. C'est « entrée » qui joue
    /// (Joel, 06/09/2026).
    Up,
    Down,
    /// Échap — annule la sélection, ferme un volet, sort d'un mode.
    Escape,
    /// `c` — comfort : ouvre le réglage, les flèches le bougent, entrée valide.
    ComfortMode,
    /// `s` — sort : change l'ordre de la collection, à l'accueil.
    Sort,
    /// `gg` et `G` — les deux bouts d'une liste, comme dans vim. `g` seul
    /// n'est rien : il attend son second (Joel, 06/09/2026).
    Top,
    Bottom,
    PlayPause,
    /// Space, the leader: show what is available. Carries the namespace
    /// that was half-typed, so `f` then space lists only the branch keys —
    /// which-key, in a terminal.
    Help(Option<char>),
    Undo,
    Repeat,
    /// `r` — resume: reprendre le dernier parcours (accueil).
    Resume,
    /// `b` — browse: parcourir à sec, sans son (écrans non connectés).
    Browse,
    Why,
    Queue,
    Quit,
    Search(String),
    Colon(String),
    /// La séquence à moitié tapée, ou vide quand elle se referme. Le lecteur
    /// **n'imprime plus rien** : l'écran appartient à la TUI, et un octet
    /// écrit derrière son dos y laisse des restes qu'elle ne sait pas
    /// effacer (relevé par Joel le 06/09/2026).
    Pending(String),
    /// Une ligne en cours de frappe (`/` ou `:`), préfixe compris.
    Typing(Option<String>),
    /// Une séquence qui ne veut rien dire.
    Unknown(String),
}

pub enum Parse {
    Done(Cmd),
    /// A valid start: wait for more.
    Pending,
    Unknown,
}

const TRACK_KEYS: [char; 8] = ['l', 's', 'b', 'm', 't', 'T', 'd', 'x'];
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
        ['p'] => Parse::Done(Cmd::PlayPause),
        ['r'] => Parse::Done(Cmd::Resume),
        ['b'] => Parse::Done(Cmd::Browse),
        ['c'] => Parse::Done(Cmd::ComfortMode),
        ['s'] => Parse::Done(Cmd::Sort),
        ['g'] => Parse::Pending,
        ['g', 'g'] => Parse::Done(Cmd::Top),
        ['G'] => Parse::Done(Cmd::Bottom),
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
                        [b'[', b'A'] => Some(Cmd::Up),
                        [b'[', b'B'] => Some(Cmd::Down),
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

            // space is the leader, not a key of the grammar: it reports
            // what can be typed, here or inside the pending namespace
            if key == ' ' {
                let namespace = pending.chars().next();
                clear_pending(&mut pending, &tx);
                if tx.send(Cmd::Help(namespace)).is_err() {
                    return;
                }
                continue;
            }

            // `/` and `:` open a line: a query is typed, not chorded
            if pending.is_empty() && (key == '/' || key == ':') {
                match read_line(&mut stdin, key, &tx) {
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

            // les octets de contrôle ne sont pas des touches : un NUL ou un
            // ^C arrivé du pty ne doit pas devenir « (inconnu) »
            if byte[0] < 0x20 && byte[0] != b'\r' && byte[0] != b'\n' {
                continue;
            }

            pending.push(key);
            match parse(&pending) {
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
                    // montrer ce qu'on attend, comme vim montre une commande
                    // à moitié tapée — mais c'est la TUI qui l'affiche
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

/// Oublier la séquence en cours, et le dire à l'écran.
fn clear_pending(pending: &mut String, tx: &UnboundedSender<Cmd>) {
    if !pending.is_empty() {
        let _ = tx.send(Cmd::Pending(String::new()));
    }
    pending.clear();
}

/// Lire une ligne : la frappe remonte à l'écran au lieu de s'écrire dessus.
/// Entrée envoie, échap annule.
fn read_line(stdin: &mut std::io::Stdin, prefix: char, tx: &UnboundedSender<Cmd>) -> Option<String> {
    let mut text = String::new();
    let _ = tx.send(Cmd::Typing(Some(prefix.to_string())));
    let mut byte = [0u8; 1];
    while stdin.read_exact(&mut byte).is_ok() {
        match byte[0] {
            b'\r' | b'\n' => return Some(text),
            0x1b => return None,
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
            "0123456789fenatlsbcgGmTdLpruwhqQ.?!\r".chars().collect();
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
        assert_eq!(parse("p").done(), Some(Cmd::PlayPause));
        assert_eq!(parse("r").done(), Some(Cmd::Resume));
        assert_eq!(parse("b").done(), Some(Cmd::Browse));
        assert_eq!(parse("c").done(), Some(Cmd::ComfortMode));
        assert_eq!(parse("s").done(), Some(Cmd::Sort));
        assert_eq!(parse("gg").done(), Some(Cmd::Top));
        assert_eq!(parse("G").done(), Some(Cmd::Bottom));
        assert!(matches!(parse("g"), Parse::Pending));
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
