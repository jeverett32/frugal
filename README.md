<h1 align="center">frugal</h1>

<p align="center">
  <strong>Cache-aware context packing for AI-assisted development.</strong>
</p>

<p align="center">
  `fgl` builds deterministic prompt context so more of your expensive prefix stays stable across runs.
</p>

<p align="center">
  <a href="https://github.com/jeverett32/frugal/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/jeverett32/frugal/ci.yml?branch=main&label=ci" alt="CI">
  </a>
  <a href="#license">
    <img src="https://img.shields.io/badge/license-MIT-4b5563" alt="MIT License">
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/rust-2021-000000?logo=rust" alt="Rust 2021">
  </a>
</p>

`frugal` is a local CLI for assembling AI context in a stable order:

1. Foundation
2. Secondary Skeletons
3. Active Zone

It is not a proxy. It does not send network requests. It does not sit between you and a model provider.

It gives you a deterministic `CONTEXT.md` or stdout stream that is easier to cache, cheaper to resend, and easier to reason about.

## Why This Exists

Most AI coding workflows pay repeatedly for broad repo context that barely changes.

`frugal` attacks that problem by separating:

- stable, high-value project context
- compact language skeletons for broad orientation
- raw active file bodies only where exact source matters

That keeps more of the prompt prefix stable across requests, which is the part most likely to benefit from provider-side prompt caching.

## What `fgl` Does

### `fgl init`

Bootstraps repo for `frugal` use.

- creates `.fgl/config.toml`
- pins `AGENTS.md` and `CLAUDE.md` into Foundation by default
- writes managed instructions into `AGENTS.md` and `CLAUDE.md`
- safe to rerun

### `fgl pack [PATH...]`

Builds markdown context pack in fixed order:

1. Foundation
2. Secondary Skeletons
3. Active Zone

Output goes to stdout by default or to a file with `--output`.

### `fgl status [PATH...]`

Prints one-line pack summary:

```text
prefix=123 active=18 ratio=6.83 files=27 langs=4
```

Token estimate uses `ceil(bytes / 4)`.

## Install

Install from local checkout:

```bash
cargo install --path .
```

Verify:

```bash
fgl --help
```

## Quick Start

Initialize repo once:

```bash
fgl init
```

Check current pack shape:

```bash
fgl status src/main.rs
```

Write context file:

```bash
fgl pack --output CONTEXT.md src/main.rs src/lib.rs
```

Pipe directly:

```bash
fgl pack src/main.rs src/lib.rs
```

## Example Workflow

Typical loop:

1. run `fgl status` before starting work
2. run `fgl pack <active-files...>` instead of reading broad repo state raw
3. keep Foundation stable
4. read raw files only when exact write/edit context is needed

That is the behavior `fgl init` writes into managed agent docs.

## Config

Default config:

```toml
version = 1

[foundation]
pinned = ["AGENTS.md", "CLAUDE.md"]

[languages]
enabled = ["python", "rust", "javascript", "typescript", "go"]
```

Rules:

- `foundation.pinned` order is preserved
- `languages.enabled` controls which languages appear in Secondary Skeletons
- Active files always stay raw and always render last

## Language Support

Current real skeletonizers:

- Python
- Rust
- Go
- JavaScript
- TypeScript

Skeleton output focuses on high-signal structure:

- function and method signatures
- classes, structs, interfaces, enums, traits, type aliases
- attached docs where supported
- top-level constants / statics / declarations

## Output Contract

`fgl pack` renders stable markdown sections:

```text
# Foundation
# Secondary Skeletons
# Active Zone
```

Each file renders as:

````text
## `path/to/file`
```lang
...
```
````

Line endings normalize to LF before rendering. Fence width expands automatically if body content already contains backticks.

## Development

Core local checks:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

In this dev environment, local C compilation may also need:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/tmp/zig-tools:$PATH"
export CFLAGS_x86_64_unknown_linux_gnu="--target=x86_64-linux-gnu.2.39"
```

## Contributing

Start with [CONTRIBUTING.md](/home/everjohn/projects/frugal/CONTRIBUTING.md).

## Security

See [SECURITY.md](/home/everjohn/projects/frugal/SECURITY.md).

## License

MIT. See [LICENSE](/home/everjohn/projects/frugal/LICENSE).
