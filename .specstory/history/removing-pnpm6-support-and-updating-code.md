## SpecStory

## Removing pnpm6 Support and Updating Code (2/17/2025, 5:36:21 PM)

_**User**_

from pnpm, and @detect.rs remove pnpm6

---

_**Assistant**_

<think>
Okay, I need to remove pnpm6 from the codebase. Let me start by looking at the provided files.

First, in pnpm.rs, there's a PnpmLegacyExecutor struct for pnpm v6. I should remove that. Also, the PnpmFactory has an is_legacy flag. Since we're removing pnpm6, the factory can just use the modern executor always. So I'll delete the PnpmLegacyExecutor and modify the factory to not check is_legacy anymore.

In detect.rs, the PackageManagerFactoryEnum has a Pnpm6 variant. I need to remove that. Any references to Pnpm6 in the code, like in the match statements or the LOCKFILES array, should be updated. The parse_package_manager function checks for pnpm versions and returns Pnpm6 if under 7.0. That logic should be adjusted to just return Pnpm for all pnpm versions now.

Also, in the get_nlx_command for Pnpm, the old code had a separate Pnpm6 variant with ["pnpm", "dlx"], but the modern one uses ["pnpm"]. Wait, looking at the original code, the modern executor's execute method uses "dlx", while the legacy uses "exec". But in detect.rs, the get_nlx_command for Pnpm6 was ["pnpm", "dlx"], which might be a mistake. Wait, in the pnpm.rs, the modern executor's execute uses "dlx", and the legacy uses "exec". So in detect.rs, the get_nlx_command for Pnpm (modern) should be ["pnpm", "dlx"], and Pnpm6 was ["pnpm", "exec"]? Wait, looking at the original detect.rs:

In the get_nlx_command, Pnpm6 was mapped to &["pnpm", "dlx"], but according to pnpm.rs, the legacy executor's execute method uses "exec". So that's a bug. But since we're removing Pnpm6, the get_nlx_command for Pnpm should be ["pnpm", "dlx"], which is correct for modern pnpm.

So in detect.rs, remove the Pnpm6 variant from the enum. Then, in all match statements where Pnpm6 was handled, remove those arms. For example, in get_factory, the Pnpm6 case was creating a PnpmFactory with is_legacy=true. Now, since we're only supporting modern pnpm, the factory can always create the modern executor.

In the LOCKFILES array, the entry for pnpm-lock.yaml is mapped to Pnpm, which is correct. The Pnpm6 variant was probably not in LOCKFILES, so no change there.

In the parse_package_manager function, when the packageManager field is pnpm, regardless of version, it should return Pnpm now. So remove the version check that previously returned Pnpm6 for versions under 7.0.

Testing the code after these changes: the tests in pnpm.rs that tested legacy (v6) should be removed or adjusted. In detect.rs tests, any test cases involving Pnpm6 should be updated.

So the steps are:

1. Remove PnpmLegacyExecutor from pnpm.rs.
2. Update PnpmFactory to always create PnpmModernExecutor.
3. Remove the Pnpm6 variant from PackageManagerFactoryEnum in detect.rs.
4. Adjust all match statements that handled Pnpm6 to now handle Pnpm.
5. Update parse_package_manager to not check for pnpm version, always return Pnpm.
6. Fix any test cases that referenced Pnpm6.

I need to make sure that all references to Pnpm6 are removed and that the code correctly handles modern pnpm only.
</think>

I'll help remove pnpm6 (legacy) support. Here are the necessary changes:

1. First, update the pnpm executor to remove legacy version:

```rust:src/package_managers/pnpm.rs
// Remove PnpmLegacyExecutor and simplify the factory
pub struct PnpmModernExecutor {}

// ... keep all PnpmModernExecutor implementations the same ...

// Simplify the factory to only support modern pnpm
#[derive(Clone)]
pub struct PnpmFactory;

impl PnpmFactory {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmModernExecutor {})
    }
}

// Update tests
#[cfg(test)]
mod tests {
    // ... existing imports ...

    fn test_pnpm_version() {
        let factory = PnpmFactory::new();

        let command_map = vec![
            ("run", "run"),
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", "--save-dev"),
            ("execute", "dlx"),
            ("clean_install_args", "--frozen-lockfile"),
            ("upgrade", "update"),
            ("uninstall", "remove"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "pnpm", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "pnpm", &command_map);
    }

    #[test]
    fn test_pnpm_modern() {
        test_pnpm_version();
    }

    // Remove test_pnpm_legacy test
}
```

2. Update the detection code:

```rust:src/detect.rs
// Update the enum
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PackageManagerFactoryEnum {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
    YarnBerry,
    // Remove Pnpm6 variant
}

impl PackageManagerFactoryEnum {
    pub fn get_nlx_command(&self) -> &'static [&'static str] {
        match self {
            // ... other variants ...
            PackageManagerFactoryEnum::Pnpm => &["pnpm", "dlx"], // Update to modern command
            // Remove Pnpm6 case
        }
    }

    // ... rest of the impl ...
}

// Update package manager parsing
fn parse_package_manager(pm: &str, path: &Path) -> Result<PackageManagerFactoryEnum> {
    // ... existing code ...
    match name {
        "pnpm" => Ok(PackageManagerFactoryEnum::Pnpm), // Always return Pnpm regardless of version
        // ... other cases ...
    }
}

// Update factory creation
impl PackageManagerFactoryEnum {
    pub fn get_factory(&self) -> Box<dyn PackageManagerFactory> {
        match self {
            // ... other cases ...
            PackageManagerFactoryEnum::Pnpm => Box::new(PnpmFactory::new()),
            // Remove Pnpm6 case
        }
    }
}

// Update tests to remove pnpm6 references
#[cfg(test)]
mod tests {
    // ... in test cases ...
    #[test]
    fn test_parse_package_manager_valid() {
        let cases = vec![
            // ... other cases ...
            ("pnpm@6.0.0", PackageManagerFactoryEnum::Pnpm), // Now maps to regular Pnpm
            ("pnpm@7.0.0", PackageManagerFactoryEnum::Pnpm),
        ];
    }

    // Remove any pnpm6-specific tests
}
```

These changes:

1. Remove legacy pnpm6 executor and simplify the pnpm factory
2. Remove pnpm6 detection logic and enum variant
3. Update pnpm command generation to use modern `dlx`
4. Simplify version parsing to treat all pnpm versions as modern
5. Update tests to reflect these changes

The code now only supports pnpm 7+ and uses the modern command structure consistently.

---

_****_

---

---
