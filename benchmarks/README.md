# Benchmarks

This folder benchmarks `@antfu/ni` against this repo's production build (`target/release/hni`) using `hyperfine`.

## Requirements

- `cargo`
- `npm`
- `hyperfine`
- `bun` (optional, only needed for bun lockfile cases)

Install `hyperfine` on macOS:

```bash
brew install hyperfine
```

## What it benchmarks

- Startup:
  - `ni --version`
- Command families with variants:
  - `ni` (plain, add pkg, dev dep, production flag, global add, frozen variants)
  - `nr` (dev with extra args, `--if-present`, build)
  - `nci`
  - `nlx`
  - `nun`
  - `nup` / `nu` (including `-i` for pnpm)
- Fixture environments:
  - npm lockfile
  - pnpm lockfile
  - yarn lockfile
  - bun lockfile
  - pnpm workspace subpackage

The benchmark uses synthetic fixture folders to avoid touching your project files.

## Run

From repo root:

```bash
./benchmarks/run.sh
```

Or:

```bash
npm run bench:ni
```

CI-style gate:

```bash
npm run bench:ni:ci
```

Cleanup artifacts:

```bash
npm run bench:ni:clean
```

## Options

Examples:

```bash
./benchmarks/run.sh --runs=50 --warmups=10
./benchmarks/run.sh --no-build
./benchmarks/run.sh --no-json
./benchmarks/run.sh --min-speedup=8
```

Supported flags:

- `--runs=<n>` measured runs per case (default `20`)
- `--warmups=<n>` warmup runs per case (default `5`)
- `--no-build` skip `cargo build --release`
- `--no-json` do not write result JSON files
- `--no-markdown` do not write markdown summary file
- `--min-speedup=<x>` fail if command geometric mean speedup (excluding startup) is below threshold (example: `8`)

## Output

- Human-readable summary in terminal
- Merged JSON: `benchmarks/results/benchmark-<timestamp>.json`
- Markdown summary: `benchmarks/results/benchmark-<timestamp>.md`
- Raw per-case `hyperfine` JSON: `benchmarks/results/raw/`

## Notes

- Script installs/updates `@antfu/ni` in `benchmarks/.cache/antfu-ni`.
- Cases that require missing binaries are auto-skipped and listed in output.
- Cases that return non-zero during warmup/measurement are auto-skipped and listed (for example, `na` on some lockfile types with current `@antfu/ni` behavior).
- The shell wrapper avoids local `node` shim collisions by preferring a non-`~/.local/bin/node` binary when available.
