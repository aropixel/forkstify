//! Web API spike — the retained path after spike-connect ruled out the
//! session token (429). Proves that ncspot's historical client id, via a
//! browser PKCE OAuth, gets a Web API token that is NOT rate-limited, and
//! does the one call the engine will lean on: resolve a title to a track
//! id (search). Reads the library too (the seed picker's call).
//!
//! Run on the host (opens a browser, writes the keyring later — for now a
//! gitignored refresh-token cache). Phone not needed.

use librespot_oauth::OAuthClientBuilder;
use std::time::Instant;

// ncspot's client id, in Spotify's extended quota mode — the de-facto
// standard of the Linux libre ecosystem (spotify-player, Omarchy-Spotify).
const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
const REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
const SCOPES: [&str; 5] = [
    "user-read-private",
    "user-library-read",
    "user-read-playback-state",
    "user-modify-playback-state",
    "streaming",
];
const REFRESH_CACHE: &str = "target/spike-webapi-refresh";

fn api_get(url: &str, token: &str) -> Result<serde_json::Value, String> {
    match ureq::get(url).set("Authorization", &format!("Bearer {token}")).call() {
        Ok(response) => response.into_json().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, response)) => {
            let retry_after = response.header("retry-after").unwrap_or("?").to_string();
            let body = response.into_string().unwrap_or_default();
            Err(format!("{code} (Retry-After: {retry_after}) — {body}"))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OAuthClientBuilder::new(CLIENT_ID, REDIRECT_URI, SCOPES.to_vec())
        .open_in_browser()
        .build()?;

    // reuse the refresh token when we have one, browser only on first run
    let token = match std::fs::read_to_string(REFRESH_CACHE) {
        Ok(refresh) if !refresh.trim().is_empty() => {
            println!("Refresh token en cache, pas de navigateur.");
            client.refresh_token_async(refresh.trim()).await?
        }
        _ => {
            println!("Ouverture du navigateur pour autoriser forkstify (client id ncspot)…");
            let token = client.get_access_token_async().await?;
            std::fs::write(REFRESH_CACHE, &token.refresh_token)?;
            token
        }
    };
    let seconds = token.expires_at.saturating_duration_since(Instant::now()).as_secs();
    println!("✓ Jeton OAuth obtenu : scopes {:?}, expire dans {seconds}s.", token.scopes);

    // point of the spike: is this client id rate-limited like the desktop one?
    let me = api_get("https://api.spotify.com/v1/me", &token.access_token)
        .map_err(|e| format!("/v1/me refuse : {e}"))?;
    println!(
        "✓ API Web : /v1/me répond ({}, produit {}) — pas de 429.",
        me["display_name"].as_str().unwrap_or("?"),
        me["product"].as_str().unwrap_or("?")
    );

    // the engine's call: resolve a title to a playable track id
    let query = "The Cure A Forest";
    let search = api_get(
        &format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=1",
            urlencoding(query)
        ),
        &token.access_token,
    )?;
    let track = &search["tracks"]["items"][0];
    println!(
        "✓ Résolution titre → id : « {query} » → {} ({}) [{}]",
        track["name"].as_str().unwrap_or("?"),
        track["artists"][0]["name"].as_str().unwrap_or("?"),
        track["uri"].as_str().unwrap_or("?"),
    );

    // the seed picker's call: the user's library
    let albums = api_get(
        "https://api.spotify.com/v1/me/albums?limit=3",
        &token.access_token,
    )?;
    let names: Vec<&str> = albums["items"]
        .as_array()
        .map(|items| items.iter().filter_map(|i| i["album"]["name"].as_str()).collect())
        .unwrap_or_default();
    println!("✓ Bibliothèque : {} albums aimés, p. ex. {names:?}.", albums["total"]);

    println!("\nSpike concluant : le client id ncspot porte l'API Web sans 429.");
    Ok(())
}

/// Minimal percent-encoding for a query string (spaces and the few chars
/// a track/artist name can carry). Enough for the spike.
fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
