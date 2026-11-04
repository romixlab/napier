# List all recipes
default:
    @just --list

# The tests. `just test`: all of them (test-all), the full suite run before landing a change. `just test --changed`:
# only what changed against main, for a quick check before finishing a change.
test *flags:
    if [ "{{flags}}" = "--changed" ]; then \
        if command -v tpm >/dev/null 2>&1; then tpm test changed -- cargo nextest run --workspace; else cargo nextest run --workspace; fi; \
    else {{just_executable()}} test-all {{flags}}; fi

# Tests (nextest for the unit and integration tests, cargo test for doctests)
test-all:
    cargo nextest run
    cargo test --doc

# Clippy (no warnings) and a format check
lint:
    cargo clippy --all-targets -- -D warnings
    cargo fmt --check

# si_dynamic is a library with no binary, so there is nothing to put on PATH: install is a release build check
install:
    cargo build --release

# Same as install: bring this PC up to date with main
deploy: install
