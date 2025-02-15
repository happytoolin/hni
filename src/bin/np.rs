use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::thread;
use log::{debug, error, info, warn};
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug)]
enum CommandResult {
    Success,
    Failure,
}

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
                        if let Err(send_err) =
                            result_sender.send((command_string.clone(), CommandResult::Failure))
                        {
                            error!("Failed to send result: {}", send_err);
                        }
                        return;
                    }
                };

                let stdout = child.stdout.take().expect("Could not capture stdout");
                let stderr = child.stderr.take().expect("Could not capture stderr");

                let stdout_reader = BufReader::new(stdout);
                let stderr_reader = BufReader::new(stderr);

                // Spawn threads to read stdout and stderr
                thread::scope(|s| {
                    let stdout_handle = s.spawn(|_| {
                        for line in stdout_reader.lines() {
                            match line {
                                Ok(line) => println!("{}", line),
                                Err(e) => error!("Error reading stdout: {}", e),
                            }
                        }
                    });

                    let stderr_handle = s.spawn(|_| {
                        for line in stderr_reader.lines() {
                            match line {
                                Ok(line) => eprintln!("{}", line),
                                Err(e) => error!("Error reading stderr: {}", e),
                            }
                        }
                    });

                    loop {
                        if signal_receiver.try_recv().is_ok() {
                            info!(
                                "Received termination signal. Stopping command: {}",
                                command_string
                            );

                            // More cross-platform compatible way to kill the process
                            if let Err(e) = child.kill() {
                                error!("Failed to kill process: {}", e);
                            }

                            // Wait for the process to fully exit
                            match child.wait() {
                                Ok(status) => {
                                    // Wait for the I/O threads to complete
                                    stdout_handle.join().unwrap();
                                    stderr_handle.join().unwrap();

                                    // For Ctrl+C, we consider it a success if the process exits cleanly
                                    info!("Command '{}' gracefully terminated", command_string);
                                    if let Err(e) = result_sender
                                        .send((command_string.clone(), CommandResult::Success))
                                    {
                                        error!("Failed to send result: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Error waiting for process to exit: {}", e);
                                    if let Err(send_err) = result_sender
                                        .send((command_string.clone(), CommandResult::Failure))
                                    {
                                        error!("Failed to send result: {}", send_err);
                                    }
                                }
                            }
                            return;
                        }

                        match child.try_wait() {
                            Ok(Some(status)) => {
                                // Wait for the I/O threads to complete
                                stdout_handle.join().unwrap();
                                stderr_handle.join().unwrap();

                                if status.success() {
                                    info!("Command '{}' completed successfully!", command_string);
                                    if let Err(e) = result_sender
                                        .send((command_string.clone(), CommandResult::Success))
                                    {
                                        error!("Failed to send result: {}", e);
                                    }
                                } else {
                                    // Only consider it a failure if the process wasn't terminated by a signal
                                    let exit_code = status.code();
                                    match exit_code {
                                        Some(code) => {
                                            error!(
                                                "Command '{}' failed with exit code: {}",
                                                command_string, code
                                            );
                                            if let Err(e) = result_sender.send((
                                                command_string.clone(),
                                                CommandResult::Failure,
                                            )) {
                                                error!("Failed to send result: {}", e);
                                            }
                                        }
                                        None => {
                                            // Process was terminated by a signal (e.g., Ctrl+C)
                                            info!(
                                                "Command '{}' was terminated by a signal",
                                                command_string
                                            );
                                            if let Err(e) = result_sender.send((
                                                command_string.clone(),
                                                CommandResult::Success,
                                            )) {
                                                error!("Failed to send result: {}", e);
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                            Ok(None) => {
                                // Sleep to avoid busy-waiting
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                            Err(e) => {
                                error!("Error waiting for process: {}", e);
                                if let Err(send_err) = result_sender
                                    .send((command_string.clone(), CommandResult::Failure))
                                {
                                    error!("Failed to send result: {}", send_err);
                                }
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
    drop(result_sender); // Drop the original sender so the loop terminates when all threads are done.
    for (command_string, result) in result_receiver {
        match result {
            CommandResult::Success => {}
            CommandResult::Failure => {
                error!("Command failed: {}", command_string);
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
