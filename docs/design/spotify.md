# Spotify: playback, authentication, API

Working note. **Decided** = recorded in `docs/decisions/`; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.
**Verified** = read in a source cited on 2026-08-30.

## Two distinct needs

1. **Getting the sound out.** Spotify only lets its music play through a
   Spotify client: the official app, the phone, a speaker, or a compatible
   client such as librespot. A third-party program never reads the audio
   itself any other way.
2. **Knowing things and acting**: reading the user's library (liked artists
   and albums, to choose the seed), searching for a track, resolving a title
   into an identifier, pushing into the queue. That is the **Web API**.

## What is verified

- **Web API quota modes** ([official docs](https://developer.spotify.com/documentation/web-api/concepts/quota-modes)):
  development mode = "*Up to 5 authenticated Spotify users*", declared by
  hand, with a Premium owner. Extended mode, since May 2025: "*Spotify only
  accepts applications from organizations (not individuals)*", an
  established company, a launched service, "*at least 250k MAUs*".
  **Consequence**: forkstify cannot obtain a shareable client id.
- **What the free ecosystem does** ([spotify-player](https://github.com/aome510/spotify-player),
  Rust, ratatui + rspotify + librespot): "*By default, spotify-player uses
  ncspot's client ID*", a historical client id already in extended mode, and
  it advises against creating one: "*clients registered today start in the
  restricted default quota mode and commonly hit 429 / 403 errors*". A grey
  area, tolerated, not guaranteed.
- **librespot** ([README](https://github.com/librespot-org/librespot)):
  "*act as a Spotify Connect receiver*", zeroconf discovery included by
  default; "*librespot only works with Spotify Premium. This will remain the
  case.*" No developer dashboard.
- **Sound with nothing installed**: spotify-player, through librespot,
  registers as a Connect device ("*registering a spotify-player device
  accessible via Spotify Connect*"). Forkstify can do the same.

## How Omarchy-Spotify does it (read in the code, 2026-08-30)

[stappmus/Omarchy-Spotify](https://github.com/stappmus/Omarchy-Spotify),
MIT, ~23,000 lines of which ~2,000 are Rust. Three pieces:

1. **Interface**: a QML plugin in the Omarchy bar's process (`Panel.qml`
   6,600 lines, `Service.qml` 4,000). Calls the Web API directly.
2. **Sound**: `omarchy-spotify-backend`, a Rust process embedding a pinned
   fork of librespot (`engine.rs`, ~800 lines). That is the Spotify Connect
   device. A private Unix socket, JSON per line (`load`, `add_to_queue`,
   `play`, `pause`, `seek`, `set_volume`…) + MPRIS. Started on demand by a
   user systemd unit, stopped after inactivity. `spotifyd` as a fallback,
   never at the same time.
3. **Data**: the Web API from the QML.

**Authentication: two browser OAuth PKCE flows, no dashboard.**

- *Web API*: client id `d420a117a32841c2b3474932e49fb54b`, "*the public
  application identity also used by spotify-player and ncspot*" — ncspot's,
  historical, in extended mode. Callback `127.0.0.1:8989/login`, refresh
  token in the GNOME Keyring, scopes limited to the visible functions.
- *Sound*: `backend authenticate` → `librespot_oauth` with
  `SessionConfig::default().client_id`, that is, **the client id of the
  official Spotify desktop client** (`65b708073fc0480ea92a077233ca87bd`).
  Librespot presents itself as the official application. The token opens a
  librespot session, stored as a reusable credential in `~/.local/state`.
- **No API token derived from the librespot session**: two grants, two
  stores. A strong hint that this is not simple — otherwise a project this
  careful would have done it.
- **Incoming zeroconf discovery disabled** (`disable_discovery = true`): you
  do not connect from the phone, you go through the browser. Zeroconf is
  used *outwards* to activate Sonos / JBL (a 700-line Python helper, a
  Diffie-Hellman key exchange done by hand).

**Other useful facts:**

- Spotify changed the API again in 2026: the *artist top tracks* endpoint
  removed, search limited to 10 results, some non-owned playlists
  unreadable. The pipe is getting poorer; the catalog, though, is ours.
- Spotify sometimes refuses a track to librespot
  (`audio_key_unavailable`); their interface has a dedicated error case. To
  be planned for.
- 320 kbps maximum; Spotify asked librespot not to work around it.
- Serious security: the binary's GitHub provenance verified, no password and
  no client secret, tokens redacted in errors. A model to follow.

## What the spike settled (2026-09-03)

Spike `src/bin/spike-connect.rs`, run by Joel with a real Premium account
and the Spotify app on the phone (plus Omarchy-Spotify installed).

- **Incoming zeroconf discovery: ✓ it works with the current app.**
  "forkstify (spike)" appears in the phone's device list, one tap sends the
  credentials over the local network, and the librespot session opens. What
  Omarchy-Spotify had disabled was therefore not broken.
  `librespot-discovery` 0.8, `libmdns` backend (pure Rust), rustls TLS —
  nothing to install on the host. The credentials are reusable (librespot
  cache), and the tap on the phone only happens once.
- **A Web API token derived from the librespot session: ✗ unusable.**
  Two routes tested, both with the official desktop client's client id (the
  session's):
  - *keymaster* (mercury, `get_token`) → **403 "Invalid request"**. The
    legacy route, which librespot itself announces as being replaced.
  - *login5* (`login5().auth_token()`, the modern route) → the token **does
    come out** (expires in 3600 s) but the Web API refuses it **on the first
    call**: `/v1/me` → **429 "API rate limit exceeded"**, `Retry-After: 47`,
    and **the 429 persists past the window**. That is the exact symptom of a
    restricted-quota client id described by spotify-player's README. The
    desktop client id is not entitled to carry our Web API calls.

  **Conclusion: the sound goes through librespot (validated end to end), but
  the Web API cannot rely on the session token.** Route 2 in the order of
  preference below is therefore dead; we go with route 1 (ncspot's client
  id, like the whole ecosystem), route 3 as a fallback.

- **Playback through an embedded librespot player: ✓ validated**
  (`src/bin/spike-play.rs`, `librespot-playback` 0.8, rodio backend → alsa,
  `libasound.so.2` present on the host). forkstify **embeds the player**
  (not just discovery + token), loads a `spotify:track:` and **the sound
  comes out of the binary** — tested by Joel on 2026-09-03 ("it works very
  well"). That is the target architecture: forkstify is itself the device,
  we drive no other device through the API. Build: `rust:1-slim` +
  `pkg-config` + `libasound2-dev` (to be pinned in a Dockerfile).
- **The Web API through browser OAuth + ncspot's client id: ✓ validated**
  (`src/bin/spike-webapi.rs`, `librespot-oauth` 0.8, PKCE, callback
  `127.0.0.1:8989/login`). The browser opens once, you authorize, the
  refresh token is cached (the browser never opens again). Tested: `/v1/me`
  (Joël, premium), `/v1/search` ("The Cure A Forest" →
  `spotify:track:4iVTSRiJAA18d3QglhyJ6Q`), `/v1/me/albums` (247 liked
  albums). The refresh with that client id returns **all** of ncspot's
  scopes (playlist, user-top-read, library-modify…), broader than asked for.
- **The 429 lesson: the throttle is at the account/IP level, not per client
  id.** Our first attempts took 429 "rate limit exceeded" on *both* client
  ids, with a **decreasing** `Retry-After` (47 → 24 → 16 s) that clears with
  rest — that is a temporary throttle triggered by hammering the API, not a
  quota block (which would be a permanent 403). **To remember for the real
  client: respect `Retry-After` and retry** (the spike does), and do not
  chain useless calls.

## Directions

### Forkstify is itself the Connect device

Rather than depending on `spotifyd` or the official client, forkstify
**embeds librespot** ([0006](../decisions/0006-rust.md) allows it without a
rewrite) and shows up as a device in the Connect list. The user installs
nothing else. That replaces the "separate the brain from the sound"
direction in [application-shape.md](application-shape.md) — the separation
stays true in the code (the engine does not know how the sound gets out),
but the sound comes out of the same binary.

### Connecting from the phone, with nothing to type

There is no QR code login for third-party applications. Connect discovery
takes its place, and it is shorter: launch forkstify, open Spotify on the
phone, tap the devices icon, choose "forkstify". The credentials arrive over
the local network, nothing to type, no browser. Condition: phone and
computer on the same network. Fallback: browser OAuth, like spotify-player.

### The Web API, in order of preference

1. **A shared historical client id** (ncspot's), like spotify-player and
   Omarchy-Spotify — the de facto standard of the free Linux ecosystem. A
   grey area, revocable by Spotify overnight. **The route chosen** after the
   spike of 2026-09-03.
2. ~~**A token from the librespot session**~~ — **ruled out by the spike**:
   keymaster answers 403, login5 returns a token the Web API refuses with a
   persistent 429 (a desktop client id in restricted quota). See the spike
   above.
3. **Every user creates their own application** on the dashboard (five
   minutes, development mode, five users) and pastes their client id into
   the configuration. Ugly but solid; worth keeping as a configuration
   option anyway, for the day the shared client id falls.

For Joel alone, route 3 always works.

### Two ways of getting the sound out

1. **Embed librespot**, like Omarchy-Spotify (`engine.rs`, 800 MIT lines to
   study). Forkstify is a Connect device, and it works on any Linux.
2. **Talk to Omarchy-Spotify's backend** through its socket (`load`,
   `add_to_queue`): forkstify is only the brain. Far less code, but it only
   works on Omarchy with that plugin installed. Good for a quick PoC, not
   for the product — unless we decide forkstify is an Omarchy project before
   it is a Linux project.

## To settle

- ~~**PoC step 0**: a *spike*~~ — **done on 2026-09-03** (see "What the
  spike settled"). Zeroconf discovery ✓, session token ✗; the fallback
  confirmed: browser OAuth + ncspot's client id.
- **Sound workstream: the four bricks validated** (2026-09-03) — zeroconf
  discovery, the librespot session, the Web API (ncspot client id), playback
  through the embedded player. Direct playback is settled: we **do not
  drive** the `/v1/me/player/*` API, forkstify plays by itself. Left to
  **wire**: hooking all of that onto `forkstify parcours` — resolving a
  segment's titles into `spotify:track:` (through `/v1/search`, done in
  spike-webapi), chaining them in the player, and handling `e` / the
  branches in real time.
- **A Linux project or an Omarchy project?** That decides between embedding
  librespot and leaning on Omarchy-Spotify's backend. The spike shows that
  embedding librespot works without Omarchy — Joel has in fact replaced
  Omarchy-Spotify with the official client in the meantime.
- **Terms of use**: librespot is not supported by Spotify. fastpotify states
  it knows of no suspended account; the risk exists and must be told to the
  user.
