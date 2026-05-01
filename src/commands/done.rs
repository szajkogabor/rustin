use crate::store::{Board, TaskStatus};
use clap::Args;

#[derive(Args)]
pub struct DoneCommand {
    /// The ID of the task to mark as done
    pub id: u32,
}

impl DoneCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        if let Some(task) = board.tasks.iter_mut().find(|t| t.id == self.id) {
            task.status = TaskStatus::Done;
            board.save()?;
            tracing::info!("Task {} moved to Done", self.id);
        } else {
            tracing::warn!("Task {} not found", self.id);
        }

        crate::commands::list::ListCommand {}.run()?;
        Ok(())
    }
}
