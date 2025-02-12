use anyhow::Result;

pub async fn command_a() -> Result<()> {
    println!("Command A executed");
    Ok(())
}
