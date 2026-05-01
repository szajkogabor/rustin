use clap::Args;

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
}

impl From<PriorityArg> for crate::store::TaskPriority {
    fn from(value: PriorityArg) -> Self {
        match value {
            PriorityArg::Low => Self::Low,
            PriorityArg::Medium => Self::Medium,
            PriorityArg::High => Self::High,
        }
    }
}

#[derive(Args)]
pub struct AddCommand {
    /// The title of the task to add
    pub title: String,

    /// Task priority
    #[arg(short, long, value_enum, default_value_t = PriorityArg::Medium)]
    pub priority: PriorityArg,
}

use crate::store::{Board, Task, TaskStatus};
use chrono::Utc;

impl AddCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        let task = Task {
            id: board.next_id,
            title: self.title.clone(),
            priority: self.priority.into(),
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
