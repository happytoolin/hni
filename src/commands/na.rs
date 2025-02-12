use anyhow::Result;
use crate::parse::parse_na;
use crate::runner::run_cli;

pub async fn command_a() -> Result<()> {
    run_cli(parse_na).await
}
