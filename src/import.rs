//! `forkstify import <url>` — reprendre les fiches d'un autre catalogue.
//!
//! [0004](../docs/decisions/0004-deux-depots-catalogue-ciblable.md) fait de
//! l'import un geste explicite ; [0010] rend `cards/` **plat, une fiche par
//! artiste**, si bien que reprendre « le jazz de quelqu'un » est une affaire
//! de **fichiers**, pas de commits. On prend l'*état* de ses fiches, jamais
//! son histoire.
//!
//! Deux règles de prudence :
//!
//! - **On n'écrase jamais une fiche qu'on a déjà.** Ses corrections sur nos
//!   artistes ne nous intéressent pas ici ; c'est le rôle d'une PR, où l'on
//!   discute. L'import n'ajoute que ce qui manque.
//! - **Les vecteurs se régénèrent ensuite**, sinon les fiches reprises ne
//!   seront atteignables que par le graphe — `vector_neighbors` ne travaille
//!   que sur ce que `vectors.jsonl` contient.
//!
//! C'est une sous-commande et non un geste d'écoute : la vectorisation
//! demande un conteneur et plusieurs minutes.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git introuvable ({e})"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Un nom de remote tiré de l'URL : `…/untel/forkstify-catalog.git` → `untel`.
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
    println!("catalogue : {}", dir.display());
    println!("source    : {url}  (remote « {remote} »)");

    // ajouter le remote est idempotent : on remet l'url au cas où
    let _ = git(dir, &["remote", "add", &remote, url]);
    git(dir, &["remote", "set-url", &remote, url])?;
    println!("\n… récupération");
    git(dir, &["fetch", "--quiet", &remote])?;

    let reference = [format!("{remote}/main"), format!("{remote}/master")]
        .into_iter()
        .find(|r| git(dir, &["rev-parse", "--verify", "--quiet", r]).is_ok())
        .ok_or_else(|| format!("ni {remote}/main ni {remote}/master"))?;

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
        println!("\n✓ rien à prendre : ce catalogue a déjà tout ce que {remote} propose");
        return Ok(());
    }

    println!("\n{} fiche(s) que ce catalogue n'a pas :", missing.len());
    for path in missing.iter().take(12) {
        let name = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".toml");
        println!("  {name}");
    }
    if missing.len() > 12 {
        println!("  … et {} de plus", missing.len() - 12);
    }

    let mut args: Vec<&str> = vec!["checkout", &reference, "--"];
    args.extend(missing.iter().map(|p| p.as_str()));
    git(dir, &args)?;
    git(dir, &["add", "cards/"])?;
    git(
        dir,
        &["commit", "-q", "-m", &format!("importe {} fiches de {remote}", missing.len())],
    )?;
    println!("\n✓ {} fiche(s) reprises, en un commit", missing.len());

    regenerate_vectors(dir);
    Ok(())
}

/// Sans vecteurs à jour, les fiches reprises n'existent que pour le graphe.
/// La vectorisation demande fastembed, donc un conteneur : on le lance s'il
/// est là, on donne la commande sinon.
fn regenerate_vectors(dir: &Path) {
    println!("\n… régénération des vecteurs (plusieurs minutes)");
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let script = format!(
        "pip install -q fastembed && python tools/vectoriser.py && chown -R {uid}:{gid} vectors tools/cache/fastembed"
    );
    let status = Command::new("docker")
        .args(["run", "--rm", "-v"])
        .arg(format!("{}:/catalogue", dir.display()))
        .args(["-w", "/catalogue", "-e", "FASTEMBED_CACHE_PATH=/catalogue/tools/cache/fastembed"])
        .args(["python:3.12-slim", "bash", "-c", &script])
        .status();
    match status {
        Ok(code) if code.success() => println!("\n✓ vecteurs à jour"),
        Ok(_) | Err(_) => {
            println!("\n⏹ vecteurs non régénérés — les fiches reprises ne seront");
            println!("   atteignables que par le graphe tant qu'ils ne le sont pas.");
            println!("\n   docker run --rm -v \"{}\":/catalogue -w /catalogue \\", dir.display());
            println!("     -e FASTEMBED_CACHE_PATH=/catalogue/tools/cache/fastembed \\");
            println!("     python:3.12-slim bash -c \"{script}\"");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_remote_prend_le_nom_du_propietaire() {
        assert_eq!(remote_name("git@github.com:untel/forkstify-catalog.git"), "untel");
        assert_eq!(remote_name("https://github.com/untel/forkstify-catalog"), "untel");
        assert_eq!(remote_name("https://github.com/untel/forkstify-catalog.git/"), "untel");
        // un nom impossible ne doit pas produire un remote invalide
        assert_eq!(remote_name("truc"), "autre");
    }
}
