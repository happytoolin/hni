use anyhow::Result;
use clap::Parser;
use crate::detect::detect;
use crate::runner::run;
use crate::{ResolvedCommand, detect::PackageManagerFactoryEnum};

#[derive(Parser, Debug)]
#[clap(name = "nr", about = "Runs a script defined in package.json")]
struct Args {
    script: Option<String>,
    #[clap(long = "if-present")]
    if_present: bool,
    #[clap(short = 'C', long = "cwd", default_value = ".")]
    cwd: String,
    #[clap(multiple = true, last = true)]
    args: Vec<String>,
}

pub async fn command_r() -> Result<()> {
    let args = Args::parse();

    let cwd = std::path::Path::new(&args.cwd);
    let package_manager_factory = match detect(Some(cwd)) {
        Ok(Some(factory)) => {
            println!("Detected package manager: {:?}", factory);
            Ok(Some(factory))
        }
        Ok(None) => {
            println!("No script provided");
            return Err(anyhow::anyhow!("No script provided"));
        }
        Err(e) => {
            println!("Error detecting package manager: {}", e);
            return Err(e);
        }
    }?;

    let mut run_args = vec![];
    if args.if_present {
        run_args.push("--if-present".to_string());
    }
    if let Some(script) = args.script {
        run_args.push(script);
        run_args.extend(args.args);
    } else {
        return Ok(());
    }

    match package_manager_factory {
        Ok(Some(package_manager_factory)) => {
            let factory = package_manager_factory.get_factory();
            let executor = factory.create_commands();
            if let Some(command) = executor.run(&run_args.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                let result = crate::runner::run(command).await;
            } else {
                return Err(anyhow::anyhow!("Failed to construct run command"));
            }
        }
        Ok(None) => {
            println!("No package manager detected");
            return Ok(());
        }
        Err(e) => {
            println!("Error detecting package manager: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
