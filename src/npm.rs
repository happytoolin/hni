// src/npm.rs

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

struct NpmExecutor {}

impl CommandExecutor for NpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        if args.len() > 1 {
            Some(vec!["npm".to_string(), "run".to_string(), args[0].to_string(), "--".to_string(), args.clone().into_iter().skip(1).collect::<Vec<&str>>().join(" ").to_string()])
        } else {
            Some(vec!["npm".to_string(), "run".to_string(), args[0].to_string()])
        }
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npx".to_string(), args.join(" ").to_string()])
    }
}

struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}
