// src/npm.rs

use crate::{CommandExecutor, PackageManagerFactory};

pub struct NpmExecutor {}

impl CommandExecutor for NpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "npm".to_string(),
            "run".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "npm".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "npm".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npx".to_string(), args.join(" ").to_string()])
    }
}

pub struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}
