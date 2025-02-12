## Future Plans

- Implement the remaining functions from the TypeScript code, such as `detectSync`, `getUserAgent`, `lookup`, `parsePackageJson`, `handlePackageManager`, and `fileExists`.
- Add unit tests for the package manager detection logic.
- Integrate the command execution logic into the `ni` command, using the abstract factory pattern to execute the commands.
- Refactor the command execution logic into separate files for each package manager (e.g., `npm.rs`, `yarn.rs`, etc.).
- Add support for more commands to the abstract factory pattern.
- Implement the remaining package managers (YarnBerry, Pnpm6)
