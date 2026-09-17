//! User configuration, read from `$XDG_CONFIG_HOME/forkstify/config.toml`
//! (or `~/.config/forkstify/config.toml`). Missing file or fields fall back
//! to defaults; a commented default file is written on first run so the
//! options are discoverable.

use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub playback: Playback,
    #[serde(default)]
    pub journey: Journey,
    #[serde(default, alias = "catalogue")]
    pub catalog: Catalogue,
    /// Every number the engine and the learned layer reason with — the
    /// dials behind the dial ([0023](../docs/decisions/0023-les-indices-du-moteur-se-reglent.md)).
    #[serde(default)]
    pub tuning: Tuning,
}

/// The engine's numbers, all of them, so "taking back the algorithm" goes
/// down to its coefficients: what a like weighs, how long a track or an
/// artist steps back after playing, how far the adventurous branch may
/// leap. The defaults are the values the code shipped with; the section
/// `[tuning]` of the configuration overrides any of them. Read through
/// [`tuning()`]: the file is loaded once, at start, and the values do not
/// move during a journey.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Tuning {
    // --- the cooldowns (0012 §2) ---
    /// A track played today keeps this share of its weight.
    pub track_cooldown_floor: f32,
    /// In days: how long the track takes to get half of the rest back.
    pub track_cooldown_half_life: f32,
    /// An artist heard today keeps this share of their pull as a branch head.
    pub artist_cooldown_floor: f32,
    /// In days: how long the artist takes to get half of the rest back.
    pub artist_cooldown_half_life: f32,

    // --- the taste (0018): al / as, tl / ts ---
    /// "less often" multiplies the artist's weight by this.
    pub less_often: f32,
    /// "more often" multiplies it by this; 0 = the mirror of `less_often`
    /// (1 / less_often), so one gesture undoes the other.
    pub more_often: f32,
    /// The artist's weight never goes under this…
    pub weight_floor: f32,
    /// …nor over this: one key cannot run away.
    pub weight_ceiling: f32,

    // --- the familiarity (0001, 0014) ---
    /// Plays (decayed) at which familiarity reaches one half.
    pub plays_reference: f64,
    /// In days: the half-life of the play counters (0014: six months).
    pub plays_half_life: f64,

    // --- the reservoir (0012 §1) ---
    /// A top's weight: the norm the others are read against.
    pub top_weight: f32,
    /// A liked track's weight at comfort 5…
    pub liked_weight_cocoon: f32,
    /// …and at comfort 0; in between, the dial slides from one to the other.
    pub liked_weight_open: f32,
    /// A door's weight on its own.
    pub door_weight: f32,
    /// What a door is multiplied by when the branch heads its way.
    pub door_bonus: f32,
    /// One track of the long tail, before the dial and the familiarity
    /// scale it down.
    pub tail_weight: f32,

    // --- the adventurous leap (cosine in the vector space) ---
    /// Under this closeness, no leap outside the graph — at comfort 5…
    pub leap_floor_cocoon: f32,
    /// …and at comfort 0.
    pub leap_floor_open: f32,
    /// Above this closeness, a leap needs no shared genre tag — at comfort 5…
    pub leap_trust_cocoon: f32,
    /// …and at comfort 0.
    pub leap_trust_open: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            track_cooldown_floor: 0.1,
            track_cooldown_half_life: 7.0,
            artist_cooldown_floor: 0.3,
            artist_cooldown_half_life: 4.0,
            less_often: 0.7,
            more_often: 0.0,
            weight_floor: 0.1,
            weight_ceiling: 3.0,
            plays_reference: 5.0,
            plays_half_life: 182.5,
            top_weight: 1.0,
            liked_weight_cocoon: 10.0,
            liked_weight_open: 2.0,
            door_weight: 0.4,
            door_bonus: 2.5,
            tail_weight: 0.25,
            leap_floor_cocoon: 0.80,
            leap_floor_open: 0.60,
            leap_trust_cocoon: 0.86,
            leap_trust_open: 0.70,
        }
    }
}

impl Tuning {
    /// The value of "more often" as the engine uses it: the mirror of
    /// "less often" unless the file says otherwise.
    pub fn more_often(&self) -> f32 {
        if self.more_often > 0.0 { self.more_often } else { 1.0 / self.less_often }
    }

    /// A number out of its range would not make the engine explainable,
    /// it would make it absurd (a negative weight, a share above one).
    /// Each odd value is said and put back to its default — the others
    /// stay as written.
    fn sane(mut self) -> Tuning {
        let default = Tuning::default();
        macro_rules! check {
            ($field:ident, $ok:expr, $range:literal) => {
                let value = self.$field;
                if !$ok(value) {
                    eprintln!(
                        "config: tuning.{} = {} is out of range ({}), using {}",
                        stringify!($field), value, $range, default.$field
                    );
                    self.$field = default.$field;
                }
            };
        }
        let share = |v: f32| v.is_finite() && (0.0..=1.0).contains(&v);
        let days = |v: f32| v.is_finite() && v > 0.0;
        let days64 = |v: f64| v.is_finite() && v > 0.0;
        let weight = |v: f32| v.is_finite() && v > 0.0;
        let cosine = |v: f32| v.is_finite() && (-1.0..=1.0).contains(&v);
        check!(track_cooldown_floor, share, "0 to 1");
        check!(track_cooldown_half_life, days, "days, above 0");
        check!(artist_cooldown_floor, share, "0 to 1");
        check!(artist_cooldown_half_life, days, "days, above 0");
        check!(less_often, |v: f32| v.is_finite() && v > 0.0 && v <= 1.0, "above 0, up to 1");
        check!(more_often, |v: f32| v.is_finite() && (v == 0.0 || v >= 1.0), "0, or 1 and above");
        check!(weight_floor, weight, "above 0");
        check!(weight_ceiling, weight, "above 0");
        check!(plays_reference, days64, "above 0");
        check!(plays_half_life, days64, "days, above 0");
        check!(top_weight, weight, "above 0");
        check!(liked_weight_cocoon, weight, "above 0");
        check!(liked_weight_open, weight, "above 0");
        check!(door_weight, weight, "above 0");
        check!(door_bonus, weight, "above 0");
        check!(tail_weight, weight, "above 0");
        check!(leap_floor_cocoon, cosine, "-1 to 1");
        check!(leap_floor_open, cosine, "-1 to 1");
        check!(leap_trust_cocoon, cosine, "-1 to 1");
        check!(leap_trust_open, cosine, "-1 to 1");
        if self.weight_floor > self.weight_ceiling {
            eprintln!("config: tuning.weight_floor is above weight_ceiling, using the defaults for both");
            self.weight_floor = default.weight_floor;
            self.weight_ceiling = default.weight_ceiling;
        }
        self
    }
}

static TUNING: OnceLock<Tuning> = OnceLock::new();

/// The numbers in force: those of the configuration once it is loaded,
/// the defaults before that (and in the tests, which load nothing).
pub fn tuning() -> &'static Tuning {
    TUNING.get_or_init(Tuning::default)
}

#[derive(Deserialize)]
pub struct Catalogue {
    /// The active catalog: a clone of the reference repository, or of its
    /// fork ([0004](../docs/decisions/0004-deux-depots-catalogue-ciblable.md):
    /// importing is cloning). Empty = `~/Work/forkstify-catalog`.
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
    /// Comfort zone (0001): 5 = cocoon, 0 = exploration. Set at opening,
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
# with c or :comfort 4. This is the value for the first ever run: once you
# change it, forkstify remembers your last choice from one launch to the next.
comfort = 3

[catalog]
# The active catalog: a clone of the reference repository, or of your fork
# of it. Empty = ~/Work/forkstify-catalog. The command-line argument
# overrides it.
# path = \"/home/me/Work/forkstify-catalog\"

[tuning]
# Every number the engine reasons with. Taking back the algorithm goes
# down to here: change one, relaunch, and the journey follows. The values
# below are the defaults — a line you remove falls back to it.

# The cooldowns. A track played today keeps track_cooldown_floor of its
# weight in the draw and gets the rest back with a half-life in days: at
# 0.1 and 7, a week later it is at 55 %, a month later at 94 %. An artist
# heard today steps back the same way as a branch head.
track_cooldown_floor = 0.1
track_cooldown_half_life = 7.0
artist_cooldown_floor = 0.3
artist_cooldown_half_life = 4.0

# The taste. as / ts \"less often\" multiplies the artist's weight by
# less_often, al / tl \"more often\" by more_often (0 = 1 / less_often, so
# one gesture undoes the other). The weight stays between floor and
# ceiling: a key discourages or favours, it never forbids.
less_often = 0.7
more_often = 0
weight_floor = 0.1
weight_ceiling = 3.0

# The familiarity. Plays decay with a half-life in days (six months), and
# familiarity reaches one half at plays_reference decayed plays.
plays_reference = 5.0
plays_half_life = 182.5

# The reservoir of an artist: each source's weight in the draw. A top is
# the norm. A liked track weighs liked_weight_cocoon at comfort 5 and
# liked_weight_open at comfort 0. A door weighs door_weight, times
# door_bonus when the branch heads its way. A track of the long tail
# weighs tail_weight, scaled down by the dial and by how familiar the
# artist is.
top_weight = 1.0
liked_weight_cocoon = 10.0
liked_weight_open = 2.0
door_weight = 0.4
door_bonus = 2.5
tail_weight = 0.25

# The adventurous leap, in closeness (cosine) in the vector space. Under
# the floor, no leap outside the graph; above the trust, a leap needs no
# shared genre tag. Each slides from its cocoon value (comfort 5) to its
# open value (comfort 0).
leap_floor_cocoon = 0.80
leap_floor_open = 0.60
leap_trust_cocoon = 0.86
leap_trust_open = 0.70
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
        let mut config = match std::fs::read_to_string(&file) {
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
        };
        config.tuning = config.tuning.sane();
        // the engine reads its numbers from here on; the first load of the
        // process wins, and every load reads the same file
        let _ = TUNING.set(config.tuning.clone());
        config
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

/// The comfort to open with: the last one chosen, if forkstify remembers
/// one, else the config default. The state file wins so an adjustment made
/// while listening survives the next launch (Joel, 14/09/2026).
pub fn comfort_at_start() -> u8 {
    let default = Config::load().journey.comfort;
    std::fs::read_to_string(state_dir().join("comfort"))
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .filter(|n| *n <= 5)
        .unwrap_or(default)
}

/// Remember the comfort just chosen, so the next launch opens on it.
pub fn remember_comfort(value: u8) {
    let _ = std::fs::write(state_dir().join("comfort"), value.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 0023: a `[tuning]` line overrides one number and leaves the others
    /// at their default; a missing section is the defaults entire.
    #[test]
    fn a_tuning_line_overrides_one_number() {
        let config: Config = toml::from_str("[tuning]\nless_often = 0.5\ndoor_bonus = 4\n").unwrap();
        assert_eq!(config.tuning.less_often, 0.5);
        assert_eq!(config.tuning.door_bonus, 4.0);
        assert_eq!(config.tuning.track_cooldown_floor, Tuning::default().track_cooldown_floor);
        assert_eq!(config.tuning.more_often(), 2.0, "the mirror of less_often");
        let bare: Config = toml::from_str("[journey]\ncomfort = 4\n").unwrap();
        assert_eq!(bare.tuning, Tuning::default());
    }

    /// An absurd number is put back to its default, the others stay.
    #[test]
    fn an_absurd_number_falls_back_alone() {
        let tuning = Tuning { track_cooldown_floor: 3.0, top_weight: 2.0, ..Tuning::default() }.sane();
        assert_eq!(tuning.track_cooldown_floor, Tuning::default().track_cooldown_floor);
        assert_eq!(tuning.top_weight, 2.0);
        let tuning = Tuning { more_often: 1.5, ..Tuning::default() }.sane();
        assert_eq!(tuning.more_often(), 1.5, "a value of its own is kept");
    }
}
