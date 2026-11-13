use std::process::ExitCode;

use clap::Parser;
use sortr::cli::Cli;
use sortr::{mover, planner, scanner};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let files = match scanner::scan(&cli.path, cli.recursive) {
        Ok(files) => files,
        Err(err) => {
            eprintln!("error: cannot read {}: {err}", cli.path.display());
            return ExitCode::from(2);
        }
    };

    let moves = planner::plan(&cli.path, &files, planner::default_category);
    let outcome = mover::execute(&moves);

    if cli.verbose {
        for m in &outcome.moved {
            println!("{} -> {}", m.from.display(), m.to.display());
        }
    }
    for err in &outcome.failed {
        eprintln!("error: {err}");
    }
    println!(
        "Moved {} file(s), {} failed, {} folder(s) created in {}",
        outcome.moved.len(),
        outcome.failed.len(),
        outcome.created_dirs.len(),
        cli.path.display()
    );

    if outcome.failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
