# hni

`hni` is a fast multicall package manager router inspired by Antfu's `ni` command family.

One binary provides:

- `hni`
- `ni`, `nr`, `nlx`, `nu`, `nun`, `nci`, `na`, `np`, `ns`
- `node` (shim mode)

## Install

### Homebrew (recommended)

```bash
brew tap happytoolin/happytap
brew install hni
hni --version
```

### Script install (Unix)

```bash
curl -fsSL https://happytoolin.com/hni/install.sh | bash
```

Fallback:

```bash
curl -fsSL https://raw.githubusercontent.com/happytoolin/hni/main/install.sh | bash
```

Environment overrides:

- `HNI_VERSION` (default: `latest`, accepts `v0.0.1` or `0.0.1`)
- `HNI_INSTALL_DIR` (default: `~/.local/bin`)
- `HNI_BASE_URL` (default: `https://happytoolin.com`)

### Script install (Windows PowerShell)

```powershell
irm https://happytoolin.com/hni/install.ps1 | iex
```

Fallback:

```powershell
irm https://raw.githubusercontent.com/happytoolin/hni/main/install.ps1 | iex
```

Optional parameters:

- `-Version latest`
- `-InstallDir "$env:LOCALAPPDATA\hni\bin"`
- `-BaseUrl "https://happytoolin.com"`

## Quick usage

```bash
hni help ni          # command-specific help
hni ni --help        # same as above
hni ni -- --help     # forward --help to npm/yarn/pnpm/bun/deno

ni                  # install dependencies
ni vite             # add dependency
ni -g eslint        # global install
ni --frozen         # lockfile-focused install

nr dev              # run script
nr                  # interactive script picker

nlx vitest          # execute package binary

nu                  # upgrade dependencies
nu --interactive    # interactive upgrade where supported

nun lodash          # uninstall dependency
nci                 # clean install
na                  # print detected package manager

np "pnpm dev" "pnpm test"   # run shell commands in parallel
ns "pnpm lint" "pnpm test"  # run shell commands sequentially

hni doctor          # inspect runtime + detection state
hni completion zsh  # print completion script
```

## Global flags

```bash
? --dry-run --print-command  # print resolved command (deprecated: prefer --debug-resolved)
--explain                    # print explain/debug block
-C <dir>                     # run as if in <dir>
-v --version                 # print versions
-h --help                    # print help
```

## Configuration

Config file locations:

- `~/.hnirc` (preferred)
- `~/.nirc` (legacy fallback)

Supported file keys:

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

Legacy `NI_*` variants are also accepted.

Internal/testing env vars:

- `HNI_SKIP_PM_CHECK`
- `HNI_REAL_NODE`
- `HNI_NODE_SHIM_ACTIVE`

## Node shim behavior

When invoked as `node`, `hni` routes package-manager verbs and batch shortcuts (`p` / `s`) while passing regular Node usage through to the real Node binary.

## Release automation

Tag pushes matching `v*` run `.github/workflows/release.yml` to:

1. Build release binaries for Linux, macOS, and Windows targets.
2. Publish GitHub release assets and `SHA256SUMS`.
3. Update `Formula/hni.rb` in `happytoolin/homebrew-happytap`.

Required secret for tap publishing:

- `HOMEBREW_TAP_GITHUB_TOKEN`

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```
