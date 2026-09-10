//! The sound: forkstify embeds the librespot player and is itself the
//! Connect device (validated by spike-play). The branch engine never
//! touches this module — it produces tracks, this plays them.

use futures_util::StreamExt;
use librespot_core::cache::Cache;
use librespot_core::{Session, SessionConfig};
use librespot_discovery::{DeviceType, Discovery};
use librespot_playback::audio_backend;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::mixer::NoOpVolume;
use librespot_playback::player::{Player, PlayerEvent, PlayerEventChannel};
use std::sync::Arc;

/// librespot's cache: the credentials the phone handed over. Taken over
/// from `target/spike-cache` once, like the other state files.
fn credentials_cache() -> std::path::PathBuf {
    let dir = crate::config::state_dir().join("librespot");
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("credentials.json");
    if !file.exists() {
        if let Ok(bytes) = std::fs::read("target/spike-cache/credentials.json") {
            let _ = std::fs::write(&file, bytes);
        }
    }
    dir
}

/// The name that shows up in the phone's device list. It is the product's
/// name in the outside world, so it says the distribution, not the spike it
/// came from.
pub const DEVICE_NAME: &str = "forkstify";

/// Are there credentials on disk? Asked before anything is opened, so the
/// home screen can say what is connected without connecting.
pub fn has_credentials() -> bool {
    Cache::new(Some(credentials_cache()), None, None, None)
        .ok()
        .and_then(|cache| cache.credentials())
        .is_some()
}

/// Advertise on the local network and wait for the phone to hand over
/// credentials. This used to live in the `spike-connect` binary; it belongs
/// in the application, because "connecting the phone" is part of the
/// product, not a setup step run once by hand.
pub async fn discover() -> Result<String, Box<dyn std::error::Error>> {
    let config = SessionConfig::default();
    let cache = Cache::new(Some(credentials_cache()), None, None, None)?;
    let mut discovery = Discovery::builder(config.device_id.clone(), config.client_id.clone())
        .name(DEVICE_NAME)
        .device_type(DeviceType::Computer)
        .launch()?;

    let credentials = discovery
        .next()
        .await
        .ok_or("discovery stopped without credentials")?;
    discovery.shutdown().await;

    // opening the session is what writes the credentials to the cache
    let session = Session::new(config, Some(cache));
    session.connect(credentials, true).await?;
    Ok(session.username())
}

pub struct Sound {
    player: Arc<Player>,
}

impl Sound {
    /// Reuse the credentials cached by the discovery step; the phone tap
    /// only ever happens once per machine.
    pub async fn connect() -> Result<Sound, Box<dyn std::error::Error>> {
        let cache = Cache::new(Some(credentials_cache()), None, None, None)?;
        let credentials = cache
            .credentials()
            .ok_or("no librespot credentials — the home screen asks the phone for them")?;

        let session = Session::new(SessionConfig::default(), Some(cache));
        session.connect(credentials, true).await?;

        let backend = audio_backend::find(None).ok_or("no audio backend")?;
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

    /// Back to the start of the track — librespot then says `Seeked`, and
    /// the progress follows.
    pub fn restart(&self) {
        self.player.seek(0);
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
