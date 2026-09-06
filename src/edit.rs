//! Les **éditions** (0013) : elles modifient une fiche et produisent un
//! commit lisible. C'est ce qui les sépare des mesures — celles-ci écrivent
//! dans `learned/` sans rien dire, celles-là laissent une trace relisible et
//! annulable, « le fork est la surcouche » ([0008]) pris au mot.
//!
//! **Les fiches sont retouchées textuellement, jamais réécrites.** Une
//! relecture par serde perdrait tout ce que le code ne modélise pas —
//! `format`, `generated`, `mbid`, `spotify`, `begin`, `origin`,
//! `description`, l'ordre des clés et les guillemets choisis à la main. Une
//! fiche est un fichier qu'un humain lit et corrige ([0002] : « le format
//! des fiches est une interface publique ») ; on y insère une ligne, on n'en
//! régénère pas le tout.

use std::path::{Path, PathBuf};

pub fn card_path(catalog_dir: &Path, slug: &str) -> PathBuf {
    catalog_dir.join("fiches").join(format!("{slug}.toml"))
}

/// Échapper une valeur pour une chaîne TOML de base.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Trouver le tableau `<clé> = [` … `]` et rendre les bornes de son contenu.
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

/// Insérer une ligne juste avant la fermeture d'un tableau. Si le tableau
/// n'existe pas, il est créé à la fin de la fiche.
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

fn door_line(title: &str, tags: &[String]) -> String {
    let list: Vec<String> = tags.iter().map(|tag| quoted(tag)).collect();
    format!(
        "  {{ track = {}, to = [{}], note = {} }},",
        quoted(title),
        list.join(", "),
        quoted("posée à l'écoute")
    )
}

fn link_line(to_slug: &str, kind: &str) -> String {
    format!(
        "  {{ to = {}, type = {}, note = {} }},",
        quoted(to_slug),
        quoted(kind),
        quoted("lié à l'écoute")
    )
}

/// Ce qu'une édition a changé, dit en une phrase — c'est le message de commit
/// et c'est aussi ce qui s'affiche à l'écran. Une seule formulation pour les
/// deux : ce que l'utilisateur lit est ce que git retiendra.
pub struct Edit {
    pub summary: String,
    pub path: PathBuf,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("fiche illisible ({e})"))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("fiche non écrite ({e})"))
}

/// `tt` — promouvoir un morceau en top.
pub fn add_top(dir: &Path, slug: &str, name: &str, title: &str) -> Result<Edit, String> {
    let path = card_path(dir, slug);
    let text = read(&path)?;
    if let Some((from, to)) = array_span(&text, "tops") {
        if text[from..to].contains(&quoted(title)) {
            return Err(format!("« {title} » est déjà un top"));
        }
    }
    let updated = insert_into_array(&text, "tops", &format!("  {},", quoted(title)));
    write(&path, &updated)?;
    Ok(Edit { summary: format!("{name} — top : + {title}"), path })
}

/// `tT` — retirer un morceau des tops.
pub fn remove_top(dir: &Path, slug: &str, name: &str, title: &str) -> Result<Edit, String> {
    let path = card_path(dir, slug);
    let text = read(&path)?;
    let needle = quoted(title);
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !(line.trim_start().starts_with(&needle) && line.trim_end().ends_with(',')))
        .collect();
    if kept.len() == text.lines().count() {
        return Err(format!("« {title} » n'est pas dans les tops"));
    }
    write(&path, &(kept.join("\n") + "\n"))?;
    Ok(Edit { summary: format!("{name} — top : − {title}"), path })
}

/// `td` — faire d'un morceau une **door** vers une direction ([0011] : `to`
/// pointe vers des tags, jamais vers un artiste).
pub fn add_door(
    dir: &Path,
    slug: &str,
    name: &str,
    title: &str,
    tags: &[String],
) -> Result<Edit, String> {
    if tags.is_empty() {
        return Err("aucune direction à donner à cette door".into());
    }
    let path = card_path(dir, slug);
    let text = read(&path)?;
    if let Some((from, to)) = array_span(&text, "doors") {
        if text[from..to].contains(&format!("track = {}", quoted(title))) {
            return Err(format!("« {title} » est déjà une door"));
        }
    }
    let updated = insert_into_array(&text, "doors", &door_line(title, tags));
    write(&path, &updated)?;
    Ok(Edit {
        summary: format!("{name} — door : {title} → {}", tags.join(", ")),
        path,
    })
}

/// `aL` — lier deux artistes ([0010] : un type fermé, une note qui explique).
pub fn add_link(
    dir: &Path,
    slug: &str,
    name: &str,
    to_slug: &str,
    to_name: &str,
    kind: &str,
) -> Result<Edit, String> {
    if slug == to_slug {
        return Err("un artiste ne se lie pas à lui-même".into());
    }
    let path = card_path(dir, slug);
    let text = read(&path)?;
    if let Some((from, to)) = array_span(&text, "links") {
        if text[from..to].contains(&format!("to = {}", quoted(to_slug))) {
            return Err(format!("{name} est déjà lié à {to_name}"));
        }
    }
    let updated = insert_into_array(&text, "links", &link_line(to_slug, kind));
    write(&path, &updated)?;
    Ok(Edit { summary: format!("{name} — lien : → {to_name} ({kind})"), path })
}

/// Le commit. Une édition qui ne laisse pas de trace relisible n'en est pas
/// une (0013) ; on ne pousse pas, en revanche — c'est l'affaire de `:sync`.
pub fn commit(dir: &Path, edit: &Edit) -> Result<(), String> {
    let relative = edit
        .path
        .strip_prefix(dir)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| edit.path.clone());
    let run = |args: &[&str]| -> Result<(), String> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map_err(|e| format!("git introuvable ({e})"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    };
    run(&["add", &relative.to_string_lossy()])?;
    run(&["commit", "-q", "-m", &edit.summary])
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = "format = 1\nname = \"The Cure\"\nmbid = \"abc\"\n\ntags = [\"post-punk\"]\n\ntops = [\n  \"A Forest\",\n  \"Lullaby\",\n]\n\nlinks = [\n  { to = \"joy-division\", type = \"scene\" },\n]\n";

    /// Le reste de la fiche doit survivre intact : c'est une interface
    /// publique, pas une structure de données à nous.
    #[test]
    fn une_insertion_ne_touche_a_rien_d_autre() {
        let out = insert_into_array(CARD, "tops", "  \"Push\",");
        assert!(out.contains("format = 1"));
        assert!(out.contains("mbid = \"abc\""));
        assert!(out.contains("  \"A Forest\",\n  \"Lullaby\",\n  \"Push\",\n]"));
        // et les autres tableaux ne bougent pas
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

    /// Le vrai risque d'une retouche textuelle : produire un TOML que plus
    /// personne ne relit. Après chaque insertion, la fiche doit encore se
    /// charger comme une fiche.
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

    /// Un titre à guillemets ou à apostrophe ne doit pas casser la fiche.
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
