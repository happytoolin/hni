// src/pnpm.rs

trait CommandExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn install(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn add(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>>;
    // Add other command methods here
}

trait PackageManagerFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor>;
}

// Pnpm
struct PnpmExecutor {}

impl CommandExecutor for PnpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "dlx".to_string(), args.join(" ").to_string()])
    }
}

struct PnpmFactory {}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmExecutor {})
    }
}
