// src/yarn.rs

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

// Yarn
struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "exec".to_string(), args.join(" ").to_string()])
    }
}

struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}
