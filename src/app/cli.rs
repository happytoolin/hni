use std::{env, ffi::OsStr, path::PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, builder::PossibleValuesParser};

use crate::app::{help::HelpTopic, init::SUPPORTED_SHELL_NAMES};
use crate::core::{
    error::{HniError, HniResult},
    types::InvocationKind,
};

#[derive(Debug, Clone)]
pub struct ParsedInvocation {
    pub cwd: PathBuf,
    pub debug: bool,
    pub explain: bool,
    pub native_override: Option<bool>,
    pub command: ParsedCommand,
    pub deprecated_debug_alias_used: bool,
}

#[derive(Debug, Clone)]
pub enum ParsedCommand {
    PrintHelp(HelpTopic),
    PrintVersions,
    Doctor,
    Completion {
        shell: Option<String>,
        program: String,
    },
    Init {
        shell: String,
    },
    InternalRealNodePath,
    InternalProfileLoop {
        invocation: InvocationKind,
        args: Vec<String>,
        iterations: usize,
    },
    Execute {
        invocation: InvocationKind,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
struct SharedFlags {
    cwd: Vec<PathBuf>,
    debug: bool,
    explain: bool,
    native_override: Option<bool>,
    help: bool,
    version: bool,
}

/// Parse command-line arguments from environment.
///
/// # Errors
///
/// Returns an error if:
/// - `argv[0]` is missing
/// - Current directory cannot be read
/// - Argument parsing fails
pub fn parse_from_env() -> HniResult<ParsedInvocation> {
    let argv = env::args().collect::<Vec<_>>();
    let Some(argv0) = argv.first() else {
        return Err(HniError::parse("missing argv[0]"));
    };

    let invocation = invocation_from_argv0(argv0);
    let (normalized_args, deprecated_debug_alias_used) = normalize_debug_aliases(&argv[1..]);
    let (shared_flags, command_args) = extract_shared_flags(&normalized_args)?;

    if invocation == InvocationKind::Hni {
        parse_hni(
            argv0,
            &command_args,
            shared_flags,
            deprecated_debug_alias_used,
        )
    } else {
        parse_alias(
            invocation,
            &command_args,
            shared_flags,
            deprecated_debug_alias_used,
        )
    }
}

fn parse_hni(
    argv0: &str,
    args: &[String],
    shared_flags: SharedFlags,
    deprecated_debug_alias_used: bool,
) -> HniResult<ParsedInvocation> {
    if args.first().is_some_and(|token| token == "help") {
        let requested_topic = args.get(1).cloned();
        if args.len() > 2 {
            return Err(HniError::parse(format!(
                "unexpected arguments for help: {}",
                args[2..].join(" ")
            )));
        }

        let mut command = ParsedCommand::PrintHelp(help_target(requested_topic)?);
        if shared_flags.version {
            command = ParsedCommand::PrintVersions;
        } else if shared_flags.help {
            command = ParsedCommand::PrintHelp(help_target_from_command(&command));
        }

        return Ok(ParsedInvocation {
            cwd: resolve_cwd(&shared_flags.cwd)?,
            debug: shared_flags.debug,
            explain: shared_flags.explain,
            native_override: shared_flags.native_override,
            command,
            deprecated_debug_alias_used,
        });
    }

    let program = normalized_program_name(argv0);
    let mut clap_args = Vec::with_capacity(args.len() + 1);
    clap_args.push(program.clone());
    clap_args.extend(args.iter().cloned());

    let matches = hni_parser()
        .try_get_matches_from(clap_args)
        .map_err(|error| HniError::parse(error.to_string()))?;

    let mut command = if let Some((name, sub_matches)) = matches.subcommand() {
        match name {
            "ni" => execute_from_subcommand(InvocationKind::Ni, sub_matches),
            "nr" => execute_from_subcommand(InvocationKind::Nr, sub_matches),
            "nlx" => execute_from_subcommand(InvocationKind::Nlx, sub_matches),
            "nu" => execute_from_subcommand(InvocationKind::Nu, sub_matches),
            "nun" => execute_from_subcommand(InvocationKind::Nun, sub_matches),
            "nci" => execute_from_subcommand(InvocationKind::Nci, sub_matches),
            "na" => execute_from_subcommand(InvocationKind::Na, sub_matches),
            "np" => execute_from_subcommand(InvocationKind::Np, sub_matches),
            "ns" => execute_from_subcommand(InvocationKind::Ns, sub_matches),
            "node" => execute_from_subcommand(InvocationKind::NodeShim, sub_matches),
            "doctor" => ParsedCommand::Doctor,
            "completion" => ParsedCommand::Completion {
                shell: sub_matches.get_one::<String>("shell").cloned(),
                program: program.clone(),
            },
            "init" => ParsedCommand::Init {
                shell: sub_matches
                    .get_one::<String>("shell")
                    .cloned()
                    .ok_or_else(|| HniError::parse("missing shell for init"))?,
            },
            "internal" => parse_internal_command(sub_matches)?,
            _ => ParsedCommand::PrintHelp(HelpTopic::Hni),
        }
    } else {
        ParsedCommand::PrintHelp(HelpTopic::Hni)
    };

    if shared_flags.version {
        command = ParsedCommand::PrintVersions;
    } else if shared_flags.help {
        command = ParsedCommand::PrintHelp(help_target_from_command(&command));
    }

    Ok(ParsedInvocation {
        cwd: resolve_cwd(&shared_flags.cwd)?,
        debug: shared_flags.debug,
        explain: shared_flags.explain,
        native_override: shared_flags.native_override,
        command,
        deprecated_debug_alias_used,
    })
}

fn parse_alias(
    invocation: InvocationKind,
    args: &[String],
    shared_flags: SharedFlags,
    deprecated_debug_alias_used: bool,
) -> HniResult<ParsedInvocation> {
    let mut forwarded_args = args.to_vec();
    let has_forwarded_args = !forwarded_args.is_empty();

    // In alias mode, treat --help/--version as passthrough flags when a target command exists.
    if has_forwarded_args {
        if shared_flags.help {
            forwarded_args.push("--help".to_string());
        }
        if shared_flags.version {
            forwarded_args.push("--version".to_string());
        }
    }

    let mut command = ParsedCommand::Execute {
        invocation,
        args: forwarded_args,
    };

    if !has_forwarded_args {
        if shared_flags.version {
            command = ParsedCommand::PrintVersions;
        } else if shared_flags.help {
            command = ParsedCommand::PrintHelp(help_topic_for_invocation(invocation));
        }
    }

    Ok(ParsedInvocation {
        cwd: resolve_cwd(&shared_flags.cwd)?,
        debug: shared_flags.debug,
        explain: shared_flags.explain,
        native_override: shared_flags.native_override,
        command,
        deprecated_debug_alias_used,
    })
}

fn execute_from_subcommand(invocation: InvocationKind, sub_matches: &ArgMatches) -> ParsedCommand {
    ParsedCommand::Execute {
        invocation,
        args: values_from(sub_matches.get_many::<String>("args")),
    }
}

fn parse_internal_command(sub_matches: &ArgMatches) -> HniResult<ParsedCommand> {
    match sub_matches.subcommand() {
        Some(("real-node-path", _)) => Ok(ParsedCommand::InternalRealNodePath),
        Some(("profile-loop", matches)) => Ok(ParsedCommand::InternalProfileLoop {
            invocation: internal_invocation(
                matches
                    .get_one::<String>("invocation")
                    .ok_or_else(|| HniError::parse("missing internal invocation"))?,
            )?,
            args: values_from(matches.get_many::<String>("args")),
            iterations: *matches
                .get_one::<usize>("iterations")
                .ok_or_else(|| HniError::parse("missing iterations"))?,
        }),
        _ => Ok(ParsedCommand::PrintHelp(HelpTopic::Hni)),
    }
}

fn values_from<'a, T: Clone + 'a>(values: Option<clap::parser::ValuesRef<'a, T>>) -> Vec<T> {
    values
        .map(|entries| entries.cloned().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn resolve_cwd(cwd_flags: &[PathBuf]) -> HniResult<PathBuf> {
    if cwd_flags.is_empty() {
        return env::current_dir().map_err(|error| {
            HniError::execution(format!("failed to read current directory: {error}"))
        });
    }

    let absolute_index = cwd_flags.iter().rposition(|segment| segment.is_absolute());
    let (mut cwd, start_index) = match absolute_index {
        Some(index) => (cwd_flags[index].clone(), index + 1),
        None => (
            env::current_dir().map_err(|error| {
                HniError::execution(format!("failed to read current directory: {error}"))
            })?,
            0,
        ),
    };

    for segment in &cwd_flags[start_index..] {
        cwd.push(segment);
    }

    Ok(cwd)
}

fn help_target(command: Option<String>) -> HniResult<HelpTopic> {
    let Some(command) = command else {
        return Ok(HelpTopic::Hni);
    };

    let normalized = command.to_ascii_lowercase();
    #[allow(clippy::match_same_arms)]
    let target = match normalized.as_str() {
        "hni" => HelpTopic::Hni,
        "ni" => HelpTopic::Ni,
        "nr" => HelpTopic::Nr,
        "nlx" => HelpTopic::Nlx,
        "nu" => HelpTopic::Nu,
        "nun" => HelpTopic::Nun,
        "nci" => HelpTopic::Nci,
        "na" => HelpTopic::Na,
        "np" => HelpTopic::Np,
        "ns" => HelpTopic::Ns,
        "node" => HelpTopic::Node,
        "init" => HelpTopic::Init,
        "doctor" | "completion" | "help" => HelpTopic::Hni,
        _ => {
            return Err(HniError::parse(format!(
                "unknown help topic '{command}'. Try: hni help"
            )));
        }
    };
    Ok(target)
}

fn help_target_from_command(command: &ParsedCommand) -> HelpTopic {
    match command {
        ParsedCommand::PrintHelp(topic) => *topic,
        ParsedCommand::Init { .. } => HelpTopic::Init,
        ParsedCommand::Execute { invocation, .. } => help_topic_for_invocation(*invocation),
        ParsedCommand::Doctor
        | ParsedCommand::Completion { .. }
        | ParsedCommand::InternalRealNodePath
        | ParsedCommand::InternalProfileLoop { .. }
        | ParsedCommand::PrintVersions => HelpTopic::Hni,
    }
}

fn help_topic_for_invocation(invocation: InvocationKind) -> HelpTopic {
    match invocation {
        InvocationKind::Hni => HelpTopic::Hni,
        InvocationKind::Ni => HelpTopic::Ni,
        InvocationKind::Nr => HelpTopic::Nr,
        InvocationKind::Nlx => HelpTopic::Nlx,
        InvocationKind::Nu => HelpTopic::Nu,
        InvocationKind::Nun => HelpTopic::Nun,
        InvocationKind::Nci => HelpTopic::Nci,
        InvocationKind::Na => HelpTopic::Na,
        InvocationKind::Np => HelpTopic::Np,
        InvocationKind::Ns => HelpTopic::Ns,
        InvocationKind::NodeShim => HelpTopic::Node,
    }
}

fn init_parser() -> Command {
    Command::new("init").arg(
        Arg::new("shell")
            .required(true)
            .value_parser(PossibleValuesParser::new(SUPPORTED_SHELL_NAMES)),
    )
}

fn internal_parser() -> Command {
    Command::new("internal")
        .hide(true)
        .subcommand(Command::new("real-node-path").hide(true))
        .subcommand(
            Command::new("profile-loop")
                .hide(true)
                .arg(
                    Arg::new("iterations")
                        .long("iterations")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("2000"),
                )
                .arg(
                    Arg::new("invocation")
                        .required(true)
                        .value_parser(PossibleValuesParser::new([
                            "ni", "nr", "nlx", "nu", "nun", "nci", "na", "node",
                        ])),
                )
                .arg(forwarded_args_arg()),
        )
}

fn hni_parser() -> Command {
    Command::new("hni")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .subcommand(command_parser("ni"))
        .subcommand(command_parser("nr"))
        .subcommand(command_parser("nlx"))
        .subcommand(command_parser("nu"))
        .subcommand(command_parser("nun"))
        .subcommand(command_parser("nci"))
        .subcommand(command_parser("na"))
        .subcommand(command_parser("np"))
        .subcommand(command_parser("ns"))
        .subcommand(command_parser("node"))
        .subcommand(Command::new("doctor"))
        .subcommand(Command::new("completion").arg(Arg::new("shell").num_args(0..=1)))
        .subcommand(init_parser())
        .subcommand(internal_parser())
}

fn command_parser(name: &'static str) -> Command {
    Command::new(name).arg(forwarded_args_arg())
}

fn forwarded_args_arg() -> Arg {
    Arg::new("args")
        .num_args(0..)
        .allow_hyphen_values(true)
        .action(ArgAction::Append)
}

fn invocation_from_argv0(argv0: &str) -> InvocationKind {
    match normalized_program_name(argv0).as_str() {
        "ni" => InvocationKind::Ni,
        "nr" => InvocationKind::Nr,
        "nlx" => InvocationKind::Nlx,
        "nu" => InvocationKind::Nu,
        "nun" => InvocationKind::Nun,
        "nci" => InvocationKind::Nci,
        "na" => InvocationKind::Na,
        "np" => InvocationKind::Np,
        "ns" => InvocationKind::Ns,
        "node" => InvocationKind::NodeShim,
        _ => InvocationKind::Hni,
    }
}

fn internal_invocation(name: &str) -> HniResult<InvocationKind> {
    match name {
        "ni" => Ok(InvocationKind::Ni),
        "nr" => Ok(InvocationKind::Nr),
        "nlx" => Ok(InvocationKind::Nlx),
        "nu" => Ok(InvocationKind::Nu),
        "nun" => Ok(InvocationKind::Nun),
        "nci" => Ok(InvocationKind::Nci),
        "na" => Ok(InvocationKind::Na),
        "node" => Ok(InvocationKind::NodeShim),
        _ => Err(HniError::parse(format!(
            "unsupported internal invocation '{name}'"
        ))),
    }
}

fn normalized_program_name(argv0: &str) -> String {
    let name = PathBuf::from(argv0)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(argv0)
        .to_ascii_lowercase();
    name.strip_suffix(".exe").unwrap_or(&name).to_string()
}

fn normalize_debug_aliases(args: &[String]) -> (Vec<String>, bool) {
    let mut normalized = Vec::with_capacity(args.len());
    let mut saw_deprecated = false;
    let mut passthrough = false;

    for arg in args {
        if passthrough {
            normalized.push(arg.clone());
            continue;
        }

        if arg == "--" {
            passthrough = true;
            normalized.push(arg.clone());
            continue;
        }

        if arg == "?" || arg == "-?" {
            saw_deprecated = true;
            normalized.push("--debug-resolved".to_string());
            continue;
        }

        normalized.push(arg.clone());
    }

    (normalized, saw_deprecated)
}

fn extract_shared_flags(args: &[String]) -> HniResult<(SharedFlags, Vec<String>)> {
    let mut flags = SharedFlags {
        cwd: Vec::new(),
        debug: false,
        explain: false,
        native_override: None,
        help: false,
        version: false,
    };
    let mut rest = Vec::new();
    let mut idx = 0;
    let mut passthrough = false;

    while idx < args.len() {
        let arg = &args[idx];
        if passthrough {
            rest.push(arg.clone());
            idx += 1;
            continue;
        }

        if arg == "--" {
            passthrough = true;
            rest.push(arg.clone());
            idx += 1;
            continue;
        }

        match arg.as_str() {
            "--debug-resolved" | "--dry-run" | "--print-command" | "-?" => {
                flags.debug = true;
                idx += 1;
            }
            "--explain" => {
                flags.explain = true;
                idx += 1;
            }
            "--native" | "--fast" => {
                set_native_override(&mut flags, true)?;
                idx += 1;
            }
            "--no-native" => {
                set_native_override(&mut flags, false)?;
                idx += 1;
            }
            "-h" | "--help" => {
                flags.help = true;
                idx += 1;
            }
            "-v" | "--version" => {
                flags.version = true;
                idx += 1;
            }
            "-C" | "--cwd" => {
                let Some(value) = args.get(idx + 1) else {
                    return Err(HniError::parse(format!("missing value for {arg}")));
                };
                flags.cwd.push(PathBuf::from(value));
                idx += 2;
            }
            _ if arg.starts_with("-C") && arg.len() > 2 => {
                flags.cwd.push(PathBuf::from(&arg[2..]));
                idx += 1;
            }
            _ if arg.starts_with("--cwd=") => {
                flags
                    .cwd
                    .push(PathBuf::from(arg.trim_start_matches("--cwd=")));
                idx += 1;
            }
            _ => {
                rest.push(arg.clone());
                idx += 1;
            }
        }
    }

    Ok((flags, rest))
}

fn set_native_override(flags: &mut SharedFlags, value: bool) -> HniResult<()> {
    match flags.native_override {
        Some(existing) if existing != value => {
            Err(HniError::parse("--native conflicts with --no-native"))
        }
        _ => {
            flags.native_override = Some(value);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_debug_aliases_without_touching_passthrough_args() {
        let (args, deprecated) = normalize_debug_aliases(&[
            "?".to_string(),
            "ni".to_string(),
            "--".to_string(),
            "?".to_string(),
        ]);

        assert!(deprecated);
        assert_eq!(args, vec!["--debug-resolved", "ni", "--", "?"]);
    }

    #[test]
    fn extracts_fast_alias_as_native_override() {
        let (flags, rest) = extract_shared_flags(&["--fast".to_string(), "dev".to_string()]).unwrap();

        assert_eq!(flags.native_override, Some(true));
        assert_eq!(rest, vec!["dev"]);
    }

    #[test]
    fn extracts_shared_flags_from_any_position_before_passthrough() {
        let (flags, rest) = extract_shared_flags(&[
            "ni".to_string(),
            "vite".to_string(),
            "--help".to_string(),
            "--".to_string(),
            "--version".to_string(),
        ])
        .unwrap();

        assert!(flags.help);
        assert_eq!(rest, vec!["ni", "vite", "--", "--version"]);
    }

    #[test]
    fn extracts_short_and_long_cwd_flag_forms() {
        let (flags, rest) = extract_shared_flags(&[
            "ni".to_string(),
            "-Ctmp".to_string(),
            "--cwd=project".to_string(),
            "vite".to_string(),
        ])
        .unwrap();

        assert_eq!(
            flags.cwd,
            vec![PathBuf::from("tmp"), PathBuf::from("project")]
        );
        assert_eq!(rest, vec!["ni", "vite"]);
    }

    #[test]
    fn missing_cwd_value_is_parse_error() {
        let err = extract_shared_flags(&["ni".to_string(), "-C".to_string()]).unwrap_err();
        assert!(err.to_string().contains("missing value for -C"));
    }

    #[test]
    fn conflicting_native_flags_are_rejected() {
        let err =
            extract_shared_flags(&["--native".to_string(), "--no-native".to_string()]).unwrap_err();
        assert!(err.to_string().contains("conflicts"));
    }

    #[test]
    fn dash_question_mark_is_normalized_as_debug_flag() {
        let (args, deprecated) = normalize_debug_aliases(&["-?".to_string()]);
        assert!(deprecated);
        assert_eq!(args, vec!["--debug-resolved"]);
    }

    #[test]
    fn alias_help_with_args_is_forwarded() {
        let shared_flags = SharedFlags {
            cwd: vec![],
            debug: false,
            explain: false,
            native_override: None,
            help: true,
            version: false,
        };

        let parsed = parse_alias(
            InvocationKind::Nlx,
            &["vitest".to_string()],
            shared_flags,
            false,
        )
        .unwrap();

        match parsed.command {
            ParsedCommand::Execute { args, .. } => {
                assert_eq!(args, vec!["vitest", "--help"]);
            }
            _ => panic!("expected execute command"),
        }
    }

    #[test]
    fn alias_help_without_args_prints_help() {
        let shared_flags = SharedFlags {
            cwd: vec![],
            debug: false,
            explain: false,
            native_override: None,
            help: true,
            version: false,
        };

        let parsed = parse_alias(InvocationKind::Nlx, &[], shared_flags, false).unwrap();

        match parsed.command {
            ParsedCommand::PrintHelp(HelpTopic::Nlx) => {}
            _ => panic!("expected nlx help command"),
        }
    }
}
