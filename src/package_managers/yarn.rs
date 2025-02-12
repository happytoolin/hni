// src/yarn.rs

use crate::{CommandExecutor, PackageManagerFactory};

pub struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "yarn".to_string(),
            "run".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "yarn".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "yarn".to_string(),
            "add".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "yarn".to_string(),
            "exec".to_string(),
            args.join(" ").to_string(),
        ])
    }
}

pub struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}
