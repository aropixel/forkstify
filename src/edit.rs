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
    catalog_dir.join("cards").join(format!("{slug}.toml"))
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

/// La note dit **d'où vient la ligne**, et depuis quand. 0010 veut qu'une
/// note explique le lien ; une ligne écrite pendant une écoute ne peut pas
/// prétendre à une explication savante, mais elle peut dire son origine —
/// ce qui permet à qui relit, plus tard ou en amont, de la peser.
fn provenance(what: &str) -> String {
    quoted(&format!("{what} à l'écoute, {}", crate::learned::today_iso()))
}

fn door_line(title: &str, tags: &[String]) -> String {
    let list: Vec<String> = tags.iter().map(|tag| quoted(tag)).collect();
    format!(
        "  {{ track = {}, to = [{}], note = {} }},",
        quoted(title),
        list.join(", "),
        provenance("posée")
    )
}

fn link_line(to_slug: &str, kind: &str) -> String {
    format!(
        "  {{ to = {}, type = {}, note = {} }},",
        quoted(to_slug),
        quoted(kind),
        provenance("rapproché")
    )
}

/// Ce qu'une édition a changé, dit en une phrase — c'est le message de commit
/// et c'est aussi ce qui s'affiche à l'écran. Une seule formulation pour les
/// deux : ce que l'utilisateur lit est ce que git retiendra.
pub struct Edit {
    pub summary: String,
    /// Le détail, quand une édition en porte plusieurs : le corps du commit
    /// dit ce que le sujet compte. Une fournée de tops en a un, un geste
    /// isolé n'en a pas besoin.
    pub body: Option<String>,
    pub path: PathBuf,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("fiche illisible ({e})"))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("fiche non écrite ({e})"))
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
        body: None,
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
    Ok(Edit { summary: format!("{name} — lien : → {to_name} ({kind})"), body: None, path })
}

/// La **fournée** de l'écran de la discographie (maquette 1a, 07/09/2026) :
/// on y corrige cinq tops d'affilée, et cinq commits pour une seule pensée
/// ne se relisent pas. Une lecture, une écriture, un commit — et le message
/// dit le compte, le corps dit les titres.
///
/// Les retraits passent avant les ajouts : promouvoir puis retirer le même
/// titre dans la même fournée doit le laisser dehors, pas dedans.
pub fn set_tops(
    dir: &Path,
    slug: &str,
    name: &str,
    adds: &[String],
    removes: &[String],
) -> Result<Edit, String> {
    if adds.is_empty() && removes.is_empty() {
        return Err("rien à écrire".into());
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
        return Err("les tops de la fiche disaient déjà cela".into());
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
        summary: format!("{name} — tops : +{} −{}", added.len(), removed.len()),
        body: Some(lines.join("\n")),
        path,
    })
}

/// La **génération d'une fiche** ([0016]) : la seule édition qui *crée* un
/// fichier au lieu d'en retoucher un. Le texte vient de `generate`, qui le
/// compose comme les 316 fiches de la référence — on ne le sérialise pas ici
/// pour la même raison que rien n'est réécrit ailleurs.
///
/// Elle refuse d'écraser une fiche existante : une génération est un ajout,
/// jamais un remplacement. Ce qui existe se corrige à la main ou par les
/// autres éditions.
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
        return Err(format!("{name} a déjà une fiche"));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("pas de dossier cards/ ({e})"))?;
    }
    write(&path, text)?;
    Ok(Edit {
        summary: format!("{name} — fiche générée"),
        body: Some(format!(
            "{tops} top(s), {links} lien(s) — MusicBrainz et Deezer.\ngenerated = true : à relire."
        )),
        path,
    })
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

/// `:mine` — ce que ce catalogue a de plus que l'amont.
///
/// La surcouche personnelle n'est pas stockée, elle se **calcule** : dans un
/// fork ([0008]), ce qui est à soi ce sont ses commits, et `git diff` les
/// rend ligne par ligne. C'est ce qui permet de garder une seule fiche par
/// artiste — pas de copie à fusionner, pas de second format — tout en
/// voyant sa propre couche.
pub fn mine(dir: &Path) -> Result<Vec<String>, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let out = std::process::Command::new("git")
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
    };

    // l'amont d'abord, l'origine à défaut : un fork a les deux, un clone
    // simple n'a que la seconde
    let base = ["upstream/main", "upstream/master", "origin/main", "origin/master"]
        .into_iter()
        .find(|reference| git(&["rev-parse", "--verify", "--quiet", reference]).is_ok())
        .ok_or("aucun amont connu — ce catalogue n'a pas de dépôt d'origine")?;

    let stat = git(&["diff", "--numstat", base, "--", "cards/"])?;
    if stat.trim().is_empty() {
        return Ok(vec![format!("rien de plus que {base} — le catalogue est celui d'origine")]);
    }

    let mut lines = vec![format!("ce que ce catalogue a de plus que {base} :")];
    for row in stat.lines() {
        let mut cols = row.split('\t');
        let (added, removed, path) = (cols.next(), cols.next(), cols.next());
        if let (Some(added), Some(removed), Some(path)) = (added, removed, path) {
            let name = path.rsplit('/').next().unwrap_or(path);
            lines.push(format!("  {name}  +{added} −{removed}"));
        }
    }

    // puis les lignes ajoutées elles-mêmes : c'est ce qu'on veut relire avant
    // de proposer quoi que ce soit en amont
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
            lines.push(format!(" … et {} lignes de plus", added.len() - 24));
        }
    }
    Ok(lines)
}
