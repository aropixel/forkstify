//! The fork facing the reference — chantier B of
//! `docs/conception/sortie.md`, screens of `Catalogue.dc.html` (Joel,
//! 20/09/2026). Three gestures under `C`: `Cd` **diff**, what this catalog
//! has beyond the reference; `Cp` **propose**, offer those cards upstream
//! as one pull request; `Cu` **update**, bring the reference into the
//! fork. Plus `:catalog`, the state in one line, and `:catalog fork <url>`,
//! the way out of the local mode.
//!
//! The personal overlay is never stored, it is **computed**
//! ([0008](../docs/decisions/0008-le-fork-est-la-surcouche.md)): in a
//! fork, what is yours is your commits, and `git diff upstream/main`
//! renders them card by card. A proposal carries the **state of the
//! cards**, never the history of `main` — which mixes in the learned
//! (0014, never upstream) and the vectors (0019, derived). An update is a
//! **merge**, not a rebase: `main` is shared by two machines that pull
//! with rebase, rewriting pushed commits would break the other one.
//!
//! Everything here blocks on git (and on the network for fetch and push):
//! the session runs it off the loop, in `spawn_blocking`.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const PROPOSAL_BRANCH: &str = "proposal";

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("git failed").to_string())
    }
}

/// Where `gh` is: on the PATH, or where a version manager keeps it —
/// launched from the desktop's bar, forkstify does not have a shell's
/// PATH, and the first `Cp` went to the browser for that (Joel,
/// 20/09/2026).
pub fn gh_command() -> std::process::Command {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{home}/.local/share/mise/shims/gh"),
        format!("{home}/.local/bin/gh"),
        "/usr/local/bin/gh".to_string(),
        "/usr/bin/gh".to_string(),
    ];
    let on_path = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| Path::new(dir).join("gh").is_file());
    if on_path {
        return std::process::Command::new("gh");
    }
    match candidates.iter().find(|c| Path::new(c).is_file()) {
        Some(found) => std::process::Command::new(found),
        None => std::process::Command::new("gh"),
    }
}

fn gh(args: &[&str]) -> Result<String, String> {
    let out = gh_command().args(args).output().map_err(|e| format!("gh not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("gh failed").to_string())
    }
}

/// Is `gh` there and logged in? The proposal goes through it when so, the
/// browser otherwise (Joel, 19/09/2026).
pub fn gh_ready() -> bool {
    gh_command().args(["auth", "status"]).output().is_ok_and(|out| out.status.success())
}

/// `owner/repo` out of a remote url.
pub fn short_repo(url: &str) -> String {
    let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = trimmed.rsplit(['/', ':']);
    let repo = parts.next().unwrap_or(trimmed);
    match parts.next() {
        Some(owner) => format!("{owner}/{repo}"),
        None => repo.to_string(),
    }
}

fn remote_url(dir: &Path, remote: &str) -> Option<String> {
    git(dir, &["remote", "get-url", remote]).ok().filter(|u| !u.is_empty())
}

/// The reference's branch, as known locally: `upstream/main` in a fork,
/// `origin/main` in a plain clone of the reference (local mode).
fn base(dir: &Path) -> Result<String, String> {
    ["upstream/main", "upstream/master", "origin/main", "origin/master"]
        .into_iter()
        .find(|reference| git(dir, &["rev-parse", "--verify", "--quiet", reference]).is_ok())
        .map(str::to_string)
        .ok_or_else(|| "no reference known — this catalog has no remote".to_string())
}

fn updated_file() -> PathBuf {
    crate::config::state_dir().join("catalog-updated")
}

/// A merge stopped on cards is waiting for git add and git commit; the
/// index is regenerated when `:catalog` resumes.
fn pending_file() -> PathBuf {
    crate::config::state_dir().join("catalog-update-pending")
}

fn slug_of(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).trim_end_matches(".toml").to_string()
}

// --- :catalog — the state in one line ---

pub struct Status {
    pub origin: Option<String>,
    pub upstream: Option<String>,
    pub local: bool,
    pub ahead: usize,
    pub behind: usize,
    /// The date of the last `Cu`, if any.
    pub updated: Option<String>,
    pub cards: usize,
    /// Cards beyond the reference (new ones).
    pub extra: usize,
    /// A merge is in progress — stopped on cards, waiting for git.
    pub merging: bool,
    /// A stopped update waits to be finished: the merge committed by hand
    /// or not yet, the index still to regenerate.
    pub pending: bool,
}

pub fn status(dir: &Path) -> Status {
    let local = crate::sync::is_local(dir);
    let origin = remote_url(dir, "origin").map(|u| short_repo(&u));
    let upstream = remote_url(dir, "upstream").map(|u| short_repo(&u));
    let (ahead, behind) = base(dir)
        .and_then(|base| git(dir, &["rev-list", "--left-right", "--count", &format!("HEAD...{base}")]))
        .ok()
        .and_then(|text| {
            let mut parts = text.split_whitespace();
            Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
        })
        .unwrap_or((0, 0));
    let cards = git(dir, &["ls-tree", "--name-only", "HEAD", "cards/"]).map(|t| t.lines().count()).unwrap_or(0);
    let extra = base(dir)
        .and_then(|base| git(dir, &["diff", "--diff-filter=A", "--name-only", &base, "--", "cards/"]))
        .map(|t| t.lines().filter(|l| !l.is_empty()).count())
        .unwrap_or(0);
    let updated = std::fs::read_to_string(updated_file()).ok().map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
    Status {
        origin,
        upstream,
        local,
        ahead,
        behind,
        updated,
        cards,
        extra,
        merging: dir.join(".git/MERGE_HEAD").exists(),
        pending: pending_file().exists(),
    }
}

impl Status {
    /// The one line of `:catalog`, and the lines under it.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.merging || self.pending {
            lines.push("⊘ an update is stopped on cards — git add, git commit, then :catalog to resume".to_string());
        }
        match (&self.origin, self.local) {
            (_, true) => lines.push("⇅ local — no fork, the learned commits and stays here · :catalog fork <url> to leave".to_string()),
            (Some(origin), false) => lines.push(format!(
                "⇅ {origin} · {} ahead · {} behind · {}",
                self.ahead,
                self.behind,
                match &self.updated {
                    Some(date) => format!("updated from the reference on {date}"),
                    None => "never updated from the reference".to_string(),
                }
            )),
            (None, false) => lines.push("⇅ no remote — the learned commits and stays here".to_string()),
        }
        lines.push(String::new());
        lines.push(format!(
            "  origin    {}",
            match (&self.origin, self.local) {
                (Some(o), false) => format!("{o} — the learned is pushed there every ten minutes"),
                (Some(o), true) => format!("{o} — the reference, read only"),
                (None, _) => "none".to_string(),
            }
        ));
        lines.push(format!("  upstream  {}", self.upstream.clone().unwrap_or_else(|| if self.local { "origin is the reference".into() } else { "none".into() })));
        lines.push(format!("  cards     {} — {} beyond the reference", self.cards, self.extra));
        lines.push(String::new());
        lines.push("  Cd the detail · Cp propose · Cu update".to_string());
        lines
    }
}

// --- Cd — what this catalog has beyond the reference ---

/// One card of the diff.
pub struct Change {
    pub slug: String,
    pub name: String,
    /// New card: `generated = true`, or written by hand.
    pub generated: bool,
    /// New card: its first tags, and how many links.
    pub summary: String,
    /// Edited card: lines added and removed.
    pub added: usize,
    pub removed: usize,
    /// Edited card: the sections touched — "tops · similar, member".
    pub sections: String,
    /// The provenance note of an added link, when there is one.
    pub note: String,
}

pub struct Diff {
    pub base: String,
    pub new: Vec<Change>,
    pub edited: Vec<Change>,
    pub removed: usize,
    pub updated: Option<String>,
}

/// The sections of a card a diff touches, from its added and removed
/// lines: tops, tags, description, mbid, spotify, the link types…
pub fn sections_of(diff: &str) -> (BTreeSet<String>, Vec<String>) {
    let mut sections = BTreeSet::new();
    let mut notes = Vec::new();
    for line in diff.lines() {
        let Some(rest) = line.strip_prefix('+').or_else(|| line.strip_prefix('-')) else { continue };
        if rest.starts_with("++") || rest.starts_with("--") {
            continue;
        }
        let trimmed = rest.trim();
        let key = trimmed.split('=').next().unwrap_or("").trim();
        let section = match key {
            "name" | "mbid" | "spotify" | "description" | "tags" | "begin" | "end" | "origin" | "generated" => key.to_string(),
            _ if trimmed.starts_with("{ to") || trimmed.starts_with("{to") => {
                if let Some(note) = trimmed.split("note = \"").nth(1) {
                    notes.push(note.split('"').next().unwrap_or("").to_string());
                }
                trimmed
                    .split("type = \"")
                    .nth(1)
                    .and_then(|t| t.split('"').next())
                    .map(str::to_string)
                    .unwrap_or_else(|| "links".to_string())
            }
            _ if trimmed.starts_with("{ track") => "doors".to_string(),
            _ if trimmed.starts_with('"') => "tops".to_string(),
            "tops" | "links" | "doors" => continue,
            _ => continue,
        };
        sections.insert(section);
    }
    (sections, notes)
}

fn card_summary(text: &str) -> (String, bool, String) {
    let mut name = String::new();
    let mut generated = false;
    let mut tags: Vec<String> = Vec::new();
    let mut links = 0;
    let mut similar = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name = ") {
            name = rest.trim_matches('"').to_string();
        } else if trimmed.starts_with("generated = true") {
            generated = true;
        } else if let Some(rest) = trimmed.strip_prefix("tags = [") {
            tags = rest.trim_end_matches(']').split(',').map(|t| t.trim().trim_matches('"').to_string()).filter(|t| !t.is_empty()).take(2).collect();
        } else if trimmed.starts_with("{ to") {
            links += 1;
            if trimmed.contains("type = \"similar\"") {
                similar += 1;
            }
        }
    }
    let mut summary = tags.join(", ");
    if links > 0 {
        if !summary.is_empty() {
            summary.push_str(" · ");
        }
        summary.push_str(&if similar == links { format!("{links} similar") } else { format!("{links} links, {similar} similar") });
    }
    (name, generated, summary)
}

pub fn diff(dir: &Path) -> Result<Diff, String> {
    let base = base(dir)?;
    let mut new = Vec::new();
    let mut edited = Vec::new();
    let status = git(dir, &["diff", "--name-status", &base, "--", "cards/"])?;
    let mut removed = 0;
    for row in status.lines() {
        let mut cols = row.split('\t');
        let (Some(kind), Some(path)) = (cols.next(), cols.next()) else { continue };
        let slug = slug_of(path);
        match kind.chars().next() {
            Some('A') => {
                let text = std::fs::read_to_string(dir.join(path)).unwrap_or_default();
                let (name, generated, summary) = card_summary(&text);
                new.push(Change {
                    name: if name.is_empty() { crate::generate::pretty(&slug) } else { name },
                    slug,
                    generated,
                    summary,
                    added: 0,
                    removed: 0,
                    sections: String::new(),
                    note: String::new(),
                });
            }
            Some('D') => removed += 1,
            _ => {
                let numstat = git(dir, &["diff", "--numstat", &base, "--", path]).unwrap_or_default();
                let mut cols = numstat.split('\t');
                let added = cols.next().and_then(|n| n.parse().ok()).unwrap_or(0);
                let removed_lines = cols.next().and_then(|n| n.parse().ok()).unwrap_or(0);
                let text = git(dir, &["diff", "-U0", &base, "--", path]).unwrap_or_default();
                let (sections, notes) = sections_of(&text);
                let name = std::fs::read_to_string(dir.join(path)).map(|t| card_summary(&t).0).unwrap_or_default();
                edited.push(Change {
                    name: if name.is_empty() { crate::generate::pretty(&slug) } else { name },
                    slug,
                    generated: false,
                    summary: String::new(),
                    added,
                    removed: removed_lines,
                    sections: sections.into_iter().collect::<Vec<_>>().join(", "),
                    note: notes.first().cloned().unwrap_or_default(),
                });
            }
        }
    }
    new.sort_by(|a, b| a.name.cmp(&b.name));
    edited.sort_by(|a, b| a.name.cmp(&b.name));
    let updated = std::fs::read_to_string(updated_file()).ok().map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
    Ok(Diff { base, new, edited, removed, updated })
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.new.is_empty() && self.edited.is_empty() && self.removed == 0
    }

    /// The overlay of `Cd`: the count and the date, the new cards, the
    /// edited ones — the cards only, never learned/ nor vectors/.
    pub fn lines(&self, at_most: usize) -> Vec<String> {
        let mut lines = Vec::new();
        if self.is_empty() {
            lines.push(format!("nothing beyond {} — the catalog is the reference's", self.base));
            return lines;
        }
        lines.push(format!(
            "{} card(s) beyond the reference · {}",
            self.new.len() + self.edited.len(),
            match &self.updated {
                Some(date) => format!("updated on {date}"),
                None => "never updated".to_string(),
            }
        ));
        lines.push(format!("+ {} new · ~ {} edited · {} removed", self.new.len(), self.edited.len(), self.removed));
        if !self.new.is_empty() {
            lines.push(String::new());
            lines.push("new".to_string());
            for change in self.new.iter().take(at_most) {
                lines.push(format!(
                    "  + {:<24} {:<10} {}",
                    change.name.chars().take(24).collect::<String>(),
                    if change.generated { "generated" } else { "written" },
                    change.summary
                ));
            }
            if self.new.len() > at_most {
                lines.push(format!("  … {} more", self.new.len() - at_most));
            }
        }
        if !self.edited.is_empty() {
            lines.push(String::new());
            lines.push("edited — these are the ones that need an ear upstream".to_string());
            for change in &self.edited {
                let mut line = format!("  ~ {:<24} +{} −{}  {}", change.name.chars().take(24).collect::<String>(), change.added, change.removed, change.sections);
                if !change.note.is_empty() {
                    line.push_str(&format!(" — {}", change.note));
                }
                lines.push(line);
            }
        }
        lines
    }

    /// The pull request, written for the reviewer (0022, English): the new
    /// cards first, one line each, to skim; the edited ones after, to read.
    pub fn title(&self) -> String {
        let generated = self.new.iter().filter(|c| c.generated).count();
        let written = self.new.len() - generated;
        let mut parts = Vec::new();
        if generated > 0 {
            parts.push(format!("{generated} generated"));
        }
        if written > 0 {
            parts.push(format!("{written} written"));
        }
        if !self.edited.is_empty() {
            parts.push(format!("{} edited", self.edited.len()));
        }
        format!("Propose {} cards ({})", self.new.len() + self.edited.len(), parts.join(", "))
    }

    pub fn body(&self) -> String {
        let mut body = String::new();
        if !self.new.is_empty() {
            body.push_str(&format!("## New cards ({}) — pipeline output, nothing to read\n\n", self.new.len()));
            for change in &self.new {
                body.push_str(&format!("- {} · {}{}\n", change.slug, if change.generated { "generated" } else { "written by hand" }, if change.summary.is_empty() { String::new() } else { format!(" · {}", change.summary) }));
            }
            body.push('\n');
        }
        if !self.edited.is_empty() {
            body.push_str(&format!("## Edited cards ({}) — please read\n\n", self.edited.len()));
            for change in &self.edited {
                body.push_str(&format!("- {} +{} −{} · {}\n", change.slug, change.added, change.removed, change.sections));
                if !change.note.is_empty() {
                    body.push_str(&format!("  source: {}\n", change.note));
                }
            }
            body.push('\n');
        }
        if self.removed > 0 {
            body.push_str(&format!("{} card(s) removed.\n\n", self.removed));
        }
        body.push_str(&crate::sync::trailer("proposal"));
        body.push('\n');
        body
    }
}

// --- Cp — propose the cards to the reference ---

pub struct Proposal {
    pub title: String,
    pub body: String,
    pub count: usize,
    /// `owner:proposal` — the head of the pull request.
    pub head: String,
    /// `owner/repo` — the reference.
    pub upstream: String,
    /// The pull request already open on this branch, updated by the push.
    pub existing: Option<String>,
    /// `gh` will create it on `y`; otherwise the browser opened already.
    pub through_gh: bool,
    /// The gh account, when `gh` is there — said on the confirmation.
    pub account: Option<String>,
}

fn worktree_dir() -> PathBuf {
    crate::config::state_dir().join("proposal")
}

/// The proposal: the state of the cards on a branch `proposal` from
/// `upstream/main`, in a worktree apart — the clone the session reads
/// never changes branch, the learned goes on committing on `main` — one
/// commit, pushed with force: one open proposal at a time, rewritten.
pub fn propose(dir: &Path) -> Result<Proposal, String> {
    if crate::sync::is_local(dir) {
        return Err("local mode — nothing to propose from: :catalog fork <url> first".to_string());
    }
    let upstream_url = remote_url(dir, "upstream").ok_or("no upstream remote — this catalog is not a fork of the reference")?;
    let origin_url = remote_url(dir, "origin").ok_or("no origin remote")?;
    git(dir, &["fetch", "--quiet", "upstream"])?;
    let diff = diff(dir)?;
    if diff.is_empty() {
        return Err("nothing beyond the reference — nothing to propose".to_string());
    }
    let wt = worktree_dir();
    if wt.join(".git").exists() {
        git(&wt, &["checkout", "-q", "-B", PROPOSAL_BRANCH, "upstream/main"])?;
    } else {
        let _ = git(dir, &["worktree", "prune"]);
        git(dir, &["worktree", "add", "-q", "-B", PROPOSAL_BRANCH, &wt.display().to_string(), "upstream/main"])?;
    }
    // the state of the cards, exactly: theirs removed, ours put in place
    let _ = std::fs::remove_dir_all(wt.join("cards"));
    git(&wt, &["checkout", "main", "--", "cards"])?;
    git(&wt, &["add", "-A", "--", "cards"])?;
    if git(&wt, &["diff", "--cached", "--quiet"]).is_ok() {
        return Err("nothing to propose once the cards are compared".to_string());
    }
    let (title, body) = (diff.title(), diff.body());
    git(&wt, &["commit", "-q", "-m", &title, "-m", &body])?;
    git(&wt, &["push", "--force", "--quiet", "origin", PROPOSAL_BRANCH])?;

    let owner = short_repo(&origin_url).split('/').next().unwrap_or("me").to_string();
    let head = format!("{owner}:{PROPOSAL_BRANCH}");
    let upstream = short_repo(&upstream_url);
    let through_gh = gh_ready();
    let account = through_gh.then(|| gh(&["api", "user", "--jq", ".login"]).ok()).flatten();
    let existing = if through_gh {
        gh(&["pr", "list", "--repo", &upstream, "--head", &head, "--state", "open", "--json", "url", "--jq", ".[0].url"])
            .ok()
            .filter(|u| !u.is_empty())
    } else {
        None
    };
    if !through_gh && existing.is_none() {
        // the browser: the comparison page, title and body in the url —
        // one reads, one clicks. Nothing leaves without that click.
        let url = format!(
            "https://github.com/{upstream}/compare/main...{}?expand=1&title={}&body={}",
            head.replace(':', ":"),
            crate::spotify::encode(&title),
            crate::spotify::encode(&body)
        );
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Ok(Proposal { title, body, count: diff.new.len() + diff.edited.len(), head, upstream, existing, through_gh, account })
}

/// `y` on the proposal: the pull request, through gh.
pub fn create_pull_request(proposal: &Proposal) -> Result<String, String> {
    gh(&[
        "pr",
        "create",
        "--repo",
        &proposal.upstream,
        "--head",
        &proposal.head,
        "--title",
        &proposal.title,
        "--body",
        &proposal.body,
    ])
    .map(|out| out.lines().last().unwrap_or("").to_string())
}

// --- Cu — bring the reference into the fork ---

pub struct Update {
    /// Cards added or changed by the reference.
    pub cards: usize,
    /// The merge stopped on these cards: (slug, what each side changed).
    pub conflicts: Vec<(String, String)>,
    pub word: String,
}

/// What each side changed on a card, said in a sentence.
fn conflict_reason(dir: &Path, base_commit: &str, path: &str) -> String {
    let describe = |against: &str| -> String {
        let text = git(dir, &["diff", "-U0", base_commit, against, "--", path]).unwrap_or_default();
        let (sections, _) = sections_of(&text);
        let list: Vec<String> = sections.into_iter().collect();
        if list.is_empty() {
            "the same lines".to_string()
        } else {
            list.join(", ")
        }
    };
    format!("upstream changed the {} · you changed the {}", describe("MERGE_HEAD"), describe("HEAD"))
}

pub fn update(dir: &Path) -> Result<Update, String> {
    update_with(dir, true)
}

/// `regenerate`: the index after a merge that changed cards — off in the
/// tests, which have no model to run.
pub fn update_with(dir: &Path, regenerate: bool) -> Result<Update, String> {
    if crate::sync::is_local(dir) {
        // the reference is origin: a pull is the update
        return crate::sync::pull(dir).map(|word| Update { cards: 0, conflicts: Vec::new(), word });
    }
    if dir.join(".git/MERGE_HEAD").exists() {
        return resume(dir);
    }
    // the learned first, as :sync does — a merge wants a clean tree
    crate::sync::commit_learned(dir)?;
    git(dir, &["fetch", "--quiet", "upstream"])?;
    let before = git(dir, &["rev-parse", "HEAD"])?;
    let base_commit = git(dir, &["merge-base", "HEAD", "upstream/main"])?;
    let merged = git(dir, &["merge", "--no-edit", "-q", "upstream/main", "-m", "Update from the reference", "-m", &crate::sync::trailer("update")]);
    if let Err(why) = merged {
        let unmerged: Vec<String> = git(dir, &["diff", "--name-only", "--diff-filter=U"])?.lines().map(str::to_string).collect();
        if unmerged.is_empty() {
            return Err(format!("merge: {why}"));
        }
        // what resolves itself: the vectors are regenerated anyway, the
        // learned is ours (0014), the tooling is theirs
        let mut conflicts = Vec::new();
        for path in &unmerged {
            if path.starts_with("vectors/") || path.starts_with("tools/") || path.starts_with("README") || path == "CONTRIBUTING.md" {
                if git(dir, &["checkout", "--theirs", "--", path]).is_err() {
                    let _ = git(dir, &["rm", "-q", "--", path]);
                }
                let _ = git(dir, &["add", "--", path]);
            } else if path.starts_with("learned/") {
                if git(dir, &["checkout", "--ours", "--", path]).is_err() {
                    let _ = git(dir, &["rm", "-q", "--", path]);
                }
                let _ = git(dir, &["add", "--", path]);
            } else {
                conflicts.push((slug_of(path), conflict_reason(dir, &base_commit, path)));
            }
        }
        if !conflicts.is_empty() {
            // stopped on cards: git gets the hand, :catalog resumes
            let _ = std::fs::write(pending_file(), &before);
            return Ok(Update { cards: 0, conflicts, word: "stopped on cards".to_string() });
        }
        git(dir, &["commit", "-q", "--no-edit"])?;
    }
    finish(dir, &before, regenerate)
}

/// After a merge — clean, or resolved by hand and committed: the index
/// regenerated when cards changed, the date remembered. An index that
/// could not be regenerated is said, never fatal: `forkstify vectors`
/// catches up, as after an import.
fn finish(dir: &Path, before: &str, regenerate: bool) -> Result<Update, String> {
    let changed = git(dir, &["diff", "--name-only", &format!("{before}..HEAD"), "--", "cards/"])?;
    let cards = changed.lines().filter(|l| !l.is_empty()).count();
    let mut regenerated = false;
    let mut caveat = String::new();
    if cards > 0 && regenerate {
        let outcome = crate::catalog::Catalog::load(dir)
            .map_err(|e| e.to_string())
            .and_then(|catalog| crate::embed::regenerate(dir, &catalog.cards));
        match outcome {
            Ok(_) => {
                git(dir, &["add", "--", "vectors"])?;
                if git(dir, &["diff", "--cached", "--quiet"]).is_err() {
                    git(dir, &["commit", "-q", "-m", "Index regenerated after the update", "-m", &crate::sync::trailer("update")])?;
                }
                regenerated = true;
            }
            Err(why) => caveat = format!(" · index not regenerated ({why}) — forkstify vectors will catch up"),
        }
    }
    let _ = std::fs::write(updated_file(), crate::learned::today_iso());
    let _ = std::fs::remove_file(pending_file());
    let word = match (cards, regenerated) {
        (0, _) => "up to date with the reference".to_string(),
        (n, true) => format!("{n} card(s) from the reference · index regenerated"),
        (n, false) => format!("{n} card(s) from the reference{caveat}"),
    };
    Ok(Update { cards, conflicts: Vec::new(), word })
}

/// `:catalog` while a merge waits: once the cards are added — and
/// committed, or not yet — the merge is completed and the index follows.
pub fn resume(dir: &Path) -> Result<Update, String> {
    resume_with(dir, true)
}

pub fn resume_with(dir: &Path, regenerate: bool) -> Result<Update, String> {
    let before = std::fs::read_to_string(pending_file()).map(|t| t.trim().to_string()).unwrap_or_default();
    let unmerged: Vec<String> = git(dir, &["diff", "--name-only", "--diff-filter=U"])?.lines().map(str::to_string).collect();
    if !unmerged.is_empty() {
        return Err(format!("{} card(s) still unmerged — git add them once resolved", unmerged.len()));
    }
    if dir.join(".git/MERGE_HEAD").exists() {
        git(dir, &["commit", "-q", "--no-edit"])?;
    }
    if before.is_empty() {
        return Err("nothing to resume".to_string());
    }
    finish(dir, &before, regenerate)
}

// --- :catalog fork <url> — out of the local mode ---

pub fn fork(dir: &Path, url: &str) -> Result<String, String> {
    if !crate::sync::is_local(dir) {
        return Err("this catalog is a fork already".to_string());
    }
    let reference = remote_url(dir, "origin").ok_or("no origin remote")?;
    if remote_url(dir, "upstream").is_none() {
        git(dir, &["remote", "add", "upstream", &reference])?;
    }
    git(dir, &["remote", "set-url", "origin", url])?;
    git(dir, &["push", "-q", "-u", "origin", "HEAD"])?;
    crate::sync::set_local(dir, false);
    Ok(format!("⇅ fork {} — the learned will be pushed there from now on", short_repo(url)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_diff_names_its_sections() {
        let diff = "\
--- a/cards/the-cure.toml
+++ b/cards/the-cure.toml
@@ -12,2 +12,3 @@
-  \"A Forest\",
+  \"A Forest\",
+  \"Lullaby\",
@@ -20 +21 @@
+  { to = \"siouxsie\", type = \"member\", note = \"linked while listening, 2026-09-06\" },
-description = \"old\"
+description = \"new\"
";
        let (sections, notes) = sections_of(diff);
        assert_eq!(sections.into_iter().collect::<Vec<_>>(), vec!["description", "member", "tops"]);
        assert_eq!(notes, vec!["linked while listening, 2026-09-06"]);
    }

    #[test]
    fn a_new_card_sums_up_in_a_line() {
        let card = "format = 1\nname = \"Codeine\"\nmbid = \"x\"\ngenerated = true\n\ntags = [\"slowcore\", \"us\", \"90s\"]\n\nlinks = [\n  { to = \"duster\", type = \"similar\" },\n  { to = \"low\", type = \"similar\" },\n]\n";
        let (name, generated, summary) = card_summary(card);
        assert_eq!(name, "Codeine");
        assert!(generated);
        assert_eq!(summary, "slowcore, us · 2 similar");
    }

    /// The pull request is written for the reviewer: the new cards to
    /// skim, the edited ones to read, the trailer last.
    #[test]
    fn the_proposal_reads_in_two_lists() {
        let diff = Diff {
            base: "upstream/main".into(),
            new: vec![Change { slug: "codeine".into(), name: "Codeine".into(), generated: true, summary: "slowcore · 4 similar".into(), added: 0, removed: 0, sections: String::new(), note: String::new() }],
            edited: vec![Change { slug: "the-cure".into(), name: "The Cure".into(), generated: false, summary: String::new(), added: 3, removed: 1, sections: "member, tops".into(), note: "listening, 09/2026".into() }],
            removed: 0,
            updated: None,
        };
        assert_eq!(diff.title(), "Propose 2 cards (1 generated, 1 edited)");
        let body = diff.body();
        assert!(body.starts_with("## New cards (1) — pipeline output, nothing to read\n\n- codeine · generated · slowcore · 4 similar\n"), "{body}");
        assert!(body.contains("## Edited cards (1) — please read\n\n- the-cure +3 −1 · member, tops\n  source: listening, 09/2026\n"), "{body}");
        assert!(body.trim_end().ends_with(&crate::sync::trailer("proposal")), "{body}");
        let lines = diff.lines(10);
        assert!(lines[1].contains("+ 1 new · ~ 1 edited · 0 removed"), "{:?}", lines);
        assert!(lines.iter().any(|l| l.contains("~ The Cure") && l.contains("+3 −1") && l.contains("member, tops")), "{:?}", lines);
    }

    /// Three repositories in a temp dir — the reference (bare), the fork
    /// (bare), and the clone the session reads — and the three gestures
    /// on them: what is beyond the reference, the proposal branch, the
    /// update that merges, and the one that stops on a card.
    #[test]
    fn diff_propose_and_update_on_real_repositories() {
        if std::process::Command::new("git").arg("--version").output().is_err() {
            eprintln!("git absent — test skipped");
            return;
        }
        let root = std::env::temp_dir().join(format!("forkstify-fork-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let sh = |dir: &Path, args: &[&str]| -> String {
            git(dir, args).unwrap_or_else(|why| panic!("git {args:?} in {}: {why}", dir.display()))
        };
        let identity = |dir: &Path| {
            sh(dir, &["config", "user.name", "Test"]);
            sh(dir, &["config", "user.email", "test@example.net"]);
        };
        let card = |name: &str, tops: &[&str]| {
            format!(
                "format = 1\nname = \"{name}\"\nmbid = \"{}\"\n\ntags = [\"post-punk\"]\n\ntops = [\n{}]\n",
                name.to_lowercase(),
                tops.iter().map(|t| format!("  \"{t}\",\n")).collect::<String>()
            )
        };

        // the reference: one card, seeded through a working clone
        let reference = root.join("reference.git");
        sh(&root, &["init", "-q", "--bare", "-b", "main", reference.to_str().unwrap()]);
        let seed = root.join("seed");
        sh(&root, &["clone", "-q", reference.to_str().unwrap(), seed.to_str().unwrap()]);
        identity(&seed);
        std::fs::create_dir_all(seed.join("cards")).unwrap();
        std::fs::write(seed.join("cards/the-cure.toml"), card("The Cure", &["A Forest"])).unwrap();
        std::fs::write(seed.join("catalog.toml"), "[proximity]\nsimilar = 4\n").unwrap();
        sh(&seed, &["add", "-A"]);
        sh(&seed, &["commit", "-q", "-m", "seed"]);
        sh(&seed, &["push", "-q", "-u", "origin", "main"]);

        // the fork (bare) and its clone: origin = the fork, upstream = the reference
        let fork_bare = root.join("fork.git");
        sh(&root, &["clone", "-q", "--bare", reference.to_str().unwrap(), fork_bare.to_str().unwrap()]);
        let clone = root.join("clone");
        sh(&root, &["clone", "-q", fork_bare.to_str().unwrap(), clone.to_str().unwrap()]);
        identity(&clone);
        sh(&clone, &["remote", "add", "upstream", reference.to_str().unwrap()]);
        sh(&clone, &["fetch", "-q", "upstream"]);

        // what is mine: a new generated card, a top added, some learned
        std::fs::write(clone.join("cards/codeine.toml"), card("Codeine", &["D"]).replace("mbid", "generated = true\nmbid")).unwrap();
        std::fs::write(clone.join("cards/the-cure.toml"), card("The Cure", &["A Forest", "Lullaby"])).unwrap();
        std::fs::create_dir_all(clone.join("learned/artists")).unwrap();
        std::fs::write(clone.join("learned/artists/the-cure.toml"), "plays = 3.0\n").unwrap();
        sh(&clone, &["add", "-A"]);
        sh(&clone, &["commit", "-q", "-m", "mine"]);

        let diff = diff(&clone).expect("diff");
        assert_eq!(diff.new.len(), 1);
        assert_eq!(diff.new[0].name, "Codeine");
        assert!(diff.new[0].generated);
        assert_eq!(diff.edited.len(), 1);
        assert_eq!(diff.edited[0].slug, "the-cure");
        assert_eq!((diff.edited[0].added, diff.edited[0].removed), (1, 0));
        assert_eq!(diff.edited[0].sections, "tops");
        assert_eq!(diff.title(), "Propose 2 cards (1 generated, 1 edited)");

        // the proposal: a branch on the fork, from the reference, the
        // cards only — no learned, one commit
        let proposal = propose(&clone).expect("propose");
        assert_eq!(proposal.count, 2);
        assert!(!proposal.through_gh || proposal.existing.is_none());
        let files = sh(&fork_bare, &["ls-tree", "-r", "--name-only", PROPOSAL_BRANCH]);
        assert!(files.contains("cards/codeine.toml"), "{files}");
        assert!(!files.contains("learned/"), "{files}");
        let history = sh(&fork_bare, &["rev-list", "--count", &format!("upstream_main..{PROPOSAL_BRANCH}").replace("upstream_main", &sh(&clone, &["rev-parse", "upstream/main"]))]);
        assert_eq!(history.trim(), "1");
        let subject = sh(&fork_bare, &["log", "-1", "--format=%s", PROPOSAL_BRANCH]);
        assert_eq!(subject, "Propose 2 cards (1 generated, 1 edited)");
        // a second proposal rewrites the same branch
        let again = propose(&clone).expect("propose again");
        assert_eq!(again.count, 2);
        assert_eq!(sh(&fork_bare, &["rev-list", "--count", &format!("{}..{PROPOSAL_BRANCH}", sh(&clone, &["rev-parse", "upstream/main"]))]).trim(), "1");

        // the reference moves on: a new card — the update merges it, and
        // commits the learned first
        std::fs::write(seed.join("cards/slowdive.toml"), card("Slowdive", &["Alison"])).unwrap();
        sh(&seed, &["add", "-A"]);
        sh(&seed, &["commit", "-q", "-m", "slowdive"]);
        sh(&seed, &["push", "-q"]);
        std::fs::write(clone.join("learned/artists/codeine.toml"), "plays = 1.0\n").unwrap();
        let update = update_with(&clone, false).expect("update");
        assert_eq!(update.cards, 1, "{}", update.word);
        assert!(update.conflicts.is_empty());
        assert!(clone.join("cards/slowdive.toml").exists());
        assert!(clone.join("cards/codeine.toml").exists());
        assert!(sh(&clone, &["status", "--porcelain"]).is_empty(), "clean after the update");
        assert!(sh(&clone, &["log", "--format=%s", "-3"]).contains("Update from the reference"));

        // both sides touch the same line of the same card: the update stops
        std::fs::write(seed.join("cards/the-cure.toml"), card("The Cure", &["A Forest", "Boys Don't Cry"])).unwrap();
        sh(&seed, &["add", "-A"]);
        sh(&seed, &["commit", "-q", "-m", "tops upstream"]);
        sh(&seed, &["push", "-q"]);
        let stopped = update_with(&clone, false).expect("update stops, no error");
        assert_eq!(stopped.conflicts.len(), 1, "{:?}", stopped.conflicts);
        assert_eq!(stopped.conflicts[0].0, "the-cure");
        assert!(stopped.conflicts[0].1.contains("tops"), "{}", stopped.conflicts[0].1);
        assert!(clone.join(".git/MERGE_HEAD").exists());
        assert!(status(&clone).merging);
        // resolved by hand, added, then :catalog resumes
        std::fs::write(clone.join("cards/the-cure.toml"), card("The Cure", &["A Forest", "Lullaby", "Boys Don't Cry"])).unwrap();
        sh(&clone, &["add", "cards/the-cure.toml"]);
        let resumed = resume_with(&clone, false).expect("resume");
        assert_eq!(resumed.cards, 1, "{}", resumed.word);
        assert!(!clone.join(".git/MERGE_HEAD").exists());
        assert!(!status(&clone).pending);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_remote_reads_as_owner_slash_repo() {
        assert_eq!(short_repo("git@github.com:kbyjoel/forkstify-catalog.git"), "kbyjoel/forkstify-catalog");
        assert_eq!(short_repo("https://github.com/aropixel/forkstify-catalog"), "aropixel/forkstify-catalog");
    }
}
