//! The files DocPlane opened lately, kept in a small text file (`recenti.txt`) in the data
//! folder: one line per file, `programma TAB percorso`.

use crate::apps::Kind;
use std::path::{Path, PathBuf};

/// How many files are remembered.
pub const MAX: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recent {
    pub kind: Kind,
    pub path: PathBuf,
}

fn code(kind: Kind) -> &'static str {
    match kind {
        Kind::Word => "word",
        Kind::Sheet => "sheet",
        Kind::Slides => "slides",
    }
}

fn from_code(text: &str) -> Option<Kind> {
    Kind::ALL.into_iter().find(|k| code(*k) == text)
}

/// Read the list; lines that make no sense are skipped.
pub fn parse(text: &str) -> Vec<Recent> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((k, p)) = line.split_once('\t') else {
            continue;
        };
        let (Some(kind), false) = (from_code(k.trim()), p.trim().is_empty()) else {
            continue;
        };
        if out.len() < MAX {
            out.push(Recent {
                kind,
                path: PathBuf::from(p.trim_end_matches(['\r', '\n'])),
            });
        }
    }
    out
}

pub fn serialize(items: &[Recent]) -> String {
    let mut s = String::new();
    for r in items.iter().take(MAX) {
        s.push_str(code(r.kind));
        s.push('\t');
        s.push_str(&r.path.to_string_lossy());
        s.push('\n');
    }
    s
}

/// Put a file first, once, and keep the list short.
pub fn add(items: &mut Vec<Recent>, kind: Kind, path: &Path) {
    items.retain(|r| r.path != path);
    items.insert(
        0,
        Recent {
            kind,
            path: path.to_path_buf(),
        },
    );
    items.truncate(MAX);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_survives_saving_and_reading() {
        let mut items = Vec::new();
        add(&mut items, Kind::Word, Path::new("a.docx"));
        add(&mut items, Kind::Sheet, Path::new("b.xlsx"));
        add(&mut items, Kind::Word, Path::new("a.docx"));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].path, Path::new("a.docx"));
        assert_eq!(parse(&serialize(&items)), items);
    }

    #[test]
    fn junk_lines_and_long_lists_are_handled() {
        assert!(parse("senza tab\n\tvuoto\nnoprog\tx\n").is_empty());
        let mut items = Vec::new();
        for i in 0..40 {
            add(&mut items, Kind::Slides, Path::new(&format!("{i}.pptx")));
        }
        assert_eq!(items.len(), MAX);
        let many: String = (0..40).map(|i| format!("word\t{i}.docx\n")).collect();
        assert_eq!(parse(&many).len(), MAX);
    }
}
