# Benchmark Suite

This is the single active benchmark suite for `hni`.

It replaces the older benchmark trees, which are preserved under [`benchmark/old/`](old/).

## Tracks

- `compare`: `@antfu/ni` vs `hni` for a very small CLI/startup-oriented set
- `native`: `hni` delegated mode vs `hni` native mode
- `runtime`: `hni` vs `bun` vs `deno` for a few comparable task-running cases

All timing uses `hyperfine`.

## Requirements

- `cargo`
- `hyperfine`
- `npm`
- `pnpm` for `pnpm` fixture cases
- `yarn` for `yarn` fixture cases
- `bun` for `bun` fixture and runtime cases
- `deno` for `deno` fixture and runtime cases

Install `hyperfine` on macOS:

```bash
brew install hyperfine
```

Install flamegraph support:

```bash
cargo install flamegraph
```

## Run

Build release binary and run all tracks:

```bash
./benchmark/run.sh
```

Run one track:

```bash
./benchmark/run.sh --track=compare
./benchmark/run.sh --track=native
./benchmark/run.sh --track=runtime
```

Smaller local run:

```bash
./benchmark/run.sh --runs=3 --warmups=1 --no-build
```

Formats:

```bash
./benchmark/run.sh --format=table
./benchmark/run.sh --format=markdown
./benchmark/run.sh --format=json
```

## Profiling

Generate flamegraphs for the default `pnpm` native/delegated cases:

```bash
./benchmark/profile.sh
```

This writes SVGs into [`benchmark/profiles/`](profiles/).

## Output

- aggregated JSON results in [`benchmark/results/`](results/)
- per-run Markdown reports in [`benchmark/results/`](results/)
- raw per-case `hyperfine` JSON in `benchmark/results/raw/<track>/`
- latest tracked snapshot in [`benchmark/LATEST.md`](LATEST.md)
- lightweight run index in [`benchmark/HISTORY.md`](HISTORY.md)
- flamegraph SVGs in [`benchmark/profiles/`](profiles/)

## Notes

- `compare` is intentionally tiny and presentational.
- `native` is the engineering benchmark for native-mode wins and regressions.
- `runtime` keeps `bun` and `deno` separate from the Antfu comparison so the story stays fair.
