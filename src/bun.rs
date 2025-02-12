// src/bun.rs

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

// Bun
struct BunExecutor {}

impl CommandExecutor for BunExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "x".to_string(), args.join(" ").to_string()])
    }
}

struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}
