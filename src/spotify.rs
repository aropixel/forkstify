//! Spotify Web API: OAuth (ncspot client id, browser once then cached
//! refresh) and the one call navigation needs — resolve a title to a
//! playable track uri. Validated by spike-webapi. Resolutions are cached
//! on disk to spare the rate limit (the 429 lesson of spotify.md).

use librespot_oauth::OAuthClientBuilder;
use std::collections::HashMap;
use std::time::Instant;

const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b"; // ncspot, extended quota
const REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
const SCOPES: [&str; 5] = [
    "user-read-private",
    "user-library-read",
    "user-read-playback-state",
    "user-modify-playback-state",
    "streaming",
];
const REFRESH_CACHE: &str = "target/spike-webapi-refresh";
const RESOLVE_CACHE: &str = "target/resolve-cache.json";

pub struct WebApi {
    token: String,
    expires_at: Instant,
    refresh_token: String,
    resolved: HashMap<String, Option<String>>,
}

impl WebApi {
    /// Reuse the cached refresh token; fall back to the browser flow once.
    pub async fn new() -> Result<WebApi, Box<dyn std::error::Error>> {
        let client = OAuthClientBuilder::new(CLIENT_ID, REDIRECT_URI, SCOPES.to_vec())
            .open_in_browser()
            .build()?;

        let token = match std::fs::read_to_string(REFRESH_CACHE) {
            Ok(refresh) if !refresh.trim().is_empty() => {
                client.refresh_token_async(refresh.trim()).await?
            }
            _ => {
                println!("Autorisation Spotify dans le navigateur (une fois)…");
                let token = client.get_access_token_async().await?;
                std::fs::write(REFRESH_CACHE, &token.refresh_token)?;
                token
            }
        };

        let resolved = std::fs::read_to_string(RESOLVE_CACHE)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();

        Ok(WebApi {
            token: token.access_token,
            expires_at: token.expires_at,
            refresh_token: token.refresh_token,
            resolved,
        })
    }

    async fn refresh_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.expires_at > Instant::now() + std::time::Duration::from_secs(30) {
            return Ok(());
        }
        let client = OAuthClientBuilder::new(CLIENT_ID, REDIRECT_URI, SCOPES.to_vec()).build()?;
        let token = client.refresh_token_async(&self.refresh_token).await?;
        self.token = token.access_token;
        self.expires_at = token.expires_at;
        Ok(())
    }

    /// Resolve "title" by "artist" to a spotify:track: uri (cached). None
    /// when Spotify has no match.
    pub async fn resolve(&mut self, title: &str, artist: &str) -> Option<String> {
        let key = format!("{artist}\u{1}{title}");
        if let Some(hit) = self.resolved.get(&key) {
            return hit.clone();
        }
        self.refresh_if_needed().await.ok()?;

        let query = format!("track:{title} artist:{artist}");
        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=1",
            encode(&query)
        );
        let uri = self
            .get_with_backoff(&url)
            .await
            .and_then(|body| body["tracks"]["items"][0]["uri"].as_str().map(String::from));

        self.resolved.insert(key, uri.clone());
        if let Ok(text) = serde_json::to_string(&self.resolved) {
            let _ = std::fs::write(RESOLVE_CACHE, text);
        }
        uri
    }

    /// GET honoring Retry-After on 429 (spotify.md: throttle is account/IP).
    async fn get_with_backoff(&self, url: &str) -> Option<serde_json::Value> {
        let token = self.token.clone();
        let url = url.to_string();
        // ureq is blocking; keep it off the async reactor
        tokio::task::spawn_blocking(move || {
            for attempt in 1..=4 {
                match ureq::get(&url)
                    .set("Authorization", &format!("Bearer {token}"))
                    .call()
                {
                    Ok(response) => return response.into_json::<serde_json::Value>().ok(),
                    Err(ureq::Error::Status(429, response)) if attempt < 4 => {
                        let wait: u64 = response
                            .header("retry-after")
                            .and_then(|h| h.parse().ok())
                            .unwrap_or(5)
                            .min(60);
                        std::thread::sleep(std::time::Duration::from_secs(wait + 1));
                    }
                    Err(_) => return None,
                }
            }
            None
        })
        .await
        .ok()
        .flatten()
    }
}

/// Percent-encode a query string component.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
