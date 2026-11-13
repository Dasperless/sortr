use std::process::ExitCode;

use clap::Parser;
use sortr::cli::Cli;
use sortr::config::{self, Rules};
use sortr::journal::Journal;
use sortr::{mover, planner, report, scanner};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let rules = match load_rules(&cli) {
        Ok(rules) => rules,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };

    let files = match scanner::scan(&cli.path, cli.recursive) {
        Ok(files) => files,
        Err(err) => {
            eprintln!("error: cannot read {}: {err}", cli.path.display());
            return ExitCode::from(2);
        }
    };

    let moves = planner::plan(&cli.path, &files, |name| rules.classify(name));

    if cli.dry_run {
        println!("{}", report::format_plan(&moves, &cli.path));
        return ExitCode::SUCCESS;
    }

    let outcome = mover::execute(&moves);

    record_journal(&cli.path, &outcome);

    if cli.verbose {
        for m in &outcome.moved {
            println!("{} -> {}", m.from.display(), m.to.display());
        }
    }
    for err in &outcome.failed {
        eprintln!("error: {err}");
    }
    println!("{}", report::format_summary(&outcome, &cli.path));

    if outcome.failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// An explicit `--config` must load; otherwise use the default config file if
/// it exists, else built-in defaults.
fn load_rules(cli: &Cli) -> Result<Rules, config::ConfigError> {
    if let Some(path) = &cli.config {
        return Rules::load(path);
    }
    match config::default_config_path() {
        Some(path) if path.is_file() => Rules::load(&path),
        _ => Ok(Rules::default()),
    }
}

/// Best effort: a journal problem must not turn a successful run into a failure.
fn record_journal(root: &std::path::Path, outcome: &mover::Outcome) {
    let result = Journal::load(root).and_then(|mut journal| {
        journal.record(outcome);
        journal.save(root)
    });
    if let Err(err) = result {
        eprintln!("warning: {err}; this run cannot be undone");
    }
}
