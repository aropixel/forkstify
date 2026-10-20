//! Synchronising the catalogue between machines (0017): `learned/` is
//! committed and pushed by the application itself, pulled at start, and
//! merged counter by counter when two machines learned at once.
//!
//! Every commit forkstify makes carries a trailer — `Forkstify: <kind>
//! <version>` — which is what lets anyone count them across public forks:
//! `gh api search/commits -f q='"Forkstify:"' --jq .total_count`.
//!
//! Commit messages are in English: they are a public interface of the
//! repository, like the file format. Everything shown on screen stays in
//! French.

use std::path::Path;
use std::process::Command;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The trailer of every commit made by forkstify. `kind` says which gesture
/// wrote it: `learned`, `edit`, `import`, `discography`…
pub fn trailer(kind: &str) -> String {
    format!("Forkstify: {kind} {VERSION}")
}

/// Run git in the catalogue, with short network timeouts: a pull at start
/// must not hang the screen when the machine is offline.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_SSH_COMMAND", "ssh -o ConnectTimeout=5 -o BatchMode=yes")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("git not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(err.lines().last().unwrap_or("git failed").to_string())
    }
}

/// Does `learned/` hold anything not committed yet?
pub fn dirty(dir: &Path) -> bool {
    git(dir, &["status", "--porcelain", "--", "learned"]).map(|s| !s.is_empty()).unwrap_or(false)
}

/// A merge or a rebase stopped half-way: nothing may be committed on top —
/// a commit of the learned would take in whatever the stopped operation
/// left on disk (Joel, 2026-10-20: a rebase that stopped had removed six
/// learned files, and the ten-minute commit recorded their removal).
pub fn in_progress(dir: &Path) -> bool {
    let git_dir = dir.join(".git");
    git_dir.join("MERGE_HEAD").exists() || git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
}

/// Commit what listening wrote. `Ok(None)` when there was nothing to
/// commit; `Ok(Some(subject))` otherwise.
pub fn commit_learned(dir: &Path) -> Result<Option<String>, String> {
    if in_progress(dir) {
        return Err("a merge or a rebase is stopped in the catalog — :catalog, or git, first".to_string());
    }
    git(dir, &["add", "-A", "--", "learned"])?;
    let staged = git(dir, &["diff", "--cached", "--name-only"])?;
    if staged.is_empty() {
        return Ok(None);
    }
    let artists = staged.lines().filter(|l| l.starts_with("learned/artists/")).count();
    let marks = staged.lines().filter(|l| l.starts_with("learned/marks/")).count();
    let mut parts = Vec::new();
    if artists > 0 {
        parts.push(format!("{artists} artist{}", if artists > 1 { "s" } else { "" }));
    }
    if marks > 0 {
        parts.push(format!("{marks} mark file{}", if marks > 1 { "s" } else { "" }));
    }
    if parts.is_empty() {
        parts.push(format!("{} file(s)", staged.lines().count()));
    }
    let subject = format!("learned: {}", parts.join(", "));
    git(dir, &["commit", "-q", "-m", &subject, "-m", &trailer("learned")])?;
    Ok(Some(subject))
}

/// Commit whatever changed in one folder of the catalogue, and nothing
/// else — what is staged or modified elsewhere stays as it is. `Ok(false)`
/// when the folder had nothing to commit.
pub fn commit_folder(dir: &Path, folder: &str, subject: &str, body: Option<&str>, kind: &str) -> Result<bool, String> {
    if git(dir, &["status", "--porcelain", "--", folder])?.is_empty() {
        return Ok(false);
    }
    git(dir, &["add", "-A", "--", folder])?;
    let trailer = trailer(kind);
    let mut args = vec!["commit", "-q", "-m", subject];
    if let Some(body) = body {
        args.extend(["-m", body]);
    }
    args.extend(["-m", trailer.as_str(), "--", folder]);
    git(dir, &args)?;
    Ok(true)
}

pub fn push(dir: &Path) -> Result<(), String> {
    if is_local(dir) {
        return Ok(());
    }
    git(dir, &["push", "-q"]).map(|_| ())
}

/// A clone of the reference without a fork of one's own (the setup's
/// "nothing — local mode"): everything works, the learned is committed
/// but never pushed. Marked in the clone's config, never versioned.
pub fn is_local(dir: &Path) -> bool {
    git(dir, &["config", "--get", "forkstify.local"]).is_ok_and(|v| v.trim() == "true")
}

pub fn set_local(dir: &Path, local: bool) {
    let _ = git(dir, &["config", "forkstify.local", if local { "true" } else { "false" }]);
}

/// Commit what we learned here, then bring in what the other machines
/// learned — rebased, so our commits sit on top, the merge driver settling
/// the counters. Returns a short word for the screen.
pub fn pull(dir: &Path) -> Result<String, String> {
    let committed = commit_learned(dir)?.is_some();
    if is_local(dir) {
        return Ok(if committed { "learned committed, local".to_string() } else { "local".to_string() });
    }
    let before = git(dir, &["rev-parse", "HEAD"])?;
    // the learned commits are rebased, so the history stays a line; but
    // an update from the reference not pushed yet is a merge, and a rebase
    // would replay the reference's whole history flat, from its first
    // commit (Joel, 2026-10-20) — then it is merged instead
    let unpushed_merge = git(dir, &["rev-list", "--merges", "@{upstream}..HEAD"]).is_ok_and(|merges| !merges.is_empty());
    let mode = if unpushed_merge { "--no-rebase" } else { "--rebase" };
    if let Err(why) = git(dir, &["pull", mode, "--no-edit", "--quiet"]) {
        // never left half-way: the catalog stays as it was before the pull
        if dir.join(".git/rebase-merge").exists() || dir.join(".git/rebase-apply").exists() {
            let _ = git(dir, &["rebase", "--abort"]);
        }
        if dir.join(".git/MERGE_HEAD").exists() {
            let _ = git(dir, &["merge", "--abort"]);
        }
        return Err(why);
    }
    let after = git(dir, &["rev-parse", "HEAD"])?;
    Ok(match (committed, before == after) {
        (true, true) => "learned committed, up to date".to_string(),
        (true, false) => "learned committed, catalog updated".to_string(),
        (false, true) => "up to date".to_string(),
        (false, false) => "catalog updated".to_string(),
    })
}

/// Commit and push now — the way out of a session.
pub fn sync(dir: &Path) -> Result<String, String> {
    let committed = commit_learned(dir)?;
    push(dir)?;
    let local = is_local(dir);
    Ok(match (committed, local) {
        (Some(subject), false) => format!("learned pushed — {subject}"),
        (Some(subject), true) => format!("learned committed, local — {subject}"),
        (None, false) => "nothing new, all pushed".to_string(),
        (None, true) => "nothing new, local".to_string(),
    })
}

/// `:sync` — the whole round, what the start does and could not: the
/// learned committed, the other machines' brought in, ours pushed. The
/// way back from "not synced" without relaunching (Joel, 2026-10-20).
pub fn resync(dir: &Path) -> Result<String, String> {
    let word = pull(dir)?;
    push(dir)?;
    Ok(if is_local(dir) { word } else { format!("{word}, pushed") })
}

/// What the status line says of a sync that failed: the reason, short —
/// "offline" was said for anything, a stopped rebase included.
pub fn failed(why: &str) -> String {
    let short: String = why.chars().take(60).collect();
    format!("⇅ not synced — {short} · :sync to retry")
}

/// Teach this clone to merge learned files through us. The driver lives in
/// the clone's config (never versioned), the attribute in
/// `.git/info/attributes` unless the repository already ships it.
pub fn ensure_merge_driver(dir: &Path) {
    let Ok(exe) = std::env::current_exe() else { return };
    let driver = format!("{} merge-learned %O %A %B", exe.display());
    let _ = git(dir, &["config", "merge.learned.name", "forkstify: three-way merge of learned counters"]);
    let _ = git(dir, &["config", "merge.learned.driver", &driver]);
    let rule = "learned/artists/*.toml merge=learned";
    let shipped = std::fs::read_to_string(dir.join(".gitattributes"))
        .map(|t| t.contains("merge=learned"))
        .unwrap_or(false);
    if shipped {
        return;
    }
    let info = dir.join(".git").join("info").join("attributes");
    let present = std::fs::read_to_string(&info).map(|t| t.contains("merge=learned")).unwrap_or(false);
    if !present {
        let _ = std::fs::create_dir_all(info.parent().unwrap());
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&info)
            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("{rule}\n").as_bytes()));
    }
}

/// `forkstify merge-learned <base> <ours> <theirs>`: git's merge driver
/// contract — the result goes into `ours`, a non-zero exit means conflict.
pub fn merge_learned(base: &Path, ours: &Path, theirs: &Path) -> Result<(), String> {
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let base_text = read(base).unwrap_or_default();
    let merged = crate::learned::merge_artist(
        if base_text.trim().is_empty() { None } else { Some(base_text.as_str()) },
        &read(ours)?,
        &read(theirs)?,
    )?;
    std::fs::write(ours, merged).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Joel's 2026-10-20: an update from the reference merged and not
    /// pushed yet, the other machine's learned pushed meanwhile. The start
    /// must merge — a rebase replayed the reference's history flat, its
    /// removal of `learned/` with it, and stopped half-way.
    #[test]
    fn an_unpushed_update_is_merged_not_replayed() {
        if std::process::Command::new("git").arg("--version").output().is_err() {
            eprintln!("git absent — test skipped");
            return;
        }
        let root = std::env::temp_dir().join(format!("forkstify-sync-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let sh = |dir: &Path, args: &[&str]| git(dir, args).unwrap_or_else(|why| panic!("git {args:?}: {why}"));
        let at = |name: &str| root.join(name).to_str().unwrap().to_string();
        let identity = |dir: &Path| {
            sh(dir, &["config", "user.name", "Test"]);
            sh(dir, &["config", "user.email", "test@example.net"]);
        };

        // the reference, which carried a learned/ once and then dropped it
        sh(&root, &["init", "-q", "--bare", "-b", "main", &at("reference.git")]);
        let seed = root.join("seed");
        sh(&root, &["clone", "-q", &at("reference.git"), &at("seed")]);
        identity(&seed);
        std::fs::create_dir_all(seed.join("learned")).unwrap();
        std::fs::create_dir_all(seed.join("cards")).unwrap();
        std::fs::write(seed.join("learned/mbid.json"), "{}\n").unwrap();
        std::fs::write(seed.join("cards/a.toml"), "format = 1\n").unwrap();
        sh(&seed, &["add", "-A"]);
        sh(&seed, &["commit", "-q", "-m", "seed"]);
        sh(&seed, &["push", "-q", "-u", "origin", "main"]);
        // the fork, taken then
        sh(&root, &["clone", "-q", "--bare", &at("reference.git"), &at("fork.git")]);
        sh(&seed, &["rm", "-q", "-r", "learned"]);
        std::fs::write(seed.join("cards/b.toml"), "format = 1\n").unwrap();
        sh(&seed, &["add", "-A"]);
        sh(&seed, &["commit", "-q", "-m", "the reference drops its learned"]);
        sh(&seed, &["push", "-q"]);

        // this machine: the update merged, the learned kept, not pushed
        let here = root.join("here");
        sh(&root, &["clone", "-q", &at("fork.git"), &at("here")]);
        identity(&here);
        sh(&here, &["remote", "add", "upstream", &at("reference.git")]);
        sh(&here, &["fetch", "-q", "upstream"]);
        sh(&here, &["merge", "--no-ff", "--no-commit", "-q", "upstream/main"]);
        sh(&here, &["restore", "--source=HEAD", "--staged", "--worktree", "--", "learned"]);
        sh(&here, &["commit", "-q", "-m", "Update from the reference"]);
        // the other machine learned and pushed
        let there = root.join("there");
        sh(&root, &["clone", "-q", &at("fork.git"), &at("there")]);
        identity(&there);
        std::fs::create_dir_all(there.join("learned/artists")).unwrap();
        std::fs::write(there.join("learned/artists/a.toml"), "plays = 1.0\n").unwrap();
        sh(&there, &["add", "-A"]);
        sh(&there, &["commit", "-q", "-m", "learned: 1 artist"]);
        sh(&there, &["push", "-q"]);

        let word = pull(&here).expect("pull");
        assert!(!in_progress(&here), "nothing left half-way");
        assert!(here.join("learned/mbid.json").exists(), "the learned stays: {word}");
        assert!(here.join("learned/artists/a.toml").exists(), "the other machine's learned came in");
        assert!(here.join("cards/b.toml").exists(), "the update stays");
        // and nothing is committed while a merge is stopped
        std::fs::write(here.join(".git/MERGE_HEAD"), sh(&here, &["rev-parse", "HEAD"])).unwrap();
        assert!(commit_learned(&here).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
