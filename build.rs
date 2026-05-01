use anyhow::Result;
use vergen::Emitter;
use vergen_git2::Git2Builder;

pub fn main() -> Result<()> {
    let git2 = Git2Builder::default().describe(true, false, None).build()?;
    Emitter::default().add_instructions(&git2)?.emit()?;
    Ok(())
}
