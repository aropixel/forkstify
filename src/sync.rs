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
/// wrote it: `learned`, `edit`, `import`.
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

/// Commit what listening wrote. `Ok(None)` when there was nothing to
/// commit; `Ok(Some(subject))` otherwise.
pub fn commit_learned(dir: &Path) -> Result<Option<String>, String> {
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

pub fn push(dir: &Path) -> Result<(), String> {
    git(dir, &["push", "-q"]).map(|_| ())
}

/// Commit what we learned here, then bring in what the other machines
/// learned — rebased, so our commits sit on top, the merge driver settling
/// the counters. Returns a short word for the screen.
pub fn pull(dir: &Path) -> Result<String, String> {
    let committed = commit_learned(dir)?.is_some();
    let before = git(dir, &["rev-parse", "HEAD"])?;
    git(dir, &["pull", "--rebase", "--quiet"])?;
    let after = git(dir, &["rev-parse", "HEAD"])?;
    Ok(match (committed, before == after) {
        (true, true) => "learned committed, up to date".to_string(),
        (true, false) => "learned committed, catalog updated".to_string(),
        (false, true) => "up to date".to_string(),
        (false, false) => "catalog updated".to_string(),
    })
}

/// Commit and push now — `:sync`, and the way out of a session.
pub fn sync(dir: &Path) -> Result<String, String> {
    let committed = commit_learned(dir)?;
    push(dir)?;
    Ok(match committed {
        Some(subject) => format!("learned pushed — {subject}"),
        None => "nothing new, all pushed".to_string(),
    })
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
