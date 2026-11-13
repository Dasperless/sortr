//! Persistent record of what each run moved, so it can be undone.
//!
//! The journal lives in `<organized dir>/.sortr/journal.json`. The directory
//! is hidden, so the scanner never touches it. Each real run appends one
//! entry; dry runs never write.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::mover::{Moved, Outcome};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    #[error("cannot read journal {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("journal {path} is corrupt: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("cannot write journal {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
}

/// One organize run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// Seconds since the Unix epoch.
    pub timestamp: u64,
    pub moves: Vec<MoveRecord>,
    /// Folders this run created (candidates for removal when undoing).
    pub created_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRecord {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl From<&Moved> for MoveRecord {
    fn from(m: &Moved) -> Self {
        Self {
            from: m.from.clone(),
            to: m.to.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Journal {
    version: u32,
    pub runs: Vec<Run>,
}

/// Where the journal for `root` is stored.
pub fn journal_path(root: &Path) -> PathBuf {
    root.join(".sortr").join("journal.json")
}

impl Journal {
    /// Loads the journal, or an empty one if none exists yet.
    pub fn load(root: &Path) -> Result<Self, JournalError> {
        let path = journal_path(root);
        match fs::read_to_string(&path) {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|source| JournalError::Parse { path, source })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(JournalError::Read { path, source }),
        }
    }

    pub fn save(&self, root: &Path) -> Result<(), JournalError> {
        let path = journal_path(root);
        let write = || -> io::Result<()> {
            fs::create_dir_all(path.parent().expect("journal path has a parent"))?;
            let copy = Journal {
                version: FORMAT_VERSION,
                runs: self.runs.clone(),
            };
            let json = serde_json::to_string_pretty(&copy).map_err(io::Error::other)?;
            // Write then rename so a crash never leaves a half-written journal.
            let tmp = path.with_extension("json.tmp");
            fs::write(&tmp, json)?;
            fs::rename(&tmp, &path)
        };
        write().map_err(|source| JournalError::Write {
            path: path.clone(),
            source,
        })
    }

    /// Appends the result of a run. Runs that moved nothing are not recorded.
    pub fn record(&mut self, outcome: &Outcome) {
        if outcome.moved.is_empty() {
            return;
        }
        self.runs.push(Run {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            moves: outcome.moved.iter().map(MoveRecord::from).collect(),
            created_dirs: outcome.created_dirs.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome() -> Outcome {
        Outcome {
            moved: vec![Moved {
                from: "/r/a.txt".into(),
                to: "/r/txt_folder/a.txt".into(),
            }],
            failed: vec![],
            created_dirs: vec!["/r/txt_folder".into()],
        }
    }

    #[test]
    fn missing_journal_loads_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Journal::load(dir.path()).unwrap().runs.is_empty());
    }

    #[test]
    fn record_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut journal = Journal::load(dir.path()).unwrap();
        journal.record(&outcome());
        journal.save(dir.path()).unwrap();

        let loaded = Journal::load(dir.path()).unwrap();
        assert_eq!(loaded.runs.len(), 1);
        assert_eq!(
            loaded.runs[0].moves[0].to,
            PathBuf::from("/r/txt_folder/a.txt")
        );
        assert_eq!(
            loaded.runs[0].created_dirs,
            vec![PathBuf::from("/r/txt_folder")]
        );
    }

    #[test]
    fn runs_accumulate() {
        let dir = tempfile::tempdir().unwrap();
        for _ in 0..2 {
            let mut journal = Journal::load(dir.path()).unwrap();
            journal.record(&outcome());
            journal.save(dir.path()).unwrap();
        }
        assert_eq!(Journal::load(dir.path()).unwrap().runs.len(), 2);
    }

    #[test]
    fn empty_runs_are_not_recorded() {
        let mut journal = Journal::default();
        journal.record(&Outcome::default());
        assert!(journal.runs.is_empty());
    }

    #[test]
    fn corrupt_journal_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".sortr")).unwrap();
        fs::write(journal_path(dir.path()), "{not json").unwrap();
        assert!(matches!(
            Journal::load(dir.path()),
            Err(JournalError::Parse { .. })
        ));
    }
}
