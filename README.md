# hni

![hni og banner](.github/og-image.png)

[![CI](https://github.com/happytoolin/hni/actions/workflows/ci.yml/badge.svg)](https://github.com/happytoolin/hni/actions/workflows/ci.yml)
[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-4F46E5.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![crates.io](https://img.shields.io/crates/v/hni?logo=rust&logoColor=white)](https://crates.io/crates/hni)
[![npm](https://img.shields.io/npm/v/%40happytoolin%2Fhni?logo=npm&logoColor=white)](https://www.npmjs.com/package/@happytoolin/hni)
![npm](https://img.shields.io/badge/npm-supported-CB3837?logo=npm&logoColor=white)
![yarn](https://img.shields.io/badge/yarn-supported-2C8EBB?logo=yarn&logoColor=white)
![pnpm](https://img.shields.io/badge/pnpm-supported-F69220?logo=pnpm&logoColor=white)
![bun](https://img.shields.io/badge/bun-supported-111111?logo=bun&logoColor=white)
![deno](https://img.shields.io/badge/deno-supported-000000?logo=deno&logoColor=white)

Fast package manager routing for `npm`, `yarn`, `pnpm`, `bun`, and `deno`.

`hni` is inspired by Antfu's [`ni`](https://github.com/antfu-collective/ni#readme), but packaged as a single multicall binary with extra shell setup for a `node` shim.

`hni` is still beta software and may have bugs.

One install gives you:

- `hni`
- `ni`, `nr`, `nlx`, `nu`, `nun`, `nci`, `na`, `np`, `ns`
- `node` shim via `hni init <shell>` (shell plugin only)

## Install

### npm (global)

```bash
npm install -g @happytoolin/hni
hni --version
```

This installs `hni` and the `ni`-family aliases (`ni`, `nr`, `nlx`, `nu`, `nun`, `nci`, `na`, `np`, `ns`) onto your global npm bin path.
The `node` shim is only enabled through `hni init <shell>`.
Under the hood, npm resolves a platform-specific optional dependency package that contains the native `hni` binary.

### Homebrew

```bash
brew tap happytoolin/happytap
brew install hni
hni --version
```

### Script install (macOS / Linux)

TODO: `https://happytoolin.com/hni/install.sh` is not live yet. Use the raw GitHub script for now:

```bash
curl -fsSL https://raw.githubusercontent.com/happytoolin/hni/main/install.sh | bash
```

Optional environment variables:

- `HNI_VERSION` - install a specific version, for example `v0.0.2`
- `HNI_INSTALL_DIR` - install somewhere other than `~/.local/bin`
- `HNI_NODE=off` - disable the `node` shim for the current environment

### Script install (PowerShell)

TODO: `https://happytoolin.com/hni/install.ps1` is not live yet. Use the raw GitHub script for now:

```powershell
irm https://raw.githubusercontent.com/happytoolin/hni/main/install.ps1 | iex
```

Optional parameters:

- `-Version latest`
- `-InstallDir "$env:LOCALAPPDATA\hni\bin"`

### Deno / JSR

Install `hni`:

```bash
deno install -gA -n hni jsr:@happytoolin/hni/hni
hni --version
```

Install alias commands (example):

```bash
deno install -gA -n ni jsr:@happytoolin/hni/ni
deno install -gA -n nr jsr:@happytoolin/hni/nr
```

## Commands

### `ni`

Install dependencies or add new ones.

```bash
ni
ni vite
ni -D vitest
ni -g eslint
ni --frozen
ni --frozen-if-present
ni --interactive
```

### `nr`

Run package scripts.

```bash
nr
nr dev
nr build
nr test -- --watch
nr --if-present lint
nr --repeat-last
```

### `nlx`

Execute binaries without adding them permanently to your project.

```bash
nlx vitest
nlx eslint .
nlx create-vite@latest
```

### `nu`

Upgrade dependencies.

```bash
nu
nu react react-dom
nu --interactive
```

### `nun`

Remove dependencies.

```bash
nun lodash
nun react react-dom
nun --multi-select
nun -g typescript
```

### `nci`

Run a clean install. If a lockfile exists, `hni` uses the package-manager-specific frozen install command.

```bash
nci
```

### `na`

Print or forward directly to the detected package manager.

```bash
na --version
na config get registry
```

### `np` / `ns`

Run shell commands in parallel or sequentially.

```bash
np "pnpm dev" "pnpm test"
ns "pnpm lint" "pnpm test"
```

### `node`

`hni` can also act as a package-manager-aware `node` shim.
Enable it by adding `hni init <shell>` to your shell config first.

```bash
node install vite
node run dev
node exec vitest
node ci
node p "echo one" "echo two"
```

Regular Node.js usage still passes through:

```bash
node script.js
node -v
node -- --trace-warnings
```

### Utilities

```bash
hni help ni
hni completion zsh
hni init bash
hni doctor
```

## Shell Setup

If you want node-shim behavior, add the init line at the end of your shell config file, after anything that manages Node or rewrites `PATH`, such as `nvm`, `mise`, `asdf`, `fnm`, or `volta`.

Do not append the `hni` directory to the end of `PATH`. Put the init line at the end of the shell config file and let it prepend the correct path for you.

### zsh

Add to `~/.zshrc`:

```bash
eval "$(hni init zsh)"
```

### bash

Add to `~/.bashrc`:

```bash
eval "$(hni init bash)"
```

### fish

Add to `~/.config/fish/config.fish`:

```fish
hni init fish | source
```

### PowerShell

Add to `$PROFILE`:

```powershell
Invoke-Expression (& hni init powershell)
```

### Nushell

Generate a stable init file, then source it from the end of `~/.config/nushell/config.nu`:

```nu
hni init nushell | save --force ~/.config/nushell/hni.nu
source ~/.config/nushell/hni.nu
```

## Global Flags

These work across `hni` and the multicall aliases:

```bash
? --dry-run --print-command
--explain
-C <dir>
-v --version
-h --help
```

Use `--` to forward flags to the underlying package manager or script:

```bash
hni ni -- --help
nr test -- --watch
```

## Configuration

Config file:

- `~/.hnirc`

Supported keys:

```ini
defaultAgent=prompt
globalAgent=npm
runAgent=node
useSfw=false
```

Environment overrides:

- `HNI_CONFIG_FILE`
- `HNI_DEFAULT_AGENT`
- `HNI_GLOBAL_AGENT`
- `HNI_USE_SFW`
- `HNI_AUTO_INSTALL`

## How It Works

`hni` detects the package manager from:

1. `packageManager` in `package.json`
2. lockfiles such as `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`, `bun.lockb`, or `deno.lock`
3. config defaults if detection is unavailable

Then it maps the command family to the right native command:

- `ni` -> install or add
- `nr` -> run or task
- `nlx` -> `npx` / `pnpm dlx` / `yarn dlx` / `bun x`
- `nu` -> update / upgrade
- `nci` -> frozen install when lockfiles exist

## Troubleshooting

### PowerShell `ni` alias conflict

PowerShell ships with a built-in `ni` alias for `New-Item`.

If that conflicts with `hni`, remove or override it in your profile before loading `hni`:

```powershell
Remove-Item Alias:ni -ErrorAction SilentlyContinue
Invoke-Expression (& hni init powershell)
```

### Check what `hni` resolved

```bash
ni vite --debug-resolved
nr dev --explain
hni doctor
```

## Benchmarking

The active benchmark suite lives in [`benchmark/`](benchmark/).

If you use [`just`](https://github.com/casey/just), the common local commands are wrapped in [`justfile`](justfile):

```bash
just build-release
just test
just test-native
just ci
just bench
```

Run all benchmark tracks with:

```bash
./benchmark/run.sh
just bench
```

Run individual tracks with:

```bash
just bench-compare
just bench-native
just bench-runtime

./benchmark/run.sh --track=compare
./benchmark/run.sh --track=native
./benchmark/run.sh --track=runtime
```

Generate flamegraphs with:

```bash
./benchmark/profile.sh
```

Tracked benchmark docs:

- current snapshot: [`benchmark/LATEST.md`](benchmark/LATEST.md)
- lightweight history: [`benchmark/HISTORY.md`](benchmark/HISTORY.md)
- native compatibility: [`docs/native-compat.md`](docs/native-compat.md)

Same-repo pull requests also run the benchmark workflow on GitHub Actions and update a sticky PR comment with the latest summary.

### Representative Results

The benchmark runner generates per-run Markdown and JSON under `benchmark/results/`, but those raw artifacts are treated as local output rather than long-lived repo data.

For commits and releases, use the tracked summary files above instead of linking directly to generated result files.

The current tracked snapshot is from March 22, 2026. It uses `5` warmups and `20` measured runs per case. The previous March 20, 2026 tracked snapshot only covered the native track with a single measured run, so the delta callouts below are useful directionally but not as a controlled apples-to-apples regression study.

Native mode is where `hni` shows the clearest gains in the current snapshot:

| Case | Delegated | Native | Gain |
| --- | ---: | ---: | ---: |
| `nr noop (npm)` | 399.19 ms | 55.25 ms | 7.23x |
| `nr noop (pnpm)` | 922.51 ms | 77.18 ms | 11.95x |
| `node run noop (pnpm)` | 741.88 ms | 70.09 ms | 10.58x |
| `nlx hello --flag (npm local bin)` | 363.95 ms | 7.55 ms | 48.21x |

Overall native-mode geometric mean in the current tracked snapshot: `3.65x` on March 22, 2026, versus `5.02x` on March 20, 2026.

Representative March 20 -> March 22 native deltas:

| Case | March 20 | March 22 |
| --- | ---: | ---: |
| `nr noop (npm)` | 7.67x | 7.23x |
| `nr noop (pnpm)` | 15.61x | 11.95x |
| `node run noop (pnpm)` | 12.23x | 10.58x |
| `nlx hello --flag (npm local bin)` | 65.70x | 48.21x |

The March 22 snapshot also adds tracked `compare` and `runtime` runs. The runtime track keeps `bun` and `deno` separate from the Antfu comparison and focuses on a small fair set of task-style commands:

| Case | `hni` | `bun` | `deno` |
| --- | ---: | ---: | ---: |
| `task noop` | 43.51 ms | 50.80 ms | 39.68 ms |
| `task hooks` | 107.08 ms | 203.35 ms | 69.51 ms |

For CLI/startup-oriented comparisons against Antfu's `ni`, the March 22 `compare` track averaged `1.46x` in `hni`'s favor overall, but the results remain command-shaped and intentionally small:

| Case | `antfu` | `hni` | Relative |
| --- | ---: | ---: | ---: |
| `ni --version` | 451.62 ms | 336.74 ms | 1.34x |
| `ni vite ? (npm)` | 6.91 ms | 11.60 ms | 0.60x |
| `nr build ? (pnpm)` | 9.26 ms | 2.46 ms | 3.77x |
| `nlx vitest ? (npm)` | 6.74 ms | 4.48 ms | 1.51x |

The `compare` track is intentionally small and should be read as a lightweight sanity check, not a broad claim about every workflow.

### Methodology

- All CLI timing uses `hyperfine`.
- `hni` is measured with the release binary from `target/release/hni`.
- `compare` benchmarks `@antfu/ni` against `hni` on a very small CLI-focused set.
- `native` benchmarks `HNI_NATIVE=false` vs `HNI_NATIVE=true`.
- `runtime` benchmarks `hni`, `bun`, and `deno` separately from the Antfu comparison to keep the chart fair.
- Raw benchmark outputs are generated locally in `benchmark/results/`; the repo keeps the curated Markdown snapshots instead of storing every generated result file.
