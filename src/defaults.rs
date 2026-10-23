//! Making the three programs the ones Windows uses for their files.
//!
//! On Windows this registers each program for its file types under the current user
//! (`HKCU\Software\Classes`, no administrator rights) and in the list of default apps. Windows 10
//! and 11 keep the user's earlier choice for a file type until the user confirms the new one, so
//! the last step opens Settings › Default apps.

// Only Windows registers programs; elsewhere the functions are exercised by the tests alone.
#![cfg_attr(not(windows), allow(dead_code))]

use crate::apps::Kind;
use std::path::Path;

/// The name Windows knows the program's file type by.
fn prog_id(kind: Kind) -> &'static str {
    match kind {
        Kind::Word => "DocPlane.WordBook",
        Kind::Sheet => "DocPlane.SheetCalc",
        Kind::Slides => "DocPlane.Flickdeck",
    }
}

fn file_type_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Word => "Documento WordBook.it",
        Kind::Sheet => "Cartella di lavoro SheetCalc.it",
        Kind::Slides => "Presentazione Flickdeck",
    }
}

fn reg(args: &[&str]) -> Vec<String> {
    let mut v = vec!["add".to_string()];
    v.extend(args.iter().map(|a| (*a).to_string()));
    v.push("/f".to_string());
    v
}

/// The `reg add` argument lists that register `kind`, whose program is `exe`.
pub fn commands(kind: Kind, exe: &Path) -> Vec<Vec<String>> {
    let id = prog_id(kind);
    let exe = exe.to_string_lossy();
    let classes = format!("HKCU\\Software\\Classes\\{id}");
    let caps = format!("HKCU\\Software\\DocPlane\\{}\\Capabilities", kind.name());
    let mut out = vec![
        reg(&[&classes, "/ve", "/d", file_type_name(kind)]),
        reg(&[
            &format!("{classes}\\DefaultIcon"),
            "/ve",
            "/d",
            &format!("\"{exe}\",0"),
        ]),
        reg(&[
            &format!("{classes}\\shell\\open\\command"),
            "/ve",
            "/d",
            &format!("\"{exe}\" \"%1\""),
        ]),
        reg(&[&caps, "/v", "ApplicationName", "/d", kind.name()]),
        reg(&[&caps, "/v", "ApplicationDescription", "/d", kind.role()]),
    ];
    for ext in kind.extensions() {
        let dotted = format!(".{ext}");
        out.push(reg(&[
            &format!("HKCU\\Software\\Classes\\{dotted}"),
            "/ve",
            "/d",
            id,
        ]));
        out.push(reg(&[
            &format!("HKCU\\Software\\Classes\\{dotted}\\OpenWithProgids"),
            "/v",
            id,
            "/t",
            "REG_NONE",
        ]));
        out.push(reg(&[
            &format!("{caps}\\FileAssociations"),
            "/v",
            &dotted,
            "/d",
            id,
        ]));
    }
    out.push(reg(&[
        "HKCU\\Software\\RegisteredApplications",
        "/v",
        &format!("DocPlane {}", kind.name()),
        "/d",
        &format!("Software\\DocPlane\\{}\\Capabilities", kind.name()),
    ]));
    out
}

/// What happened when the programs were registered.
#[derive(Debug, Default)]
pub struct Outcome {
    pub done: Vec<Kind>,
    pub missing: Vec<Kind>,
}

#[cfg(windows)]
pub fn apply(find: impl Fn(Kind) -> Option<std::path::PathBuf>) -> Result<Outcome, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const NO_WINDOW: u32 = 0x0800_0000;
    let mut outcome = Outcome::default();
    for kind in Kind::ALL {
        let Some(exe) = find(kind) else {
            outcome.missing.push(kind);
            continue;
        };
        for args in commands(kind, &exe) {
            let status = Command::new("reg")
                .args(&args)
                .creation_flags(NO_WINDOW)
                .output()
                .map_err(|e| format!("Impossibile scrivere nel registro di Windows: {e}"))?;
            if !status.status.success() {
                return Err(format!(
                    "Windows ha rifiutato la registrazione di {}.",
                    kind.name()
                ));
            }
        }
        outcome.done.push(kind);
    }
    // Tell Settings to look at the new entries, then show the page where the choice is confirmed.
    let _ = Command::new("cmd")
        .args(["/C", "start", "", "ms-settings:defaultapps"])
        .creation_flags(NO_WINDOW)
        .spawn();
    Ok(outcome)
}

#[cfg(not(windows))]
pub fn apply(_find: impl Fn(Kind) -> Option<std::path::PathBuf>) -> Result<Outcome, String> {
    Err("Impostare i programmi predefiniti è possibile solo su Windows.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_type_of_a_program_is_registered_once() {
        for kind in Kind::ALL {
            let cmds = commands(kind, Path::new("C:\\Programmi\\DocPlane\\prog.exe"));
            for ext in kind.extensions() {
                let key = format!("HKCU\\Software\\Classes\\.{ext}");
                let n = cmds
                    .iter()
                    .filter(|c| c.get(1).is_some_and(|k| *k == key))
                    .count();
                assert_eq!(n, 1, "{ext}");
            }
        }
    }

    #[test]
    fn the_open_command_runs_the_program_with_the_file() {
        let cmds = commands(Kind::Slides, Path::new("D:\\a b\\Flickdeck.exe"));
        let open = cmds.iter().find(|c| {
            c.get(1)
                .is_some_and(|k| k.ends_with("shell\\open\\command"))
        });
        assert_eq!(
            open.and_then(|c| c.get(4)).map(String::as_str),
            Some("\"D:\\a b\\Flickdeck.exe\" \"%1\"")
        );
    }

    #[test]
    fn every_command_forces_and_names_a_key() {
        for kind in Kind::ALL {
            for c in commands(kind, Path::new("x.exe")) {
                assert_eq!(c.first().map(String::as_str), Some("add"));
                assert_eq!(c.last().map(String::as_str), Some("/f"));
                assert!(c.get(1).is_some_and(|k| k.starts_with("HKCU\\")));
            }
        }
    }
}
