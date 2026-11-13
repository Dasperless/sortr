# Architecture

Planning is separate from execution: the planner is pure, the mover does the
only disk writes. That one split makes dry runs, reporting and undo simple.

```text
 CLI args ──► config::Rules ─────────────┐
     │                                   ▼
     └──► scanner::scan ──► planner::plan ──► Vec<PlannedMove>
                                  │
                       --dry-run ─┤─► report::format_plan
                                  ▼
                           mover::execute ──► Outcome ──► journal::record
                                                   │
                                                   └──► report::format_summary

 sortr undo ──► journal::load ──► undo::undo_last ──► journal::save
```

| Module | Responsibility |
|---|---|
| `main.rs` | Thin glue: parse args, call the library, print, choose the exit code. |
| `cli` | `clap` definitions (`OrganizeArgs`, `undo` subcommand). |
| `scanner` | Lists candidate files (skips symlinks and hidden entries). |
| `planner` | Extension parsing, default categories, builds `PlannedMove`s from a classifier closure. No I/O. |
| `config` | TOML rules compiled to `Rules::classify`; validates category names. |
| `mover` | Executes moves: atomic name reservation (`create_new`), rename or copy+delete, collects `Outcome`. |
| `error` | `MoveError` (carries paths). Other modules define their own typed errors. |
| `report` | Pure text formatting for plans and summaries. |
| `journal` | Reads and writes `.sortr/journal.json`; records runs. |
| `undo` | Reverts the latest journaled run without overwriting. |

## Extension points

- **New classification rule:** supply a different classifier to `planner::plan`
  (the closure takes the file name and returns the category).
- **New output format (e.g. JSON):** add a formatter beside `report`; do not
  print from the library modules.
- **New safety behaviour:** put it in `mover`/`undo`, with a temp-dir test.

## Testing

Unit tests sit next to the code. `tests/` runs the real binary against temp
directories (`tests/common::Sandbox`), isolating `HOME`/`XDG_CONFIG_HOME`.
