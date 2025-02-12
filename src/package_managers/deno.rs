// src/deno.rs

use crate::{CommandExecutor, PackageManagerFactory};

pub fn deno_execute(args: Vec<&str>) -> Option<Vec<String>> {
    Some(vec![
        "deno".to_string(),
        "run".to_string(),
        format!("npm:{}", args[0]),
    ])
}

pub struct DenoExecutor {}

impl CommandExecutor for DenoExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "deno".to_string(),
            "task".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "deno".to_string(),
            "install".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec![
            "deno".to_string(),
            "add".to_string(),
            args.join(" ").to_string(),
        ])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        deno_execute(args)
    }
}

pub struct DenoFactory {}

impl PackageManagerFactory for DenoFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(DenoExecutor {})
    }
}
