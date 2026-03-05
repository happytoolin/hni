use std::path::Path;

use crate::{
    core::{
        config::{DefaultAgent, HniConfig, RunAgent},
        detect::detect,
        types::DetectionSource,
    },
    platform::node::resolve_real_node_path,
};

pub fn print_doctor(cwd: &Path, config: &HniConfig) {
    println!("hni doctor");
    println!();
    println!("cwd: {}", cwd.display());
    println!(
        "config_file: {}",
        config
            .config_path
            .as_ref()
            .map_or_else(|| "none".to_string(), |p| p.display().to_string())
    );
    println!(
        "defaultAgent: {}",
        format_default_agent(config.default_agent)
    );
    println!("globalAgent: {}", config.global_agent.display_name());
    println!("runAgent: {}", format_run_agent(config.run_agent));
    println!("useSfw: {}", config.use_sfw);
    println!("autoInstall(env): {}", config.auto_install);
    println!();

    match detect(cwd, config) {
        Ok(detection) => {
            println!(
                "detected_agent: {}",
                detection
                    .agent
                    .map_or_else(|| "none".to_string(), |pm| pm.display_name().to_string())
            );
            println!(
                "detection_source: {}",
                detection_source_label(detection.source)
            );
            println!("has_lockfile: {}", detection.has_lock);
            if let Some(version_hint) = detection.version_hint {
                println!("version_hint: {version_hint}");
            }
        }
        Err(err) => {
            println!("detection_error: {err}");
        }
    }

    println!();
    match resolve_real_node_path() {
        Ok(path) => println!("real_node: {}", path.display()),
        Err(err) => println!("real_node: unavailable ({err})"),
    }

    println!();
    println!("package_manager_binaries:");
    for (label, bin) in [
        ("npm", "npm"),
        ("yarn", "yarn"),
        ("pnpm", "pnpm"),
        ("bun", "bun"),
        ("deno", "deno"),
    ] {
        let state = if which::which(bin).is_ok() {
            "ok"
        } else {
            "missing"
        };
        println!("  {label:<5} {state}");
    }
}

fn format_default_agent(value: DefaultAgent) -> &'static str {
    match value {
        DefaultAgent::Prompt => "prompt",
        DefaultAgent::Agent(pm) => pm.display_name(),
    }
}

fn format_run_agent(value: RunAgent) -> &'static str {
    match value {
        RunAgent::PackageManager => "package-manager",
        RunAgent::Node => "node",
    }
}

fn detection_source_label(value: DetectionSource) -> &'static str {
    match value {
        DetectionSource::PackageManagerField => "packageManager field",
        DetectionSource::Lockfile => "lockfile",
        DetectionSource::Config => "config defaultAgent",
        DetectionSource::Fallback => "fallback (npm in PATH)",
        DetectionSource::None => "none",
    }
}
