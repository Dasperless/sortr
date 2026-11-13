use std::process::ExitCode;

use clap::Parser;
use sortr::cli::Cli;
use sortr::{mover, planner, report, scanner};

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

    if cli.dry_run {
        println!("{}", report::format_plan(&moves, &cli.path));
        return ExitCode::SUCCESS;
    }

    let outcome = mover::execute(&moves);

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
