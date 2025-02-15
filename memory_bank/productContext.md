# Product Context: nirs

**Problem:** Developers often work on multiple projects that use different package managers (npm, yarn, pnpm, bun). Remembering the specific commands for each package manager can be tedious and error-prone. This leads to wasted time and frustration.

**Solution:** `nirs` provides a unified command-line interface that abstracts away the differences between package managers. Developers can use the same commands (e.g., `ni` for install, `nr` for run) regardless of the underlying package manager.

**How it Works:** `nirs` automatically detects the package manager used in a project by looking for lock files (e.g., `yarn.lock`, `package-lock.json`). It then translates the `nirs` commands into the appropriate commands for the detected package manager.

**User Experience Goals:**

- **Simplicity:** The commands should be easy to remember and use.
- **Consistency:** The same commands should work across all projects.
- **Efficiency:** Developers should be able to perform common tasks quickly.
- **Transparency:** The tool should provide clear logging so developers understand what's happening.
- **Flexibility:** The tool should handle common use cases and edge cases.

**User Stories:**

- As a developer, I want to install dependencies with a single command, regardless of the project's package manager.
- As a developer, I want to run scripts with a single command, regardless of the project's package manager.
- As a developer, I want to upgrade packages easily.
- As a developer, I want to uninstall packages easily.
- As a developer, I want to see clear logs of what the tool is doing.
