# frugal

`frugal` (`fgl`) builds stable, cache-friendly context packs for AI-assisted development.

It is not a proxy. It scans repo, keeps broad context in deterministic skeleton form, leaves active files raw, then prints markdown to stdout or writes `CONTEXT.md`.

## Install

From repo checkout:

```bash
cargo install --path .
```

Then verify:

```bash
fgl --help
```

## Commands

### `fgl init`

Bootstraps repo for `frugal` use.

- creates `.fgl/config.toml`
- pins `AGENTS.md` and `CLAUDE.md` into Foundation by default
- writes managed `frugal` instructions into `AGENTS.md` and `CLAUDE.md`
- safe to rerun

### `fgl pack [PATH...]`

Builds markdown pack in fixed order:

1. Foundation
2. Secondary Skeletons
3. Active Zone

Default writes to stdout:

```bash
fgl pack src/main.rs
```

Write file directly:

```bash
fgl pack --output CONTEXT.md src/main.rs src/lib.rs
```

Shell redirect still works:

```bash
fgl pack src/main.rs > CONTEXT.md
```

### `fgl status [PATH...]`

Prints one-line summary:

```text
prefix=123 active=18 ratio=6.83 files=27 langs=4
```

Meaning:

- `prefix`: estimated tokens in Foundation + Secondary
- `active`: estimated tokens in Active Zone
- `ratio`: `prefix / active`
- `files`: selected file count
- `langs`: selected language count

Token estimate uses `ceil(bytes / 4)`.

## Config

`fgl init` creates:

```toml
version = 1

[foundation]
pinned = ["AGENTS.md", "CLAUDE.md"]

[languages]
enabled = ["python", "rust", "javascript", "typescript", "go"]
```

`foundation.pinned` order preserved.

`languages.enabled` controls which source languages enter Secondary Skeletons.

## Skeleton coverage

Current real extractors:

- Python
- Rust
- Go
- JavaScript
- TypeScript

Secondary files stay deterministic and compact. Active files remain raw.

## Development

Run tests:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/tmp/zig-tools:$PATH"
export CFLAGS_x86_64_unknown_linux_gnu="--target=x86_64-linux-gnu.2.39"
cargo test
```

