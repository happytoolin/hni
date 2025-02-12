use anyhow::Result;
use crate::parse::parse_nci;
use crate::runner::run_cli;

pub async fn command_ci() -> Result<()> {
    run_cli(parse_nci).await
}
