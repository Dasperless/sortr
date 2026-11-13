//! Command-line interface definition.

use std::path::PathBuf;

use clap::Parser;

/// Organize files into folders by type.
#[derive(Debug, Parser)]
#[command(name = "sortr", version, about)]
pub struct Cli {
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
        assert_eq!(cli.path, PathBuf::from("."));
        assert!(!cli.recursive && !cli.verbose && !cli.dry_run);
    }

    #[test]
    fn parses_flags_and_path() {
        let cli = Cli::parse_from(["sortr", "-r", "-v", "-n", "-c", "r.toml", "/tmp/x"]);
        assert_eq!(cli.config, Some(PathBuf::from("r.toml")));
        assert_eq!(cli.path, PathBuf::from("/tmp/x"));
        assert!(cli.recursive && cli.verbose && cli.dry_run);
    }
}
