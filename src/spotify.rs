//! Spotify Web API: OAuth (ncspot client id, browser once then cached
//! refresh) and the one call navigation needs — resolve a title to a
//! playable track uri. Validated by spike-webapi. Resolutions are cached
//! on disk to spare the rate limit (the 429 lesson of spotify.md).
//!
//! A lookup that did not go through is never a "no match": the two are
//! separate outcomes (`Resolved`), only the real absence is cached, and a
//! dead token says so instead of looking like a missing track.

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
/// The refresh token and the title → uri cache, under the state dir (0021);
/// taken over from `target/` the first time.
fn refresh_cache() -> std::path::PathBuf {
    crate::config::state_file("web-refresh-token", "target/spike-webapi-refresh")
}
fn resolve_cache() -> std::path::PathBuf {
    crate::config::state_file("resolve-cache.json", "target/resolve-cache.json")
}

/// Outcome of a title lookup. `Absent` is an answer from Spotify (worth
/// caching); `Failed` means the question never got asked (never cached).
pub enum Resolved {
    Track(String),
    Absent,
    Failed(String),
}

pub struct WebApi {
    token: String,
    expires_at: Instant,
    refresh_token: String,
    resolved: HashMap<String, Option<String>>,
    prefer_studio: bool,
}

/// Does this text (a track or album name) mark a live recording? Token-based
/// so "deliver" doesn't count as "live".
fn is_live(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("en public") {
        return true;
    }
    lower
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| matches!(word, "live" | "unplugged" | "concert" | "vivo"))
}

/// Is there a refresh token on disk? Asked by the home screen before
/// anything is opened, so it can say what is connected without connecting.
pub fn has_refresh() -> bool {
    std::fs::read_to_string(refresh_cache()).is_ok_and(|t| !t.trim().is_empty())
}

/// One row of a track search: enough to tell two versions of a title apart.
#[derive(Clone, Debug)]
pub struct SearchHit {
    pub title: String,
    pub artist: String,
    pub uri: String,
    pub album: String,
    /// The release year alone — "1964" — or empty when Spotify has none.
    pub year: String,
    pub duration_ms: u64,
}

impl WebApi {
    /// Reuse the cached refresh token; fall back to the browser flow once.
    pub async fn new(prefer_studio: bool) -> Result<WebApi, Box<dyn std::error::Error>> {
        let client = OAuthClientBuilder::new(CLIENT_ID, REDIRECT_URI, SCOPES.to_vec())
            .open_in_browser()
            .build()?;

        let cached = std::fs::read_to_string(refresh_cache())
            .ok()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty());

        // a refresh token that no longer works is not a fatal error: it
        // just means the authorization has to be asked for again
        let token = match &cached {
            Some(refresh) => match client.refresh_token_async(refresh).await {
                Ok(token) => token,
                // nothing is printed here: the TUI holds the screen and
                // shows the current step ("web api authorization…")
                Err(_) => client.get_access_token_async().await?,
            },
            None => client.get_access_token_async().await?,
        };

        let resolved = std::fs::read_to_string(resolve_cache())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();

        let mut api = WebApi {
            token: token.access_token,
            expires_at: token.expires_at,
            // the refresh we came in with; a rotated one replaces it below.
            // Taking it from the response alone would wipe it whenever the
            // refresh answer carries none.
            refresh_token: cached.unwrap_or_default(),
            resolved,
            prefer_studio,
        };
        api.keep_refresh(token.refresh_token);
        Ok(api)
    }

    async fn refresh_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.expires_at > Instant::now() + std::time::Duration::from_secs(30) {
            return Ok(());
        }
        let client = OAuthClientBuilder::new(CLIENT_ID, REDIRECT_URI, SCOPES.to_vec()).build()?;
        let token = client.refresh_token_async(&self.refresh_token).await?;
        self.token = token.access_token;
        self.expires_at = token.expires_at;
        self.keep_refresh(token.refresh_token);
        Ok(())
    }

    /// Spotify's PKCE flow rotates refresh tokens: the one we just used may
    /// already be dead, so the new one has to replace it in memory *and* on
    /// disk. An empty field means the response carried none — keep the old.
    fn keep_refresh(&mut self, refresh: String) {
        if refresh.is_empty() || refresh == self.refresh_token {
            return;
        }
        self.refresh_token = refresh;
        if let Err(e) = std::fs::write(refresh_cache(), &self.refresh_token) {
            eprintln!("refresh token not saved ({e})");
        }
    }

    /// What the cache already knows of "title" by "artist", without
    /// touching the network: the screen shows first, the lookup runs behind.
    pub fn cached(&self, title: &str, artist: &str) -> Option<Resolved> {
        let key = format!("{artist}\u{1}{title}");
        self.resolved.get(&key).map(|hit| match hit {
            Some(uri) => Resolved::Track(uri.clone()),
            None => Resolved::Absent,
        })
    }

    /// Resolve "title" by "artist" to a spotify:track: uri (cached).
    pub async fn resolve(&mut self, title: &str, artist: &str) -> Resolved {
        let key = format!("{artist}\u{1}{title}");
        if let Some(hit) = self.resolved.get(&key) {
            return match hit {
                Some(uri) => Resolved::Track(uri.clone()),
                None => Resolved::Absent,
            };
        }
        if let Err(e) = self.refresh_if_needed().await {
            return Resolved::Failed(format!("Spotify token expired ({e})"));
        }

        // when preferring studio, fetch several and pick the first non-live —
        // unless the requested title itself asks for a live version
        let limit = if self.prefer_studio && !is_live(title) { 8 } else { 1 };
        let query = format!("track:{title} artist:{artist}");
        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit={limit}",
            encode(&query)
        );
        // no body = the call never went through; that is not an answer and
        // must not be remembered as one
        let Some(body) = self.get_with_backoff(&url).await else {
            return Resolved::Failed("the Spotify API did not answer".to_string());
        };
        let uri = (|| {
            let items = body["tracks"]["items"].as_array()?;
            let studio = items.iter().find(|item| {
                let name = item["name"].as_str().unwrap_or("");
                let album = item["album"]["name"].as_str().unwrap_or("");
                !is_live(name) && !is_live(album)
            });
            // the first studio hit if any, else the top result
            studio
                .or_else(|| items.first())
                .and_then(|item| item["uri"].as_str().map(String::from))
        })();

        self.resolved.insert(key, uri.clone());
        if let Ok(text) = serde_json::to_string(&self.resolved) {
            let _ = std::fs::write(resolve_cache(), text);
        }
        match uri {
            Some(uri) => Resolved::Track(uri),
            None => Resolved::Absent,
        }
    }

    /// Free-text track search (the `:search` modal): a few results, best
    /// match first. Album, year and length come along — a title Spotify
    /// holds five times (studio, Olympia, best-of…) is otherwise five
    /// identical rows (Joel, 09/09/2026).
    pub async fn search_tracks(
        &mut self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, String> {
        // an unreachable Spotify is not an empty result — say which it is
        if let Err(e) = self.refresh_if_needed().await {
            return Err(format!("Spotify token expired ({e})"));
        }
        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit={}",
            encode(query),
            limit
        );
        let Some(body) = self.get_with_backoff(&url).await else {
            return Err("the Spotify API did not answer".to_string());
        };
        let Some(items) = body["tracks"]["items"].as_array() else {
            return Ok(Vec::new());
        };
        let hits = items
            .iter()
            .filter_map(|track| {
                Some(SearchHit {
                    title: track["name"].as_str()?.to_string(),
                    artist: track["artists"][0]["name"].as_str()?.to_string(),
                    uri: track["uri"].as_str()?.to_string(),
                    album: track["album"]["name"].as_str().unwrap_or_default().to_string(),
                    year: track["album"]["release_date"]
                        .as_str()
                        .and_then(|date| date.get(..4))
                        .unwrap_or_default()
                        .to_string(),
                    duration_ms: track["duration_ms"].as_u64().unwrap_or_default(),
                })
            })
            .collect();
        Ok(hits)
    }

    /// Harvest an artist's discography — the long tail, 0012 §1's fourth
    /// source. Albums and singles, then their tracks in batches of twenty,
    /// which keeps a whole artist to a handful of calls. Returns the tracks
    /// with their uris, so nothing here will ever need resolving.
    pub async fn discography(
        &mut self,
        spotify_id: &str,
    ) -> Result<Vec<crate::discography::TailTrack>, String> {
        if let Err(e) = self.refresh_if_needed().await {
            return Err(format!("Spotify token expired ({e})"));
        }
        let url = format!(
            "https://api.spotify.com/v1/artists/{spotify_id}/albums\
             ?include_groups=album,single&limit=50"
        );
        let Some(body) = self.get_with_backoff(&url).await else {
            return Err("the Spotify API did not answer".to_string());
        };
        let album_ids: Vec<String> = body["items"]
            .as_array()
            .map(|items| {
                items.iter().filter_map(|a| a["id"].as_str().map(String::from)).collect()
            })
            .unwrap_or_default();

        let mut tracks = Vec::new();
        for chunk in album_ids.chunks(20) {
            let url = format!("https://api.spotify.com/v1/albums?ids={}", chunk.join(","));
            let Some(body) = self.get_with_backoff(&url).await else {
                // a partial harvest is still worth keeping: the tail is a
                // reservoir, not an inventory
                break;
            };
            let Some(albums) = body["albums"].as_array() else { break };
            for album in albums {
                let album_name = album["name"].as_str().unwrap_or("").to_string();
                // date and rank give the discography its order; the type
                // separates albums from singles
                let released = album["release_date"].as_str().unwrap_or("").to_string();
                let single = album["album_type"].as_str() != Some("album");
                let Some(items) = album["tracks"]["items"].as_array() else { continue };
                for track in items {
                    let (Some(title), Some(uri)) =
                        (track["name"].as_str(), track["uri"].as_str())
                    else {
                        continue;
                    };
                    tracks.push(crate::discography::TailTrack {
                        title: title.to_string(),
                        uri: uri.to_string(),
                        album: album_name.clone(),
                        released: released.clone(),
                        number: track["track_number"].as_u64().unwrap_or(0) as u32,
                        duration_ms: track["duration_ms"].as_u64().unwrap_or(0) as u32,
                        single,
                    });
                }
            }
        }
        Ok(tracks)
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
pub fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
