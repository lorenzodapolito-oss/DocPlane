//! The dictionaries the three programs share: Hunspell spelling dictionaries and MyThes
//! thesauri in one `dictionaries` folder beside DocPlane.it. Each program finds the folder by
//! itself (it sits beside it) and DocPlane.it also names it in `DOCPLANE_DICTIONARIES` when it
//! starts one.
//!
//! The dictionaries are not part of this repository: they are downloaded from LibreOffice's
//! dictionaries repository, each under its own licence (the Italian one is GPL 3), and kept in
//! a separate folder as plain data files.

use std::path::{Path, PathBuf};
use std::process::Command;

const BASE: &str = "https://raw.githubusercontent.com/LibreOffice/dictionaries/master";

/// A language of the suite and the files that make it up (path in the repository, file name).
pub struct Language {
    pub name: &'static str,
    pub code: &'static str,
    files: &'static [(&'static str, &'static str)],
}

pub const LANGUAGES: [Language; 2] = [
    Language {
        name: "italiano",
        code: "it_IT",
        files: &[
            ("it_IT/it_IT.aff", "it_IT.aff"),
            ("it_IT/it_IT.dic", "it_IT.dic"),
            ("it_IT/hyph_it_IT.dic", "hyph_it_IT.dic"),
            ("it_IT/th_it_IT_v2.dat", "th_it_IT_v2.dat"),
        ],
    },
    Language {
        name: "inglese",
        code: "en_US",
        files: &[
            ("en/en_US.aff", "en_US.aff"),
            ("en/en_US.dic", "en_US.dic"),
            ("en/hyph_en_US.dic", "hyph_en_US.dic"),
            ("en/th_en_US_v2.dat", "th_en_US_v2.dat"),
        ],
    },
];

/// What is installed for a language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub spelling: bool,
    pub thesaurus: bool,
}

/// The shared folder: `dictionaries` beside DocPlane.it.
pub fn folder(exe_dir: &Path) -> PathBuf {
    exe_dir.join("dictionaries")
}

pub fn status(dir: &Path, lang: &Language) -> Status {
    let has = |name: &str| dir.join(name).is_file();
    Status {
        spelling: has(&format!("{}.aff", lang.code)) && has(&format!("{}.dic", lang.code)),
        thesaurus: has(&format!("th_{}_v2.dat", lang.code)),
    }
}

/// "italiano ✓ inglese ✓", or what is missing.
pub fn summary(dir: &Path) -> String {
    LANGUAGES
        .iter()
        .map(|l| {
            let s = status(dir, l);
            let what = match (s.spelling, s.thesaurus) {
                (true, true) => "ortografia e sinonimi".to_string(),
                (true, false) => "solo ortografia".to_string(),
                (false, true) => "solo sinonimi".to_string(),
                (false, false) => "non installato".to_string(),
            };
            format!("{}: {what}", l.name)
        })
        .collect::<Vec<_>>()
        .join("  ·  ")
}

/// The program that downloads files: `curl` (Windows 10 and later, macOS and Linux all have it).
fn curl() -> &'static str {
    if cfg!(windows) { "curl.exe" } else { "curl" }
}

/// Download one file to `dest` (through a temporary name, so an interrupted download leaves nothing
/// that looks like a dictionary).
fn fetch(url: &str, dest: &Path) -> Result<(), String> {
    let tmp = dest.with_extension("part");
    let mut cmd = Command::new(curl());
    cmd.args([
        "--location",
        "--fail",
        "--silent",
        "--show-error",
        "--max-time",
        "600",
        "--output",
    ])
    .arg(&tmp)
    .arg(url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("impossibile avviare curl: {e}"))?;
    if !out.status.success() {
        let _ = std::fs::remove_file(&tmp);
        let why = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{url}: {}", why.trim()));
    }
    std::fs::rename(&tmp, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

/// Download the dictionaries into `dir`; `progress` is told each file as it starts. Stops at the
/// first error.
pub fn download(dir: &Path, progress: &mut dyn FnMut(&str)) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for lang in &LANGUAGES {
        for (path, name) in lang.files {
            progress(name);
            fetch(&format!("{BASE}/{path}"), &dir.join(name))?;
        }
    }
    std::fs::write(dir.join("LEGGIMI.txt"), NOTICE).map_err(|e| format!("LEGGIMI.txt: {e}"))
}

const NOTICE: &str = "Dizionari del pacchetto DocPlane.it\r\n\
\r\n\
Questa cartella è letta da WordBook.it, SheetCalc.it e Flickdeck.it: controllo ortografico\r\n\
(file .aff e .dic, formato Hunspell), sinonimi (file th_*.dat, formato MyThes) e sillabazione\r\n\
(file hyph_*.dic).\r\n\
\r\n\
I file sono scaricati dal repository https://github.com/LibreOffice/dictionaries, ognuno con la\r\n\
propria licenza, indicata nei file README di quel repository. Il dizionario italiano (LibreItalia,\r\n\
Estensione linguistica italiana) è distribuito con licenza GNU GPL 3. I programmi del pacchetto non\r\n\
includono questi dati: li leggono soltanto.\r\n\
\r\n\
Per aggiungere una lingua copia qui i file Hunspell della lingua (per esempio fr_FR.aff e fr_FR.dic).\r\n";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn status_follows_the_files_in_the_folder() {
        let dir = std::env::temp_dir().join(format!("docplane-dict-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let it = &LANGUAGES[0];
        assert_eq!(
            status(&dir, it),
            Status {
                spelling: false,
                thesaurus: false
            }
        );
        std::fs::write(dir.join("it_IT.aff"), b"").unwrap();
        assert_eq!(status(&dir, it).spelling, false, "the .dic is missing too");
        std::fs::write(dir.join("it_IT.dic"), b"").unwrap();
        std::fs::write(dir.join("th_it_IT_v2.dat"), b"").unwrap();
        assert_eq!(
            status(&dir, it),
            Status {
                spelling: true,
                thesaurus: true
            }
        );
        assert!(summary(&dir).contains("italiano: ortografia e sinonimi"));
        assert!(summary(&dir).contains("inglese: non installato"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_language_lists_the_files_its_status_looks_for() {
        for l in &LANGUAGES {
            let names: Vec<&str> = l.files.iter().map(|f| f.1).collect();
            assert!(names.contains(&format!("{}.aff", l.code).as_str()));
            assert!(names.contains(&format!("{}.dic", l.code).as_str()));
            assert!(names.contains(&format!("th_{}_v2.dat", l.code).as_str()));
            assert!(
                l.files
                    .iter()
                    .all(|(p, n)| p.ends_with(n) && !n.contains('/') && !n.contains(".."))
            );
        }
    }
}
