//! Reverts the most recent run recorded in the journal.
//!
//! Undo never overwrites anything: a file is restored only if its current
//! location still holds a file and its original location is free. Anything
//! else is reported and kept in the journal so undo can be retried.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::journal::{Journal, JournalError, MoveRecord};

#[derive(Debug, thiserror::Error)]
pub enum UndoError {
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error("nothing to undo in {0}")]
    NothingToUndo(PathBuf),
}

/// Why a single file could not be restored.
#[derive(Debug)]
pub struct Skipped {
    pub record: MoveRecord,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct UndoOutcome {
    pub restored: Vec<MoveRecord>,
    pub skipped: Vec<Skipped>,
    pub removed_dirs: Vec<PathBuf>,
}

/// Undoes the latest run for `root`.
pub fn undo_last(root: &std::path::Path) -> Result<UndoOutcome, UndoError> {
    let mut journal = Journal::load(root)?;
    let Some(mut run) = journal.runs.pop() else {
        return Err(UndoError::NothingToUndo(root.to_path_buf()));
    };

    let mut outcome = UndoOutcome::default();
    let mut remaining = Vec::new();

    // Reverse order mirrors the order the moves were made in.
    for record in run.moves.iter().rev() {
        match restore(record) {
            Ok(()) => outcome.restored.push(record.clone()),
            Err(reason) => {
                remaining.push(record.clone());
                outcome.skipped.push(Skipped {
                    record: record.clone(),
                    reason,
                });
            }
        }
    }

    // Only remove folders this run created, and only if now empty
    // (`remove_dir` refuses to delete a non-empty directory).
    let mut still_present = Vec::new();
    for dir in run.created_dirs.iter().rev() {
        match fs::remove_dir(dir) {
            Ok(()) => outcome.removed_dirs.push(dir.clone()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) => still_present.push(dir.clone()),
        }
    }

    // Keep whatever could not be undone so the user can retry later.
    if !remaining.is_empty() {
        remaining.reverse();
        run.moves = remaining;
        run.created_dirs = still_present;
        journal.runs.push(run);
    }
    journal.save(root)?;
    Ok(outcome)
}

fn restore(record: &MoveRecord) -> Result<(), String> {
    if !record.to.is_file() {
        return Err(format!("{} no longer exists", record.to.display()));
    }
    if record.from.exists() {
        return Err(format!("{} already exists", record.from.display()));
    }
    if let Some(parent) = record.from.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if fs::rename(&record.to, &record.from).is_ok() {
        return Ok(());
    }
    fs::copy(&record.to, &record.from).map_err(|e| e.to_string())?;
    fs::remove_file(&record.to).map_err(|e| {
        let _ = fs::remove_file(&record.from);
        e.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mover, planner, scanner};
    use std::fs::write;

    /// Organizes `root` for real and records the run, like the CLI does.
    fn organize(root: &std::path::Path) {
        let files = scanner::scan(root, false).unwrap();
        let moves = planner::plan(root, &files, planner::default_category);
        let outcome = mover::execute(&moves);
        let mut journal = Journal::load(root).unwrap();
        journal.record(&outcome);
        journal.save(root).unwrap();
    }

    #[test]
    fn restores_files_and_removes_created_folders() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "A").unwrap();
        write(dir.path().join("b.png"), "B").unwrap();
        organize(dir.path());
        assert!(!dir.path().join("a.txt").exists());

        let out = undo_last(dir.path()).unwrap();

        assert_eq!(out.restored.len(), 2);
        assert!(out.skipped.is_empty());
        assert_eq!(fs::read_to_string(dir.path().join("a.txt")).unwrap(), "A");
        assert!(dir.path().join("b.png").exists());
        assert!(!dir.path().join("txt_folder").exists());
        assert!(Journal::load(dir.path()).unwrap().runs.is_empty());
    }

    #[test]
    fn only_the_latest_run_is_undone() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "").unwrap();
        organize(dir.path());
        write(dir.path().join("b.txt"), "").unwrap();
        organize(dir.path());

        undo_last(dir.path()).unwrap();

        assert!(dir.path().join("b.txt").exists());
        assert!(dir.path().join("txt_folder/a.txt").exists());
        assert_eq!(Journal::load(dir.path()).unwrap().runs.len(), 1);
    }

    #[test]
    fn missing_or_blocked_files_are_skipped_and_kept_for_retry() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "").unwrap();
        write(dir.path().join("b.txt"), "").unwrap();
        organize(dir.path());
        // The user deleted one organized file and put a new file at the
        // other's original location.
        fs::remove_file(dir.path().join("txt_folder/a.txt")).unwrap();
        write(dir.path().join("b.txt"), "new").unwrap();

        let out = undo_last(dir.path()).unwrap();

        assert!(out.restored.is_empty());
        assert_eq!(out.skipped.len(), 2);
        assert_eq!(fs::read_to_string(dir.path().join("b.txt")).unwrap(), "new");
        assert!(dir.path().join("txt_folder/b.txt").exists());
        let journal = Journal::load(dir.path()).unwrap();
        assert_eq!(journal.runs.len(), 1);
        assert_eq!(journal.runs[0].moves.len(), 2);
    }

    #[test]
    fn folders_with_other_content_are_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join("a.txt"), "").unwrap();
        organize(dir.path());
        write(dir.path().join("txt_folder/user_file.md"), "keep").unwrap();

        let out = undo_last(dir.path()).unwrap();

        assert!(out.removed_dirs.is_empty());
        assert!(dir.path().join("txt_folder/user_file.md").exists());
        assert!(dir.path().join("a.txt").exists());
    }

    #[test]
    fn nothing_to_undo_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            undo_last(dir.path()),
            Err(UndoError::NothingToUndo(_))
        ));
    }
}
