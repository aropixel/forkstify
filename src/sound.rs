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

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn events(&self) -> PlayerEventChannel {
        self.player.get_player_event_channel()
    }
}

/// Has this event ended the current track (so we can load the next)?
pub fn is_track_over(event: &PlayerEvent) -> bool {
    matches!(
        event,
        PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. } | PlayerEvent::Unavailable { .. }
    )
}
