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

/// `ae`: the card was edited by hand, in `$EDITOR`; what changed is the
/// diff, the commit only says who and how.
pub fn edited_by_hand(dir: &Path, slug: &str, name: &str) -> Edit {
    Edit { summary: format!("{name} — edited by hand"), body: None, path: card_path(dir, slug), also: Vec::new() }
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
/// It refuses to overwrite an existing card: a generation is an addition.
/// Replacing one is a separate, explicit act — `regenerate_card`.
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

/// Rewrite an existing card from the sources — `:generate <name> <mbid>`
/// over a card that exists (Joel, 23/09/2026). The whole text goes, hand
/// edits included: that is what you want when the card was born under the
/// wrong artist, and the commit is the undo when it was not. `was` names
/// who the card used to be, for the commit.
pub fn regenerate_card(
    dir: &Path,
    slug: &str,
    name: &str,
    text: &str,
    tops: usize,
    links: usize,
    was: &str,
) -> Result<Edit, String> {
    let path = card_path(dir, slug);
    if !path.exists() {
        return Err(format!("{name} has no card to regenerate"));
    }
    // the old identity, for the record: the commit says who this was
    let before = read(&path)?;
    let old_mbid = before
        .lines()
        .find_map(|line| line.strip_prefix("mbid = ").map(|v| v.trim_matches('"')))
        .unwrap_or("no mbid");
    let was = format!("{was} ({old_mbid})");
    write(&path, text)?;
    Ok(Edit {
        summary: format!("{name} — card regenerated"),
        body: Some(format!(
            "Was {was}.\n{tops} top(s), {links} link(s) — MusicBrainz and Deezer.\ngenerated = true: to review."
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
    fn an_insertion_touches_nothing_else() {
        let out = insert_into_array(CARD, "tops", "  \"Push\",");
        assert!(out.contains("format = 1"));
        assert!(out.contains("mbid = \"abc\""));
        assert!(out.contains("  \"A Forest\",\n  \"Lullaby\",\n  \"Push\",\n]"));
        // and the other arrays do not move
        assert!(out.contains("{ to = \"joy-division\", type = \"scene\" },"));
    }

    #[test]
    fn a_missing_array_is_created_at_the_end_of_the_card() {
        let out = insert_into_array(CARD, "doors", "  { track = \"A Forest\" },");
        assert!(out.trim_end().ends_with("doors = [\n  { track = \"A Forest\" },\n]"));
        assert!(out.contains("tops = ["));
    }

    #[test]
    fn the_bounds_of_the_right_array() {
        let (from, to) = array_span(CARD, "tops").expect("tops");
        assert!(CARD[from..to].contains("A Forest"));
        assert!(!CARD[from..to].contains("joy-division"));
    }

    /// The real risk of a textual patch: producing a TOML nobody can read
    /// back. After every insertion, the card must still load as a card.
    #[test]
    fn the_card_stays_readable_after_each_edit() {
        let mut text = CARD.to_string();
        text = insert_into_array(&text, "tops", "  \"Push\",");
        text = insert_into_array(
            &text,
            "doors",
            &door_line("A Forest", &["post-punk".into(), "atmospherique".into()]),
        );
        text = insert_into_array(&text, "links", "  { to = \"siouxsie\", type = \"member\" },");

        let card: crate::catalog::Card =
            toml::from_str(&text).expect("the card must still parse");
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
    fn a_tricky_title_still_goes_through() {
        let text = insert_into_array(CARD, "tops", &format!("  {},", quoted("L'\"autre\" titre")));
        let card: crate::catalog::Card = toml::from_str(&text).expect("lisible");
        assert!(card.tops.contains(&"L'\"autre\" titre".to_string()));
    }

    #[test]
    fn quotes_are_escaped() {
        assert_eq!(quoted("A \"Forest\""), "\"A \\\"Forest\\\"\"");
    }
}
