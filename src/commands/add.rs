use clap::Args;

#[derive(Args)]
pub struct AddCommand {
    /// The title of the task to add
    pub title: String,
}

use crate::store::{Board, Task, TaskStatus};
use chrono::Utc;

impl AddCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        let task = Task {
            id: board.next_id,
            title: self.title.clone(),
            status: TaskStatus::Todo,
            created_at: Utc::now(),
        };

        board.next_id += 1;
        board.tasks.push(task.clone());
        board.save()?;

        tracing::info!("Task added successfully: [{}] {}", task.id, task.title);

        crate::commands::list::ListCommand {}.run()?;
        Ok(())
    }
}
