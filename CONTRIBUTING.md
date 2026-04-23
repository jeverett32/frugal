# Contributing

Thanks for contributing to `frugal`.

## Before Opening Work

- open an issue first for non-trivial changes
- keep scope tight
- prefer small PRs over large refactors
- add or update tests for behavior changes

## Local Setup

```bash
git clone https://github.com/jeverett32/frugal.git
cd frugal
cargo test
```

In this environment, local builds may also need:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/tmp/zig-tools:$PATH"
export CFLAGS_x86_64_unknown_linux_gnu="--target=x86_64-linux-gnu.2.39"
```

## Required Checks

Before opening a PR, run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Project Shape

High-level modules:

- `src/init.rs` bootstraps `.fgl/` and managed docs
- `src/discovery.rs` selects Foundation, Secondary, Active files
- `src/pack.rs` materializes and renders markdown output
- `src/status.rs` computes one-line pack metrics
- `src/languages/` contains per-language skeletonizers

## Adding Language Support

New language work should:

1. keep output deterministic
2. prefer omission over unstable heuristics
3. extract high-signal structure only
4. add fixture-backed tests under `tests/fixtures/languages/<lang>/`
5. add integration test file in `tests/`

## Style

- preserve stable ordering
- avoid hidden network behavior
- keep CLI surface small
- prefer explicit tests for contracts and regressions
