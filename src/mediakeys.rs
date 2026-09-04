//! MPRIS D-Bus interface, so the desktop's media keys (⏮ ⏭ ⏯) and tools
//! like playerctl drive forkstify. The keys are captured by the desktop and
//! forwarded to the active MPRIS player; here we register as one and turn
//! each call into a Control the listen loop already knows how to handle.
//!
//! The mpris-server Player keeps its callbacks in a RefCell (single-thread),
//! so this runs on the current-thread runtime via spawn_local; the callbacks
//! only forward a Control over a channel, keeping D-Bus off the audio path.

use mpris_server::{PlaybackStatus, Player};
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
        .playback_status(PlaybackStatus::Playing)
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
