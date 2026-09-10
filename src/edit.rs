//! The **edits** (0013): they change a card and produce a readable commit.
//! That is what sets them apart from measures — those write to `learned/`
//! without a word, these leave a trace that can be reread and undone,
//! "the fork is the overlay" ([0008]) taken at its word.
//!
//! **Cards are patched textually, never rewritten.** A round trip through
//! serde would lose everything the code does not model — `format`,
//! `generated`, `mbid`, `spotify`, `begin`, `origin`, `description`, the
//! key order and the quotes chosen by hand. A card is a file a human reads
//! and corrects ([0002]: "the card format is a public interface"); a line
//! is inserted into it, the whole is not regenerated.

use std::path::{Path, PathBuf};

pub fn card_path(catalog_dir: &Path, slug: &str) -> PathBuf {
    catalog_dir.join("cards").join(format!("{slug}.toml"))
}

/// Escape a value for a basic TOML string.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Find the array `<key> = [` … `]` and return the bounds of its content.
fn array_span(text: &str, key: &str) -> Option<(usize, usize)> {
    let head = format!("\n{key} = [");
    let start = if text.starts_with(&format!("{key} = [")) {
        0
    } else {
        text.find(&head)? + 1
    };
    let open = text[start..].find('[')? + start;
    let close = text[open..].find("\n]")? + open + 1;
    Some((open + 1, close))
}

/// Insert a line just before an array closes. If the array does not
/// exist, it is created at the end of the card.
fn insert_into_array(text: &str, key: &str, line: &str) -> String {
    match array_span(text, key) {
        Some((_, close)) => format!("{}{line}\n{}", &text[..close], &text[close..]),
        None => {
            let mut out = text.trim_end().to_string();
            out.push_str(&format!("\n\n{key} = [\n{line}\n]\n"));
            out
        }
    }
}

/// The note says **where the line comes from**, and since when. 0010 wants
/// a note to explain the link; a line written during a listen cannot claim
/// a learned explanation, but it can state its origin — which lets whoever
/// rereads it, later or upstream, weigh it.
fn provenance(what: &str) -> String {
    quoted(&format!("{what} while listening, {}", crate::learned::today_iso()))
}

fn door_line(title: &str, tags: &[String]) -> String {
    let list: Vec<String> = tags.iter().map(|tag| quoted(tag)).collect();
    format!(
        "  {{ track = {}, to = [{}], note = {} }},",
        quoted(title),
        list.join(", "),
        provenance("set")
    )
}

fn link_line(to_slug: &str, kind: &str) -> String {
    format!(
        "  {{ to = {}, type = {}, note = {} }},",
        quoted(to_slug),
        quoted(kind),
        provenance("linked")
    )
}

/// What an edit changed, said in one sentence — it is the commit message
/// and also what shows on screen. One wording for both: what the user reads
/// is what git will keep.
pub struct Edit {
    pub summary: String,
    /// The detail, when an edit carries several: the commit body says what
    /// the subject counts. A batch of tops has one, a lone gesture does not
    /// need it.
    pub body: Option<String>,
    pub path: PathBuf,
    /// What else the same edit touched — the index, when a card comes with
    /// its vector (0019): one thought, one commit.
    pub also: Vec<PathBuf>,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("card unreadable ({e})"))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("card not written ({e})"))
}

/// `td` — make a track a **door** towards a direction ([0011]: `to` points
/// to tags, never to an artist).
pub fn add_door(
    dir: &Path,
    slug: &str,
    name: &str,
    title: &str,
    tags: &[String],
) -> Result<Edit, String> {
    if tags.is_empty() {
        return Err("no direction to give this door".into());
    }
    let path = card_path(dir, slug);
    let text = read(&path)?;
    if let Some((from, to)) = array_span(&text, "doors") {
        if text[from..to].contains(&format!("track = {}", quoted(title))) {
            return Err(format!("\"{title}\" is already a door"));
        }
    }
    let updated = insert_into_array(&text, "doors", &door_line(title, tags));
    write(&path, &updated)?;
    Ok(Edit {
        summary: format!("{name} — door: {title} → {}", tags.join(", ")),
        body: None,
        path,
        also: Vec::new(),
    })
}

/// `aL` — link two artists ([0010]: a closed type, a note that explains).
pub fn add_link(
    dir: &Path,
    slug: &str,
    name: &str,
    to_slug: &str,
    to_name: &str,
    kind: &str,
) -> Result<Edit, String> {
    if slug == to_slug {
        return Err("an artist does not link to itself".into());
    }
    let path = card_path(dir, slug);
    let text = read(&path)?;
    if let Some((from, to)) = array_span(&text, "links") {
        if text[from..to].contains(&format!("to = {}", quoted(to_slug))) {
            return Err(format!("{name} is already linked to {to_name}"));
        }
    }
    let updated = insert_into_array(&text, "links", &link_line(to_slug, kind));
    write(&path, &updated)?;
    Ok(Edit { summary: format!("{name} — link: → {to_name} ({kind})"), body: None, path, also: Vec::new() })
}

/// The **batch** of the discography screen (mockup 1a, 07/09/2026): five
/// tops get fixed in a row there, and five commits for a single thought do
/// not reread well. One read, one write, one commit — and the message says
/// the count, the body says the titles.
///
/// Removals go before additions: promoting then removing the same title in
/// the same batch must leave it out, not in.
pub fn set_tops(
    dir: &Path,
    slug: &str,
    name: &str,
    adds: &[String],
    removes: &[String],
) -> Result<Edit, String> {
    if adds.is_empty() && removes.is_empty() {
        return Err("nothing to write".into());
    }
    let path = card_path(dir, slug);
    let mut text = read(&path)?;

    let mut removed = Vec::new();
    for title in removes {
        let needle = quoted(title);
        let kept: Vec<&str> = text
            .lines()
            .filter(|line| {
                !(line.trim_start().starts_with(&needle) && line.trim_end().ends_with(','))
            })
            .collect();
        if kept.len() != text.lines().count() {
            text = kept.join("\n") + "\n";
            removed.push(title.clone());
        }
    }

    let mut added = Vec::new();
    for title in adds {
        if let Some((from, to)) = array_span(&text, "tops") {
            if text[from..to].contains(&quoted(title)) {
                continue;
            }
        }
        text = insert_into_array(&text, "tops", &format!("  {},", quoted(title)));
        added.push(title.clone());
    }

    if added.is_empty() && removed.is_empty() {
        return Err("the card's tops already said so".into());
    }
    write(&path, &text)?;

    let mut lines = Vec::new();
    for title in &added {
        lines.push(format!("+ {title}"));
    }
    for title in &removed {
        lines.push(format!("− {title}"));
    }
    Ok(Edit {
        summary: format!("{name} — tops: +{} −{}", added.len(), removed.len()),
        body: Some(lines.join("\n")),
        path,
        also: Vec::new(),
    })
}

/// **Generating a card** ([0016]): the only edit that *creates* a file
/// instead of patching one. The text comes from `generate`, which composes
/// it like the 316 reference cards — it is not serialized here, for the
/// same reason nothing is rewritten elsewhere.
///
/// It refuses to overwrite an existing card: a generation is an addition,
/// never a replacement. What exists is fixed by hand or by the other
/// edits.
pub fn create_card(
    dir: &Path,
    slug: &str,
    name: &str,
    text: &str,
    tops: usize,
    links: usize,
) -> Result<Edit, String> {
    let path = card_path(dir, slug);
    if path.exists() {
        return Err(format!("{name} already has a card"));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("no cards/ folder ({e})"))?;
    }
    write(&path, text)?;
    Ok(Edit {
        summary: format!("{name} — card generated"),
        body: Some(format!(
            "{tops} top(s), {links} link(s) — MusicBrainz and Deezer.\ngenerated = true: to review."
        )),
        path,
        also: Vec::new(),
    })
}

/// The commit. An edit that leaves no readable trace is not one (0013);
/// no push, though — that is `:sync`'s business.
pub fn commit(dir: &Path, edit: &Edit) -> Result<(), String> {
    let relative = |path: &Path| -> String {
        path.strip_prefix(dir).unwrap_or(path).to_string_lossy().to_string()
    };
    let run = |args: &[&str]| -> Result<(), String> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map_err(|e| format!("git not found ({e})"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    };
    for path in std::iter::once(&edit.path).chain(&edit.also) {
        run(&["add", &relative(path)])?;
    }
    let mut args = vec!["commit".to_string(), "-q".to_string(), "-m".to_string(), edit.summary.clone()];
    if let Some(body) = &edit.body {
        args.push("-m".to_string());
        args.push(body.clone());
    }
    args.push("-m".to_string());
    args.push(crate::sync::trailer("edit"));
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&args)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = "format = 1\nname = \"The Cure\"\nmbid = \"abc\"\n\ntags = [\"post-punk\"]\n\ntops = [\n  \"A Forest\",\n  \"Lullaby\",\n]\n\nlinks = [\n  { to = \"joy-division\", type = \"scene\" },\n]\n";

    /// The rest of the card must survive intact: it is a public interface,
    /// not a data structure of ours.
    #[test]
    fn une_insertion_ne_touche_a_rien_d_autre() {
        let out = insert_into_array(CARD, "tops", "  \"Push\",");
        assert!(out.contains("format = 1"));
        assert!(out.contains("mbid = \"abc\""));
        assert!(out.contains("  \"A Forest\",\n  \"Lullaby\",\n  \"Push\",\n]"));
        // and the other arrays do not move
        assert!(out.contains("{ to = \"joy-division\", type = \"scene\" },"));
    }

    #[test]
    fn un_tableau_absent_se_cree_en_fin_de_fiche() {
        let out = insert_into_array(CARD, "doors", "  { track = \"A Forest\" },");
        assert!(out.trim_end().ends_with("doors = [\n  { track = \"A Forest\" },\n]"));
        assert!(out.contains("tops = ["));
    }

    #[test]
    fn les_bornes_du_bon_tableau() {
        let (from, to) = array_span(CARD, "tops").expect("tops");
        assert!(CARD[from..to].contains("A Forest"));
        assert!(!CARD[from..to].contains("joy-division"));
    }

    /// The real risk of a textual patch: producing a TOML nobody can read
    /// back. After every insertion, the card must still load as a card.
    #[test]
    fn la_fiche_reste_lisible_apres_chaque_edition() {
        let mut text = CARD.to_string();
        text = insert_into_array(&text, "tops", "  \"Push\",");
        text = insert_into_array(
            &text,
            "doors",
            &door_line("A Forest", &["post-punk".into(), "atmospherique".into()]),
        );
        text = insert_into_array(&text, "links", &link_line("siouxsie", "member"));

        let card: crate::catalog::Card =
            toml::from_str(&text).expect("la fiche doit encore se lire");
        assert_eq!(card.name, "The Cure");
        assert!(card.tops.contains(&"Push".to_string()));
        assert!(card.tops.contains(&"A Forest".to_string()));
        assert_eq!(card.doors.len(), 1);
        assert_eq!(card.doors[0].track, "A Forest");
        assert_eq!(card.doors[0].to, vec!["post-punk", "atmospherique"]);
        assert_eq!(card.links.len(), 2);
        assert!(card.links.iter().any(|l| l.to == "siouxsie" && l.kind == "member"));
    }

    /// A title with quotes or an apostrophe must not break the card.
    #[test]
    fn un_titre_retors_passe_quand_meme() {
        let text = insert_into_array(CARD, "tops", &format!("  {},", quoted("L'\"autre\" titre")));
        let card: crate::catalog::Card = toml::from_str(&text).expect("lisible");
        assert!(card.tops.contains(&"L'\"autre\" titre".to_string()));
    }

    #[test]
    fn les_guillemets_sont_echappes() {
        assert_eq!(quoted("A \"Forest\""), "\"A \\\"Forest\\\"\"");
    }
}

/// `:mine` — what this catalog has beyond upstream.
///
/// The personal overlay is not stored, it is **computed**: in a fork
/// ([0008]), what is yours is your commits, and `git diff` renders them
/// line by line. That is what allows a single card per artist — no copy to
/// merge, no second format — while still seeing one's own layer.
pub fn mine(dir: &Path) -> Result<Vec<String>, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let out = std::process::Command::new("git")
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
    };

    // upstream first, origin as a fallback: a fork has both, a plain
    // clone only the latter
    let base = ["upstream/main", "upstream/master", "origin/main", "origin/master"]
        .into_iter()
        .find(|reference| git(&["rev-parse", "--verify", "--quiet", reference]).is_ok())
        .ok_or("no known upstream — this catalog has no origin repository")?;

    let stat = git(&["diff", "--numstat", base, "--", "cards/"])?;
    if stat.trim().is_empty() {
        return Ok(vec![format!("nothing beyond {base} — the catalog is the original one")]);
    }

    let mut lines = vec![format!("what this catalog has beyond {base}:")];
    for row in stat.lines() {
        let mut cols = row.split('\t');
        let (added, removed, path) = (cols.next(), cols.next(), cols.next());
        if let (Some(added), Some(removed), Some(path)) = (added, removed, path) {
            let name = path.rsplit('/').next().unwrap_or(path);
            lines.push(format!("  {name}  +{added} −{removed}"));
        }
    }

    // then the added lines themselves: that is what we want to reread
    // before proposing anything upstream
    let diff = git(&["diff", "-U0", base, "--", "cards/"])?;
    let added: Vec<&str> = diff
        .lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .collect();
    if !added.is_empty() {
        lines.push(String::new());
        for line in added.iter().take(24) {
            lines.push(format!(" {}", line[1..].trim()));
        }
        if added.len() > 24 {
            lines.push(format!(" … and {} more lines", added.len() - 24));
        }
    }
    Ok(lines)
}
