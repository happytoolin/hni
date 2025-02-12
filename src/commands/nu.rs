use anyhow::Result;
use crate::parse::parse_nu;
use crate::runner::run_cli;

pub async fn command_u() -> Result<()> {
    run_cli(parse_nu).await
}
