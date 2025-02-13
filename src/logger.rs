use env_logger::Builder;
use std::io::Write;

/// Initialize the logger with nice formatting and environment variable configuration
pub fn init() {
    let env = env_logger::Env::default().filter_or("LOG_LEVEL", "info");

    Builder::from_env(env)
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn => "\x1b[33m",  // Yellow
                log::Level::Info => "\x1b[32m",  // Green
                log::Level::Debug => "\x1b[36m", // Cyan
                log::Level::Trace => "\x1b[37m", // White
            };

            let reset = "\x1b[0m";
            let bold = "\x1b[1m";

            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };

            writeln!(
                buf,
                "{}{:>5}{} {} {}{}{} {}",
                level_style,
                record.level(),
                reset,
                chrono::Local::now().format("%H:%M:%S"),
                bold,
                target,
                reset,
                record.args()
            )
        })
        .format_target(true)
        .format_timestamp(None)
        .init();

    log::debug!(
        "Logger initialized with environment: LOG_LEVEL={}",
        std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
    );
}
