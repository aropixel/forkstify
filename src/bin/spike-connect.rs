//! Spotify Connect spike — step 0 of the sound work. Tests the two
//! unverified points of docs/design/spotify.md:
//!
//! 1. Inbound zeroconf discovery with the current phone app: forkstify
//!    appears in the phone's device list, tapping it sends credentials
//!    over the local network (Omarchy-Spotify disabled this without
//!    saying why).
//! 2. A Web API token drawn from the librespot session, with no client
//!    id of our own (nobody in the ecosystem does this; low confidence).
//!
//! Run it on the host, phone on the same network. Whatever the outcome,
//! the fallback is known: browser OAuth + ncspot's client id.

use futures_util::StreamExt;
use librespot_core::cache::Cache;
use librespot_core::{Session, SessionConfig};
use librespot_discovery::{DeviceType, Discovery};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SessionConfig::default();
    // reusable credentials, so the phone tap happens once per machine
    let cache = Cache::new(Some("target/spike-cache"), None, None, None)?;

    let credentials = match cache.credentials() {
        Some(saved) => {
            println!("Identifiants en cache (target/spike-cache), pas besoin du téléphone.");
            saved
        }
        None => {
            let mut discovery =
                Discovery::builder(config.device_id.clone(), config.client_id.clone())
                    .name("forkstify (spike)")
                    .device_type(DeviceType::Computer)
                    .launch()?;

            println!("En attente sur le réseau local (mDNS).");
            println!("Sur le téléphone : Spotify → un morceau → l'icône des appareils → « forkstify (spike) ».");

            let credentials = discovery
                .next()
                .await
                .ok_or("découverte interrompue sans identifiants")?;
            println!("\n✓ Point 1 — zeroconf entrant : identifiants reçus du téléphone.");
            discovery.shutdown().await;
            credentials
        }
    };

    let session = Session::new(config, Some(cache));
    session.connect(credentials, true).await?;
    println!("✓ Session librespot ouverte (utilisateur : {}).", session.username());

    // keymaster (mercury) answered 403 "Invalid request" on 03/09/2026 —
    // the legacy path is closing. login5 is the modern one, used by
    // librespot itself for its internal calls.
    let token = session.login5().auth_token().await?;
    println!(
        "✓ Jeton login5 tiré de la session (expire dans {:?}).",
        token.expires_in
    );

    let me: serde_json::Value = match ureq::get("https://api.spotify.com/v1/me")
        .set("Authorization", &format!("Bearer {}", token.access_token))
        .call()
    {
        Ok(response) => response.into_json()?,
        Err(ureq::Error::Status(code, response)) => {
            // diagnose: 429/403 are the known symptoms of a client id
            // in restricted quota (spotify-player README)
            let retry_after = response.header("retry-after").unwrap_or("?").to_string();
            let body = response.into_string().unwrap_or_default();
            println!(
                "✗ Point 2 — /v1/me refuse : {code}, Retry-After: {retry_after}, corps : {body}"
            );
            return Err(format!("API Web refusée ({code})").into());
        }
        Err(e) => return Err(e.into()),
    };
    println!(
        "✓ Point 2 — API Web : /v1/me répond ({}, produit {}).",
        me["display_name"].as_str().unwrap_or("?"),
        me["product"].as_str().unwrap_or("?")
    );

    // the call the seed picker will need: the user's library
    let albums: serde_json::Value = ureq::get("https://api.spotify.com/v1/me/albums?limit=3")
        .set("Authorization", &format!("Bearer {}", token.access_token))
        .call()?
        .into_json()?;
    let names: Vec<&str> = albums["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|i| i["album"]["name"].as_str())
                .collect()
        })
        .unwrap_or_default();
    println!("✓ Bibliothèque lisible : {} albums aimés, p. ex. {names:?}.",
        albums["total"]);

    println!("\nSpike concluant : les deux points non vérifiés tiennent.");
    Ok(())
}
