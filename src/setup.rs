//! The setup after the installation — workstream A of
//! `docs/design/before-release.md`, screens of `Installation.dc.html` (Joel,
//! 20/09/2026). Seven steps, in the order of the cahier: the catalog, the
//! git identity, the connection, the library, the playlists, the comfort,
//! the coverage. **Nothing to type one does not already know**; every step
//! can be skipped (escape) and replayed alone later (`:setup`, or
//! `:library` for steps 4, 5 and 7).
//!
//! The first launch is simply `forkstify` **without a readable catalog**:
//! it opens the setup instead of failing. The setup is a screen of the
//! session like home (0021): one column, the flow, the prompt last; its
//! keys are the modal's — `j`/`k`, space ticks, enter validates, escape
//! skips — with the digits back for a step's choices (`keys::parse_setup`).
use crate::keys::{self, Cmd};
use crate::library::{self, Library};
use crate::tui::{SetupRow, SetupView, Toast, Tui};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc::UnboundedReceiver;

/// The reference catalog — the upstream of every fork.
const REFERENCE_REPO: &str = "aropixel/forkstify-catalog";
const REFERENCE_URL: &str = "https://github.com/aropixel/forkstify-catalog.git";
const STEPS: usize = 7;
const TOAST_SECONDS: u64 = 5;

/// What to replay: everything (`:setup`, a list to pick from), or the
/// library alone (`:library` — steps 4, 5 and 7, one after the other).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Replay {
    All,
    Library,
}

pub enum Outcome {
    /// The catalog to open home on.
    Ready(PathBuf),
    Quit,
}

/// How a step ended.
#[derive(PartialEq)]
enum Flow {
    Next,
    Skip,
    Quit,
}

/// The setup, first launch or replayed. `dir` is the catalog when there is
/// one already (a replay); `None` on the first launch.
pub fn run(
    rx: &mut UnboundedReceiver<Cmd>,
    tui: &mut Tui,
    replay: Option<Replay>,
    dir: Option<PathBuf>,
) -> anyhow::Result<Outcome> {
    keys::set_setup(true);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let local = tokio::task::LocalSet::new();
    let mut setup = Setup {
        rx,
        tui,
        dir,
        web: None,
        me: None,
        library: None,
        toast: None,
        generated: 0,
        replaying: replay.is_some(),
    };
    let outcome = local.block_on(&rt, setup.go(replay));
    keys::set_setup(false);
    keys::set_text(false, "");
    Ok(outcome)
}

struct Setup<'a> {
    rx: &'a mut UnboundedReceiver<Cmd>,
    tui: &'a mut Tui,
    dir: Option<PathBuf>,
    web: Option<crate::spotify::WebApi>,
    me: Option<String>,
    library: Option<Library>,
    toast: Option<(String, Instant)>,
    generated: usize,
    replaying: bool,
}

// --- git and gh, blocking, kept short ---

fn git(dir: Option<&Path>, args: &[&str]) -> Result<String, String> {
    let mut command = std::process::Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let out = command.args(args).output().map_err(|e| format!("git not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("git failed").to_string())
    }
}

fn gh(args: &[&str]) -> Result<String, String> {
    let out = crate::fork::gh_command().args(args).output().map_err(|e| format!("gh not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("gh failed").to_string())
    }
}

/// Is `gh` there, and logged in?
#[derive(PartialEq, Clone, Copy)]
enum Gh {
    Ready,
    NotLoggedIn,
    Absent,
}

fn gh_state() -> Gh {
    match crate::fork::gh_command().args(["auth", "status"]).output() {
        Ok(out) if out.status.success() => Gh::Ready,
        Ok(_) => Gh::NotLoggedIn,
        Err(_) => Gh::Absent,
    }
}

fn remote_url(dir: &Path, remote: &str) -> Option<String> {
    git(Some(dir), &["remote", "get-url", remote]).ok()
}

/// `owner/repo` out of a remote url, for the screen.
fn short_repo(url: &str) -> String {
    let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = trimmed.rsplit(['/', ':']);
    let repo = parts.next().unwrap_or(trimmed);
    match parts.next() {
        Some(owner) => format!("{owner}/{repo}"),
        None => repo.to_string(),
    }
}

fn identity(dir: Option<&Path>, scope: &str) -> (Option<String>, Option<String>) {
    let get = |key: &str| git(dir, &["config", scope, "--get", key]).ok().filter(|v| !v.is_empty());
    (get("user.name"), get("user.email"))
}

fn tilde(path: &Path) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let text = path.display().to_string();
    match text.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => text,
    }
}

fn setup_date_file() -> PathBuf {
    crate::config::state_dir().join("setup-date")
}

impl Setup<'_> {
    // --- the screen ---

    fn say(&mut self, text: impl Into<String>) {
        self.toast = Some((text.into(), Instant::now()));
    }

    fn status(&self) -> Vec<(String, bool)> {
        let status = crate::home::Status::read();
        let reauth = crate::spotify::needs_reauthorization();
        let mut out = vec![
            (if status.librespot { "✓ librespot" } else { "⏹ no sound" }.to_string(), status.librespot),
            (
                match (status.web, reauth) {
                    (true, false) => "✓ api web".to_string(),
                    (true, true) => "↻ api web — authorize again".to_string(),
                    (false, _) => "⏹ no api web".to_string(),
                },
                status.web && !reauth,
            ),
        ];
        out.push(match &self.dir {
            Some(dir) if crate::sync::is_local(dir) => ("⇅ local".to_string(), true),
            Some(dir) if remote_url(dir, "origin").is_some() => ("⇅ fork".to_string(), true),
            Some(_) => ("⇅ no remote".to_string(), false),
            None => ("no catalog yet".to_string(), false),
        });
        out
    }

    fn draw(&mut self, step: usize, rows: &[SetupRow], prompt: &str) {
        let (label, gauge) = if step == 0 {
            ("setup".to_string(), (0, 0))
        } else if step > STEPS {
            (if step == 8 { "home" } else { ":setup" }.to_string(), (0, 0))
        } else {
            (format!("step {step} / {STEPS}"), (step, STEPS))
        };
        let toast = self.toast.as_ref().filter(|(_, at)| at.elapsed().as_secs() < TOAST_SECONDS).map(|(text, _)| Toast {
            text: text.clone(),
            tone: crate::tui::tone_of(text),
            sticky: false,
        });
        let view = SetupView { step: label, gauge, status: self.status(), rows, prompt: prompt.to_string(), toast };
        let _ = self.tui.draw_setup(&view);
    }

    async fn key(&mut self) -> Cmd {
        self.rx.recv().await.unwrap_or(Cmd::Quit)
    }

    /// A line typed in place: the field lights up, every keystroke redraws,
    /// enter gives the text, escape gives nothing.
    async fn read_field(&mut self, step: usize, rows: impl Fn(&str) -> Vec<SetupRow>, prompt: &str, initial: &str) -> Option<String> {
        keys::set_text(true, initial);
        let mut value = initial.to_string();
        let out = loop {
            let shown = rows(&value);
            self.draw(step, &shown, prompt);
            match self.key().await {
                Cmd::Typing(Some(line)) => value = line,
                Cmd::Auto => break Some(value.trim().to_string()),
                Cmd::Escape => break None,
                _ => {}
            }
        };
        keys::set_text(false, "");
        out
    }

    // --- the flow ---

    async fn go(&mut self, replay: Option<Replay>) -> Outcome {
        match replay {
            Some(Replay::Library) => {
                for step in [4, 5, 7] {
                    if self.step(step).await == Flow::Quit {
                        return Outcome::Quit;
                    }
                }
                return match self.dir.clone() {
                    Some(dir) => Outcome::Ready(dir),
                    None => Outcome::Quit,
                };
            }
            Some(Replay::All) => return self.replay_list().await,
            None => {}
        }

        // screen 0: the first launch
        loop {
            let rows = self.intro_rows();
            self.draw(0, &rows, "[⏎ begin · q]");
            match self.key().await {
                Cmd::Auto => break,
                Cmd::Quit | Cmd::Escape => return Outcome::Quit,
                _ => {}
            }
        }
        for step in 1..=STEPS {
            match self.step(step).await {
                Flow::Quit => return Outcome::Quit,
                Flow::Skip if step == 1 && self.dir.is_none() => {
                    // without a catalog nothing can be written: the setup
                    // has nowhere to go
                    let rows = vec![
                        SetupRow::Title("no catalog".into()),
                        SetupRow::Text("nothing can be written without one — the learned, the cards, the library all live in it.".into()),
                        SetupRow::Blank,
                        SetupRow::Muted("⏎ back to step 1 · q quit".into()),
                    ];
                    loop {
                        self.draw(1, &rows, "[⏎ step 1 · q]");
                        match self.key().await {
                            Cmd::Auto => break,
                            Cmd::Quit => return Outcome::Quit,
                            _ => {}
                        }
                    }
                    if self.step(1).await == Flow::Quit || self.dir.is_none() {
                        return Outcome::Quit;
                    }
                }
                _ => {}
            }
        }
        let _ = std::fs::write(setup_date_file(), crate::learned::today_iso());
        self.recap().await
    }

    async fn step(&mut self, n: usize) -> Flow {
        self.toast = None;
        match n {
            1 => self.step_catalog().await,
            2 => self.step_identity().await,
            3 => self.step_connection().await,
            4 => self.step_library().await,
            5 => self.step_playlists().await,
            6 => self.step_comfort().await,
            7 => self.step_coverage().await,
            _ => Flow::Next,
        }
    }

    fn intro_rows(&self) -> Vec<SetupRow> {
        vec![
            SetupRow::Notice("⏹ no readable catalog".into()),
            SetupRow::Blank,
            SetupRow::Title("setup".into()),
            SetupRow::Text("nothing to type you do not already know. every step skips with escape, and replays later with :setup.".into()),
            SetupRow::Blank,
            SetupRow::Rule("the seven steps".into()),
            SetupRow::Step { n: 1, done: None, label: "the catalog".into(), note: "the url of your fork, or nothing".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 2, done: None, label: "the git identity".into(), note: "name and mail, if missing".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 3, done: None, label: "the connection".into(), note: "the phone and the browser".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 4, done: None, label: "the library".into(), note: "liked tracks, albums, followed artists".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 5, done: None, label: "the playlists".into(), note: "the ones to tick".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 6, done: None, label: "the comfort".into(), note: "a number 0–5 — default 3".into(), state: String::new(), cursor: false },
            SetupRow::Step { n: 7, done: None, label: "the coverage".into(), note: "generate the missing cards of your top".into(), state: String::new(), cursor: false },
        ]
    }

    // --- step 1: the catalog ---

    async fn step_catalog(&mut self) -> Flow {
        let gh = gh_state();
        let gh_note = match gh {
            Gh::Ready => "gh detected, logged in",
            Gh::NotLoggedIn => "gh detected, not logged in — gh auth login first",
            Gh::Absent => "gh is not installed",
        };
        let existing = self.dir.clone().filter(|d| d.join(".git").exists());
        let mut selected = if existing.is_some() { 0 } else { 1 };
        loop {
            let target = crate::config::default_catalog_dir();
            let mut rows = vec![
                SetupRow::Title("the catalog".into()),
                SetupRow::Text("the cards live in a git repository. the normal case is a fork of the reference: it receives the learned, and the sync pushes to origin.".into()),
                SetupRow::Blank,
            ];
            if let Some(dir) = &existing {
                rows.push(SetupRow::Choice { n: 0, label: "keep it as it is".into(), note: tilde(dir), selected: selected == 0 });
                rows.push(SetupRow::Kv { key: "origin".into(), value: remote_url(dir, "origin").map(|u| short_repo(&u)).unwrap_or_else(|| "none".into()) });
                rows.push(SetupRow::Kv { key: "upstream".into(), value: remote_url(dir, "upstream").map(|u| short_repo(&u)).unwrap_or_else(|| if crate::sync::is_local(dir) { "origin is the reference (local mode)".into() } else { "none".into() }) });
                rows.push(SetupRow::Blank);
            }
            rows.push(SetupRow::Choice { n: 1, label: "I have a fork already".into(), note: "paste its url".into(), selected: selected == 1 });
            rows.push(SetupRow::Choice { n: 2, label: "fork it for me".into(), note: gh_note.into(), selected: selected == 2 });
            rows.push(SetupRow::Dim(format!("gh repo fork {REFERENCE_REPO} --clone")));
            rows.push(SetupRow::Choice { n: 3, label: "nothing — local mode".into(), note: "the reference cloned, ⇅ local".into(), selected: selected == 3 });
            rows.push(SetupRow::Dim("everything works, the learned is committed but not pushed, and home says so. :catalog fork <url> leaves it later, losing nothing.".into()));
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Rule("what will be written".into()));
            rows.push(SetupRow::Kv { key: "path".into(), value: format!("{} (xdg)", tilde(&target)) });
            rows.push(SetupRow::Kv { key: "origin".into(), value: "your fork — or the reference in local mode".into() });
            rows.push(SetupRow::Kv { key: "upstream".into(), value: REFERENCE_REPO.into() });
            rows.push(SetupRow::Kv { key: "config".into(), value: "[catalog] path in config.toml".into() });
            let prompt = if existing.is_some() { "[0-3 choose · ⏎ validate · esc skip · q]" } else { "[1-3 choose · ⏎ validate · esc skip · q]" };
            self.draw(1, &rows, prompt);
            match self.key().await {
                Cmd::Digit(n) if n <= 3 && (n > 0 || existing.is_some()) => selected = n,
                Cmd::Down => selected = (selected + 1).min(3),
                Cmd::Up => selected = selected.saturating_sub(if existing.is_some() { 1 } else { usize::from(selected > 1) }),
                Cmd::Auto => match selected {
                    0 => return Flow::Next,
                    1 => {
                        let step_rows = rows.iter().map(|_| ()).count();
                        let _ = step_rows;
                        let url = self
                            .read_field(
                                1,
                                |value| {
                                    vec![
                                        SetupRow::Title("the catalog".into()),
                                        SetupRow::Text("the url of your fork — the one git clone takes.".into()),
                                        SetupRow::Blank,
                                        SetupRow::Field { label: "url".into(), value: value.to_string(), active: true },
                                        SetupRow::Dim("e.g. https://github.com/you/forkstify-catalog — or git@github.com:you/forkstify-catalog.git".into()),
                                    ]
                                },
                                "[⏎ clone · esc back]",
                                "",
                            )
                            .await;
                        match url {
                            Some(url) if !url.is_empty() => {
                                if self.install(Cloner::Git(url), false).await {
                                    return Flow::Next;
                                }
                            }
                            _ => {}
                        }
                    }
                    2 => {
                        if gh != Gh::Ready {
                            self.say(format!("⏹ {gh_note}"));
                            continue;
                        }
                        if self.install(Cloner::Gh, false).await {
                            return Flow::Next;
                        }
                    }
                    _ => {
                        if self.install(Cloner::Git(REFERENCE_URL.to_string()), true).await {
                            return Flow::Next;
                        }
                    }
                },
                Cmd::Escape => return Flow::Skip,
                Cmd::Quit => return Flow::Quit,
                _ => {}
            }
        }
    }

    /// Clone into the xdg place — or take what is already there — set the
    /// remotes, the local mark, the config. True when the catalog is ready.
    async fn install(&mut self, cloner: Cloner, local: bool) -> bool {
        let target = crate::config::default_catalog_dir();
        let rows = vec![
            SetupRow::Title("the catalog".into()),
            SetupRow::Blank,
            SetupRow::Notice(format!("… cloning into {} — a minute, the cards and their vectors", tilde(&target))),
        ];
        self.draw(1, &rows, "[cloning…]");
        let already = target.join(".git").exists();
        let target_for_task = target.clone();
        let result: Result<(), String> = tokio::task::spawn_blocking(move || {
            if let Some(parent) = target_for_task.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match (&cloner, already) {
                (Cloner::Git(url), false) => git(None, &["clone", "--quiet", url, &target_for_task.display().to_string()]).map(|_| ()),
                (Cloner::Git(url), true) => git(Some(&target_for_task), &["remote", "set-url", "origin", url]).map(|_| ()),
                (Cloner::Gh, _) => {
                    // fork if not yet forked (idempotent), then clone through
                    // gh: it sets the protocol one chose, and adds upstream
                    gh(&["repo", "fork", REFERENCE_REPO, "--clone=false"])?;
                    let login = gh(&["api", "user", "--jq", ".login"])?;
                    let repo = format!("{login}/forkstify-catalog");
                    if already {
                        let url = gh(&["repo", "view", &repo, "--json", "url", "--jq", ".url"])?;
                        git(Some(&target_for_task), &["remote", "set-url", "origin", &url]).map(|_| ())
                    } else {
                        gh(&["repo", "clone", &repo, &target_for_task.display().to_string()]).map(|_| ())
                    }
                }
            }
        })
        .await
        .unwrap_or_else(|e| Err(format!("clone interrupted ({e})")));
        if let Err(why) = result {
            self.say(format!("⏹ {why}"));
            return false;
        }
        crate::sync::set_local(&target, local);
        if !local && remote_url(&target, "upstream").is_none() {
            let _ = git(Some(&target), &["remote", "add", "upstream", REFERENCE_URL]);
        }
        crate::sync::ensure_merge_driver(&target);
        if let Err(why) = crate::config::set_catalog_path(&target) {
            self.say(format!("⏹ {why}"));
        }
        self.dir = Some(target.clone());
        self.say(format!("✓ catalog at {}{}", tilde(&target), if local { " — local mode" } else { "" }));
        true
    }

    // --- step 2: the git identity ---

    async fn step_identity(&mut self) -> Flow {
        let Some(dir) = self.dir.clone() else {
            self.say("(no catalog — nothing to sign)");
            return Flow::Skip;
        };
        let (global_name, global_mail) = identity(None, "--global");
        let (local_name, local_mail) = identity(Some(&dir), "--local");
        let complete = (global_name.is_some() && global_mail.is_some()) || (local_name.is_some() && local_mail.is_some());
        if complete && !self.replaying {
            self.say(if local_name.is_some() { "✓ git identity — local to the clone" } else { "✓ git identity — from the global config" });
            return Flow::Next;
        }
        let rows = |name: &str, mail: &str, active: usize| {
            vec![
                SetupRow::Title("the git identity".into()),
                SetupRow::Text("forkstify commits the learned every ten minutes. a commit without user.name fails — and this machine has none.".into()),
                SetupRow::Blank,
                SetupRow::Field { label: "name".into(), value: name.to_string(), active: active == 0 },
                SetupRow::Field { label: "mail".into(), value: mail.to_string(), active: active == 1 },
                SetupRow::Blank,
                SetupRow::Dim("written in the clone — git config --local, never global. the identity of the catalog's commits, not the machine's.".into()),
            ]
        };
        let prompt = "[⏎ next field · esc skip — the learned will not commit · q]";
        let start_name = local_name.or(global_name).unwrap_or_default();
        let start_mail = local_mail.or(global_mail).unwrap_or_default();
        let Some(name) = self.read_field(2, |v| rows(v, &start_mail, 0), prompt, &start_name).await else {
            self.say("(skipped — the learned will not commit until an identity is set)");
            return Flow::Skip;
        };
        let Some(mail) = self.read_field(2, |v| rows(&name, v, 1), prompt, &start_mail).await else {
            return Flow::Skip;
        };
        if name.is_empty() || mail.is_empty() {
            self.say("(empty — nothing written)");
            return Flow::Skip;
        }
        match git(Some(&dir), &["config", "--local", "user.name", &name])
            .and_then(|_| git(Some(&dir), &["config", "--local", "user.email", &mail]))
        {
            Ok(_) => self.say(format!("✓ commits signed {name} <{mail}>, in the clone")),
            Err(why) => self.say(format!("⏹ {why}")),
        }
        Flow::Next
    }

    // --- step 3: the connection ---

    async fn step_connection(&mut self) -> Flow {
        let prefer_studio = crate::config::Config::load().playback.prefer_studio;
        let mut discovering: Option<std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error>>>>>> = None;
        loop {
            let status = crate::home::Status::read();
            let reauth = crate::spotify::needs_reauthorization();
            if !status.librespot && discovering.is_none() {
                discovering = Some(Box::pin(crate::sound::discover()));
            }
            let mut rows = vec![
                SetupRow::Title("the connection".into()),
                SetupRow::Text("two independent authorizations, nothing to type: the phone announces the device, the browser grants the api.".into()),
                SetupRow::Blank,
                SetupRow::Choice { n: 1, label: "librespot — the sound".into(), note: String::new(), selected: !status.librespot },
                SetupRow::Dim(format!("the credentials come from the phone over zeroconf. open spotify on the phone, \"available devices\", then pick \"{}\".", crate::sound::DEVICE_NAME)),
                SetupRow::Notice(if status.librespot { "✓ device announced".into() } else { "… waiting for the phone".into() }),
                SetupRow::Blank,
                SetupRow::Choice { n: 2, label: "the web api — the tracks, the library, the playlists".into(), note: String::new(), selected: !status.web || reauth },
                SetupRow::Dim("an oauth authorization in the browser — seven scopes now.".into()),
                SetupRow::Muted("✓ the five of before — playback, state, tracks".into()),
                SetupRow::Muted("↻ user-follow-read — the followed artists (step 4)".into()),
                SetupRow::Muted("↻ playlist-read-private — your playlists, private ones included (step 5)".into()),
            ];
            if reauth {
                rows.push(SetupRow::Notice("↻ authorized before these two: the browser once more. this is not a failure.".into()));
            }
            rows.push(SetupRow::Notice(if status.web && !reauth { "✓ authorized".into() } else { "o  open the authorization page".into() }));
            let prompt = "[o authorize · ⏎ continue · esc skip — steps 4, 5 and 7 skipped · q]";
            self.draw(3, &rows, prompt);

            let cmd = match discovering.as_mut() {
                Some(future) => {
                    tokio::select! {
                        found = future => {
                            discovering = None;
                            match found {
                                Ok(_) => self.say("✓ device announced — the phone handed the credentials"),
                                Err(why) => self.say(format!("⏹ discovery: {why}")),
                            }
                            continue;
                        }
                        cmd = self.rx.recv() => cmd.unwrap_or(Cmd::Quit),
                    }
                }
                None => self.key().await,
            };
            match cmd {
                Cmd::Open => {
                    if reauth {
                        crate::spotify::forget_authorization();
                    }
                    self.say("… the browser opens — authorize forkstify there");
                    let rows_wait = vec![SetupRow::Title("the connection".into()), SetupRow::Blank, SetupRow::Notice("… waiting for the browser's answer".into())];
                    self.draw(3, &rows_wait, "[authorizing in the browser…]");
                    match crate::spotify::WebApi::new(prefer_studio).await {
                        Ok(web) => {
                            self.web = Some(web);
                            self.say("✓ api web authorized — seven scopes");
                        }
                        Err(why) => self.say(format!("⏹ authorization: {why}")),
                    }
                }
                Cmd::Auto => {
                    if status.web && !reauth {
                        return Flow::Next;
                    }
                    self.say("(the api is not authorized — o to open the browser, esc to skip)");
                }
                Cmd::Escape => return Flow::Skip,
                Cmd::Quit => return Flow::Quit,
                _ => {}
            }
        }
    }

    /// The api, opened once and kept across the steps. None when it is not
    /// authorized — the caller says so and skips.
    async fn api(&mut self) -> bool {
        if self.web.is_some() {
            return true;
        }
        if !crate::spotify::has_refresh() || crate::spotify::needs_reauthorization() {
            self.say("(the api is not authorized — step 3 first)");
            return false;
        }
        let prefer_studio = crate::config::Config::load().playback.prefer_studio;
        match crate::spotify::WebApi::new(prefer_studio).await {
            Ok(web) => {
                self.web = Some(web);
                true
            }
            Err(why) => {
                self.say(format!("⏹ api: {why}"));
                false
            }
        }
    }

    // --- step 4: the library ---

    async fn step_library(&mut self) -> Flow {
        let Some(dir) = self.dir.clone() else {
            self.say("(no catalog — nowhere to write the library)");
            return Flow::Skip;
        };
        if !self.api().await {
            return Flow::Skip;
        }
        let previous = Library::load(&dir);
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<library::Progress>();
        let (done_tx, mut done_rx) = tokio::sync::mpsc::unbounded_channel::<(crate::spotify::WebApi, Result<Library, String>)>();
        let mut web = self.web.take().expect("api opened");
        let task = tokio::task::spawn_local(async move {
            let result = library::harvest(&mut web, previous.as_ref(), &progress_tx).await;
            let _ = done_tx.send((web, result));
        });
        let (mut tracks, mut albums, mut followed) = ((0usize, 0usize), (0usize, 0usize), 0usize);
        let mut done: Option<Library> = None;
        loop {
            let mut rows = vec![
                SetupRow::Title("the library".into()),
                SetupRow::Text("what spotify already knows of you. it is the matter of the ranking — and so of everything forkstify proposes next.".into()),
                SetupRow::Blank,
                SetupRow::Bar { label: "liked tracks".into(), done: tracks.0, total: tracks.1, note: "the main artist only — guests are not liked".into() },
                SetupRow::Bar { label: "liked albums".into(), done: albums.0, total: albums.1, note: "×3 in the ranking".into() },
                SetupRow::Bar { label: "followed artists".into(), done: followed, total: 0, note: "+8 in the ranking".into() },
                SetupRow::Blank,
                SetupRow::Dim("/me/tracks · /me/albums · /me/following?type=artist — paged, ~2 min for 5 000 tracks".into()),
            ];
            if let Some(library) = &done {
                rows.push(SetupRow::Blank);
                rows.push(SetupRow::Notice(format!("✓ learned/library.toml written — {} artists ranked", library.artists.len())));
                rows.push(SetupRow::Dim("one file, english vocabulary (0022): name, spotify, liked_tracks, liked_albums, followed, playlist_tracks, score, sources. it replaces classement.json and the artistes-*.json — the old stays read as long as it is there.".into()));
            }
            let prompt = if done.is_some() { "[⏎ continue · q]" } else { "[harvesting… · esc cancel · q]" };
            self.draw(4, &rows, prompt);
            if done.is_some() {
                match self.key().await {
                    Cmd::Auto | Cmd::Escape => return Flow::Next,
                    Cmd::Quit => return Flow::Quit,
                    _ => {}
                }
                continue;
            }
            tokio::select! {
                progress = progress_rx.recv() => match progress {
                    Some(library::Progress::Tracks { done, total }) => tracks = (done, total),
                    Some(library::Progress::Albums { done, total }) => albums = (done, total),
                    Some(library::Progress::Followed { done }) => followed = done,
                    Some(library::Progress::Playlist { .. }) | None => {}
                },
                finished = done_rx.recv() => {
                    let Some((web, result)) = finished else { continue };
                    self.web = Some(web);
                    match result {
                        Ok(library) => {
                            tracks = (library.liked_tracks() as usize, library.liked_tracks() as usize);
                            albums = (library.liked_albums() as usize, library.liked_albums() as usize);
                            followed = library.followed();
                            match library.save(&dir) {
                                Ok(()) => self.say("✓ library harvested"),
                                Err(why) => self.say(format!("⏹ {why}")),
                            }
                            self.library = Some(library.clone());
                            done = Some(library);
                        }
                        Err(why) => {
                            self.say(format!("⏹ harvest: {why}"));
                            return Flow::Skip;
                        }
                    }
                },
                cmd = self.rx.recv() => match cmd.unwrap_or(Cmd::Quit) {
                    Cmd::Escape => {
                        task.abort();
                        self.say("(harvest cancelled — nothing written)");
                        return Flow::Skip;
                    }
                    Cmd::Quit => {
                        task.abort();
                        return Flow::Quit;
                    }
                    _ => {}
                },
            }
        }
    }

    // --- step 5: the playlists ---

    async fn step_playlists(&mut self) -> Flow {
        let Some(dir) = self.dir.clone() else {
            self.say("(no catalog — nowhere to write the library)");
            return Flow::Skip;
        };
        if !self.api().await {
            return Flow::Skip;
        }
        let Some(mut library) = self.library.clone().or_else(|| Library::load(&dir)) else {
            self.say("(harvest the library first — step 4)");
            return Flow::Skip;
        };
        let rows_wait = vec![SetupRow::Title("the playlists".into()), SetupRow::Blank, SetupRow::Notice("… listing your playlists".into())];
        self.draw(5, &rows_wait, "[listing…]");
        let me = match self.me.clone() {
            Some(me) => me,
            None => match self.web.as_mut().expect("api opened").me().await {
                Ok(me) => {
                    self.me = Some(me.clone());
                    me
                }
                Err(why) => {
                    self.say(format!("⏹ {why}"));
                    return Flow::Skip;
                }
            },
        };
        let all = match library::list_playlists(self.web.as_mut().expect("api opened"), &me).await {
            Ok(list) => list,
            Err(why) => {
                self.say(format!("⏹ {why}"));
                return Flow::Skip;
            }
        };
        let mut ticked: std::collections::HashSet<String> = library.playlists.iter().map(|p| p.id.clone()).collect();
        let mut cursor = 0usize;
        let mut filter = String::new();
        loop {
            let visible: Vec<&library::PlaylistInfo> = all
                .iter()
                .filter(|p| filter.is_empty() || p.name.to_lowercase().contains(&filter.to_lowercase()))
                .collect();
            cursor = cursor.min(visible.len().saturating_sub(1));
            let window = 12usize;
            let first = cursor.saturating_sub(window / 2).min(visible.len().saturating_sub(window));
            let ticked_tracks: usize = all.iter().filter(|p| ticked.contains(&p.id)).map(|p| p.tracks).sum();
            let mut rows = vec![
                SetupRow::Title("the playlists".into()),
                SetupRow::Text("a ticked playlist counts its artists ×1, like a liked track. tick the ones that sound like you — not the ones spotify made for you.".into()),
                SetupRow::Blank,
            ];
            if !filter.is_empty() {
                rows.push(SetupRow::Muted(format!("/{filter}")));
            }
            for (i, playlist) in visible.iter().enumerate().skip(first).take(window) {
                rows.push(SetupRow::Check {
                    on: ticked.contains(&playlist.id),
                    name: playlist.name.clone(),
                    count: format!("{} tracks", playlist.tracks),
                    owner: playlist.owner.clone(),
                    cursor: i == cursor,
                });
            }
            let hidden = visible.len().saturating_sub(first + window);
            rows.push(SetupRow::Dim(format!(
                "── {}{} ticked · {} tracks",
                if hidden > 0 { format!("{hidden} more · ") } else { String::new() },
                ticked.len(),
                ticked_tracks
            )));
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Dim("remembered in learned/library.toml — :library harvests them again without asking.".into()));
            self.draw(5, &rows, "[space tick · j/k · /text filter · ⏎ harvest the ticked · esc skip]");
            match self.key().await {
                Cmd::Help(None) => {
                    if let Some(playlist) = visible.get(cursor) {
                        if !ticked.remove(&playlist.id) {
                            ticked.insert(playlist.id.clone());
                        }
                    }
                }
                Cmd::Down => cursor = (cursor + 1).min(visible.len().saturating_sub(1)),
                Cmd::Up => cursor = cursor.saturating_sub(1),
                Cmd::Top => cursor = 0,
                Cmd::Bottom => cursor = visible.len().saturating_sub(1),
                Cmd::Search(text) => {
                    filter = text;
                    cursor = 0;
                }
                Cmd::Escape if !filter.is_empty() => filter.clear(),
                Cmd::Escape => return Flow::Skip,
                Cmd::Quit => return Flow::Quit,
                Cmd::Auto => {
                    let chosen: Vec<library::PlaylistInfo> = all.iter().filter(|p| ticked.contains(&p.id)).cloned().collect();
                    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<library::Progress>();
                    let (done_tx, mut done_rx) = tokio::sync::mpsc::unbounded_channel::<(crate::spotify::WebApi, Library, Result<(), String>)>();
                    let mut web = self.web.take().expect("api opened");
                    let mut harvesting = library.clone();
                    let chosen_for_task = chosen.clone();
                    let task = tokio::task::spawn_local(async move {
                        let result = library::harvest_playlists(&mut web, &mut harvesting, &chosen_for_task, &progress_tx).await;
                        let _ = done_tx.send((web, harvesting, result));
                    });
                    let mut bars: Vec<(String, usize, usize)> = chosen.iter().map(|p| (p.name.clone(), 0, p.tracks)).collect();
                    loop {
                        let mut rows = vec![SetupRow::Title("the playlists".into()), SetupRow::Blank];
                        for (name, done, total) in &bars {
                            rows.push(SetupRow::Bar { label: name.chars().take(16).collect(), done: *done, total: *total, note: String::new() });
                        }
                        self.draw(5, &rows, "[harvesting the ticked playlists… · esc cancel]");
                        tokio::select! {
                            progress = progress_rx.recv() => {
                                if let Some(library::Progress::Playlist { name, done, total }) = progress {
                                    if let Some(bar) = bars.iter_mut().find(|(n, ..)| *n == name) {
                                        *bar = (name, done, total.max(done));
                                    }
                                }
                            }
                            finished = done_rx.recv() => {
                                let Some((web, harvested, result)) = finished else { continue };
                                self.web = Some(web);
                                match result {
                                    Ok(()) => {
                                        library = harvested;
                                        match library.save(&dir) {
                                            Ok(()) => self.say(format!("✓ {} playlists counted, remembered", chosen.len())),
                                            Err(why) => self.say(format!("⏹ {why}")),
                                        }
                                        self.library = Some(library);
                                        return Flow::Next;
                                    }
                                    Err(why) => {
                                        self.say(format!("⏹ playlists: {why}"));
                                        break;
                                    }
                                }
                            }
                            cmd = self.rx.recv() => match cmd.unwrap_or(Cmd::Quit) {
                                Cmd::Escape => { task.abort(); self.say("(cancelled — the previous counts stay)"); break; }
                                Cmd::Quit => { task.abort(); return Flow::Quit; }
                                _ => {}
                            },
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // --- step 6: the comfort ---

    async fn step_comfort(&mut self) -> Flow {
        let mut value = crate::config::comfort_at_start();
        loop {
            let mut rows = vec![
                SetupRow::Title("the comfort zone".into()),
                SetupRow::Text("how far forkstify strays from what you know, when it chooses for you.".into()),
                SetupRow::Blank,
                SetupRow::Muted(format!(
                    "{}  {}  {}",
                    (0..5).map(|i| if i < value { '█' } else { '░' }).collect::<String>(),
                    value,
                    crate::listen::comfort_word(value)
                )),
                SetupRow::Dim("h/l to set · c<n> or cc while listening".into()),
                SetupRow::Blank,
            ];
            let hints = [
                (5, "your regulars, nothing else"),
                (4, "the known, a step aside now and then"),
                (3, "the known, and a step aside per branch"),
                (2, "the neighbourhood, the neglected come back"),
                (1, "far leaps, the long tail counts"),
                (0, "the neglected and the long tail come first"),
            ];
            for (level, hint) in hints {
                rows.push(SetupRow::Choice {
                    n: level as usize,
                    label: crate::listen::comfort_word(level).to_string(),
                    note: hint.into(),
                    selected: level == value,
                });
            }
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Dim("« she chooses alone when you don't » (0001) — the comfort only rules that moment.".into()));
            self.draw(6, &rows, "[h/l set · 0-5 directly · ⏎ validate — default 3]");
            match self.key().await {
                Cmd::Digit(n) if n <= 5 => value = n as u8,
                Cmd::Next | Cmd::Up => value = (value + 1).min(5),
                Cmd::Prev | Cmd::Down => value = value.saturating_sub(1),
                Cmd::Auto => {
                    crate::config::remember_comfort(value);
                    self.say(format!("✓ comfort {value} — {}", crate::listen::comfort_word(value)));
                    return Flow::Next;
                }
                Cmd::Escape => return Flow::Skip,
                Cmd::Quit => return Flow::Quit,
                _ => {}
            }
        }
    }

    // --- step 7: the coverage ---

    async fn step_coverage(&mut self) -> Flow {
        let Some(dir) = self.dir.clone() else {
            self.say("(no catalog)");
            return Flow::Skip;
        };
        let Some(library) = self.library.clone().or_else(|| Library::load(&dir)) else {
            self.say("(harvest the library first — step 4)");
            return Flow::Skip;
        };
        let mut catalog = match crate::catalog::Catalog::load(&dir) {
            Ok(catalog) => catalog,
            Err(why) => {
                self.say(format!("⏹ catalog: {why}"));
                return Flow::Skip;
            }
        };
        let coverage = library::coverage(&library, library::card_finder(&catalog));
        let base_rows = |coverage: &library::Coverage| {
            let mut rows = vec![
                SetupRow::Title("the coverage".into()),
                SetupRow::Text(format!(
                    "{} of your {} most present artists have a card. lower, it thins out: {} artist(s) above score {} have none.",
                    coverage.top.0,
                    coverage.top.1,
                    coverage.missing_total,
                    library::COVERAGE_FLOOR
                )),
                SetupRow::Blank,
            ];
            for (label, with, total) in &coverage.tiers {
                rows.push(SetupRow::Bar { label: label.clone(), done: *with, total: *total, note: "with a card".into() });
            }
            rows.push(SetupRow::Blank);
            rows
        };
        if coverage.missing.is_empty() {
            let mut rows = base_rows(&coverage);
            rows.push(SetupRow::Notice(format!("✓ everything above score {} has a card", library::COVERAGE_FLOOR)));
            loop {
                self.draw(7, &rows, "[⏎ continue · q]");
                match self.key().await {
                    Cmd::Auto | Cmd::Escape => return Flow::Next,
                    Cmd::Quit => return Flow::Quit,
                    _ => {}
                }
            }
        }
        let n = coverage.missing.len();
        loop {
            let mut rows = base_rows(&coverage);
            rows.push(SetupRow::Text(format!(
                "generate {n} to cover everything with a score ≥ {}? (~3 s each — {} s · musicbrainz then deezer · one commit)",
                library::COVERAGE_FLOOR,
                n * 3
            )));
            rows.push(SetupRow::Names(coverage.missing.iter().map(|(name, _)| name.clone()).collect()));
            if coverage.missing_total > n {
                rows.push(SetupRow::Dim(format!("{} more above the floor, for a later :library", coverage.missing_total - n)));
            }
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Dim("they are born generated = true, with their vectors — and Cp will propose them to the reference one day.".into()));
            self.draw(7, &rows, "[o generate · esc later — :library proposes it again · q]");
            match self.key().await {
                Cmd::Open => break,
                Cmd::Escape => return Flow::Skip,
                Cmd::Quit => return Flow::Quit,
                _ => {}
            }
        }

        // the generation: one card after the other in the background —
        // MusicBrainz asks for its second between two — each written as it
        // comes, the vectors and the one commit at the end
        let names: Vec<String> = coverage.missing.iter().map(|(name, _)| name.clone()).collect();
        let known = std::sync::Arc::new(crate::generate::Known::of(&catalog));
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<(usize, String, Result<crate::generate::Draft, String>)>();
        let names_for_task = names.clone();
        let task = tokio::task::spawn_local(async move {
            for (i, name) in names_for_task.iter().enumerate() {
                let (slug, hint, known) = (crate::generate::slugify(name), name.clone(), known.clone());
                let result = tokio::task::spawn_blocking(move || crate::generate::draft(&slug, Some(&hint), None, &known))
                    .await
                    .unwrap_or_else(|e| Err(format!("interrupted ({e})")));
                if progress_tx.send((i, name.clone(), result)).is_err() {
                    return;
                }
            }
        });
        let mut written: Vec<String> = Vec::new();
        let mut failed: Vec<String> = Vec::new();
        let mut current = names.first().cloned().unwrap_or_default();
        let mut done = 0usize;
        loop {
            let mut rows = base_rows(&coverage);
            rows.push(SetupRow::Bar { label: "generating".into(), done, total: n, note: if done < n { format!("… {current}") } else { String::new() } });
            for name in &failed {
                rows.push(SetupRow::Notice(format!("⏹ {name}")));
            }
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Dim("esc stops after the current card — what is written is committed.".into()));
            self.draw(7, &rows, "[generating… · esc stop · q]");
            if done >= n {
                break;
            }
            tokio::select! {
                progress = progress_rx.recv() => {
                    let Some((i, name, result)) = progress else { break };
                    done = i + 1;
                    current = names.get(i + 1).cloned().unwrap_or_default();
                    match result {
                        Ok(draft) => match toml::from_str::<crate::catalog::Card>(&draft.toml) {
                            Ok(card) => match crate::edit::create_card(&dir, &draft.slug, &draft.name, &draft.toml, draft.tops, draft.links) {
                                Ok(_) => {
                                    catalog.cards.insert(draft.slug.clone(), card);
                                    written.push(draft.slug);
                                }
                                Err(why) => failed.push(format!("{name} — {why}")),
                            },
                            Err(why) => failed.push(format!("{name} — {why}")),
                        },
                        Err(why) => failed.push(format!("{name} — {why}")),
                    }
                }
                cmd = self.rx.recv() => match cmd.unwrap_or(Cmd::Quit) {
                    Cmd::Escape => { task.abort(); break; }
                    Cmd::Quit => { task.abort(); break; }
                    _ => {}
                },
            }
        }
        if written.is_empty() {
            self.say("(no card generated)");
            return Flow::Skip;
        }
        // the vectors (0019), then one commit for the batch, like import
        let mut rows = base_rows(&coverage);
        rows.push(SetupRow::Notice(format!("… vectorizing {} card(s){}", written.len(), if crate::embed::model_cached() { "" } else { " — first run: the model downloads (241 MB)" })));
        self.draw(7, &rows, "[vectorizing…]");
        let texts: Vec<String> = written.iter().map(|slug| crate::embed::text_of(slug, &catalog.cards[slug], &catalog.cards)).collect();
        let vectors = tokio::task::spawn_blocking(move || crate::embed::embed(&texts)).await.unwrap_or_else(|e| Err(format!("interrupted ({e})")));
        let mut vectorized = 0;
        match vectors {
            Ok(vectors) => {
                for (slug, vector) in written.iter().zip(&vectors) {
                    if crate::embed::write_vector(&dir, slug, vector).is_ok() {
                        vectorized += 1;
                    }
                }
            }
            Err(why) => self.say(format!("⏹ vectors: {why} — forkstify vectors will catch up")),
        }
        let commit = git(Some(&dir), &["add", "cards", "vectors"]).and_then(|_| {
            git(
                Some(&dir),
                &[
                    "commit",
                    "-q",
                    "-m",
                    &format!("library: {} cards generated", written.len()),
                    "-m",
                    &format!("{}\n\n{}", written.join("\n"), crate::sync::trailer("edit")),
                ],
            )
        });
        match commit {
            Ok(_) => self.say(format!("✓ {} card(s) generated, {vectorized} vectorized · one commit", written.len())),
            Err(why) => self.say(format!("⏹ commit: {why}")),
        }
        self.generated = written.len();
        let mut rows = base_rows(&coverage);
        rows.push(SetupRow::Notice(format!("✓ {} card(s) generated — generated = true, to reread at leisure", written.len())));
        for name in &failed {
            rows.push(SetupRow::Notice(format!("⏹ {name}")));
        }
        loop {
            self.draw(7, &rows, "[⏎ continue · q]");
            match self.key().await {
                Cmd::Auto | Cmd::Escape => return Flow::Next,
                Cmd::Quit => return Flow::Quit,
                _ => {}
            }
        }
    }

    // --- screen 8: out of the setup ---

    async fn recap(&mut self) -> Outcome {
        let Some(dir) = self.dir.clone() else { return Outcome::Quit };
        let library = self.library.clone().or_else(|| Library::load(&dir));
        let with_card = match (crate::catalog::Catalog::load(&dir), &library) {
            (Ok(catalog), Some(library)) => {
                let known = library::card_finder(&catalog);
                library.artists.iter().filter(|a| known(&a.name)).count()
            }
            _ => 0,
        };
        let mut rows = vec![
            SetupRow::Notice("✓ setup done".into()),
            SetupRow::Notice(format!(
                "✓ catalog — {}",
                if crate::sync::is_local(&dir) { "the reference, local mode".to_string() } else { remote_url(&dir, "origin").map(|u| format!("fork {}", short_repo(&u))).unwrap_or_else(|| tilde(&dir)) }
            )),
        ];
        if let Some(library) = &library {
            rows.push(SetupRow::Notice(format!(
                "✓ library harvested — {} tracks, {} albums, {} followed, {} playlist(s)",
                library.liked_tracks(),
                library.liked_albums(),
                library.followed(),
                library.playlists.len()
            )));
        }
        if self.generated > 0 {
            rows.push(SetupRow::Notice(format!("✓ {} cards generated", self.generated)));
        }
        rows.push(SetupRow::Blank);
        if let Some(library) = &library {
            rows.push(SetupRow::Text(format!("{} artists ranked, {with_card} with a card. the rest happens at home.", library.artists.len())));
        } else {
            rows.push(SetupRow::Text("the rest happens at home.".into()));
        }
        rows.push(SetupRow::Blank);
        rows.push(SetupRow::Rule("to come back to the setup".into()));
        rows.push(SetupRow::Kv { key: ":setup".into(), value: "replays the seven steps — those already done are ticked, and skip with ⏎".into() });
        rows.push(SetupRow::Kv { key: ":library".into(), value: "replays steps 4, 5 and 7 only".into() });
        loop {
            self.draw(8, &rows, "[⏎ home · q]");
            match self.key().await {
                Cmd::Auto | Cmd::Escape => return Outcome::Ready(dir),
                Cmd::Quit => return Outcome::Quit,
                _ => {}
            }
        }
    }

    // --- screen 9: `:setup` replayed — a list to pick a step from ---

    async fn replay_list(&mut self) -> Outcome {
        let mut cursor = 0usize;
        loop {
            let Some(dir) = self.dir.clone() else {
                return Outcome::Quit;
            };
            let library = self.library.clone().or_else(|| Library::load(&dir));
            let status = crate::home::Status::read();
            let reauth = crate::spotify::needs_reauthorization();
            let (local_name, _) = identity(Some(&dir), "--local");
            let (global_name, global_mail) = identity(None, "--global");
            let comfort = crate::config::comfort_at_start();
            let missing = match (crate::catalog::Catalog::load(&dir), &library) {
                (Ok(catalog), Some(library)) => library::coverage(library, library::card_finder(&catalog)).missing_total,
                _ => 0,
            };
            let date = std::fs::read_to_string(setup_date_file()).ok().map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
            let steps: Vec<(bool, String, String)> = vec![
                (
                    true,
                    format!(
                        "the catalog — {}",
                        if crate::sync::is_local(&dir) { "local mode".to_string() } else { remote_url(&dir, "origin").map(|u| format!("fork {}", short_repo(&u))).unwrap_or_else(|| "no remote".into()) }
                    ),
                    String::new(),
                ),
                (
                    local_name.is_some() || (global_name.is_some() && global_mail.is_some()),
                    format!("the git identity — {}", if local_name.is_some() { "local to the clone" } else if global_name.is_some() { "from the global config" } else { "none" }),
                    String::new(),
                ),
                (
                    status.connected() && !reauth,
                    format!("the connection — {}", if reauth { "two scopes more: the browser once again" } else if status.connected() { "both tokens, seven scopes" } else { "incomplete" }),
                    String::new(),
                ),
                (
                    library.as_ref().is_some_and(|l| !l.harvested.is_empty()),
                    match &library {
                        Some(l) => format!("the library — {} tracks, {} albums, {} followed", l.liked_tracks(), l.liked_albums(), l.followed()),
                        None => "the library — not harvested".into(),
                    },
                    library.as_ref().map(|l| format!("harvested {}", l.harvested)).unwrap_or_default(),
                ),
                (
                    library.as_ref().is_some_and(|l| !l.playlists.is_empty()),
                    match &library {
                        Some(l) => format!("the playlists — {} ticked, remembered", l.playlists.len()),
                        None => "the playlists".into(),
                    },
                    String::new(),
                ),
                (true, format!("the comfort — {comfort}, {}", crate::listen::comfort_word(comfort)), String::new()),
                (
                    missing == 0,
                    if missing == 0 {
                        format!("the coverage — everything above score {} has a card", library::COVERAGE_FLOOR)
                    } else {
                        format!("the coverage — {missing} artist(s) ≥ {} without a card", library::COVERAGE_FLOOR)
                    },
                    if missing == 0 { String::new() } else { "to do".into() },
                ),
            ];
            let mut rows = vec![
                SetupRow::Title(match &date {
                    Some(date) => format!("setup of {date}"),
                    None => "setup".into(),
                }),
                SetupRow::Muted("replay a step".into()),
                SetupRow::Blank,
            ];
            for (i, (done, label, state)) in steps.iter().enumerate() {
                rows.push(SetupRow::Step { n: i + 1, done: Some(*done), label: label.clone(), note: String::new(), state: state.clone(), cursor: i == cursor });
            }
            rows.push(SetupRow::Blank);
            rows.push(SetupRow::Dim("replaying step 4 reads the library again and rewrites learned/library.toml — the ranking changes, nothing else moves. the learned (plays, bans, likes) is never touched by the setup.".into()));
            self.draw(9, &rows, "[⏎ replay the step · j/k · 1-7 directly · esc leave]");
            match self.key().await {
                Cmd::Down => cursor = (cursor + 1).min(STEPS - 1),
                Cmd::Up => cursor = cursor.saturating_sub(1),
                Cmd::Top => cursor = 0,
                Cmd::Bottom => cursor = STEPS - 1,
                Cmd::Digit(n) if (1..=STEPS).contains(&n) => {
                    cursor = n - 1;
                    if self.step(n).await == Flow::Quit {
                        return Outcome::Quit;
                    }
                }
                Cmd::Auto => {
                    if self.step(cursor + 1).await == Flow::Quit {
                        return Outcome::Quit;
                    }
                }
                Cmd::Escape | Cmd::Quit => return Outcome::Ready(dir),
                _ => {}
            }
        }
    }
}

/// How the catalog gets here.
enum Cloner {
    Git(String),
    Gh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_remote_reads_as_owner_slash_repo() {
        assert_eq!(short_repo("git@github.com:kbyjoel/forkstify-catalog.git"), "kbyjoel/forkstify-catalog");
        assert_eq!(short_repo("https://github.com/aropixel/forkstify-catalog"), "aropixel/forkstify-catalog");
        assert_eq!(short_repo("forkstify-catalog"), "forkstify-catalog");
    }

    #[test]
    fn home_folds_to_a_tilde() {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            assert_eq!(tilde(&PathBuf::from(&home).join("x")), "~/x");
        }
        assert_eq!(tilde(Path::new("/tmp/x")), "/tmp/x");
    }
}
