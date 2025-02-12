use anyhow::Result;
use clap::Parser;
use crate::detect::detect;
use crate::runner::run;
use crate::ResolvedCommand;

#[derive(Parser, Debug)]
#[clap(name = "nr", about = "Runs a script defined in package.json")]
struct Args {
    script: Option<String>,
    #[clap(long = "if-present")]
    if_present: bool,
    #[clap(short = 'C', long = "cwd", default_value = ".")]
    cwd: String,
}

pub async fn command_r() -> Result<()> {
    let args = Args::parse();

    let package_manager_factory = detect(Some(args.cwd.clone()))?;

    let mut run_args = vec![];
    if args.if_present {
        run_args.push("--if-present".to_string());
    }
    if let Some(script) = args.script {
        run_args.push(script);
    } else {
        println!("No script provided");
        return Ok(());
    }

    match package_manager_factory {
        Ok(Some(package_manager_factory)) => {
            let factory = package_manager_factory.get_factory();
            let executor = factory.create_commands();
            if let Some(command) = executor.run(run_args.iter().map(|s| s.as_str()).collect()) {
                crate::runner::run(command).await?;
            } else {
                println!("Failed to construct run command");
            }
        }
        Ok(None) => {
            println!("No package manager detected");
        }
        Err(e) => {
            println!("Error detecting package manager: {}", e);
        }
    }

    Ok(())
}
