//! Executes a plan on disk, safely.
//!
//! Guarantees:
//! - existing files are never overwritten (collisions get a ` (n)` suffix);
//! - cross-filesystem moves fall back to copy + delete;
//! - every failure is reported with its path and the run continues.

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::error::MoveError;
use crate::planner::PlannedMove;

/// A move that was carried out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Result of executing a plan.
#[derive(Debug, Default)]
pub struct Outcome {
    pub moved: Vec<Moved>,
    pub failed: Vec<MoveError>,
    /// Destination folders that did not exist before the run.
    pub created_dirs: Vec<PathBuf>,
}

/// Carries out every planned move, collecting successes and failures.
pub fn execute(moves: &[PlannedMove]) -> Outcome {
    let mut outcome = Outcome::default();
    for planned in moves {
        if let Err(err) = execute_one(planned, &mut outcome) {
            outcome.failed.push(err);
        }
    }
    outcome
}

fn execute_one(planned: &PlannedMove, outcome: &mut Outcome) -> Result<(), MoveError> {
    let dir = &planned.dest_dir;
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|source| MoveError::CreateDir {
            path: dir.clone(),
            source,
        })?;
        outcome.created_dirs.push(dir.clone());
    }

    let name = planned.source.file_name().unwrap_or_default();
    let to = reserve_destination(dir, name).map_err(|source| MoveError::Reserve {
        dir: dir.clone(),
        source,
    })?;

    if let Err(source) = move_file(&planned.source, &to) {
        // Release the placeholder we created so we do not leave empty files.
        let _ = fs::remove_file(&to);
        return Err(MoveError::Move {
            from: planned.source.clone(),
            to,
            source,
        });
    }

    outcome.moved.push(Moved {
        from: planned.source.clone(),
        to,
    });
    Ok(())
}

/// Atomically claims a free file name in `dir` by creating an empty file with
/// `create_new`, so two writers can never pick the same name. The caller then
/// moves the real file over the placeholder.
fn reserve_destination(dir: &Path, name: &OsStr) -> io::Result<PathBuf> {
    let mut n = 0u32;
    loop {
        let candidate = dir.join(candidate_name(name, n));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err(e),
        }
    }
}

/// `report.pdf` -> `report.pdf` (n = 0), `report (1).pdf`, `report (2).pdf`...
fn candidate_name(name: &OsStr, n: u32) -> std::ffi::OsString {
    if n == 0 {
        return name.to_os_string();
    }
    let path = Path::new(name);
    let stem = path.file_stem().unwrap_or(name).to_string_lossy();
    match path.extension() {
        Some(ext) => format!("{stem} ({n}).{}", ext.to_string_lossy()).into(),
        None => format!("{stem} ({n})").into(),
    }
}

/// Renames, falling back to copy + delete (e.g. across filesystems).
fn move_file(from: &Path, to: &Path) -> io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    fs::copy(from, to)?;
    if let Err(e) = fs::remove_file(from) {
        // Keep the original only; do not leave a duplicate behind.
        let _ = fs::remove_file(to);
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_to_string, write};

    fn planned(source: PathBuf, dest_dir: PathBuf) -> PlannedMove {
        PlannedMove { source, dest_dir }
    }

    #[test]
    fn moves_file_and_creates_folder() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.txt");
        write(&src, "hello").unwrap();
        let dest = dir.path().join("txt_folder");

        let out = execute(&[planned(src.clone(), dest.clone())]);

        assert!(out.failed.is_empty());
        assert_eq!(out.created_dirs, vec![dest.clone()]);
        assert!(!src.exists());
        assert_eq!(read_to_string(dest.join("a.txt")).unwrap(), "hello");
    }

    #[test]
    fn never_overwrites_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("txt_folder");
        fs::create_dir(&dest).unwrap();
        write(dest.join("a.txt"), "old").unwrap();
        write(dest.join("a (1).txt"), "older").unwrap();
        let src = dir.path().join("a.txt");
        write(&src, "new").unwrap();

        let out = execute(&[planned(src, dest.clone())]);

        assert!(out.failed.is_empty());
        assert_eq!(out.moved[0].to, dest.join("a (2).txt"));
        assert_eq!(read_to_string(dest.join("a.txt")).unwrap(), "old");
        assert_eq!(read_to_string(dest.join("a (1).txt")).unwrap(), "older");
        assert_eq!(read_to_string(dest.join("a (2).txt")).unwrap(), "new");
    }

    #[test]
    fn collision_naming_without_extension() {
        assert_eq!(candidate_name(OsStr::new("Makefile"), 0), "Makefile");
        assert_eq!(candidate_name(OsStr::new("Makefile"), 2), "Makefile (2)");
        assert_eq!(candidate_name(OsStr::new("a.tar.gz"), 1), "a.tar (1).gz");
    }

    #[test]
    fn reports_failures_and_continues() {
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("good.txt");
        write(&good, "x").unwrap();
        let missing = dir.path().join("missing.txt");
        let dest = dir.path().join("txt_folder");

        let out = execute(&[
            planned(missing, dest.clone()),
            planned(good.clone(), dest.clone()),
        ]);

        assert_eq!(out.failed.len(), 1);
        assert!(out.failed[0].to_string().contains("missing.txt"));
        assert_eq!(out.moved.len(), 1);
        assert!(dest.join("good.txt").exists());
        // The failed move must not leave an empty placeholder behind.
        assert!(!dest.join("missing.txt").exists());
    }

    #[test]
    fn destination_folder_blocked_by_a_file_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.txt");
        write(&src, "x").unwrap();
        let dest = dir.path().join("txt_folder");
        write(&dest, "i am a file").unwrap();

        let out = execute(&[planned(src.clone(), dest)]);

        assert_eq!(out.failed.len(), 1);
        assert!(src.exists());
    }
}
