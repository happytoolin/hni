use std::process::ExitCode;

fn main() -> ExitCode {
    match hni::app::dispatch::run_from_env() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
