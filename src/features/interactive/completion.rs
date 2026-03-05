use std::collections::BTreeSet;

pub fn completion_script_bash(command: &str) -> String {
    format!(
        r#"###-begin-{command}-completion-###
_{command}_completion() {{
  local cword="${{COMP_CWORD}}"
  local words=("${{COMP_WORDS[@]}}")
  COMPREPLY=($(COMP_CWORD=$cword COMP_LINE="$COMP_LINE" {command} --completion "${{words[@]}}"))
}}
complete -F _{command}_completion {command}
###-end-{command}-completion-###
"#
    )
}

pub fn completion_script_zsh(command: &str) -> String {
    format!(
        r#"#compdef {command}
_{command}() {{
  local -a suggestions
  suggestions=($({command} --completion "$words[@]"))
  _describe '{command} scripts' suggestions
}}
compdef _{command} {command}
"#
    )
}

pub fn completion_script_fish(command: &str) -> String {
    format!(
        r"function __{command}_complete
  set -lx COMP_LINE (commandline -cp)
  set -lx COMP_CWORD (count (commandline -opc))
  {command} --completion (commandline -opc)
end
complete -f -c {command} -a '(__{command}_complete)'
"
    )
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
