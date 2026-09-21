//! `forkstify validate [catalog]` — what the reference's GitHub action runs
//! on every pull request (workstream B, "Review on the reference side" in
//! `docs/design/before-release.md`): every card reads, carries `format = 1`, a
//! name, an `mbid` **unique across the catalog** — that is what catches a
//! "Ye" proposed while `kanye-west` exists —, a file named as a slug, and
//! links whose targets are slugs and whose types are the closed list of
//! [0010](../docs/decisions/0010-revised-format-links-without-doors.md).
//!
//! A link to a card that does not exist is **not** an error: it is a
//! proposal ([0016](../docs/decisions/0016-broad-base-and-on-the-fly-generation.md)).
//! A card whose name no longer matches its file ("Ye" at `kanye-west`) is
//! a warning: the file name is the key, the name may move on.
use std::collections::HashMap;
use std::path::Path;

/// The closed list of link types (0010), what `catalog.toml` may extend.
const LINK_TYPES: [&str; 6] = ["member", "collab", "similar", "family", "scene", "influence"];

#[derive(Default, Debug)]
pub struct Report {
    pub cards: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

fn is_slug(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// One card's text against the rules. `slug` is the file stem.
pub fn check_card(slug: &str, text: &str, types: &[String], report: &mut Report) -> Option<String> {
    let table: toml::Table = match toml::from_str(text) {
        Ok(table) => table,
        Err(e) => {
            report.errors.push(format!("{slug}: unreadable TOML — {}", e.message()));
            return None;
        }
    };
    if let Err(e) = toml::from_str::<crate::catalog::Card>(text) {
        report.errors.push(format!("{slug}: {}", e.message()));
        return None;
    }
    if !is_slug(slug) {
        report.errors.push(format!("{slug}: the file name is not a slug (lower case, digits, hyphens)"));
    }
    match table.get("format").and_then(|v| v.as_integer()) {
        Some(1) => {}
        Some(other) => report.errors.push(format!("{slug}: format = {other}, expected 1")),
        None => report.errors.push(format!("{slug}: no format")),
    }
    let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
    if name.is_empty() {
        report.errors.push(format!("{slug}: no name"));
    } else if crate::generate::slugify(name) != slug {
        report.warnings.push(format!("{slug}: named \"{name}\" — the file would be {}", crate::generate::slugify(name)));
    }
    let mbid = table.get("mbid").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if mbid.is_empty() {
        report.errors.push(format!("{slug}: no mbid"));
    } else if !crate::generate::is_mbid(&mbid) {
        report.errors.push(format!("{slug}: mbid \"{mbid}\" is not a MusicBrainz id"));
    }
    if let Some(links) = table.get("links").and_then(|v| v.as_array()) {
        for link in links {
            let to = link.get("to").and_then(|v| v.as_str()).unwrap_or("");
            if !is_slug(to) {
                report.errors.push(format!("{slug}: link to \"{to}\" — the target is not a slug"));
            }
            let kind = link.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if !types.iter().any(|t| t == kind) {
                report.errors.push(format!("{slug}: link type \"{kind}\" — not one of {}", types.join(", ")));
            }
        }
    }
    (!mbid.is_empty()).then_some(mbid)
}

pub fn validate(dir: &Path) -> Report {
    let mut report = Report::default();
    let mut types: Vec<String> = LINK_TYPES.iter().map(|t| t.to_string()).collect();
    match std::fs::read_to_string(dir.join("catalog.toml")) {
        Ok(text) => match toml::from_str::<toml::Table>(&text) {
            Ok(table) => {
                if let Some(proximity) = table.get("proximity").and_then(|v| v.as_table()) {
                    let declared: Vec<String> = proximity.keys().filter(|k| !types.contains(k)).cloned().collect();
                    types.extend(declared);
                }
            }
            Err(e) => report.errors.push(format!("catalog.toml: unreadable — {}", e.message())),
        },
        Err(_) => report.warnings.push("no catalog.toml — the built-in proximities apply".to_string()),
    }
    let mut files: Vec<_> = match std::fs::read_dir(dir.join("cards")) {
        Ok(entries) => entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "toml")).collect(),
        Err(_) => {
            report.errors.push("no cards/ folder".to_string());
            return report;
        }
    };
    files.sort();
    let mut seen: HashMap<String, String> = HashMap::new();
    for path in files {
        let slug = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) => {
                report.errors.push(format!("{slug}: {e}"));
                continue;
            }
        };
        report.cards += 1;
        if let Some(mbid) = check_card(&slug, &text, &types, &mut report) {
            if let Some(other) = seen.insert(mbid.clone(), slug.clone()) {
                report.errors.push(format!("{slug} and {other}: the same mbid {mbid} — one artist, two cards"));
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types() -> Vec<String> {
        LINK_TYPES.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn a_sound_card_passes() {
        let mut report = Report::default();
        let card = "format = 1\nname = \"The Cure\"\nmbid = \"69ee3720-a7cb-4402-b48d-a02c366f2bcf\"\n\nlinks = [\n  { to = \"siouxsie\", type = \"member\" },\n]\n";
        let mbid = check_card("the-cure", card, &types(), &mut report);
        assert!(report.ok(), "{:?}", report.errors);
        assert!(report.warnings.is_empty());
        assert_eq!(mbid.as_deref(), Some("69ee3720-a7cb-4402-b48d-a02c366f2bcf"));
    }

    #[test]
    fn every_rule_has_its_message() {
        let mut report = Report::default();
        let card = "format = 2\nname = \"Ye\"\nmbid = \"nope\"\n\nlinks = [\n  { to = \"Kid Cudi\", type = \"buddy\" },\n]\n";
        check_card("kanye-west", card, &types(), &mut report);
        let errors = report.errors.join("\n");
        assert!(errors.contains("format = 2"), "{errors}");
        assert!(errors.contains("not a MusicBrainz id"), "{errors}");
        assert!(errors.contains("the target is not a slug"), "{errors}");
        assert!(errors.contains("link type \"buddy\""), "{errors}");
        assert!(report.warnings.iter().any(|w| w.contains("named \"Ye\"")), "{:?}", report.warnings);
        let mut report = Report::default();
        check_card("Bad Slug", "name = \"x\"\n", &types(), &mut report);
        let errors = report.errors.join("\n");
        assert!(errors.contains("not a slug"), "{errors}");
        assert!(errors.contains("no format"), "{errors}");
        assert!(errors.contains("no mbid"), "{errors}");
        let mut report = Report::default();
        assert!(check_card("x", "name = [\n", &types(), &mut report).is_none());
        assert!(report.errors[0].contains("unreadable"));
    }

    /// The same artist twice, under two files: the error the reference's
    /// action is there to catch.
    #[test]
    fn the_same_mbid_twice_is_an_error() {
        let root = std::env::temp_dir().join(format!("forkstify-validate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("cards")).unwrap();
        std::fs::write(root.join("catalog.toml"), "[proximity]\ncover = 2\n").unwrap();
        let card = |name: &str| format!("format = 1\nname = \"{name}\"\nmbid = \"164f0d73-1234-4e2c-8743-d77bf2191051\"\nlinks = [ {{ to = \"x\", type = \"cover\" }} ]\n");
        std::fs::write(root.join("cards/kanye-west.toml"), card("Ye")).unwrap();
        std::fs::write(root.join("cards/ye.toml"), card("Ye")).unwrap();
        let report = validate(&root);
        assert_eq!(report.cards, 2);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("ye and kanye-west: the same mbid"), "{}", report.errors[0]);
        // a type declared in catalog.toml is accepted; the renamed card warns
        assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
        let _ = std::fs::remove_dir_all(&root);
    }
}
