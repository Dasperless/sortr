//! Command-line interface definition.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Organize files into folders by type.
#[derive(Debug, Parser)]
#[command(name = "sortr", version, about, args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub organize: OrganizeArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Undo the most recent run in a directory.
    Undo {
        /// Directory that was organized (defaults to the current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct OrganizeArgs {
    /// Directory to organize (defaults to the current directory).
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Also organize files in subdirectories.
    #[arg(short, long)]
    pub recursive: bool,

    /// TOML file with category rules (see sortr.example.toml).
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Show what would be moved without changing anything.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Print every move, not just the summary.
    #[arg(short, long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn defaults_to_current_directory() {
        let cli = Cli::parse_from(["sortr"]);
        assert!(cli.command.is_none());
        let a = cli.organize;
        assert_eq!(a.path, PathBuf::from("."));
        assert!(!a.recursive && !a.verbose && !a.dry_run);
    }

    #[test]
    fn parses_flags_and_path() {
        let cli = Cli::parse_from(["sortr", "-r", "-v", "-n", "-c", "r.toml", "/tmp/x"]);
        let a = cli.organize;
        assert_eq!(a.path, PathBuf::from("/tmp/x"));
        assert_eq!(a.config, Some(PathBuf::from("r.toml")));
        assert!(a.recursive && a.verbose && a.dry_run);
    }

    #[test]
    fn parses_undo_subcommand() {
        let cli = Cli::parse_from(["sortr", "undo", "/tmp/x"]);
        match cli.command {
            Some(Command::Undo { path }) => assert_eq!(path, PathBuf::from("/tmp/x")),
            other => panic!("expected undo, got {other:?}"),
        }
    }
}
