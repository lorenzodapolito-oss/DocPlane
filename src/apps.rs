//! The three programs of the suite: how to recognise their files, where to find them and how to
//! start them.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Word,
    Sheet,
    Slides,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Word, Kind::Sheet, Kind::Slides];

    /// The product name.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Word => "WordBook.it",
            Kind::Sheet => "SheetCalc.it",
            Kind::Slides => "Flickdeck.it",
        }
    }

    /// What the program is for, as written under its name.
    pub fn role(self) -> &'static str {
        match self {
            Kind::Word => "Videoscrittura",
            Kind::Sheet => "Foglio di calcolo",
            Kind::Slides => "Presentazione",
        }
    }

    /// The file types the program opens (lower case, no dot).
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Kind::Word => &["docx", "dotx", "odt", "rtf", "txt", "md", "html", "htm"],
            Kind::Sheet => &["xlsx", "xlsm", "xlsb", "ods", "csv", "tsv"],
            Kind::Slides => &["pptx", "ppsx", "potx", "odp"],
        }
    }

    /// The program for a file, by its extension.
    pub fn for_path(path: &Path) -> Option<Kind> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Kind::ALL
            .into_iter()
            .find(|k| k.extensions().contains(&ext.as_str()))
    }

    /// Names the program file may have, most specific first.
    fn exe_names(self) -> &'static [&'static str] {
        match (self, cfg!(windows)) {
            (Kind::Word, true) => &["WordBook.it.exe", "wordbook.exe"],
            (Kind::Word, false) => &["WordBook.it", "wordbook"],
            (Kind::Sheet, true) => &["SheetCalc.it.exe", "gridcraft.exe"],
            (Kind::Sheet, false) => &["SheetCalc.it", "gridcraft"],
            (Kind::Slides, true) => &["Flickdeck.it.exe", "Flickdeck.exe", "flickdeck.exe"],
            (Kind::Slides, false) => &["Flickdeck.it", "Flickdeck", "flickdeck"],
        }
    }

    /// Folders next to DocPlane that may hold the program instead of DocPlane's own folder.
    fn folders(self) -> Vec<&'static str> {
        match self {
            Kind::Slides => vec!["Flickdeck.it", "Flickdeck"],
            _ => vec![self.name()],
        }
    }
}

/// The program file of `kind` in `dir`, or in a folder named after it, or in `dir/programmi`.
pub fn find_exe(kind: Kind, dir: &Path) -> Option<PathBuf> {
    let mut places = vec![dir.to_path_buf()];
    for f in kind.folders() {
        places.push(dir.join(f));
    }
    places.push(dir.join("programmi"));
    for place in places {
        for name in kind.exe_names() {
            let candidate = place.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Start `exe`, optionally with a file to open. The program runs on its own: closing DocPlane
/// does not close it. `dictionaries` is the shared dictionaries folder, handed over so the program
/// finds it wherever it is installed.
pub fn launch(exe: &Path, file: Option<&Path>, dictionaries: Option<&Path>) -> Result<(), String> {
    let mut cmd = Command::new(exe);
    if let Some(d) = dictionaries {
        cmd.env("DOCPLANE_DICTIONARIES", d);
    }
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    if let Some(f) = file {
        cmd.arg(f);
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("Impossibile avviare {}: {e}", exe.display()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("docplane-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn files_go_to_the_right_program() {
        assert_eq!(
            Kind::for_path(Path::new("a/Lettera.DOCX")),
            Some(Kind::Word)
        );
        assert_eq!(Kind::for_path(Path::new("conti.xlsx")), Some(Kind::Sheet));
        assert_eq!(Kind::for_path(Path::new("dati.csv")), Some(Kind::Sheet));
        assert_eq!(Kind::for_path(Path::new("idee.pptx")), Some(Kind::Slides));
        assert_eq!(Kind::for_path(Path::new("foto.png")), None);
        assert_eq!(Kind::for_path(Path::new("senza_estensione")), None);
    }

    #[test]
    fn no_extension_belongs_to_two_programs() {
        for a in Kind::ALL {
            for b in Kind::ALL {
                if a != b {
                    assert!(a.extensions().iter().all(|e| !b.extensions().contains(e)));
                }
            }
        }
    }

    #[test]
    fn programs_are_found_beside_docplane_or_in_a_named_folder() {
        let dir = temp_dir("find");
        assert_eq!(find_exe(Kind::Word, &dir), None);
        let name = Kind::Word.exe_names()[0];
        std::fs::write(dir.join(name), b"").unwrap();
        assert_eq!(find_exe(Kind::Word, &dir), Some(dir.join(name)));
        let sub = dir.join("Flickdeck.it");
        std::fs::create_dir_all(&sub).unwrap();
        let slides = Kind::Slides.exe_names()[0];
        std::fs::write(sub.join(slides), b"").unwrap();
        assert_eq!(find_exe(Kind::Slides, &dir), Some(sub.join(slides)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_program_is_an_error_not_a_panic() {
        assert!(launch(Path::new("/non/esiste/docplane-test"), None, None).is_err());
    }
}
