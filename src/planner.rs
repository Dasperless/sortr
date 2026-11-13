//! Decides where each file should go, without touching the disk.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Folder used for files that have no usable extension.
pub const NO_EXTENSION: &str = "no_extension";

/// A single planned move: `source` should end up inside `dest_dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub dest_dir: PathBuf,
}

/// Lower-cased extension of a file name, if it has one.
///
/// Only the last extension counts (`a.tar.gz` -> `gz`). A leading dot is not
/// an extension separator (`.bashrc` has none), and an empty trailing
/// extension (`name.`) is treated as none.
pub fn extension_of(file_name: &OsStr) -> Option<String> {
    Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .filter(|e| !e.is_empty())
}

/// Default category: `<ext>_folder`, matching the original Python script,
/// or `no_extension` when there is no extension.
pub fn default_category(file_name: &OsStr) -> String {
    match extension_of(file_name) {
        Some(ext) => format!("{ext}_folder"),
        None => NO_EXTENSION.to_string(),
    }
}

/// Builds the list of moves for `files`, placing each into
/// `root/<category>`, where the category comes from `classify`.
///
/// Files that already sit in their destination folder are left alone, which
/// makes running the organizer twice a no-op.
pub fn plan<F>(root: &Path, files: &[PathBuf], classify: F) -> Vec<PlannedMove>
where
    F: Fn(&OsStr) -> String,
{
    files
        .iter()
        .filter_map(|source| {
            let name = source.file_name()?;
            let dest_dir = root.join(classify(name));
            if source.parent() == Some(dest_dir.as_path()) {
                return None;
            }
            Some(PlannedMove {
                source: source.clone(),
                dest_dir,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ext(name: &str) -> Option<String> {
        extension_of(OsStr::new(name))
    }

    #[test]
    fn extension_parsing() {
        assert_eq!(ext("photo.jpg").as_deref(), Some("jpg"));
        assert_eq!(ext("PHOTO.JPG").as_deref(), Some("jpg"));
        assert_eq!(ext("archive.tar.gz").as_deref(), Some("gz"));
        assert_eq!(ext("README"), None);
        assert_eq!(ext(".bashrc"), None);
        assert_eq!(ext("trailing."), None);
    }

    #[test]
    fn default_categories() {
        assert_eq!(default_category(OsStr::new("a.PDF")), "pdf_folder");
        assert_eq!(default_category(OsStr::new("Makefile")), NO_EXTENSION);
    }

    #[test]
    fn plans_each_file_into_its_category() {
        let root = Path::new("/r");
        let files = vec![PathBuf::from("/r/a.txt"), PathBuf::from("/r/b.png")];
        let moves = plan(root, &files, default_category);
        assert_eq!(
            moves,
            vec![
                PlannedMove {
                    source: "/r/a.txt".into(),
                    dest_dir: "/r/txt_folder".into()
                },
                PlannedMove {
                    source: "/r/b.png".into(),
                    dest_dir: "/r/png_folder".into()
                },
            ]
        );
    }

    #[test]
    fn files_already_in_place_are_skipped() {
        let root = Path::new("/r");
        let files = vec![
            PathBuf::from("/r/txt_folder/a.txt"),
            PathBuf::from("/r/sub/b.txt"),
        ];
        let moves = plan(root, &files, default_category);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].source, PathBuf::from("/r/sub/b.txt"));
    }

    #[test]
    fn custom_classifier_is_used() {
        let files = vec![PathBuf::from("/r/a.txt")];
        let moves = plan(Path::new("/r"), &files, |_| "docs".to_string());
        assert_eq!(moves[0].dest_dir, PathBuf::from("/r/docs"));
    }
}
