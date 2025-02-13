use std::path::Path;

use crate::{
    detect::{detect, PackageManagerFactoryEnum},
    CommandExecutor,
};

use anyhow::{Context, Result};

pub fn execute_command(cwd: &Path, action: &str, args: Vec<&str>) -> Result<std::process::Command> {
    let package_manager = detect(cwd)
        .context("Failed to detect package manager")?
        .ok_or(anyhow::anyhow!("No package manager detected"))?;

    let factory = package_manager.get_factory();
    let executor = factory.create_commands();

    let command = match action {
        "run" => executor
            .run(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "install" => executor
            .install(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "add" => executor
            .add(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "execute" => executor
            .execute(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "upgrade" => executor
            .upgrade(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "uninstall" => executor
            .uninstall(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        "clean_install" => executor
            .clean_install(args)
            .ok_or(anyhow::anyhow!("Command not found")),
        _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
    }?;

    let mut cmd = std::process::Command::new(command.bin);
    cmd.args(command.args);

    Ok(cmd)
}
