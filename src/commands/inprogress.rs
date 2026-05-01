use crate::store::{Board, TaskStatus};
use clap::Args;

#[derive(Args)]
pub struct InprogressCommand {
    /// The ID of the task to move to in-progress
    pub id: u32,
}

impl InprogressCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        if let Some(task) = board.tasks.iter_mut().find(|t| t.id == self.id) {
            task.status = TaskStatus::InProgress;
            board.save()?;
            tracing::info!("Task {} moved to In Progress", self.id);
        } else {
            tracing::warn!("Task {} not found", self.id);
        }

        crate::commands::list::ListCommand {}.run()?;
        Ok(())
    }
}
