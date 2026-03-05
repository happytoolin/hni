use std::collections::BTreeSet;

use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};

pub fn completion_script_bash(command: &str) -> String {
    generate_completion(command, Bash)
}

pub fn completion_script_zsh(command: &str) -> String {
    generate_completion(command, Zsh)
}

pub fn completion_script_fish(command: &str) -> String {
    generate_completion(command, Fish)
}

pub fn completion_candidates(
    prefix: &str,
    scripts: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let prefix = prefix.trim();
    let set: BTreeSet<String> = scripts
        .into_iter()
        .filter(|script| script.starts_with(prefix))
        .collect();
    set.into_iter().collect()
}

fn generate_completion<G>(command: &str, generator: G) -> String
where
    G: clap_complete::Generator,
{
    let mut cmd = clap::Command::new("nr")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            clap::Arg::new("debug")
                .short('?')
                .long("debug-resolved")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("cwd")
                .short('C')
                .value_name("DIR")
                .action(clap::ArgAction::Append),
        )
        .arg(
            clap::Arg::new("completion")
                .long("completion")
                .hide(true)
                .num_args(0..)
                .action(clap::ArgAction::Append),
        )
        .arg(
            clap::Arg::new("completion-bash")
                .long("completion-bash")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("completion-zsh")
                .long("completion-zsh")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("completion-fish")
                .long("completion-fish")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("args")
                .num_args(0..)
                .allow_hyphen_values(true)
                .action(clap::ArgAction::Append),
        );

    let mut output = Vec::new();
    generate(generator, &mut cmd, command, &mut output);
    String::from_utf8_lossy(&output).into_owned()
}
