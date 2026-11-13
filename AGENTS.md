# AGENTS.md

Instructions for AI coding agents (and a useful summary for humans). This is the
single source of truth; `CLAUDE.md` and `.github/copilot-instructions.md` point
here. Update this file when you learn something an agent keeps getting wrong.

## What this project is

`sortr` is a Rust CLI that organizes files in a directory into folders by type,
with dry runs, configurable rules and undo. It rewrote an earlier Python script.
It **moves users' files**, so safety beats cleverness.

Read before changing behaviour:
- [`docs/SPEC.md`](docs/SPEC.md): what the tool must do (the contract).
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): modules and data flow.

## Commands (run these before saying you are done)

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Minimum supported Rust version is 1.85 (`rust-version` in `Cargo.toml`).

## How to work here: process over magic

AI output is a draft for a human to review, not an answer to trust.

1. **Start from an issue and a plan.** State the plan (what, which files, how you
   will verify) before writing code. If the request is ambiguous, ask.
2. **Small, reviewable steps.** One issue or concern per pull request. If a
   change grows past what a person can review comfortably, split it by value
   (each part should still work and be tested on its own).
3. **Verify, don't assume.** Add or update tests with the change. Run the
   commands above and read the output. Never say "done" or "tests pass" without
   having run them; if something was not run or failed, say so.
4. **Explain the change.** PR descriptions say what changed, why, how it was
   tested, and what a reviewer should look at hardest.
5. **Simplest thing that meets the spec.** No speculative features, abstractions
   or dependencies. Ask before adding a dependency or doing a large refactor.
6. **Never weaken checks to get green.** Do not skip, delete or loosen a test,
   lint or CI job to make it pass; fix the cause or raise it.

## Code conventions

- Idiomatic Rust, formatted with `rustfmt`, clean under `clippy -D warnings`.
- Errors are typed (`thiserror`) and carry the path involved. **Never swallow an
  error** (the old Python script's `except: continue` is the anti-pattern).
- No `unwrap()`/`expect()` in non-test code unless an invariant makes failure
  impossible, and say why in a comment.
- Keep *planning* (pure, no disk writes) separate from *execution* (`mover`).
  New behaviour should normally extend the plan, not add disk writes elsewhere.
- Comments explain *why*, not *what*.

## Safety rules (this tool touches real files)

- Never overwrite an existing file. Collisions get a ` (n)` suffix.
- Anything that changes the disk must be skippable with `--dry-run` and
  recordable for `undo`.
- Config-supplied names (categories) must not be able to escape the target
  directory.
- Tests use temp directories only (`tempfile`). Never touch the real home
  directory or the developer's config: integration tests isolate
  `HOME`/`XDG_CONFIG_HOME`.

## Boundaries

- Do not hand-edit generated files: `CHANGELOG.md`, and the version fields in
  `Cargo.toml`/`Cargo.lock` (semantic-release owns them).
- Do not change `.github/workflows/` without explaining the effect; they cannot
  be fully tested locally.
- Do not commit secrets, and do not push to or force-push `main`.

## GitHub Actions: pin to commit SHAs

Tags can be moved or re-pointed, so every `uses:` in `.github/workflows/` must
reference the **full 40-character commit SHA of the latest release tag**, with
the tag in a trailing comment:

```yaml
- uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
```

- Look up the latest release tag first, then resolve it to a commit (for an
  annotated tag use the commit it points to, not the tag object):
  `git ls-remote --tags https://github.com/<owner>/<repo> | sort -V`.
  Do not guess versions from memory; they go stale.
- Never use a branch or floating tag (`@v4`, `@stable`, `@main`).
- Some actions select a version by ref (for example `dtolnay/rust-toolchain@stable`).
  When pinning those, pass the version as an input instead
  (`with: toolchain: stable`).
- After bumping, read the action's `action.yml` to confirm the inputs and
  outputs we use still exist, since new major versions can rename them.
- Dependabot (`.github/dependabot.yml`) updates SHA pins and their comments.

## Commits and pull requests

- Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `ci:`, `chore:`,
  `refactor:`. Releases are computed from them. PR titles must follow the same
  format (enforced by CI; PRs are squash-merged).
- Link the issue. Fill in the PR template.

## Definition of done

- The three commands above pass.
- Behaviour changes are covered by tests and reflected in `docs/SPEC.md` and the
  README.
- A human has reviewed the change.
