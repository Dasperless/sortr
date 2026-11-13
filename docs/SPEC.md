# sortr specification

The behavioural contract. If behaviour changes, change this file in the same
pull request. Tests should trace back to the statements here.

## Purpose

Organize the files in a directory into sub-folders, safely and reversibly.

## Command line

```text
sortr [OPTIONS] [PATH]      PATH defaults to "."
sortr undo [PATH]           PATH defaults to "."
```

Options: `-n/--dry-run`, `-r/--recursive`, `-c/--config FILE`, `-v/--verbose`.
A directory literally named `undo` must be given as `./undo`.

## Selection

1. Only regular files are candidates. Symlinks are never followed or moved.
2. Names starting with `.` (files and directories) are ignored, so `.git`,
   `.env` and sortr's own `.sortr/` are never touched.
3. Without `--recursive`, only the top level of PATH is scanned.

## Classification

1. The extension is the text after the last `.` of the file name, lower-cased.
   `.bashrc` and `name.` have no extension.
2. With a rule file: the category named for that extension.
3. Otherwise `<extension>_folder`; with no extension, `no_extension`.

## Moving

1. A file moves to `PATH/<category>/` (created when missing), even when found in
   a sub-directory during `--recursive`.
2. A file already in its destination folder is left alone, so a second run is a
   no-op.
3. An existing file is never overwritten; the new file is renamed
   `name (n).ext` with the smallest free `n`.
4. A failure on one file is reported (path and cause) and does not stop the
   others. Cross-filesystem moves fall back to copy + delete; a failed delete
   never leaves a duplicate.

## Dry run

`--dry-run` prints the planned moves and a summary. It changes nothing and does
not write a journal.

## Configuration

TOML with a `[categories]` table of `category = ["ext", ...]`.

- Extensions are case-insensitive; a leading dot is allowed.
- Category names must be plain folder names (no separators, not `.`/`..`).
- An extension in two categories is an error.
- `--config` must load or the run fails (exit 2). Without it, the file in the
  platform config dir (`<config dir>/sortr/config.toml`) is used if present.

## Journal and undo

1. Each real run that moved at least one file appends its moves and created
   folders to `PATH/.sortr/journal.json` (versioned, atomically written).
2. `sortr undo` reverts the most recent run: restore files in reverse order,
   then remove folders that run created if now empty.
3. Undo never overwrites. A file that is missing, or whose original location is
   taken, is reported and stays in the journal for a retry.
4. A journal write failure warns but does not fail the run.

## Exit codes

`0` success. `1` some files failed to move/restore. `2` the run could not start
(unreadable directory, invalid or missing config, nothing to undo).

## Non-goals

Content-based classification, watching a directory, merging duplicates,
renaming by metadata, and a GUI.
