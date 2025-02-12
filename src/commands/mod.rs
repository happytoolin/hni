pub mod na;
pub mod nci;
pub mod ni;
pub mod nlx;
pub mod nr;
pub mod nu;
pub mod nun;

use anyhow::Result;

pub async fn handle_command(command: &str) -> Result<()> {
    match command {
        "na" => na::command_a().await,
        "nci" => nci::command_ci().await,
        "ni" => ni::interactive_install().await,
        "nlx" => nlx::command_lx().await,
        "nr" => {
            println!("Executing nr command");
            let result = nr::command_r().await;
            println!("nr command result: {:?}", result);
            result
        },
        "nu" => nu::command_u().await,
        "nun" => nun::command_un().await,
        _ => {
            println!("Unknown command: {}", command);
            Ok(())
        }
    }
}
