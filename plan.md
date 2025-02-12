Below is the final, detailed plan for porting the TypeScript `ni` tool to Rust. Each major section includes a confidence score (from 1 to 10) reflecting our confidence in the approach and design choices based on experience and best practices.

---

## 1. Project Setup & Structure  
**Confidence Score: 9/10**

### 1.1. Initialize the Cargo Project  
- **Steps:**
  1. Open your terminal.
  2. Create a new binary project:
     ```bash
     cargo new ni-rs --bin
     cd ni-rs
     ```
  3. Run a simple `cargo run` to ensure the project builds.
- **Rationale:**  
  A standard Cargo project setup is well understood and provides a solid starting point.

### 1.2. Directory and File Organization  
- **Proposed Layout:**
  ```
  ni-rs/
  ├── Cargo.toml
  ├── src/
  │   ├── main.rs              // CLI entry point
  │   ├── lib.rs               // Shared library logic (if needed)
  │   ├── config.rs            // Configuration management
  │   ├── detect.rs            // Package manager detection
  │   ├── parse.rs             // Command parsing & translation
  │   ├── runner.rs            // Command execution
  │   ├── fetch.rs             // NPM package search
  │   ├── fs.rs                // File system utilities
  │   ├── storage.rs           // Command history and caching
  │   ├── utils.rs             // Utilities (error types, logging, etc.)
  │   ├── completion.rs        // Shell completions
  │   └── commands/            // Command-specific implementations
  │       ├── mod.rs           // Command dispatcher
  │       ├── ni.rs            // Install command
  │       ├── nr.rs            // Run command
  │       ├── nci.rs           // Clean install command
  │       ├── na.rs            // Agent alias command
  │       ├── nu.rs            // Upgrade command
  │       ├── nun.rs           // Uninstall command
  │       └── nlx.rs           // Execute command
  ```
- **Rationale:**  
  This modular structure separates concerns clearly. It enables parallel development, improves maintainability, and scales well as features are added.

---

## 2. Core Dependencies  
**Confidence Score: 9/10**

### 2.1. Dependency List  
Add the following to your `Cargo.toml`:
```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "env"] }     # CLI parsing
tokio = { version = "1.0", features = ["full"] }              # Async runtime
reqwest = { version = "0.11", features = ["json"] }           # HTTP client
serde = { version = "1.0", features = ["derive"] }            # Serialization/deserialization
serde_json = "1.0"                                            # JSON parsing
config = "0.13"                                               # Configuration file management
dialoguer = "0.11"                                            # Interactive CLI prompts
inquire = "0.6.2"                                             # Alternative interactive CLI library
skim = "0.10"                                                 # Fuzzy search integration
console = "0.15.7"                                            # Terminal formatting
duct = "0.13.6"                                               # Command execution
which = "4.4.0"                                               # Binary lookup in PATH
directories = "5.0.1"                                         # OS-specific directories
tempfile = "3.8.0"                                            # Temporary file creation
async-trait = "0.1.74"                                        # Async trait support
lazy_static = "1.4.0"                                          # Static initialization
anyhow = "1.0.75"                                              # Error handling
thiserror = "1.0.50"                                           # Custom error definitions
```
- **Rationale:**  
  These libraries are standard choices in the Rust ecosystem. They cover CLI parsing, async operations, configuration, interactivity, and robust error handling.

---

## 3. Detailed Implementation Components  
**Overall Confidence Score for Component Designs: 8/10**

### 3.1. Configuration Management  
**File:** `src/config.rs`  
- **Objective:** Load user settings from configuration files (e.g., `~/.nirc`) and environment variables, with default values.
- **Plan:**
  1. Define a configuration struct using Serde.
  2. Implement a load function that merges file and environment settings.
- **Example Code:**
  ```rust
  use config::Config;
  use serde::Deserialize;

  #[derive(Debug, Deserialize)]
  pub struct NiConfig {
      #[serde(default = "default_agent")]
      pub default_agent: String,
      #[serde(default = "global_agent")]
      pub global_agent: String,
  }

  fn default_agent() -> String { "prompt".into() }
  fn global_agent() -> String { "npm".into() }

  impl NiConfig {
      pub fn load() -> anyhow::Result<Self> {
          let mut settings = Config::new();
          // Optionally load from "~/.nirc"
          // settings.merge(config::File::with_name("~/.nirc")).ok();
          // Merge environment variables prefixed with NI
          // settings.merge(config::Environment::with_prefix("NI")).ok();
          settings.try_into().map_err(Into::into)
      }
  }
  ```
- **Confidence:**  
  **8/10** – This approach is standard and leverages popular crates, though real-world configuration merging might require additional error handling.

### 3.2. Package Manager Detection  
**File:** `src/detect.rs`  
- **Objective:** Determine the active package manager (npm, yarn, pnpm, or bun) by checking for specific lock files and the existence of binaries.
- **Plan:**
  1. Check for lock files like `pnpm-lock.yaml` or `yarn.lock`.
  2. Use the `which` crate as a fallback to detect globally installed tools.
- **Example Code:**
  ```rust
  use which::which;
  use std::path::Path;

  pub enum PackageManager {
      Npm,
      Yarn,
      Pnpm,
      Bun,
  }

  impl PackageManager {
      pub fn detect(cwd: &Path) -> Option<Self> {
          if cwd.join("pnpm-lock.yaml").exists() {
              return Some(Self::Pnpm);
          }
          if cwd.join("yarn.lock").exists() {
              return Some(Self::Yarn);
          }
          if which("npm").is_ok() {
              return Some(Self::Npm);
          }
          None
      }
  }
  ```
- **Confidence:**  
  **9/10** – The detection logic is straightforward and robust for most projects. It may require adjustments for edge cases (e.g., custom package managers).

### 3.3. Command Parsing & Translation  
**File:** `src/parse.rs`  
- **Objective:** Transform CLI arguments into a structured command, mapping shorthand flags to full options.
- **Plan:**
  1. Define a `ResolvedCommand` struct to hold the executable name and arguments.
  2. Convert flags like `-D` to `--save-dev` where applicable.
- **Example Code:**
  ```rust
  use crate::detect::PackageManager;

  pub struct ResolvedCommand {
      pub bin: String,
      pub args: Vec<String>,
  }

  pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
      match agent {
          PackageManager::Npm => {
              let args_str = args.join(" ");
              ResolvedCommand {
                  bin: "npm".into(),
                  args: vec!["install".into(), args_str],
              }
          }
          // Additional mapping for Yarn, Pnpm, etc.
      }
  }
  ```
- **Confidence:**  
  **8/10** – The concept is solid. Refinements may be necessary to handle more complex argument transformations or additional package managers.

### 3.4. Interactive Prompts & Package Search  
**File:** `src/commands/ni.rs`  
- **Objective:** Use interactive prompts (with `inquire` or `dialoguer`) for package selection and installation mode.
- **Plan:**
  1. Query the NPM registry (using an async function in `fetch.rs`).
  2. Present the results using fuzzy search and selection prompts.
- **Example Code:**
  ```rust
  use inquire::Select;
  use anyhow::Result;
  use crate::fetch::fetch_npm_packages;

  pub async fn interactive_install() -> Result<()> {
      let packages = fetch_npm_packages("react").await?;
      
      let package_choice = Select::new("Choose a package:", packages)
          .with_page_size(15)
          .prompt()?;
      
      let mode = Select::new("Install as:", vec!["prod", "dev", "peer"])
          .prompt()?;
      
      // Build command args based on selections
      Ok(())
  }
  ```
- **Confidence:**  
  **7/10** – Interactive CLI design is inherently subjective. This approach is flexible but might require UX refinements based on user feedback.

### 3.5. Command Execution  
**File:** `src/runner.rs`  
- **Objective:** Execute the constructed command safely and report errors if execution fails.
- **Plan:**
  1. Use the `duct` crate (or Rust’s standard `Command`) to spawn the process.
  2. Ensure proper error propagation if the command fails.
- **Example Code:**
  ```rust
  use duct::cmd;
  use crate::parse::ResolvedCommand;
  use std::path::Path;

  pub fn execute(command: ResolvedCommand, cwd: &Path) -> anyhow::Result<()> {
      cmd(command.bin, command.args)
          .dir(cwd)
          .run()?;
      Ok(())
  }
  ```
- **Confidence:**  
  **9/10** – Using `duct` is a reliable way to manage external processes, and this design cleanly encapsulates the execution logic.

### 3.6. Additional Modules and Enhancements

#### 3.6.1. File System & History Utilities  
- **Files:** `src/fs.rs` and `src/storage.rs`
- **Objective:** Implement common file operations and manage command history/caching.
- **Confidence:**  
  **8/10** – These utilities are straightforward; however, design decisions (like cache invalidation) may require iteration.

#### 3.6.2. Shell Completions  
- **File:** `src/completion.rs`
- **Objective:** Generate shell completion scripts using `clap`’s capabilities.
- **Confidence:**  
  **9/10** – Clap provides built-in support for completions, making this feature highly reliable.

#### 3.6.3. Error Handling & Utility Functions  
- **File:** `src/utils.rs` (or `src/utils/error.rs`)
- **Objective:** Define custom error types using `thiserror` to standardize error handling.
- **Example Code:**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum NiError {
      #[error("Package manager detection failed")]
      DetectionFailure,
      
      #[error("Unsupported command for {agent}: {command}")]
      UnsupportedCommand {
          agent: String,
          command: String,
      },
      
      #[error("I/O error: {0}")]
      Io(#[from] std::io::Error),
  }
  ```
- **Confidence:**  
  **9/10** – Using `thiserror` and `anyhow` for error management is a best practice in Rust projects.

#### 3.6.4. Volta Integration and Async Support  
- **File:** `src/exec/volta.rs` (if applicable)
- **Objective:** Optionally integrate Volta or similar tools; leverage async/await for network and I/O operations.
- **Confidence:**  
  **7/10** – The integration details depend on specific Volta use cases; however, using `tokio` ensures robust async support.

---

## 4. CLI Structure and Command Dispatching  
**Confidence Score: 9/10**

### 4.1. CLI Parsing with Clap  
- **File:** `src/main.rs`
- **Plan:**
  1. Define the CLI structure using Clap’s derive macros.
  2. Use subcommands to handle each functionality (ni, nr, etc.).
- **Example Code:**
  ```rust
  use clap::{Parser, Subcommand};

  #[derive(Parser)]
  #[command(name = "ni", version, about = "Rust port of the ni tool")]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }

  #[derive(Subcommand)]
  enum Commands {
      /// Install dependencies interactively
      Ni {
          /// Packages to install
          packages: Vec<String>,
          /// Install as development dependency
          #[arg(short, long)]
          dev: bool,
      },
      /// Run a script with history support
      Nr {
          /// The script to execute
          script: String,
      },
      // Additional commands: nci, na, nu, nun, nlx, etc.
  }

  fn main() -> anyhow::Result<()> {
      let cli = Cli::parse();
      match cli.command {
          Commands::Ni { packages, dev } => {
              use std::env;
              use crate::detect::PackageManager;
              use crate::parse::parse_ni;
              use crate::runner::execute;
              let cwd = env::current_dir()?;
              let manager = PackageManager::detect(&cwd).unwrap_or(PackageManager::Npm);
              let args: Vec<String> = packages;
              let resolved = parse_ni(manager, &args);
              execute(resolved, &cwd)?;
          },
          Commands::Nr { script } => {
              // Dispatch to run command implementation
          },
          // Additional command dispatching...
      }
      Ok(())
  }
  ```
- **Confidence:**  
  **9/10** – Using Clap for CLI parsing is proven and effective. The design cleanly separates subcommands.

---

## 5. Testing Strategy  
**Confidence Score: 9/10**

### 5.1. Unit Testing  
- **Plan:**  
  Write tests within each module using `#[cfg(test)]` to verify individual components.
- **Example:** Testing package manager detection:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use std::fs;

      #[test]
      fn test_npm_detection() {
          let temp_dir = tempfile::tempdir().unwrap();
          fs::write(temp_dir.path().join("package-lock.json"), "").unwrap();
          let detected = PackageManager::detect(temp_dir.path());
          assert!(matches!(detected, Some(PackageManager::Npm)));
      }
  }
  ```
- **Confidence:**  
  **9/10** – A strong testing strategy is critical. Unit tests will catch regressions early.

### 5.2. Integration Testing  
- **Plan:**  
  Create integration tests in a separate `tests/` directory using crates like `assert_cmd` to simulate real CLI usage.
- **Example:**
  ```rust
  // tests/integration_test.rs
  use assert_cmd::Command;

  #[test]
  fn test_install_flow() {
      Command::cargo_bin("ni")
          .unwrap()
          .arg("ni")
          .arg("react")
          .assert()
          .success();
  }
  ```
- **Confidence:**  
  **9/10** – Integration tests ensure that the full workflow operates as expected.

### 5.3. Golden Testing  
- **Plan:**  
  Compare output strings (or command-line outputs) against expected “golden” files to ensure consistent behavior.
- **Confidence:**  
  **8/10** – Golden tests are effective for ensuring stable outputs but may require maintenance as features evolve.

---

## 6. Progressive Enhancement Phases  
**Confidence Score: 9/10**

### 6.1. Phase 1 – Minimum Viable Product (MVP)
- **Focus:**  
  - Basic configuration loading.
  - Package manager detection.
  - Command parsing and synchronous execution.
  - A simple CLI interface via Clap.
- **Goal:**  
  Validate that the tool can interpret commands and execute a basic install.
- **Confidence:**  
  **9/10** – Building an MVP first allows iterative improvement and early feedback.

### 6.2. Phase 2 – Interactive and Async Enhancements
- **Focus:**  
  - Add interactive package search (with `inquire`/`dialoguer` and `skim`).
  - Convert blocking operations to async (using `tokio`).
- **Goal:**  
  Enhance user experience with non-blocking interactions.
- **Confidence:**  
  **8/10** – Async enhancements are common and well-supported; some edge cases may require fine-tuning.

### 6.3. Phase 3 – Cross-Platform & Performance Optimizations
- **Focus:**  
  - Handle platform-specific issues (e.g., Windows paths).
  - Optional integration with Volta.
  - Performance improvements (caching, parallel searches with Rayon if needed).
- **Goal:**  
  Ensure the tool is robust and performs well on all major platforms.
- **Confidence:**  
  **8/10** – Cross-platform support and performance tuning are achievable with careful testing and iteration.

---

## 7. Rust-Specific Optimizations  
**Confidence Score: 9/10**

### 7.1. Type-Driven APIs and Pattern Matching  
- **Plan:**  
  Use enums for commands and package managers, leveraging pattern matching for clarity.
- **Confidence:**  
  **9/10** – Rust’s type system and pattern matching are among its strongest features and well-suited for this project.

### 7.2. Memory Safety and Zero-Cost Abstractions  
- **Plan:**  
  Favor `&str` over `String` where possible and design functions with minimal cloning.
- **Confidence:**  
  **9/10** – This approach is intrinsic to Rust and ensures high performance with safe memory management.

### 7.3. Async/Await for I/O  
- **Plan:**  
  Use `tokio` to handle network requests and file I/O asynchronously.
- **Confidence:**  
  **9/10** – Async/await in Rust is mature and will keep the UI responsive during long-running operations.

---

## 8. Final Integration, Documentation, & CI  
**Confidence Score: 9/10**

### 8.1. Documentation  
- **Plan:**  
  - Write comprehensive Rust doc comments (`///`) for all public APIs.
  - Generate documentation with `cargo doc` and update it regularly.
- **Confidence:**  
  **9/10** – Good documentation is key to maintainability and is straightforward using Rust’s tools.

### 8.2. Continuous Integration (CI)  
- **Plan:**  
  - Set up CI pipelines (using GitHub Actions, Travis CI, etc.) to run tests on every commit.
  - Include cross-platform testing to catch OS-specific issues.
- **Confidence:**  
  **9/10** – CI is a standard practice that greatly improves code quality and stability.

### 8.3. Code Review and Iterative Improvement  
- **Plan:**  
  - Regularly review code and update tests/documentation as features evolve.
  - Monitor performance and refactor as needed.
- **Confidence:**  
  **9/10** – Iterative improvement and code review are essential practices that will ensure long-term project success.

---

## 9. Conclusion  
**Overall Confidence Score: 9/10**

This detailed plan outlines each step—from initial project setup to advanced interactive and async features. With careful attention to modularity, testing, and Rust-specific optimizations, the approach is well-grounded in best practices. We are highly confident (9/10) that this plan will lead to a robust and high-performance Rust version of the `ni` tool.

Happy coding!