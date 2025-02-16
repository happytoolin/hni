# Tech Context: nirs

**Technologies Used:**

- **Language:** Rust
- **Build System:** Cargo
- **Package Managers (Supported):** npm, yarn, pnpm, bun
- **Testing:** `cargo test`
- **Logging:** Custom logging module (`src/logger.rs`)
- **Configuration:** `config` crate for `.nirs` file (TOML, YAML, JSON, etc.), environment variables. See `memory_bank/config.md` for details.

**Development Setup:**

- **Requirements:** Rust 1.70 or later, Cargo
- **Building:** `cargo build`
- **Running Tests:** `cargo test`

**Technical Constraints:**

- The tool must be able to detect the package manager correctly based on lock files.
- The tool must be able to execute commands for different package managers.
- The tool should handle errors gracefully.
- The tool should be performant.
