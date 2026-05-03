use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ArchiveCommand;

impl ArchiveCommand {
    pub fn run(&self) -> Result<()> {
        let mut board = Board::load()?;
        let count = board.archive_done();

        if count > 0 {
            board.save()?;
            tracing::info!("Archived {} done task(s)", count);
        } else {
            tracing::info!("No done tasks to archive");
        }

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}
