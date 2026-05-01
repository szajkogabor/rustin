use crate::store::Board;
use clap::Args;

#[derive(Args)]
pub struct RemoveCommand {
    /// The ID of the task to remove
    pub id: u32,
}

impl RemoveCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        let initial_len = board.tasks.len();
        board.tasks.retain(|t| t.id != self.id);

        if board.tasks.len() < initial_len {
            board.save()?;
            tracing::info!("Task {} removed", self.id);
        } else {
            tracing::warn!("Task {} not found", self.id);
        }

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}
