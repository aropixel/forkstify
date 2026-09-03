//! Playback spike — the last sound brick. forkstify embeds the librespot
//! player (not just discovery + token) and plays a track through the host
//! audio, proving the target architecture of docs/conception/spotify.md:
//! "forkstify est lui-même l'appareil Connect". No Web API player control,
//! no other device — the sound comes out of this binary.
//!
//! Reuses the credentials cached by spike-connect (target/spike-cache),
//! so no phone tap. Run on the host (needs audio + Premium):
//!
//!   ./target/release/spike-play [spotify:track:...]
//!
//! Default track: The Cure — A Forest (resolved by spike-webapi).

use librespot_core::{Session, SessionConfig, SpotifyUri};
use librespot_core::cache::Cache;
use librespot_playback::audio_backend;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::mixer::NoOpVolume;
use librespot_playback::player::{Player, PlayerEvent};

const DEFAULT_TRACK: &str = "spotify:track:4iVTSRiJAA18d3QglhyJ6Q"; // A Forest

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_TRACK.to_string());
    let track = SpotifyUri::from_uri(&uri)?;

    let cache = Cache::new(Some("target/spike-cache"), None, None, None)?;
    let credentials = cache
        .credentials()
        .ok_or("pas d'identifiants en cache — lance d'abord spike-connect")?;

    let session = Session::new(SessionConfig::default(), Some(cache));
    session.connect(credentials, true).await?;
    println!("✓ Session librespot ouverte (utilisateur : {}).", session.username());

    // embed the player with the default audio backend (rodio → alsa)
    let backend = audio_backend::find(None).ok_or("aucun backend audio")?;
    let player = Player::new(
        PlayerConfig::default(),
        session,
        Box::new(NoOpVolume),
        move || backend(None, AudioFormat::default()),
    );

    let mut events = player.get_player_event_channel();
    println!("Chargement de {uri} …");
    player.load(track, true, 0);

    // play ~15s, then stop — enough to hear it and see the events
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        tokio::select! {
            event = events.recv() => match event {
                Some(PlayerEvent::Playing { .. }) => println!("✓ Lecture en cours — le son doit sortir maintenant."),
                Some(PlayerEvent::TrackChanged { audio_item }) => {
                    println!("  piste : {} ({} ms)", audio_item.name, audio_item.duration_ms);
                }
                Some(PlayerEvent::Unavailable { .. }) => {
                    println!("✗ Piste indisponible pour librespot (audio_key_unavailable ?).");
                    break;
                }
                Some(PlayerEvent::EndOfTrack { .. }) | None => break,
                _ => {}
            },
            _ = tokio::time::sleep_until(deadline) => {
                println!("(15 s écoulées, on arrête)");
                break;
            }
        }
    }

    player.stop();
    println!("\nSpike concluant : le son sort du binaire forkstify.");
    Ok(())
}
