use clap::Args;

#[derive(Args)]
pub struct AddCommand {
    /// The title of the task to add
    pub title: String,

    /// Task priority
    #[arg(short, long, value_enum, default_value_t = crate::store::TaskPriority::Medium)]
    pub priority: crate::store::TaskPriority,

    /// Task kind
    #[arg(short, long, value_enum, default_value_t = crate::store::TaskKind::Feature)]
    pub kind: crate::store::TaskKind,

    /// Optional description
    #[arg(short, long)]
    pub description: Option<String>,
}

use crate::store::{Board, Task, TaskStatus};
use chrono::Utc;

impl AddCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        let task = Task {
            id: board.next_id,
            title: self.title.clone(),
            priority: self.priority,
            kind: self.kind,
            description: self.description.clone(),
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
