//! Discovers the files that are candidates for organizing.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Returns the regular files under `root`, sorted for deterministic output.
///
/// - Only the top level is scanned unless `recursive` is true.
/// - Symbolic links are never followed or returned.
/// - Hidden entries (names starting with `.`) are skipped, including hidden
///   directories such as `.git`, so they are never moved.
pub fn scan(root: &Path, recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect(root, recursive, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_file() {
            out.push(entry.path());
        } else if file_type.is_dir() && recursive {
            collect(&entry.path(), recursive, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, write};

    fn names(root: &Path, files: &[PathBuf]) -> Vec<String> {
        files
            .iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn top_level_only_by_default() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "").unwrap();
        create_dir(dir.path().join("sub")).unwrap();
        write(dir.path().join("sub/b.txt"), "").unwrap();

        let files = scan(dir.path(), false).unwrap();
        assert_eq!(names(dir.path(), &files), ["a.txt"]);
    }

    #[test]
    fn recursive_descends_into_directories() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "").unwrap();
        create_dir(dir.path().join("sub")).unwrap();
        write(dir.path().join("sub/b.txt"), "").unwrap();

        let files = scan(dir.path(), true).unwrap();
        assert_eq!(names(dir.path(), &files), ["a.txt", "sub/b.txt"]);
    }

    #[test]
    fn skips_hidden_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join(".hidden"), "").unwrap();
        create_dir(dir.path().join(".git")).unwrap();
        write(dir.path().join(".git/config"), "").unwrap();
        write(dir.path().join("a.txt"), "").unwrap();

        let files = scan(dir.path(), true).unwrap();
        assert_eq!(names(dir.path(), &files), ["a.txt"]);
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("real.txt"), "").unwrap();
        std::os::unix::fs::symlink(dir.path().join("real.txt"), dir.path().join("link.txt"))
            .unwrap();

        let files = scan(dir.path(), false).unwrap();
        assert_eq!(names(dir.path(), &files), ["real.txt"]);
    }

    #[test]
    fn missing_directory_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scan(&dir.path().join("nope"), false).is_err());
    }
}
