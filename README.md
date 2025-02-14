# nirs

A Rust implementation of the [ni](https://github.com/antfu/ni) command-line tool for simplified package management. It automatically detects your package manager (npm, yarn, pnpm, etc.) and provides a unified interface for common operations.

## Installation

```bash
cargo install nirs
```

## Commands

- `ni` - Install packages
  ```bash
  ni           # Install all dependencies
  ni react     # Install react
  ni react -D  # Install react as dev dependency
  ni -g yarn   # Install yarn globally
  ```

- `nr` - Run scripts
  ```bash
  nr           # Run default script (start)
  nr dev       # Run dev script
  nr build     # Run build script
  ```

- `nlx` - Execute packages (like npx)
  ```bash
  nlx vitest   # Run vitest
  nlx prettier . --write  # Run prettier
  ```

- `nu` - Upgrade packages
  ```bash
  nu           # Upgrade all packages
  nu react     # Upgrade react
  ```

- `nun` - Uninstall packages
  ```bash
  nun react    # Uninstall react
  ```

- `nci` - Clean install
  ```bash
  nci          # Clean install (like npm ci)
  ```

- `na` - Show agent info
  ```bash
  na           # Show detected package manager and its version
  ```
  Example output:
  ```
  INFO  10:00:00 na Using package manager: npm
  INFO  10:00:00 na Getting version information
  INFO  10:00:00 na Executing: npm with args: ["--version"]
  9.6.7
  ```

## Package Manager Detection

The tool automatically detects your package manager based on lockfiles in the following order:
1. `bun.lock` or `bun.lockb` → Bun
2. `pnpm-lock.yaml` → PNPM
3. `yarn.lock` → Yarn
4. `package-lock.json` or `npm-shrinkwrap.json` → npm
5. Fallback to npm if no lockfile is found

## Logging

The tool provides detailed logging with different verbosity levels. You can control the log level using the `LOG_LEVEL` environment variable:

```bash
# Available levels (from least to most verbose):
LOG_LEVEL=error cargo run --bin ni react  # Only errors (red)
LOG_LEVEL=warn cargo run --bin ni react   # Warnings (yellow) and errors
LOG_LEVEL=info cargo run --bin ni react   # Normal output (green) - default
LOG_LEVEL=debug cargo run --bin ni react  # Detailed debugging (cyan)
LOG_LEVEL=trace cargo run --bin ni react  # Very detailed debugging (white)
```

The logs include:
- Log level with appropriate color
- Timestamp
- Module name
- Detailed operation information

Example debug output:
```
DEBUG 19:04:51 nirs::logger Logger initialized with environment: LOG_LEVEL=debug
DEBUG 19:04:51 ni Current working directory: /path/to/project
DEBUG 19:04:51 ni Parsed arguments: ["react", "-D"]
INFO  19:04:51 ni Installing packages with arguments: ["react", "-D"]
INFO  19:04:51 nirs::detect Detecting package manager in directory: /path/to/project
DEBUG 19:04:51 nirs::detect Checking for 6 known lockfile patterns
INFO  19:04:51 nirs::detect Found package manager npm (lockfile: package-lock.json)
DEBUG 19:04:51 nirs::detect Creating factory for package manager: npm
INFO  19:04:51 ni Executing: npm with args: ["install", "react", "-D"]
```

## Development

Requirements:
- Rust 1.70 or later
- Cargo

Building:
```bash
cargo build
```

Running tests:
```bash
cargo test
```

## License

MIT 