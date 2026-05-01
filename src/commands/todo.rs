use crate::store::{Board, TaskStatus};
use clap::Args;

#[derive(Args)]
pub struct TodoCommand {
    /// The ID of the task to move to todo
    pub id: u32,
}

impl TodoCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        if let Some(task) = board.tasks.iter_mut().find(|t| t.id == self.id) {
            task.status = TaskStatus::Todo;
            board.save()?;
            tracing::info!("Task {} moved to Todo", self.id);
        } else {
            tracing::warn!("Task {} not found", self.id);
        }

        crate::commands::list::ListCommand {}.run()?;
        Ok(())
    }
}
