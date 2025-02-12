// src/bun.rs

use crate::{CommandExecutor, PackageManagerFactory};

pub struct BunExecutor {}

impl CommandExecutor for BunExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "bun".to_string(),
            "run".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "bun".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "bun".to_string(),
            "add".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "bun".to_string(),
            "x".to_string(),
            args.join(" ").to_string(),
        ])
    }
}

pub struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}
