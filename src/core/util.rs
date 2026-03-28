use std::process::ExitCode;

pub fn exit_code_from_status(code: Option<i32>) -> ExitCode {
    code.map_or_else(|| ExitCode::from(1), exit_code_from_code)
}

pub fn exit_code_from_code(code: i32) -> ExitCode {
    let code = u8::try_from(code).unwrap_or(1);
    ExitCode::from(code)
}
