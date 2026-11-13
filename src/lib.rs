//! sortr: a safe command-line file organizer.
//!
//! The library is split so that *planning* (deciding what should move where)
//! is separate from *execution* (touching the disk). This keeps dry runs and
//! undo simple and makes the logic easy to test.

pub mod planner;
pub mod scanner;
