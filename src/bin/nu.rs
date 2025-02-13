use anyhow::Result;
use nirs::parse::parse_nu;
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let cwd = env::current_dir()?;
    let agent = nirs::detect::detect(cwd.as_path()).unwrap().unwrap();
    parse_nu(agent, &args);
    Ok(())
}
