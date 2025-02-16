# Configuration

`nirs` can be configured using a `nirs.toml`, `nirs.json`, or `nirs.yaml` file in the project directory or in the home directory.

## Options

### `default_package_manager`

Specifies the default package manager to use if no lockfile is found.

**Type:** String

**Possible values:** `"npm"`, `"yarn"`, `"pnpm"`, `"bun"`

**Default value:** `"npm"`

## Example `.nirs` file

```toml
default_package_manager = "yarn"
```
