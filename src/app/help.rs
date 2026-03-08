use clap::{Arg, ArgAction, Command, builder::PossibleValuesParser, value_parser};

use crate::app::init::SUPPORTED_SHELL_NAMES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Hni,
    Ni,
    Nr,
    Nlx,
    Nu,
    Nun,
    Nci,
    Na,
    Np,
    Ns,
    Node,
    Init,
}

pub fn print_help(topic: HelpTopic) {
    let mut cmd = help_command(topic);
    let _ = cmd.print_long_help();
    println!();
}

pub fn help_command(topic: HelpTopic) -> Command {
    match topic {
        HelpTopic::Hni => top_level_help(),
        HelpTopic::Ni => command_help(
            "ni",
            "install or add dependencies",
            "Routes installs to the package manager detected from packageManager or lockfile.",
            "Examples:\n\
             \n\
             ni                   Install dependencies\n\
             ni vite              Add dependency\n\
             ni -D vitest         Add dev dependency\n\
             ni --interactive     Search and choose a package interactively\n\
             ni --frozen          Use lockfile-only install (nci behavior)\n\
             ni -- --help         Forward --help to underlying package manager\n\
             ni -g npm-check-updates",
        ),
        HelpTopic::Nr => command_help(
            "nr",
            "run package scripts",
            "Runs scripts through the detected package manager and keeps npm '--' behavior consistent.",
            "Examples:\n\
             \n\
             nr                   Run 'start'\n\
             nr dev               Run dev script\n\
             nr test -- --watch   Pass extra args to script\n\
             nr --if-present lint Skip failure if script is missing\n\
             nr --repeat-last      Re-run last script",
        ),
        HelpTopic::Nlx => command_help(
            "nlx",
            "execute package binaries",
            "Runs package binaries without permanently installing them (npx / pnpm dlx / yarn dlx / bun x).",
            "Examples:\n\
             \n\
             nlx vite@latest\n\
             nlx eslint .\n\
             nlx degit user/repo app",
        ),
        HelpTopic::Nu => command_help(
            "nu",
            "upgrade dependencies",
            "Upgrades dependencies using package-manager-specific update commands.",
            "Examples:\n\
             \n\
             nu\n\
             nu react react-dom\n\
             nu --interactive      Interactive mode when supported",
        ),
        HelpTopic::Nun => command_help(
            "nun",
            "remove dependencies",
            "Uninstalls dependencies. Supports interactive multi-select mode.",
            "Examples:\n\
             \n\
             nun lodash\n\
             nun react react-dom\n\
             nun --multi-select    Interactive multi-select\n\
             nun -g typescript",
        ),
        HelpTopic::Nci => command_help(
            "nci",
            "clean install",
            "Performs lockfile-clean install when lockfile exists; falls back to install otherwise.",
            "Examples:\n\
             \n\
             nci\n\
             nci --prefer-offline",
        ),
        HelpTopic::Na => command_help(
            "na",
            "package manager alias",
            "Forwards arguments directly to the detected package manager binary.",
            "Examples:\n\
             \n\
             na --version\n\
             na config get registry\n\
             na cache clean --force",
        ),
        HelpTopic::Np => command_help(
            "np",
            "run shell commands in parallel",
            "Runs each argument as a separate shell command concurrently. Returns first non-zero code.",
            "Examples:\n\
             \n\
             np \"npm:test\" \"npm:lint\"\n\
             np \"echo one\" \"echo two\"",
        ),
        HelpTopic::Ns => command_help(
            "ns",
            "run shell commands sequentially",
            "Runs each argument in order and stops at first failure.",
            "Examples:\n\
             \n\
             ns \"npm run build\" \"npm run test\"\n\
             ns \"echo pre\" \"echo post\"",
        ),
        HelpTopic::Node => with_global_flags(
            Command::new("node")
                .about("package-manager-aware node shim")
                .long_about(
                    "Interprets npm-like verbs and routes them through hni command resolution.\n\
                     Non-routed invocations pass through to the real Node.js binary.",
                )
                .arg(command_args_arg())
                .after_help(
                    "Passthrough examples:\n\
                     \n\
                     node script.js\n\
                     node -v\n\
                     node -- --trace-warnings\n\
                     \n\
                     Routed examples:\n\
                     \n\
                     node install vite\n\
                     node run dev -- --port=3000\n\
                     node p \"echo one\" \"echo two\"\n\
                     \n\
                     Routed verbs: p, s, install|i, add, run, exec|x|dlx, update|upgrade, uninstall|remove, ci",
                ),
        ),
        HelpTopic::Init => with_global_flags(
            Command::new("init")
                .about("print shell init code for node shim precedence")
                .long_about(
                    "Prints shell-specific init code that captures the current real Node.js binary\n\
                     and then prepends the hni install dir so the node shim takes precedence.\n\
                     Add the generated line at the end of your shell config, after nvm/mise/asdf/fnm/volta init.",
                )
                .arg(init_shell_arg())
                .after_help(
                    "Examples:\n\
                     \n\
                     hni init bash\n\
                     hni init zsh\n\
                     hni init fish\n\
                     hni init powershell\n\
                     hni init nushell",
                ),
        ),
    }
}

fn top_level_help() -> Command {
    with_global_flags(
        Command::new("hni")
            .about("use the right package manager")
            .long_about(
                "hni is a multicall package-manager router.\n\
                 It powers commands like ni, nr, nlx, nu, nun, nci, na, np, ns, and node.",
            )
            .subcommand(Command::new("ni").about("install or add dependencies"))
            .subcommand(Command::new("nr").about("run package scripts"))
            .subcommand(Command::new("nlx").about("execute package binaries"))
            .subcommand(Command::new("nu").about("upgrade dependencies"))
            .subcommand(Command::new("nun").about("remove dependencies"))
            .subcommand(Command::new("nci").about("clean install"))
            .subcommand(Command::new("na").about("package manager alias"))
            .subcommand(Command::new("np").about("run shell commands in parallel"))
            .subcommand(Command::new("ns").about("run shell commands sequentially"))
            .subcommand(Command::new("node").about("package-manager-aware node shim"))
            .subcommand(Command::new("init").about("print shell init code"))
            .subcommand(Command::new("doctor").about("print environment and detection diagnostics"))
            .subcommand(Command::new("completion").about("print shell completion script"))
            .after_help(
                "Quick examples:\n\
                 \n\
                 ni vite\n\
                 ni --explain react -D\n\
                 nr dev -- --port=3000\n\
                 nlx create-vite@latest\n\
                 nu --interactive\n\
                 nun --multi-select\n\
                 np \"echo one\" \"echo two\"\n\
                 ns \"npm run build\" \"npm run test\"\n\
                 hni init bash\n\
                 hni doctor\n\
                 hni help ni\n\
                 hni completion zsh\n\
                 node install react",
            ),
    )
}

fn command_help(
    name: &'static str,
    about: &'static str,
    long_about: &'static str,
    examples: &'static str,
) -> Command {
    with_global_flags(
        Command::new(name)
            .about(about)
            .long_about(long_about)
            .arg(command_args_arg())
            .after_help(examples),
    )
}

fn with_global_flags(cmd: Command) -> Command {
    cmd.disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("debug")
                .short('?')
                .long("debug-resolved")
                .help("print resolved command and exit (aliases: --dry-run, --print-command)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("explain")
                .long("explain")
                .help("print detection + resolution details and exit")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("cwd")
                .short('C')
                .value_name("DIR")
                .help("run as if in <dir>")
                .value_parser(value_parser!(std::path::PathBuf))
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("show versions")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("show help")
                .action(ArgAction::SetTrue),
        )
}

fn command_args_arg() -> Arg {
    Arg::new("args")
        .value_name("ARGS")
        .help("arguments forwarded to the resolved command")
        .num_args(0..)
        .allow_hyphen_values(true)
        .action(ArgAction::Append)
}

fn init_shell_arg() -> Arg {
    Arg::new("shell")
        .value_name("SHELL")
        .help("shell to initialize")
        .required(true)
        .value_parser(PossibleValuesParser::new(SUPPORTED_SHELL_NAMES))
}
