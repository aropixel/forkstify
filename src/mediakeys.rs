//! MPRIS D-Bus interface, so the desktop's media keys (⏮ ⏭ ⏯) and tools
//! like playerctl drive forkstify. The keys are captured by the desktop and
//! forwarded to the active MPRIS player; here we register as one and turn
//! each call into a Control the listen loop already knows how to handle.
//!
//! The mpris-server Player keeps its callbacks in a RefCell (single-thread),
//! so this runs on the current-thread runtime via spawn_local; the callbacks
//! only forward a Control over a channel, keeping D-Bus off the audio path.

use mpris_server::{Metadata, PlaybackStatus, Player, Time, TrackId};
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Copy, Debug)]
pub enum Control {
    Next,
    Previous,
    PlayPause,
    Stop,
}

/// Register the MPRIS player and spawn its D-Bus task. Returns the Player,
/// which must be kept alive (dropping it unregisters the interface) and lets
/// us push the playback status back to the desktop.
pub async fn start(tx: UnboundedSender<Control>) -> Result<Player, Box<dyn std::error::Error>> {
    let player = Player::builder("forkstify")
        .identity("forkstify")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .can_go_previous(true)
        .can_control(true)
        // nothing on air until the mirror says so — declared "playing"
        // here, the bar danced during a :search (Joel, 10/09/2026)
        .playback_status(PlaybackStatus::Stopped)
        .build()
        .await?;

    let forward = |tx: UnboundedSender<Control>, control: Control| {
        move |_: &Player| {
            let _ = tx.send(control);
        }
    };
    player.connect_next(forward(tx.clone(), Control::Next));
    player.connect_previous(forward(tx.clone(), Control::Previous));
    player.connect_play_pause(forward(tx.clone(), Control::PlayPause));
    player.connect_play(forward(tx.clone(), Control::PlayPause));
    player.connect_pause(forward(tx.clone(), Control::PlayPause));
    player.connect_stop(forward(tx, Control::Stop));

    tokio::task::spawn_local(player.run());
    Ok(player)
}

/// What the desktop is told about the track (0021): the same state the
/// screen draws, pushed by the listen loop whenever it changes. `next` is
/// "title — artist" of the queue's head, the one thing MPRIS has no word
/// for — it travels as the custom key `forkstify:next`.
#[derive(Clone, PartialEq, Default)]
pub struct Shown {
    pub title: String,
    pub artist: String,
    pub next: String,
    pub length_ms: u32,
    /// Some(true) plays, Some(false) is paused, None: nothing on air.
    pub playing: Option<bool>,
    /// Counts the tracks, so `mpris:trackid` changes with each one.
    pub serial: u64,
}

/// Push a track and its state to the desktop. The D-Bus calls are async
/// and the Player is single-thread: they run as a local task, off the
/// caller's path. Errors are the desktop's problem, not the music's.
/// `Seeked` is how MPRIS says the needle jumped — a start, a resume, a
/// new track: without it a desktop keeps the position it read at
/// discovery, zero, and its bar never moves (Joel, 10/09/2026).
pub fn publish(player: &Rc<Player>, shown: &Shown, track_changed: bool, position_ms: u32) {
    let player = player.clone();
    let shown = shown.clone();
    tokio::task::spawn_local(async move {
        if track_changed {
            let trackid = TrackId::try_from(format!("/org/forkstify/track/{}", shown.serial))
                .unwrap_or(TrackId::NO_TRACK);
            let mut metadata = Metadata::builder()
                .trackid(trackid)
                .title(shown.title.clone())
                .artist([shown.artist.clone()])
                .length(Time::from_millis(i64::from(shown.length_ms)))
                .build();
            metadata.set("forkstify:next", Some(shown.next.clone()));
            let _ = player.set_metadata(metadata).await;
        }
        let status = match shown.playing {
            Some(true) => PlaybackStatus::Playing,
            Some(false) => PlaybackStatus::Paused,
            None => PlaybackStatus::Stopped,
        };
        let _ = player.set_playback_status(status).await;
        let position = Time::from_millis(i64::from(position_ms));
        player.set_position(position);
        let _ = player.seeked(position).await;
    });
}

/// The needle, for the desktop's progress bar. Cheap: a stored value the
/// desktop reads when it asks.
pub fn position(player: &Rc<Player>, position_ms: u32) {
    player.set_position(Time::from_millis(i64::from(position_ms)));
}

/// The needle jumped without anything else changing — `h` restarting
/// the track, a correction from librespot: say so, or the desktop's bar
/// keeps counting from where it was.
pub fn seeked(player: &Rc<Player>, position_ms: u32) {
    let player = player.clone();
    tokio::task::spawn_local(async move {
        let position = Time::from_millis(i64::from(position_ms));
        player.set_position(position);
        let _ = player.seeked(position).await;
    });
}
