use anyhow::Result;

pub trait NirsCommand {
    fn run(&self, args: Vec<String>) -> Result<()>;
    fn before_run(&self) -> Result<()> {
        Ok(())
    }
    fn after_run(&self) -> Result<()> {
        Ok(())
    }
}
