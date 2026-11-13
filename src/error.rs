//! Error types.

use std::io;
use std::path::PathBuf;

/// A failure while moving a single file. Carries the path so the user can see
/// exactly what went wrong instead of errors being swallowed.
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("cannot create folder {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },

    #[error("cannot reserve a destination name in {dir}: {source}")]
    Reserve { dir: PathBuf, source: io::Error },

    #[error("cannot move {from} to {to}: {source}")]
    Move {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
}
