use anyhow::Result;
use crate::parse::parse_nun;
use crate::runner::run_cli;

pub async fn command_un() -> Result<()> {
    run_cli(parse_nun).await
}
