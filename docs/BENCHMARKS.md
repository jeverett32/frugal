# Benchmarks

`frugal` should be evaluated on two axes:

1. how much context size drops versus naive raw-file packing
2. whether the pack prefix stays stable when only active files change

## What To Measure

- raw repo context estimated tokens
- `fgl pack` estimated tokens
- change in estimated tokens after a small active-file edit
- whether the pack prefix before `# Active Zone` stays byte-identical

## Benchmark Script

From repo root:

```bash
cargo build
python3 scripts/benchmark.py /path/to/repo path/to/active_file.py --fgl target/debug/fgl
```

Output is JSON with:

- raw token estimate before edit
- pack token estimate before edit
- raw token estimate after edit
- pack token estimate after edit
- SHA-256 hashes
- `pack_prefix_unchanged`

## Suggested Benchmark Repos

- medium Python repo
- medium TypeScript repo
- mixed-language monorepo
- one of your real daily-driver repos

## What Good Looks Like

Good benchmark result usually means:

- `pack_before_tokens` much smaller than `raw_before_tokens`
- `pack_prefix_unchanged = true` after small active-file edits
- raw snapshot hash changes broadly while packed prefix stays stable

## Notes

- token estimates here use the same rough `ceil(bytes / 4)` heuristic as `fgl status`
- this is not provider billing truth
- it is still useful for comparing relative prompt size and stability
