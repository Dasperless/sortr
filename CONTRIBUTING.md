# Contributing

Thanks for helping with sortr. These rules apply equally to people and to AI
assistants (see [`AGENTS.md`](AGENTS.md) for the agent-specific details).

## Workflow

1. **Start from an issue.** Describe the change there first; for anything
   non-trivial agree the approach before coding. Behaviour changes update
   [`docs/SPEC.md`](docs/SPEC.md) in the same PR.
2. **Keep PRs small and focused.** One concern per PR. If it is too long to
   review comfortably, split it into parts that each deliver value and pass on
   their own.
3. **Test with the change.** Unit tests next to the code; end-to-end tests in
   `tests/` use temp directories only.
4. **Run the checks locally:**

   ```sh
   cargo fmt --all -- --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   ```

5. **Open a PR** and fill in the template. AI-generated code gets the same
   scrutiny as any other: a human must read and understand it before merging.

## Branch names

`<type>/<small-description>`: a Conventional Commit type, a slash, then a short,
meaningful, readable description in lowercase words joined by hyphens (usually
2 to 4 words). Examples: `fix/release-version-bump`, `feat/json-output`,
`docs/branch-naming`. Avoid issue numbers, dates, session ids and vague names
like `fix/stuff`.

## Commit and PR title format

[Conventional Commits](https://www.conventionalcommits.org/). Releases are
computed from them, and PRs are squash-merged, so the **PR title** is what
counts.

| Prefix | Effect on the next release |
|---|---|
| `feat:` | minor |
| `fix:`, `perf:` | patch |
| `feat!:` or a `BREAKING CHANGE:` footer | major |
| `docs:`, `test:`, `ci:`, `chore:`, `refactor:` | no release |

## Do not edit by hand

`CHANGELOG.md` and the `version` in `Cargo.toml`/`Cargo.lock` are written by the
release pipeline.

## Safety

sortr moves people's files. Never overwrite, never swallow errors, and never
let tests touch real user directories.
