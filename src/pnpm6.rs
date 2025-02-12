// src/pnpm6.rs

use crate::{CommandExecutor, PackageManagerFactory};

pub struct Pnpm6Executor {}

impl CommandExecutor for Pnpm6Executor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "pnpm".to_string(),
            "run".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "pnpm".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "pnpm".to_string(),
            "add".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "pnpm".to_string(),
            "dlx".to_string(),
            args.join(" ").to_string(),
        ])
    }
}

pub struct Pnpm6Factory {}

impl PackageManagerFactory for Pnpm6Factory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(Pnpm6Executor {})
    }
}
