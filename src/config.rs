//! User configuration, read from `$XDG_CONFIG_HOME/forkstify/config.toml`
//! (or `~/.config/forkstify/config.toml`). Missing file or fields fall back
//! to defaults; a commented default file is written on first run so the
//! options are discoverable.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub playback: Playback,
    #[serde(default)]
    pub journey: Journey,
    #[serde(default, alias = "catalogue")]
    pub catalog: Catalogue,
}

#[derive(Deserialize)]
pub struct Catalogue {
    /// Le catalogue actif : un clone du dépôt de référence, ou de son fork
    /// ([0004](../docs/decisions/0004-deux-depots-catalogue-ciblable.md) :
    /// importer, c'est cloner). Vide = `~/Work/forkstify-catalog`.
    #[serde(default)]
    pub path: String,
}

impl Default for Catalogue {
    fn default() -> Self {
        Catalogue { path: String::new() }
    }
}

#[derive(Deserialize)]
pub struct Journey {
    /// Comfort zone (0001): 5 = cocon, 0 = exploration. Set at opening,
    /// adjustable in the journey with `:comfort`.
    #[serde(default = "three")]
    pub comfort: u8,
}

fn three() -> u8 {
    3
}

impl Default for Journey {
    fn default() -> Self {
        Journey { comfort: three() }
    }
}

#[derive(Deserialize)]
pub struct Playback {
    /// Prefer studio versions over live ones when resolving a title.
    #[serde(default = "yes")]
    pub prefer_studio: bool,
}

fn yes() -> bool {
    true
}

impl Default for Playback {
    fn default() -> Self {
        Playback { prefer_studio: yes() }
    }
}

const TEMPLATE: &str = "\
# forkstify — configuration

[playback]
# Prefer studio versions over live ones when resolving a track.
prefer_studio = true

[journey]
# Comfort zone, from 0 to 5: 5 = cocoon (stay with what you know),
# 0 = exploration (head for what you don't). Adjustable while listening
# with c or :comfort 4.
comfort = 3

[catalog]
# The active catalog: a clone of the reference repository, or of your fork
# of it. Empty = ~/Work/forkstify-catalog. The command-line argument
# overrides it.
# path = \"/home/me/Work/forkstify-catalog\"
";

fn path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"));
    base.join("forkstify").join("config.toml")
}

impl Config {
    pub fn load() -> Config {
        let file = path();
        match std::fs::read_to_string(&file) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                eprintln!("config unreadable ({e}), using defaults");
                Config::default()
            }),
            Err(_) => {
                // first run: drop a commented default so it's discoverable
                if let Some(dir) = file.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = std::fs::write(&file, TEMPLATE);
                Config::default()
            }
        }
    }
}

/// Where tokens and caches live: `$XDG_STATE_HOME/forkstify`, else
/// `~/.local/state/forkstify`. They used to sit in `target/`, relative to
/// the directory the binary was launched from — which a launch from the
/// desktop's bar no longer has (0021).
pub fn state_dir() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/state"));
    let dir = base.join("forkstify");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// A state file, taken over from its old place under `target/` the first
/// time — so nobody has to authorize Spotify again after the move.
pub fn state_file(name: &str, legacy: &str) -> PathBuf {
    let path = state_dir().join(name);
    if !path.exists() {
        if let Ok(bytes) = std::fs::read(legacy) {
            let _ = std::fs::write(&path, bytes);
        }
    }
    path
}
