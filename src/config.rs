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
# Privilégier les versions studio plutôt que live à la résolution d'un titre.
prefer_studio = true
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
                eprintln!("config illisible ({e}), valeurs par défaut");
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
