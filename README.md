# sortr

A fast, safe command-line file organizer written in Rust. It sorts the files in
a directory (a Downloads folder, say) into sub-folders by file type, and it can
undo what it did.

> **sortr replaces the original Python script** (`organize.py`, formerly the
> `Organizador-de-Archivos` repository). Same idea, rewritten with dry runs,
> undo, configurable rules and no silent failures.

## Install

From source (requires Rust 1.85 or newer):

```sh
cargo install --git https://github.com/Dasperless/sortr
```

Prebuilt binaries for Linux, macOS and Windows (x86_64 and aarch64) are
attached to each [GitHub release](https://github.com/Dasperless/sortr/releases)
together with `.sha256` checksums.

## Usage

```text
sortr [OPTIONS] [PATH]        organize PATH (default: current directory)
sortr undo [PATH]             undo the most recent run in PATH
```

| Option | Meaning |
|---|---|
| `-n`, `--dry-run` | Show what would be moved; change nothing. |
| `-r`, `--recursive` | Also organize files in sub-directories. |
| `-c`, `--config FILE` | Use category rules from a TOML file. |
| `-v`, `--verbose` | Print every move. |

Try it safely first:

```sh
sortr --dry-run ~/Downloads
sortr ~/Downloads
sortr undo ~/Downloads     # changed your mind
```

### What it does

- Files are moved into `<PATH>/<category>/`. By default the category is
  `<extension>_folder` (`photo.PNG` goes to `png_folder/`); files without an
  extension go to `no_extension/`. Extensions are case-insensitive and only the
  last one counts (`a.tar.gz` is a `gz` file).
- **Nothing is overwritten.** If the destination already has `a.txt`, the new
  file becomes `a (1).txt`.
- **Hidden files and directories** (names starting with `.`) and **symlinks**
  are never touched.
- Running it twice is harmless: files already in their folder stay put.
- Errors are reported per file with the path and the cause, and the rest of the
  run continues.

### Category rules

Group extensions into folders of your choice with a TOML file:

```toml
[categories]
images = ["jpg", "jpeg", "png"]
documents = ["pdf", "txt", "md"]
```

```sh
sortr --config sortr.toml ~/Downloads
```

See [`sortr.example.toml`](sortr.example.toml) for a fuller example. If you do
not pass `--config`, sortr loads `config.toml` from its config directory when it
exists (`~/.config/sortr/config.toml` on Linux). Extensions that no rule lists
still use `<extension>_folder`. Category names must be plain folder names;
`../x` or `a/b` are rejected, and so is an extension listed in two categories.

### Undo

Every real run is recorded in `<PATH>/.sortr/journal.json`. `sortr undo`
reverses the latest run: it restores files to where they were and removes the
folders that run created, if they are now empty. It never overwrites anything:
if you deleted an organized file, or a new file now occupies the original
location, that file is reported and skipped, and stays in the journal so you can
retry. Dry runs are never recorded.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Some files could not be moved (or restored by `undo`). |
| `2` | The run could not start: unreadable directory, bad config, nothing to undo. |

## Development

```sh
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

The code separates *planning* (`scanner`, `planner`) from *execution*
(`mover`), which is what makes dry runs and undo straightforward. Releases are
automated from [Conventional Commits](https://www.conventionalcommits.org/)
with semantic-release: use `feat:`, `fix:` and similar prefixes in PR titles.
