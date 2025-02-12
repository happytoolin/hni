use anyhow::Result;
use crate::parse::parse_nlx;
use crate::runner::run_cli;

pub async fn command_lx() -> Result<()> {
    run_cli(parse_nlx).await
}
