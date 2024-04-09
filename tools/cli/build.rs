use anyhow::Result;
use vergen::EmitBuilder;

pub fn main() -> Result<()> {
    println!("running build");
    EmitBuilder::builder().all_build().all_git().emit()?;
    Ok(())
}
