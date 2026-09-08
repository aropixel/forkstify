//! L'écran de la **discographie** (`ad`, `:discography`) — maquette 1a de
//! `Discographie.dc.html`, arbitrée par Joel le 07/09/2026.
//!
//! Il naît d'un constat : les éditions (0013) ne s'exercent que sur le
//! morceau qui sonne, si bien que redresser les tops d'un artiste
//! demanderait de le poncer en entier. Ici on voit tout d'un coup — et
//! surtout **quel album porte les écoutes**, qui est la vraie question
//! quand on n'aime qu'un disque sur douze.
//!
//! Trois choix de forme, qui expliquent le code :
//!
//! * **Les albums sont pliés.** Cent quatre-vingt-sept titres deviennent
//!   douze lignes, et seul l'album sous le curseur s'ouvre. Une
//!   discographie ne se lit pas, elle se survole.
//! * **Les éditions s'accumulent et partent en un seul commit** (⏎). On en
//!   fait cinq d'affilée : cinq commits pour une seule pensée ne se
//!   relisent pas. C'est l'inverse d'une mesure (`tl`, `tb`), qui est
//!   silencieuse, immédiate, et que 0017 balaie toute seule.
//! * **Rien n'est réécrit ici** : l'écran prépare, `edit::set_tops` écrit.
//!   Le module ne connaît ni le disque ni git.

use crate::catalog::Card;
use crate::discography::{clean_title, normalize, TailTrack};
use crate::learned::Learned;

/// L'ordre des albums. Le chronologique est celui de la mémoire ; l'autre
/// répond à « où vont mes écoutes », qui est la question qu'on vient poser.
#[derive(Clone, Copy, PartialEq)]
pub enum Sort {
    Chrono,
    Plays,
}

/// Le filtre de provenance — une touche, pas un menu : corriger les tops se
/// fait sur neuf lignes, pas sur cent quatre-vingt-sept.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Tops,
    Liked,
    Banned,
}

impl Filter {
    pub fn next(self) -> Filter {
        match self {
            Filter::All => Filter::Tops,
            Filter::Tops => Filter::Liked,
            Filter::Liked => Filter::Banned,
            Filter::Banned => Filter::All,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Filter::All => "tout",
            Filter::Tops => "♪ tops",
            Filter::Liked => "♥ aimés",
            Filter::Banned => "⊘ bannis",
        }
    }
}

pub struct Track {
    /// Le titre lisible — celui de Spotify, ou celui de la fiche pour un top
    /// que la discographie ne rend pas.
    pub title: String,
    /// La chaîne **exacte** de la fiche quand ce morceau y est un top. Elle
    /// n'est pas toujours celle de Spotify, et c'est elle qu'un retrait doit
    /// viser — sinon `remove_top` ne trouve rien alors que le ♪ s'affiche.
    pub card_top: Option<String>,
    pub number: u32,
    pub duration_ms: u32,
    pub liked: bool,
    pub banned: bool,
    pub plays: f64,
    pub days: Option<i64>,
}

impl Track {
    pub fn is_top(&self) -> bool {
        self.card_top.is_some()
    }
}

pub struct Album {
    pub title: String,
    pub year: Option<u16>,
    pub single: bool,
    /// Un top de la fiche que la discographie ne contient pas : coquille,
    /// live, titre de compilation. C'est la moitié « audit » de l'écran.
    pub orphan: bool,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn plays(&self) -> f64 {
        self.tracks.iter().map(|t| t.plays).sum()
    }

    pub fn tops(&self) -> usize {
        self.tracks.iter().filter(|t| t.is_top()).count()
    }

    pub fn liked(&self) -> usize {
        self.tracks.iter().filter(|t| t.liked).count()
    }

    pub fn banned(&self) -> usize {
        self.tracks.iter().filter(|t| t.banned).count()
    }
}

/// Une édition préparée, pas encore écrite.
pub struct Pending {
    pub add: bool,
    /// La chaîne à écrire dans la fiche, ou à en retirer.
    pub title: String,
    /// Ce qui la justifie, en trois mots — elle s'affiche à côté.
    pub why: String,
}

/// Où l'on est. Une **identité**, pas un numéro de ligne : plier un album
/// change le nombre de lignes au-dessus, et un index y survivrait mal.
#[derive(Clone, Copy, PartialEq)]
pub struct Cursor {
    pub album: usize,
    pub track: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Row {
    Album(usize),
    Track(usize, usize),
}

pub struct Summary {
    pub albums: usize,
    pub titles: usize,
    pub tops: usize,
    pub liked: usize,
    pub banned: usize,
    pub tail: usize,
    pub plays: f64,
    /// Combien d'albums portent la majorité des écoutes, et quelle part.
    pub carrying: usize,
    pub carrying_pct: u32,
    pub never: usize,
}

pub struct Explore {
    pub slug: String,
    pub name: String,
    pub generated: bool,
    /// La matière première, gardée pour pouvoir tout reconstruire après une
    /// écriture ou une mesure : le catalogue en mémoire, lui, ne bouge pas.
    raw: Vec<TailTrack>,
    tops: Vec<String>,
    stats: Vec<(String, f64, Option<i64>, bool, bool)>,
    playing: Option<String>,
    pub albums: Vec<Album>,
    pub sort: Sort,
    pub filter: Filter,
    pub query: String,
    pub cursor: Cursor,
    /// L'album sous le curseur est ouvert, sauf si l'on a tout replié (`h`).
    pub folded: bool,
    pub pending: Vec<Pending>,
    /// Ce que l'écran vient de dire — la ligne de notice, en bas.
    pub notice: String,
    /// Échap une première fois avec des éditions en attente ne ferme pas :
    /// il prévient. C'est le garde-fou que 1a se doit d'avoir.
    pub confirm_close: bool,
    /// La discographie arrive : l'écran s'est ouvert sur les tops, le reste
    /// se charge derrière (Joel, 08/09/2026).
    pub loading: bool,
}

impl Explore {
    pub fn open(
        slug: &str,
        card: &Card,
        tail: &[TailTrack],
        learned: &Learned,
        playing: Option<&str>,
    ) -> Explore {
        let mut screen = Explore {
            slug: slug.to_string(),
            name: card.name.clone(),
            generated: card.generated,
            raw: tail.to_vec(),
            tops: card.tops.clone(),
            stats: learned.track_table(slug),
            playing: playing.map(normalize),
            albums: Vec::new(),
            sort: Sort::Chrono,
            filter: Filter::All,
            query: String::new(),
            cursor: Cursor { album: 0, track: None },
            folded: false,
            pending: Vec::new(),
            notice: String::new(),
            confirm_close: false,
            loading: false,
        };
        screen.build();
        screen
    }

    /// La discographie est arrivée : on reconstruit sur la nouvelle matière
    /// sans perdre ce que l'utilisateur a déjà fait ici (tri, filtre,
    /// éditions en attente, curseur — borné par `build`).
    pub fn reload(&mut self, tail: &[TailTrack], learned: &Learned) {
        self.raw = tail.to_vec();
        self.stats = learned.track_table(&self.slug);
        self.loading = false;
        self.notice = format!("✓ discographie chargée — {} titres", tail.len());
        self.build();
    }

    /// Reconstruire les albums depuis la matière première. Appelé à
    /// l'ouverture, après une mesure, et après une écriture.
    fn build(&mut self) {
        // un morceau par titre normalisé : Spotify livre la même chanson
        // cinq fois (album, single, réédition). On garde la plus ancienne
        // occurrence d'album — c'est celle dont on se souvient.
        let mut kept: Vec<&TailTrack> = Vec::new();
        for track in &self.raw {
            let key = normalize(&track.title);
            match kept.iter().position(|k| normalize(&k.title) == key) {
                None => kept.push(track),
                Some(index) => {
                    // un album plutôt qu'un single, et la plus ancienne
                    // parution : c'est la version dont on se souvient
                    let rank = |t: &TailTrack| (t.single, t.year().unwrap_or(u16::MAX));
                    if rank(track) < rank(kept[index]) {
                        kept[index] = track;
                    }
                }
            }
        }

        let stat = |title: &str, stats: &[(String, f64, Option<i64>, bool, bool)]| {
            let key = normalize(title);
            stats
                .iter()
                .find(|(name, ..)| normalize(name) == key)
                .map(|(_, plays, days, liked, banned)| (*plays, *days, *liked, *banned))
                .unwrap_or((0.0, None, false, false))
        };

        let mut albums: Vec<Album> = Vec::new();
        let mut claimed: Vec<String> = Vec::new();
        for track in kept {
            let key = normalize(&track.title);
            let card_top = self
                .tops
                .iter()
                .find(|top| normalize(top) == key)
                .map(|top| {
                    claimed.push(top.clone());
                    top.clone()
                });
            let (plays, days, liked, banned) = stat(&track.title, &self.stats);
            let row = Track {
                title: track.title.clone(),
                card_top,
                number: track.number,
                duration_ms: track.duration_ms,
                liked,
                banned,
                plays,
                days,
            };
            match albums.iter_mut().find(|a| a.title == track.album) {
                Some(album) => album.tracks.push(row),
                None => albums.push(Album {
                    title: track.album.clone(),
                    year: track.year(),
                    single: track.single,
                    orphan: false,
                    tracks: vec![row],
                }),
            }
        }

        // les tops que la discographie ne rend pas : ils tombent en fin de
        // liste plutôt que de disparaître — c'est là qu'on voit les coquilles
        let orphans: Vec<&String> =
            self.tops.iter().filter(|top| !claimed.contains(top)).collect();
        if !orphans.is_empty() {
            let tracks = orphans
                .into_iter()
                .map(|top| {
                    let (plays, days, liked, banned) = stat(top, &self.stats);
                    Track {
                        title: top.clone(),
                        card_top: Some(top.clone()),
                        number: 0,
                        duration_ms: 0,
                        liked,
                        banned,
                        plays,
                        days,
                    }
                })
                .collect();
            albums.push(Album {
                title: "tops hors discographie".into(),
                year: None,
                single: false,
                orphan: true,
                tracks,
            });
        }

        self.albums = albums;
        self.clamp();
    }

    /// Reprendre ce que l'appris sait, après une mesure faite ici.
    pub fn refresh(&mut self, learned: &Learned) {
        self.stats = learned.track_table(&self.slug);
        self.build();
    }

    /// Ce qui sonne a changé.
    pub fn now_playing(&mut self, title: Option<&str>) {
        self.playing = title.map(normalize);
    }

    pub fn is_playing(&self, track: &Track) -> bool {
        self.playing.as_deref() == Some(normalize(&track.title).as_str())
    }

    // --- ce qui se voit -----------------------------------------------------

    fn visible_track(&self, track: &Track) -> bool {
        let passes = match self.filter {
            Filter::All => true,
            Filter::Tops => track.is_top(),
            Filter::Liked => track.liked,
            Filter::Banned => track.banned,
        };
        passes && (self.query.is_empty() || contains(&track.title, &self.query))
    }

    fn visible_album(&self, index: usize) -> bool {
        let album = &self.albums[index];
        // un album dont le nom répond à la recherche s'ouvre en entier
        if !self.query.is_empty() && contains(&album.title, &self.query) {
            return album.tracks.iter().any(|t| match self.filter {
                Filter::All => true,
                Filter::Tops => t.is_top(),
                Filter::Liked => t.liked,
                Filter::Banned => t.banned,
            });
        }
        album.tracks.iter().any(|t| self.visible_track(t))
    }

    /// Les indices des morceaux affichés d'un album, dans l'ordre courant.
    pub fn tracks_of(&self, index: usize) -> Vec<usize> {
        let album = &self.albums[index];
        let named = !self.query.is_empty() && contains(&album.title, &self.query);
        let mut rows: Vec<usize> = (0..album.tracks.len())
            .filter(|i| {
                let track = &album.tracks[*i];
                if named {
                    match self.filter {
                        Filter::All => true,
                        Filter::Tops => track.is_top(),
                        Filter::Liked => track.liked,
                        Filter::Banned => track.banned,
                    }
                } else {
                    self.visible_track(track)
                }
            })
            .collect();
        match self.sort {
            Sort::Chrono => rows.sort_by_key(|i| (album.tracks[*i].number, i.to_owned())),
            Sort::Plays => rows.sort_by(|a, b| {
                album.tracks[*b]
                    .plays
                    .total_cmp(&album.tracks[*a].plays)
                    .then(album.tracks[*a].number.cmp(&album.tracks[*b].number))
            }),
        }
        rows
    }

    /// L'ordre des albums : chronologique, ou par écoutes. Les singles et
    /// les orphelins ferment la marche dans les deux cas.
    pub fn order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.albums.len()).filter(|i| self.visible_album(*i)).collect();
        match self.sort {
            Sort::Chrono => order.sort_by_key(|i| {
                let album = &self.albums[*i];
                (album.orphan, album.single, album.year.unwrap_or(u16::MAX), album.title.clone())
            }),
            Sort::Plays => order.sort_by(|a, b| {
                let (x, y) = (&self.albums[*a], &self.albums[*b]);
                (x.orphan, x.single)
                    .cmp(&(y.orphan, y.single))
                    .then(y.plays().total_cmp(&x.plays()))
                    .then(x.year.cmp(&y.year))
            }),
        }
        order
    }

    /// Les lignes de l'écran, dans l'ordre : les albums, et les morceaux de
    /// celui qui est ouvert.
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        for index in self.order() {
            rows.push(Row::Album(index));
            if !self.folded && self.cursor.album == index {
                for track in self.tracks_of(index) {
                    rows.push(Row::Track(index, track));
                }
            }
        }
        rows
    }

    pub fn at(&self, row: Row) -> bool {
        match row {
            Row::Album(a) => self.cursor.album == a && self.cursor.track.is_none(),
            Row::Track(a, t) => self.cursor.album == a && self.cursor.track == Some(t),
        }
    }

    fn position(&self) -> usize {
        self.rows().iter().position(|row| self.at(*row)).unwrap_or(0)
    }

    fn adopt(&mut self, row: Row) {
        self.cursor = match row {
            Row::Album(a) => Cursor { album: a, track: None },
            Row::Track(a, t) => Cursor { album: a, track: Some(t) },
        };
    }

    /// Le curseur reste sur quelque chose qui existe — après un filtre, un
    /// tri, une reconstruction.
    fn clamp(&mut self) {
        let rows = self.rows();
        if rows.is_empty() {
            self.cursor = Cursor { album: 0, track: None };
            return;
        }
        if !rows.iter().any(|row| self.at(*row)) {
            let fallback = self
                .order()
                .first()
                .copied()
                .unwrap_or(0);
            self.cursor = Cursor { album: fallback, track: None };
        }
    }

    pub fn move_by(&mut self, step: isize) {
        let rows = self.rows();
        if rows.is_empty() {
            return;
        }
        let here = self.position() as isize;
        let next = (here + step).clamp(0, rows.len() as isize - 1) as usize;
        self.adopt(rows[next]);
        // descendre sur un album l'ouvre : c'est ce qui fait qu'on parcourt
        // douze lignes au lieu de deux cents
        self.clamp();
    }

    pub fn go_top(&mut self) {
        if let Some(first) = self.rows().first() {
            self.adopt(*first);
        }
    }

    pub fn go_bottom(&mut self) {
        if let Some(last) = self.rows().last() {
            self.adopt(*last);
        }
    }

    /// `h` replie tout, `l` rouvre l'album sous le curseur.
    pub fn fold(&mut self) {
        self.folded = true;
        self.cursor.track = None;
    }

    pub fn unfold(&mut self) {
        self.folded = false;
    }

    pub fn toggle_sort(&mut self) {
        self.sort = match self.sort {
            Sort::Chrono => Sort::Plays,
            Sort::Plays => Sort::Chrono,
        };
        self.notice = match self.sort {
            Sort::Chrono => "ordre : chronologique".into(),
            Sort::Plays => "ordre : mes écoutes d'abord".into(),
        };
        self.clamp();
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.notice = format!("filtre : {}", self.filter.word());
        self.clamp();
    }

    pub fn search(&mut self, query: &str) {
        self.query = query.trim().to_string();
        self.notice = if self.query.is_empty() {
            "filtre levé".into()
        } else {
            format!("filtre : « {} »", self.query)
        };
        self.clamp();
    }

    pub fn sort_word(&self) -> &'static str {
        match self.sort {
            Sort::Chrono => "chronologique",
            Sort::Plays => "mes écoutes d'abord",
        }
    }

    // --- ce qu'on prépare ---------------------------------------------------

    pub fn track(&self) -> Option<&Track> {
        let album = self.albums.get(self.cursor.album)?;
        album.tracks.get(self.cursor.track?)
    }

    fn why(track: &Track) -> String {
        let mut said = Vec::new();
        if track.plays >= 0.5 {
            said.push(format!("{:.0} écoutes", track.plays));
        }
        if track.liked {
            said.push("aimé".into());
        }
        if track.banned {
            said.push("banni".into());
        }
        if said.is_empty() {
            "jamais écouté".into()
        } else {
            said.join(", ")
        }
    }

    /// Une édition déjà préparée sur ce titre — pour ne pas en empiler deux.
    fn pending_on(&self, title: &str) -> Option<usize> {
        let key = normalize(title);
        self.pending.iter().position(|p| normalize(&p.title) == key)
    }

    /// `A` — promouvoir l'album : ses titres les plus écoutés qui ne sont pas
    /// encore des tops. C'est le grain du problème (« je n'aime que cet
    /// album »), et le geste qui n'existe nulle part ailleurs.
    pub fn top_album(&mut self, most: usize) {
        let Some(album) = self.albums.get(self.cursor.album) else { return };
        let name = album.title.clone();
        let mut candidates: Vec<(String, f64, String)> = album
            .tracks
            .iter()
            .filter(|track| !track.is_top() && !track.banned && track.plays >= 0.5)
            .map(|track| (track.title.clone(), track.plays, Self::why(track)))
            .collect();
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        candidates.truncate(most);
        if candidates.is_empty() {
            self.notice = format!("(rien à promouvoir dans {name} — aucun titre écouté hors des tops)");
            return;
        }
        let mut written = 0;
        for (title, _, why) in candidates {
            if self.pending_on(&title).is_some() {
                continue;
            }
            self.pending.push(Pending { add: true, title: clean_title(&title), why });
            written += 1;
        }
        self.notice = format!("♪+ {written} titre(s) de {name} — en attente");
    }

    /// `u` — défaire la dernière édition préparée. Ici l'annulation est
    /// gratuite : rien n'est écrit tant qu'on n'a pas validé.
    pub fn undo(&mut self) {
        match self.pending.pop() {
            Some(edit) => {
                self.notice =
                    format!("↺ {} {} — annulé", if edit.add { "♪+" } else { "♪−" }, edit.title)
            }
            None => self.notice = "(rien à annuler)".into(),
        }
    }

    pub fn adds(&self) -> Vec<String> {
        self.pending.iter().filter(|p| p.add).map(|p| p.title.clone()).collect()
    }

    pub fn removes(&self) -> Vec<String> {
        self.pending.iter().filter(|p| !p.add).map(|p| p.title.clone()).collect()
    }

    /// Ce que la fournée deviendra dans la fiche — affiché avant d'appuyer,
    /// et c'est le sujet du commit.
    pub fn commit_line(&self) -> String {
        format!("{} — tops : +{} −{}", self.name, self.adds().len(), self.removes().len())
    }

    /// Les éditions ont été écrites : l'écran adopte les tops qu'il vient de
    /// poser, sans relire le catalogue — celui-ci ne rechargera qu'au
    /// prochain lancement, et l'écran, lui, doit dire vrai tout de suite.
    pub fn written(&mut self) {
        for edit in &self.pending {
            let key = normalize(&edit.title);
            if edit.add {
                if !self.tops.iter().any(|top| normalize(top) == key) {
                    self.tops.push(edit.title.clone());
                }
            } else {
                self.tops.retain(|top| normalize(top) != key);
            }
        }
        self.pending.clear();
        self.confirm_close = false;
        self.build();
    }

    // --- l'en-tête ----------------------------------------------------------

    pub fn summary(&self) -> Summary {
        let tracks: Vec<&Track> = self.albums.iter().flat_map(|a| a.tracks.iter()).collect();
        let plays: f64 = tracks.iter().map(|t| t.plays).sum();
        let tops = tracks.iter().filter(|t| t.is_top()).count();
        let liked = tracks.iter().filter(|t| t.liked && !t.is_top()).count();
        let banned = tracks.iter().filter(|t| t.banned).count();

        let mut shares: Vec<f64> = self.albums.iter().map(Album::plays).collect();
        shares.sort_by(|a, b| b.total_cmp(a));
        let (mut carrying, mut carried) = (0, 0.0);
        for share in &shares {
            if plays <= 0.0 || carried / plays >= 0.75 {
                break;
            }
            carried += share;
            carrying += 1;
        }
        Summary {
            albums: self.albums.iter().filter(|a| !a.orphan).count(),
            titles: tracks.len(),
            tops,
            liked,
            banned,
            tail: tracks.len().saturating_sub(tops + liked + banned),
            plays,
            carrying,
            carrying_pct: if plays > 0.0 { (carried / plays * 100.0).round() as u32 } else { 0 },
            never: self.albums.iter().filter(|a| !a.orphan && a.plays() < 0.5).count(),
        }
    }
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Card;

    fn tail() -> Vec<TailTrack> {
        let track = |title: &str, album: &str, year: &str, number: u32, single: bool| TailTrack {
            title: title.into(),
            uri: format!("spotify:track:{title}"),
            album: album.into(),
            released: year.into(),
            number,
            duration_ms: 200_000,
            single,
        };
        vec![
            track("Cross Bones Style", "Moon Pix", "1998-09-22", 1, false),
            track("Metal Heart", "Moon Pix", "1998-09-22", 2, false),
            track("Colors and the Kids", "Moon Pix", "1998-09-22", 4, false),
            // la même chanson, en single et remasterisée : elle ne doit pas
            // faire deux lignes
            track("Metal Heart - 2015 Remaster", "Metal Heart", "2015", 1, true),
            track("Sea Of Love", "The Covers Record", "2000", 1, false),
        ]
    }

    fn card() -> Card {
        let mut card: Card = toml::from_str(
            "format = 1\nname = \"Cat Power\"\nmbid = \"x\"\ntops = [\"Cross Bones Style\", \"Song to Bobby\"]\n",
        )
        .expect("fiche");
        card.generated = false;
        card
    }

    fn screen() -> Explore {
        Explore::open("cat-power", &card(), &tail(), &Learned::blank(), Some("Metal Heart"))
    }

    #[test]
    fn une_chanson_ne_parait_qu_une_fois() {
        let screen = screen();
        let titles: Vec<&str> =
            screen.albums.iter().flat_map(|a| a.tracks.iter()).map(|t| t.title.as_str()).collect();
        assert_eq!(titles.iter().filter(|t| t.starts_with("Metal Heart")).count(), 1);
        // et c'est la version d'album qui est gardée, pas le single
        assert!(titles.contains(&"Metal Heart"));
    }

    #[test]
    fn les_albums_sont_dans_l_ordre_et_les_tops_marques() {
        let screen = screen();
        let order = screen.order();
        assert_eq!(screen.albums[order[0]].title, "Moon Pix");
        assert_eq!(screen.albums[order[1]].title, "The Covers Record");
        let moon = &screen.albums[order[0]];
        assert!(moon.tracks.iter().find(|t| t.title == "Cross Bones Style").unwrap().is_top());
        assert!(!moon.tracks.iter().find(|t| t.title == "Metal Heart").unwrap().is_top());
    }

    /// Un top de la fiche que Spotify ne rend pas ne disparaît pas : c'est
    /// exactement ce qu'on vient chercher ici.
    #[test]
    fn un_top_hors_discographie_tombe_en_fin_de_liste() {
        let screen = screen();
        let last = screen.albums.last().expect("un album");
        assert!(last.orphan);
        assert_eq!(last.tracks[0].title, "Song to Bobby");
    }

    #[test]
    fn ce_qui_est_ecrit_devient_vrai_a_l_ecran() {
        // « A » ne promeut que ce qui a été écouté : une écoute de Metal Heart
        let mut learned = Learned::blank();
        learned.played("cat-power", "Metal Heart");
        let mut screen = Explore::open("cat-power", &card(), &tail(), &learned, Some("Metal Heart"));
        // « A » sur Moon Pix, un seul titre : le plus écouté hors tops
        screen.cursor = Cursor { album: 0, track: Some(1) };
        screen.top_album(1);
        assert_eq!(screen.adds(), vec!["Metal Heart".to_string()]);
        assert_eq!(screen.commit_line(), "Cat Power — tops : +1 −0");
        screen.undo();
        assert!(screen.pending.is_empty(), "u défait, gratuitement");
        screen.top_album(1);
        screen.written();
        assert!(screen.pending.is_empty());
        let moon = &screen.albums[0];
        assert!(moon.tracks.iter().find(|t| t.title == "Metal Heart").unwrap().is_top());
    }

    #[test]
    fn le_filtre_ne_garde_que_les_tops() {
        let mut screen = screen();
        screen.cycle_filter();
        assert_eq!(screen.filter, Filter::Tops);
        let rows = screen.rows();
        let tracks: Vec<Row> = rows.iter().filter(|r| matches!(r, Row::Track(..))).copied().collect();
        for row in tracks {
            let Row::Track(a, t) = row else { unreachable!() };
            assert!(screen.albums[a].tracks[t].is_top());
        }
    }

    /// Le curseur est une identité : replier ne doit pas le perdre.
    #[test]
    fn le_curseur_survit_au_pliage() {
        let mut screen = screen();
        screen.move_by(1);
        assert_eq!(screen.cursor.track, Some(0));
        screen.fold();
        assert_eq!(screen.cursor.track, None);
        assert_eq!(screen.rows().len(), screen.order().len(), "tout est plié");
        screen.unfold();
        assert!(screen.rows().len() > screen.order().len());
    }

    #[test]
    fn l_ecran_dit_ou_vont_les_ecoutes() {
        let screen = screen();
        let summary = screen.summary();
        assert_eq!(summary.albums, 2);
        assert_eq!(summary.tops, 2, "un top d'album et un orphelin");
        assert_eq!(summary.titles, 5, "quatre titres et le top orphelin");
    }
}
