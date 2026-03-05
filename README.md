# hni

`hni` is a multicall package-manager router inspired by the `ni` command family.

It provides one binary (`hni`) plus command aliases (`ni`, `nr`, `nlx`, `nu`, `nun`, `nci`, `na`, `np`, `ns`, `node`).

## Install

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/happytoolin/hni/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/happytoolin/hni/main/install.ps1'))
```

## Core Commands

```bash
ni                 # install dependencies
ni vite            # add dependency
ni -g eslint       # global install
ni --frozen        # frozen install

nr dev             # run script
nr                 # interactive script picker

nlx vitest         # execute package binary

nu                 # upgrade dependencies
nun lodash         # uninstall dependency
nci                # clean install (falls back when lockfile is missing)
na                 # print detected package manager

np "pnpm dev" "pnpm test"             # run shell commands in parallel
ns "pnpm lint" "pnpm test"            # run shell commands sequentially (stop on first failure)

node p "pnpm dev" "pnpm test"         # same as np via node shim
node s "pnpm lint" "pnpm test"        # same as ns via node shim
node -p "1 + 1"                       # passthrough to real node

hni doctor                            # inspect detection/config/runtime state
hni completion zsh                    # print shell completion script
```

## Global Flags

```bash
? --dry-run --print-command  # print resolved command instead of executing
--explain                    # print debug/explain block
-C <dir>                     # run as if in <dir>
-v --version                 # print versions
-h --help                    # print help
```

## Configuration

Config file: `~/.hnirc` (legacy fallback: `~/.nirc`)

```ini
defaultAgent=prompt
globalAgent=npm
runAgent=node
useSfw=false
```

Environment variables:

- `HNI_CONFIG_FILE`
- `HNI_DEFAULT_AGENT`
- `HNI_GLOBAL_AGENT`
- `HNI_USE_SFW`
- `HNI_AUTO_INSTALL`

Internal/testing env vars (not regular user config): `HNI_SKIP_PM_CHECK`, `HNI_REAL_NODE`.

Legacy `NI_*` config env names are still accepted for compatibility.

## Node Shim

When invoked as `node`, `hni` routes package-manager verbs (`install`, `run`, `add`, etc.), plus `p` and `s` for batch execution, and passes through normal Node usage (`node file.js`, `node -v`, `node -p`, `node -- ...`) to the real Node binary.

## Development

```bash
cargo fmt
cargo test
```
