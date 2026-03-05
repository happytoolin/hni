pub fn shell_escape(input: &str) -> String {
    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '=' | '@'))
    {
        return input.to_string();
    }

    let escaped = input.replace('"', "\\\"");
    format!("\"{escaped}\"")
}
