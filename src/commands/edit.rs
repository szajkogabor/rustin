use crate::store::{Board, TaskKind, TaskPriority};
use clap::Args;

#[derive(Args)]
pub struct EditCommand {
    /// ID of the task to edit
    pub id: u32,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New priority
    #[arg(short, long, value_enum)]
    pub priority: Option<TaskPriority>,

    /// New kind
    #[arg(short, long, value_enum)]
    pub kind: Option<TaskKind>,

    /// New description (pass empty string to clear)
    #[arg(short, long)]
    pub description: Option<String>,
}

impl EditCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        let task = board
            .tasks
            .iter_mut()
            .find(|t| t.id == self.id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", self.id))?;

        if let Some(title) = &self.title {
            task.title = title.clone();
        }
        if let Some(priority) = self.priority {
            task.priority = priority;
        }
        if let Some(kind) = self.kind {
            task.kind = kind;
        }
        if let Some(description) = &self.description {
            task.description = if description.is_empty() {
                None
            } else {
                Some(description.clone())
            };
        }

        board.save()?;

        crate::commands::list::ListCommand {}.run()?;
        Ok(())
    }
}
