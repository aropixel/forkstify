//! `forkstify import <url>` — take over the cards of another catalog.
//!
//! [0004](../docs/decisions/0004-two-repositories-targetable-catalog.md) makes
//! the import an explicit gesture; [0010] makes `cards/` **flat, one card
//! per artist**, so taking over "someone's jazz" is a matter of **files**,
//! not commits. We take the *state* of their cards, never their history.
//!
//! Two rules of caution:
//!
//! - **A card we already have is never overwritten.** Their corrections to
//!   our artists are not of interest here; that is what a PR is for, where
//!   things get discussed. The import only adds what is missing.
//! - **Vectors are regenerated in the same commit** (0019), otherwise the
//!   imported cards would only be reachable through the graph —
//!   `vector_neighbors` works only on what `vectors.jsonl` holds. And they
//!   would not be alone: a card's vectors cite its neighbours, so a new
//!   neighbour changes the text of those pointing at it.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git not found ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// A remote name taken from the URL: `…/untel/forkstify-catalog.git` → `untel`.
fn remote_name(url: &str) -> String {
    let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = trimmed.rsplit(['/', ':']);
    let _repo = parts.next();
    parts
        .next()
        .map(|owner| owner.replace(|c: char| !c.is_alphanumeric() && c != '-', "-"))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "autre".to_string())
}

pub fn run(dir: &Path, url: &str) -> Result<(), String> {
    let remote = remote_name(url);
    println!("catalog: {}", dir.display());
    println!("source:  {url}  (remote \"{remote}\")");

    // adding the remote is idempotent: the url is reset just in case
    let _ = git(dir, &["remote", "add", &remote, url]);
    git(dir, &["remote", "set-url", &remote, url])?;
    println!("\n… fetching");
    git(dir, &["fetch", "--quiet", &remote])?;

    let reference = [format!("{remote}/main"), format!("{remote}/master")]
        .into_iter()
        .find(|r| git(dir, &["rev-parse", "--verify", "--quiet", r]).is_ok())
        .ok_or_else(|| format!("neither {remote}/main nor {remote}/master"))?;

    let theirs: HashSet<String> = git(dir, &["ls-tree", "--name-only", &reference, "cards/"])?
        .lines()
        .map(str::to_string)
        .collect();
    let ours: HashSet<String> = git(dir, &["ls-tree", "--name-only", "HEAD", "cards/"])?
        .lines()
        .map(str::to_string)
        .collect();

    let mut missing: Vec<&String> = theirs.difference(&ours).collect();
    missing.sort();
    if missing.is_empty() {
        println!("\n✓ nothing to take: this catalog already has everything {remote} offers");
        return Ok(());
    }

    println!("\n{} card(s) this catalog lacks:", missing.len());
    for path in missing.iter().take(12) {
        let name = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".toml");
        println!("  {name}");
    }
    if missing.len() > 12 {
        println!("  … and {} more", missing.len() - 12);
    }

    let mut args: Vec<&str> = vec!["checkout", &reference, "--"];
    args.extend(missing.iter().map(|p| p.as_str()));
    git(dir, &args)?;
    // the whole index, not just the new ones: their neighbours cite them
    let vectors = match crate::catalog::Catalog::load(dir) {
        Ok(catalog) => crate::embed::regenerate(dir, &catalog.cards),
        Err(why) => Err(why.to_string()),
    };
    match &vectors {
        Ok(n) => println!("✓ {n} vectors recomputed"),
        Err(why) => println!("⏹ vectors not recomputed ({why}) — \"forkstify vectors\" will catch up"),
    }
    git(dir, &["add", "cards/", "vectors/"])?;
    git(
        dir,
        &[
            "commit",
            "-q",
            "-m",
            &format!("import: {} cards from {remote}", missing.len()),
            "-m",
            &crate::sync::trailer("import"),
        ],
    )?;
    println!("\n✓ {} card(s) taken in, in one commit", missing.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_remote_takes_the_owners_name() {
        assert_eq!(remote_name("git@github.com:untel/forkstify-catalog.git"), "untel");
        assert_eq!(remote_name("https://github.com/untel/forkstify-catalog"), "untel");
        assert_eq!(remote_name("https://github.com/untel/forkstify-catalog.git/"), "untel");
        // an impossible name must not produce an invalid remote
        assert_eq!(remote_name("truc"), "autre");
    }
}
