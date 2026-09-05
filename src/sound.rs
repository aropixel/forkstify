//! The sound: forkstify embeds the librespot player and is itself the
//! Connect device (validated by spike-play). The branch engine never
//! touches this module — it produces tracks, this plays them.

use librespot_core::cache::Cache;
use librespot_core::{Session, SessionConfig};
use librespot_playback::audio_backend;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::mixer::NoOpVolume;
use librespot_playback::player::{Player, PlayerEvent, PlayerEventChannel};
use std::sync::Arc;

const CREDENTIALS_CACHE: &str = "target/spike-cache";

pub struct Sound {
    player: Arc<Player>,
}

impl Sound {
    /// Reuse the credentials cached by the discovery step; the phone tap
    /// only ever happens once per machine.
    pub async fn connect() -> Result<Sound, Box<dyn std::error::Error>> {
        let cache = Cache::new(Some(CREDENTIALS_CACHE), None, None, None)?;
        let credentials = cache.credentials().ok_or(
            "pas d'identifiants librespot en cache — lance d'abord ./target/release/spike-connect",
        )?;

        let session = Session::new(SessionConfig::default(), Some(cache));
        session.connect(credentials, true).await?;

        let backend = audio_backend::find(None).ok_or("aucun backend audio")?;
        let player = Player::new(
            PlayerConfig::default(),
            session,
            Box::new(NoOpVolume),
            move || backend(None, AudioFormat::default()),
        );
        Ok(Sound { player })
    }

    pub fn play(&self, uri: librespot_core::SpotifyUri) {
        self.player.load(uri, true, 0);
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn events(&self) -> PlayerEventChannel {
        self.player.get_player_event_channel()
    }
}

/// If this event ends a track (finished, stopped, or unplayable), the play
/// request id it belongs to — so the caller can ignore events left over
/// from a track it already moved past.
pub fn track_over(event: &PlayerEvent) -> Option<u64> {
    match event {
        PlayerEvent::EndOfTrack { play_request_id, .. }
        | PlayerEvent::Stopped { play_request_id, .. }
        | PlayerEvent::Unavailable { play_request_id, .. } => Some(*play_request_id),
        _ => None,
    }
}

/// Did this event end a track by *playing it through*? A skip or a failure
/// also ends it, but they are not a listen and must not count as one (0014).
pub fn track_finished(event: &PlayerEvent) -> bool {
    matches!(event, PlayerEvent::EndOfTrack { .. })
}

/// The id of a newly started play request (emitted at the top of load).
pub fn request_started(event: &PlayerEvent) -> Option<u64> {
    match event {
        PlayerEvent::PlayRequestIdChanged { play_request_id } => Some(*play_request_id),
        _ => None,
    }
}
