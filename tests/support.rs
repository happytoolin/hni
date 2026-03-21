#![allow(dead_code)]

use std::{
    ffi::OsStr,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn with_env_lock<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    f()
}

/// Set a process environment variable for tests.
pub fn set_var<K, V>(key: K, value: V)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // SAFETY: Callers must hold the global env lock via `with_env_lock`, ensuring
    // serialized environment mutation within the test process.
    unsafe { std::env::set_var(key, value) };
}

/// Remove a process environment variable for tests.
pub fn remove_var<K>(key: K)
where
    K: AsRef<OsStr>,
{
    // SAFETY: Callers must hold the global env lock via `with_env_lock`, ensuring
    // serialized environment mutation within the test process.
    unsafe { std::env::remove_var(key) };
}

/// Run hni with the given arguments and extra environment variables.
pub fn run_hni(args: Vec<&str>, extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(hni_executable_path());
    cmd.args(args)
        .env("HNI_AUTO_INSTALL", "false")
        .env_remove("HNI_REAL_NODE")
        .env_remove("HNI_NODE_SHIM_ACTIVE");

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    cmd.output().expect("failed to run hni")
}

pub fn real_node_supports_run() -> bool {
    let output = match Command::new("node").arg("--help").output() {
        Ok(output) => output,
        Err(_) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("--run")
}

/// Get the path to the hni executable.
pub fn hni_executable_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_hni") {
        return PathBuf::from(path);
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(if cfg!(windows) { "hni.exe" } else { "hni" });
    path
}
