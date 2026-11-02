# Working on si_dynamic

Guidance for AI agents and contributors. Read this before changing code.

si_dynamic parses SI quantities and units from text at runtime: `4.7 uF`, `10mV`, `4k7` (resistance), units with
prefixes and exponents, and (planned) unit expressions like `m/s²`. Published on crates.io. Its main user is
rvariant (`Variant::SI`), and through it egui_tabular and mx3, where it parses component parameters typed by people
or imported from CSV and supplier data, so lenient input and exact results both matter.

## FEATURES.md is the source of truth

[FEATURES.md](FEATURES.md) lists every feature with its status, every known bug, and what is planned, with stable
IDs per area (`UNIT-2`, `NUM-3`, ...).

- **Read the relevant area before starting.** Several known bugs are recorded with the input that triggers them.
- **Name IDs with a short slug when talking to the user** (answers, plans, summaries, tables):
  `UNIT-6 gram-prefix`, never a bare `UNIT-6`. The slug is 2-4 kebab-case words from the item's title. Commit
  messages, CHANGELOG and code `TODO`s keep the bare ID.
- **Update it in the same commit** as the code: mark items ✅/🚧/🐛 with a pointer to the code, add bugs you find
  but don't fix (next free ID of the area), move obsolete items to *Dropped and superseded*. Never renumber or
  reuse IDs.
- Don't track status anywhere else (README checklists, TODO files). Code `TODO`s that matter reference an ID:
  `// TODO(EXPR-1): ...`.

## CHANGELOG.md records every change

[CHANGELOG.md](CHANGELOG.md) is the history, FEATURES.md the current state; keep both.

- Every change a user of the crate would notice gets an entry under `## [Unreleased]` in the same commit:
  `### Added`, `### Changed`, `### Fixed`, `### Removed`. Short, with the feature ID in parentheses. Mark API
  breaks with **Breaking:**, and say when an input now parses to a different value.
- Pure refactors and typo fixes don't need an entry.
- A commit that bumps the version (see *Versions*) moves the `[Unreleased]` entries under a new
  `## [x.y.z] - YYYY-MM-DD` heading and leaves `[Unreleased]` empty, so every version has its own section.
- Questions like "what's new" or "what changed since X" are answered from CHANGELOG.md, newest sections first
  (the user's version or date as the cutoff), with FEATURES.md for current status.

The tpm repo's `/sync-repos` reads this file to log progress, so a missing entry means work nobody sees.

## Layout

A Cargo workspace, two crates under `crates/`, root `Cargo.toml` holding `[workspace.package]` (shared edition,
license, repository) and `[workspace.dependencies]`.

- `crates/si_dynamic` — the values: parses SI quantities and units from text. Its own `README.md`,
  `CHANGELOG.md`-worthy history lives in the root `CHANGELOG.md` below.
  - `grammar/si.pest` — the pest grammar: numbers, unit words, expressions (`m/s²`, `W/(m·K)`), superscripts, the
    special resistance syntax (`4k7`, the `*_unconditional` prefixes). A unit word is resolved in Rust
    (`parse::resolve_word`): a whole known unit wins (`Pa`, `cd`, `m`), then a prefix plus a known unit (`kPa`,
    `mΩ`), then the old split for unknown words (`mVDC` is milli + `VDC`).
  - `src/dim.rs` — `Dim` (SI base-unit exponents, ww_si's order) and `Scale` (how a unit becomes coherent SI).
  - `src/unit.rs` — `Prefix`, `BaseUnit` (SI base and derived units, `Ratio`, registered `Named`, `Unknown` words),
    `Unit` (prefix, base, exponent), `UnitExpr` (products, quotients, powers).
  - `src/registry.rs` — `Registry` and `NamedUnit`: units and spellings known at run time.
  - `src/parse.rs` — from the pest tree to `Quantity` and `UnitExpr`, word resolution and prefix splitting.
  - `src/lib.rs` — `Quantity`, `OhmF32::parse`, `Error`, tests at the bottom.
- `crates/napier` — the expression language over si_dynamic's values, named after John Napier. Empty for now
  (`publish = false`); depends on `si_dynamic` by path.

The si_dynamic model follows ww_si (`~/git/wire_weaver/ww_stdlib/ww_si`): same `Dim` order and `Scale` semantics.
Nothing about a non-SI unit is built in beyond `%`, `‰`, `ppm`; callers register theirs in a `Registry`.

rvariant (`../rvariant/src/si.rs`) is the main consumer of si_dynamic. After an API or behaviour change, build and
test rvariant against this checkout (it takes si_dynamic by path) and say what it needs to follow.

## Commands

```sh
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo test
cd ../rvariant && cargo test --all-features   # main downstream user
```

Before declaring a change done: build, clippy without warnings, fmt, tests, and rvariant's tests.

## Code conventions

- Parse errors are `Error` values, never panics: no `unwrap`/`expect` on input-derived data, no `println!` left in
  library code.
- A new accepted spelling goes into the grammar and a test in the same commit. Check it doesn't change how an
  existing input parses (the prefix split in `resolve_word` makes this easy to break: `m`, `T`, `G`, `R`, `cd`, `Pa`).
- Don't lose precision silently: keep the number text in `Quantity`, convert at the edge.

## Tests

Unit tests in `src/lib.rs` (`mod tests`), one per kind of input, asserting prefix, base, exponent and value. A bug
fix starts with a failing test with the exact input that misbehaves.

## Commits

Conventional Commits: `feat: ...`, `fix: ...`, `refactor: ...`, `build: ...`. Short imperative summary, blank
line, body with what and why; reference feature IDs (`fix: gram prefix no longer doubles to kkg (UNIT-6)`).

Commit on your own initiative (tpm CLAUDE.md "Commits are free, prod is gated"): work happens on a session branch
(`tpm work new SLUG`), each finished step is one commit with FEATURES.md and CHANGELOG.md updated in it. When the
work is done and the user agrees, `tpm land` puts it on main; pushing is the housekeeping timer's job. Anything that
reaches clients (prod deploy, firmware/OTA release, registry publish) still waits for the user's OK.

## Versions

Landing bumps the version, not each work commit (tpm CLAUDE.md "Landing bumps the version"), so any build from main
traces back to a release commit. **A branch never touches a version** (a manifest's `version =`, a new
`## [x.y.z]` CHANGELOG heading, a tag): not a session's call, in this workspace or any crate in it.
- Session commits add CHANGELOG entries under `[Unreleased]` without bumping. `tpm land` turns them into the next
  version's section with the manifest bump in one `release: x.y.z` commit, per changed crate: minor for
  Added/Changed/Removed/Deprecated, else patch; major only when the owner says so (`tpm land --version`).
  Docs-only, CI-only and no-behaviour-change refactors land without a bump.
- CLIs print version, git SHA and build time in `--version`, e.g. `tool 0.4.2 (a1b2c3d-dirty, built 3 Oct 2026
  18:20)`: a small `build.rs` without extra crates (`git rev-parse --short HEAD`, `-dirty` when
  `git status --porcelain` isn't empty, `rerun-if-changed` on `.git/HEAD` and `.git/index`, `unknown` without
  git). Firmware reports the same through `fw_info`. When touching a CLI that lacks it, add it.
