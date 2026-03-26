use std::{
    path::Path,
    process::{Command, Stdio},
};

pub fn shell_command(command_string: &str) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command_string]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command_string]);
        cmd
    }
}

pub fn configure_command(mut command: Command, cwd: &Path) -> Command {
    command
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
}

pub fn shell_escape(input: &str) -> String {
    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '=' | '@'))
    {
        return input.to_string();
    }

    let double_quoted = format!("\"{}\"", input.replace('"', "\\\""));
    if shlex::split(&double_quoted).is_some_and(|parts| parts.len() == 1 && parts[0] == input) {
        return double_quoted;
    }

    shlex::try_quote(input)
        .map(std::borrow::Cow::into_owned)
        .unwrap_or(double_quoted)
}
