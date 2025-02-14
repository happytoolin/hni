# nirs

A Rust implementation of the [ni](https://github.com/antfu/ni) command-line tool for simplified package management. It automatically detects your package manager (npm, yarn, pnpm, etc.) and provides a unified interface for common operations.

## Installation

To install `nirs` and all its associated binaries (na, ni, nlx, nr, nun) using a single command, follow these steps:

1. **Run the following command** in your terminal:

   - **Linux/macOS:**

     ```bash
     curl -sSL https://github.com/spa5k/nirs/releases/latest/download/install.sh | bash
     ```

   - **Windows:**

     ```powershell
     iex ((New-Object System.Net.WebClient).DownloadString('https://github.com/spa5k/nirs/releases/latest/download/install.ps1'))
     ```

     (Run PowerShell as Administrator if you want to install to `C:\Program Files\nirs`)

   This command will:

   - Download the installation script.
   - Detect your operating system and architecture.
   - Download all the correct binaries from the latest release.
   - Install the binaries to a suitable location (`~/bin` on Linux/macOS, `~\AppData\Local\Programs\nirs` on Windows, or a system-wide directory if necessary).
   - Add the installation directory to your PATH environment variable.

2. **Open a new terminal** or restart your system for the PATH changes to take effect.

Alternatively, you can install `nirs` directly from cargo:

```bash
cargo install nirs
```

However, this will only install the `nirs` binary itself, and not the associated binaries (na, ni, nlx, nr, nun). You will need to install those separately using the method described above.

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
