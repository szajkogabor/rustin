use anyhow::Result;
use vergen::Emitter;

pub fn main() -> Result<()> {
    let describe = vergen_git2::Git2::builder()
        .describe(true, true, None)
        .build();

    Emitter::default().add_instructions(&describe)?.emit()?;
    Ok(())
}
