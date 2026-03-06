use std::process::ExitCode;

fn main() -> ExitCode {
    match hni::app::dispatch::run_from_env() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", hni::app::error_report::render_error(&err));
            ExitCode::from(1)
        }
    }
}
