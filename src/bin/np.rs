use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::thread;
use log::{debug, error, info, warn};
use nirs::{logger, NirsCommand};
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum CommandResult {
    Success,
    Failure,
    Interrupted,
}

struct NpCommand;

impl NirsCommand for NpCommand {
    fn run(&self, args: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        debug!("Current working directory: {}", cwd.display());

        if args.is_empty() {
            warn!("No commands provided to run in parallel.");
            return Ok(());
        }

        info!("Running commands in parallel...");

        // Use crossbeam channels for signal handling
        let (signal_sender, signal_receiver): (Sender<()>, Receiver<()>) = unbounded();

        ctrlc::set_handler(move || {
            if let Err(e) = signal_sender.send(()) {
                error!("Failed to send signal: {}", e);
            }
        })
        .expect("Error setting Ctrl-C handler");

        let (result_sender, result_receiver): (
            Sender<(String, CommandResult)>,
            Receiver<(String, CommandResult)>,
        ) = unbounded();

        thread::scope(|s| {
            for command_string in args {
                let command_string = command_string.clone();
                let signal_receiver = signal_receiver.clone();
                let result_sender = result_sender.clone();
                let cwd = cwd.clone();

                s.spawn(move |_| {
                    let start_time = Instant::now();
                    debug!("Executing command: {}", command_string);

                    let mut cmd = if cfg!(target_os = "windows") {
                        let mut cmd = Command::new("cmd");
                        cmd.args(["/C", &command_string]);
                        cmd
                    } else {
                        let mut cmd = Command::new("sh");
                        cmd.args(["-c", &command_string]);
                        cmd
                    };

                    cmd.current_dir(&cwd);
                    cmd.stdout(Stdio::piped());
                    cmd.stderr(Stdio::piped());

                    let mut child = match cmd.spawn() {
                        Ok(child) => child,
                        Err(e) => {
                            error!("Failed to spawn command '{}': {}", command_string, e);
                            let _ =
                                result_sender.send((command_string.clone(), CommandResult::Failure));
                            return;
                        }
                    };

                    let stdout = child.stdout.take().expect("Could not capture stdout");
                    let stderr = child.stderr.take().expect("Could not capture stderr");

                    let stdout_reader = BufReader::new(stdout);
                    let stderr_reader = BufReader::new(stderr);

                    // Spawn threads to read stdout and stderr
                    let _ = thread::scope(|s| {
                        let _stdout_handle = s.spawn(|_| {
                            for line in stdout_reader.lines() {
                                match line {
                                    Ok(line) => println!("{}", line),
                                    Err(e) => error!("Error reading stdout: {}", e),
                                }
                            }
                        });

                        let _stderr_handle = s.spawn(|_| {
                            for line in stderr_reader.lines() {
                                match line {
                                    Ok(line) => eprintln!("{}", line),
                                    Err(e) => error!("Error reading stderr: {}", e),
                                }
                            }
                        });

                        loop {
                            // Check for termination signal first
                            if signal_receiver.try_recv().is_ok() {
                                info!(
                                    "Termination signal received for command: {}",
                                    command_string
                                );
                                if let Err(e) = child.kill() {
                                    error!("Failed to kill process: {}", e);
                                }

                                let exit_status = child
                                    .wait()
                                    .map_err(|e| error!("Error waiting for process: {}", e))
                                    .ok();

                                let result = match exit_status.and_then(|s| s.code()) {
                                    Some(0) => {
                                        info!(
                                            "Command exited cleanly after signal: {}",
                                            command_string
                                        );
                                        CommandResult::Success
                                    }
                                    _ => {
                                        warn!(
                                            "Command interrupted before clean exit: {}",
                                            command_string
                                        );
                                        CommandResult::Interrupted
                                    }
                                };

                                let duration = start_time.elapsed();
                                info!(
                                    "Command '{}' terminated after {:.2?}",
                                    command_string, duration
                                );

                                let _ = result_sender.send((command_string.clone(), result));
                                return;
                            }

                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    let duration = start_time.elapsed();
                                    let result = if status.success() {
                                        info!(
                                            "Command '{}' succeeded in {:.2?}",
                                            command_string, duration
                                        );
                                        CommandResult::Success
                                    } else {
                                        match status.code() {
                                            Some(code) => {
                                                error!(
                                                    "Command '{}' failed with exit code {} in {:.2?}",
                                                    command_string, code, duration
                                                );
                                                CommandResult::Failure
                                            }
                                            None => {
                                                warn!(
                                                    "Command '{}' terminated by signal in {:.2?}",
                                                    command_string, duration
                                                );
                                                CommandResult::Interrupted
                                            }
                                        }
                                    };

                                    let _ = result_sender.send((command_string.clone(), result));
                                    return;
                                }
                                Ok(None) => {
                                    // Reduce sleep time for more responsive termination
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                                Err(e) => {
                                    error!("Error waiting for process: {}", e);
                                    let _ = result_sender
                                        .send((command_string.clone(), CommandResult::Failure));
                                    return;
                                }
                            }
                        }
                    });
                });
            }
        })
        .unwrap();

        // Ensure all threads complete and collect results
        let mut failed = false;
        let mut interrupted = false;
        drop(result_sender); // Drop the original sender so the loop terminates when all threads are done.
        for (command_string, result) in result_receiver {
            match result {
                CommandResult::Success => {}
                CommandResult::Failure => {
                    error!("Command failed: {}", command_string);
                    failed = true;
                }
                CommandResult::Interrupted => {
                    warn!("Command interrupted: {}", command_string);
                    interrupted = true;
                }
            }
        }

        if failed {
            std::process::exit(1);
        } else if interrupted {
            info!("Operations interrupted by user");
            std::process::exit(130); // Standard SIGINT exit code
        }

        info!("All commands completed successfully!");
        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    let args: Vec<String> = env::args().skip(1).collect();
    let command = NpCommand;
    command.run(args)
}
