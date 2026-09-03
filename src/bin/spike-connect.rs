//! Spotify Connect spike — step 0 of the sound work. Tests the two
//! unverified points of docs/conception/spotify.md:
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
use librespot_core::{Session, SessionConfig};
use librespot_discovery::{DeviceType, Discovery};

const SCOPES: &str = "user-read-private,user-library-read";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SessionConfig::default();

    let mut discovery = Discovery::builder(config.device_id.clone(), config.client_id.clone())
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

    let session = Session::new(config, None);
    session.connect(credentials, false).await?;
    println!("✓ Session librespot ouverte (utilisateur : {}).", session.username());

    let token = session.token_provider().get_token(SCOPES).await?;
    println!(
        "✓ Jeton tiré de la session : scopes {:?}, expire dans {:?}.",
        token.scopes, token.expires_in
    );

    let me: serde_json::Value = ureq::get("https://api.spotify.com/v1/me")
        .set("Authorization", &format!("Bearer {}", token.access_token))
        .call()?
        .into_json()?;
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
