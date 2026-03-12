use std::process::Command;

use crate::{
    core::{
        detect::detect,
        resolve::{ResolveContext, version_command_for_pm},
    },
    platform::node::{REAL_NODE_ENV, path_with_real_node_priority, resolve_real_node_path},
};

pub fn print_versions(ctx: &ResolveContext) {
    println!("hni       v{}", env!("CARGO_PKG_VERSION"));

    let node_version = resolve_real_node_path().ok().and_then(|path| {
        run_version(
            path.to_string_lossy().to_string(),
            vec!["--version".into()],
            ctx,
        )
    });
    if let Some(version) = node_version {
        println!("node       {version}");
    }

    if let Ok(detected) = detect(&ctx.cwd, &ctx.config) {
        if let Some(agent) = detected.agent {
            let (program, args) = version_command_for_pm(agent);
            if let Some(version) = run_version(program, args, ctx) {
                println!("agent      {} ({version})", agent.display_name());
            } else {
                println!("agent      {}", agent.display_name());
            }
        } else {
            println!("agent      none");
        }
    }

    let global = ctx.config.global_agent;
    let (program, args) = version_command_for_pm(global);
    if let Some(version) = run_version(program, args, ctx) {
        println!("global     {} ({version})", global.display_name());
    } else {
        println!("global     {}", global.display_name());
    }
}

fn run_version(program: String, args: Vec<String>, ctx: &ResolveContext) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(&ctx.cwd);

    if let Ok(real_node) = resolve_real_node_path() {
        command.env(REAL_NODE_ENV, &real_node);
        if let Some(path) = path_with_real_node_priority(&real_node, std::env::var_os("PATH")) {
            command.env("PATH", path);
        }
    }

    let output = command.output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}
