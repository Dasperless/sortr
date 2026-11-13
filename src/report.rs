//! Human-readable output for plans and results.

use std::fmt::Write;
use std::path::Path;

use crate::mover::Outcome;
use crate::planner::PlannedMove;

/// Lists planned moves, one per line, plus a summary line. Used by `--dry-run`.
///
/// Destinations show the target folder and original file name; if a name is
/// already taken when the real run happens, the mover adds a ` (n)` suffix.
pub fn format_plan(moves: &[PlannedMove], root: &Path) -> String {
    let mut out = String::new();
    for m in moves {
        let name = m.source.file_name().unwrap_or_default();
        let _ = writeln!(
            out,
            "{} -> {}",
            m.source.display(),
            m.dest_dir.join(name).display()
        );
    }
    let _ = write!(
        out,
        "Dry run: {} file(s) would be moved in {} (nothing was changed)",
        moves.len(),
        root.display()
    );
    out
}

/// One-line summary of a real run.
pub fn format_summary(outcome: &Outcome, root: &Path) -> String {
    format!(
        "Moved {} file(s), {} failed, {} folder(s) created in {}",
        outcome.moved.len(),
        outcome.failed.len(),
        outcome.created_dirs.len(),
        root.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mover::Moved;
    use std::path::PathBuf;

    #[test]
    fn plan_lists_moves_and_states_nothing_changed() {
        let moves = vec![PlannedMove {
            source: "/r/a.txt".into(),
            dest_dir: "/r/txt_folder".into(),
        }];
        let text = format_plan(&moves, Path::new("/r"));
        let first = text.lines().next().unwrap();
        assert!(first.starts_with("/r/a.txt -> "));
        assert!(first.ends_with("a.txt"));
        assert!(text.ends_with("1 file(s) would be moved in /r (nothing was changed)"));
    }

    #[test]
    fn empty_plan_still_has_summary() {
        let text = format_plan(&[], Path::new("/r"));
        assert_eq!(
            text,
            "Dry run: 0 file(s) would be moved in /r (nothing was changed)"
        );
    }

    #[test]
    fn summary_counts() {
        let outcome = Outcome {
            moved: vec![Moved {
                from: PathBuf::from("a"),
                to: PathBuf::from("b"),
            }],
            failed: vec![],
            created_dirs: vec![PathBuf::from("d")],
        };
        assert_eq!(
            format_summary(&outcome, Path::new("/r")),
            "Moved 1 file(s), 0 failed, 1 folder(s) created in /r"
        );
    }
}
