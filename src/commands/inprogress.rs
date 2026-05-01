use crate::store::{Board, StatusTransition, TaskStatus};
use chrono::Utc;
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
            let from = task.status.clone();
            task.transitions.push(StatusTransition {
                from,
                to: TaskStatus::InProgress,
                at: Utc::now(),
            });
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
