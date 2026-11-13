# Decision log

Short records of choices that are not obvious from the code. Add a new entry at
the bottom; do not rewrite old ones, supersede them.

## 1. Plan first, execute second
`planner` is pure and `mover` is the only module that moves files. This makes
`--dry-run` exact (same plan as a real run), keeps most logic unit-testable
without a disk, and lets rules and undo evolve independently.

## 2. Never overwrite; reserve names atomically
A destination name is claimed with `create_new`, then the file is moved over the
placeholder. Two writers cannot pick the same name, and an existing file can
never be clobbered. Cost: a short-lived empty placeholder file.

## 3. Hidden files and symlinks are skipped
The Python version moved dotfiles. Moving `.env`, `.gitignore` or a `.git`
directory's contents is surprising and hard to reverse, so they are ignored. This
also keeps sortr's own `.sortr/` journal out of the scan.

## 4. TOML for rules
Readable, comment-friendly, well supported in Rust (`serde` + `toml`). Category
names are restricted to plain folder names so a config can never move files
outside the target directory.

## 5. Journal stored beside the organized directory
`<dir>/.sortr/journal.json` travels with the folder, needs no data-dir lookup,
and is per-directory, so `sortr undo <dir>` is unambiguous. Trade-off: it only
supports undoing runs recorded under the directory that was organized.

## 6. Conventional Commits and semantic-release
Versions, changelog and releases are derived from commit messages, so there is
no manual release step. PRs are squash-merged with a validated title.

## 7. Agent instructions in one file
`AGENTS.md` is canonical; tool-specific files only point to it, so instructions
cannot drift between assistants.
