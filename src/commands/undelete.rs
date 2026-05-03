use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct UndeleteCommand {
    /// The ID of the task to restore
    pub id: u32,
}

impl UndeleteCommand {
    pub fn run(&self) -> Result<()> {
        let mut board = Board::load()?;

        if board.undelete(self.id) {
            board.save()?;
            tracing::info!("Task {} restored", self.id);
        } else {
            tracing::warn!("Task {} not found in bin", self.id);
        }

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}
