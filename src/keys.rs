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

use std::sync::atomic::{AtomicBool, Ordering};
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
    /// `cc` — comfort : ouvre le réglage, les flèches le bougent, entrée valide.
    ComfortMode,
    /// `c<n>` — la zone de confort, d'un coup (Joel, 08/09/2026).
    Comfort(u8),
    /// `s` — sort : change l'ordre de la collection, à l'accueil.
    Sort,
    /// `gg` et `G` — les deux bouts d'une liste, comme dans vim. `g` seul
    /// n'est rien : il attend son second (Joel, 06/09/2026).
    Top,
    Bottom,
    /// `J` / `K` — déplacer la ligne surlignée d'un cran dans la file, tout
    /// de suite (Joel, 08/09/2026). La touche contraire annule.
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
    Repeat,
    /// `r` — resume: reprendre le dernier parcours (accueil).
    Resume,
    /// `b` — browse: parcourir à sec, sans son (écrans non connectés).
    Browse,
    Why,
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

    // — les touches d'une modale, qui a sa propre table —
    /// `e` — mettre à la file le morceau sous le curseur, sans fermer :
    /// une édition ne sonne qu'au prochain lancement, la file, elle, sonne
    /// ce soir.
    Enqueue,
    /// `v` — la vue : cycler le filtre de provenance.
    Filter,
    /// `A` — promouvoir l'album entier, au grain du problème.
    AlbumTop,
}

/// Une modale prend le clavier et lui donne **sa** table — `keybindings.md`
/// le prévoit depuis le début (« un mode à part, avec sa propre table »).
/// Le lecteur de touches vit dans un fil et ne connaît pas l'état de
/// l'écran : c'est donc un atomique, posé à l'ouverture et rendu à la
/// fermeture. La séquence en cours est oubliée au changement, sans quoi un
/// `t` tapé d'un côté se compléterait de l'autre.
static MODAL: AtomicBool = AtomicBool::new(false);
/// Text mode (Joel, 08/09/2026): the search modal owns the keyboard — every
/// printable key is typed, arrows move, enter takes, escape closes, tab
/// toggles the scope. No grammar, or « cros » would fire c, r, o, s.
static TEXT: AtomicBool = AtomicBool::new(false);

/// Ce que la ligne du mode texte contient à l'ouverture — `:search bowie`
/// ouvre la modale déjà remplie, et la frappe suivante doit **continuer**
/// ce mot, pas l'effacer (Joel, 09/09/2026).
static TEXT_LINE: Mutex<String> = Mutex::new(String::new());

pub fn set_text(on: bool, start: &str) {
    if let Ok(mut line) = TEXT_LINE.lock() {
        line.clear();
        line.push_str(start);
    }
    TEXT.store(on, Ordering::Relaxed);
}

fn text() -> bool {
    TEXT.load(Ordering::Relaxed)
}

pub fn set_modal(on: bool) {
    MODAL.store(on, Ordering::Relaxed);
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
const TRACK_KEYS: [char; 9] = ['l', 's', 'b', 'm', 't', 'T', 'd', 'x', 'i'];
const ARTIST_KEYS: [char; 7] = ['l', 's', 'b', 'e', 'L', 'd', 'g'];
/// Dans la modale de la discographie, `t` ne sert qu'à ce qui a un sens sur
/// une ligne de liste : les deux éditions et les deux mesures.
// plus de `tt` / `tT` ici non plus (Joel, 08/09/2026) : la modale aime,
// bannit, met à la file — et `A` promeut un album d'un coup
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
        // c est un namespace depuis le 08/09/2026 : c<n> règle, cc ouvre la
        // jauge — c seul ne peut plus être complet sans casser la grammaire
        ['c'] => Parse::Pending,
        ['c', 'c'] => Parse::Done(Cmd::ComfortMode),
        ['c', d] if ('0'..='5').contains(d) => Parse::Done(Cmd::Comfort(*d as u8 - b'0')),
        ['s'] => Parse::Done(Cmd::Sort),
        // la vue de la collection, aimés ⇄ tous — le même mot que dans la
        // discographie (Joel, 09/09/2026)
        ['v'] => Parse::Done(Cmd::Filter),
        ['g'] => Parse::Pending,
        ['g', 'g'] => Parse::Done(Cmd::Top),
        ['G'] => Parse::Done(Cmd::Bottom),
        ['J'] => Parse::Done(Cmd::MoveDown),
        ['K'] => Parse::Done(Cmd::MoveUp),
        ['u'] => Parse::Done(Cmd::Undo),
        ['.'] => Parse::Done(Cmd::Repeat),
        ['?'] => Parse::Done(Cmd::Why),
        ['q'] => Parse::Done(Cmd::Quit),
        ['\r'] | ['\n'] => Parse::Done(Cmd::Auto),

        _ => Parse::Unknown,
    }
}

/// La table de la modale de la discographie (maquette 1a). Elle est
/// **sans préfixe** comme l'autre, et elle emprunte à vim ce que la
/// grammaire de l'écoute laisse libre : `j`/`k` descendent et montent,
/// `h`/`l` plient et déplient — un axe vertical, cette fois.
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

/// One byte from the terminal, straight from the descriptor: std's stdin
/// buffers, and a buffered « ESC [ A » would hide its tail from `poll` —
/// the escape would then look alone, and « [ » « A » would be typed.
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

/// After an ESC: the tail of an escape sequence (« [ A » for ↑, or « O A »
/// in application mode) if it is already there, `None` for the Escape key
/// alone. The reader used to wait for two more bytes whatever happened, so
/// a lone escape needed two more keystrokes to pass — « je dois souvent
/// appuyer plusieurs fois » (Joel, 08/09/2026). A terminal delivers a
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
        // la ligne du mode texte : elle vit ici, l'écran n'en voit que l'état
        let mut line = String::new();
        let mut was_text = text();

        while let Some(b) = raw_byte() {
            let byte = [b];
            let key = b as char;

            if text() != was_text {
                was_text = !was_text;
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

            // l'écran a changé de table sous nos pieds : la séquence en
            // cours appartenait à l'autre
            if modal() != was_modal {
                was_modal = !was_modal;
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
                match read_line(key, &tx) {
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
            let outcome = if was_modal { parse_modal(&pending) } else { parse(&pending) };
            match outcome {
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
fn read_line(prefix: char, tx: &UnboundedSender<Cmd>) -> Option<String> {
    let mut text = String::new();
    let _ = tx.send(Cmd::Typing(Some(prefix.to_string())));
    while let Some(b) = raw_byte() {
        match b {
            b'\r' | b'\n' => return Some(text),
            // une flèche dans une ligne ne fait rien, mais ses octets ne
            // doivent pas retomber dans la grammaire
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
        assert_eq!(parse("ag").done(), Some(Cmd::Artist('g')));
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
        // 0018 : « tt » se lit encore — la discographie s'en sert — mais
        // c'est la session qui le refuse en écoute
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
