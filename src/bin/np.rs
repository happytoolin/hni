use anyhow::Result;
use log::{debug, error, info, warn};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    nirs::logger::init();

    let cwd = env::current_dir()?;
    debug!("Current working directory: {}", cwd.display());

    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        warn!("No commands provided to run in parallel.");
        return Ok(());
    }

    info!("Running commands in parallel...");

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&term))?;
    signal_hook::flag::register(SIGINT, Arc::clone(&term))?;

    let mut handles = vec![];
    let _child_processes: Vec<u32> = vec![];
    let mut failed = false;

    for command_string in args {
        let command_string = command_string.clone();
        let cwd = cwd.clone();
        let term = Arc::clone(&term);

        let handle = thread::spawn(move || -> Result<bool> {
            debug!("Executing command: {}", command_string);

            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(&command_string);
            cmd.current_dir(cwd);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn()?;

            let stdout = child.stdout.take().expect("Could not capture stdout");
            let stderr = child.stderr.take().expect("Could not capture stderr");

            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            // Spawn threads to read stdout and stderr
            let stdout_handle = thread::spawn(move || {
                let reader = BufReader::new(stdout_reader);
                for line in reader.lines() {
                    match line {
                        Ok(line) => println!("{}", line),
                        Err(e) => error!("Error reading stdout: {}", e),
                    }
                }
            });

            let stderr_handle = thread::spawn(move || {
                let reader = BufReader::new(stderr_reader);
                for line in reader.lines() {
                    match line {
                        Ok(line) => eprintln!("{}", line),
                        Err(e) => error!("Error reading stderr: {}", e),
                    }
                }
            });

            loop {
                if term.load(Ordering::Relaxed) {
                    info!(
                        "Received termination signal. Stopping command: {}",
                        command_string
                    );
                    // Attempt to kill the child process
                    if let Err(e) = Command::new("kill")
                        .arg("-s")
                        .arg("SIGINT")
                        .arg(child.id().to_string())
                        .status()
                    {
                        error!("Failed to kill process: {}", e);
                    }
                    break;
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        stdout_handle.join().unwrap();
                        stderr_handle.join().unwrap();
                        if status.success() {
                            info!("Command completed successfully!");
                            return Ok(true);
                        } else {
                            error!("Command failed with exit code: {:?}", status.code());
                            return Ok(false);
                        }
                    }
                    Ok(None) => {
                        // Sleep to avoid busy-waiting
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Error waiting for process: {}", e);
                        return Err(e.into());
                    }
                }
            }

            Ok(false)
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.join() {
            Ok(result) => match result {
                Ok(success) => {
                    if !success {
                        failed = true;
                    }
                }
                Err(e) => {
                    error!("Command failed: {:?}", e);
                    failed = true;
                }
            },
            Err(e) => {
                error!("Thread panicked: {:?}", e);
                failed = true;
            }
        }
    }

    if failed {
        std::process::exit(1);
    }

    info!("All commands completed!");
    Ok(())
}
