use anyhow::Result;
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
    pub fn run(&self) -> Result<()> {
        let mut board = Board::load()?;

        let task = Task {
            id: board.next_id,
            title: self.title.clone(),
            priority: self.priority,
            kind: self.kind,
            description: self.description.clone(),
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
        };

        board.next_id += 1;
        board.tasks.push(task.clone());
        board.save()?;

        tracing::info!("Task added successfully: [{}] {}", task.id, task.title);

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{Board, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::Utc;

    fn make_task(id: u32, title: &str) -> Task {
        Task {
            id,
            title: title.to_string(),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
        }
    }

    #[test]
    fn new_task_starts_with_todo_status() {
        let task = make_task(1, "Write tests");
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn new_task_has_empty_transitions() {
        let task = make_task(1, "Task");
        assert!(task.transitions.is_empty());
    }

    #[test]
    fn adding_task_increments_next_id() {
        let mut board = Board::default();
        assert_eq!(board.next_id, 1);
        let task = make_task(board.next_id, "First");
        board.next_id += 1;
        board.tasks.push(task);
        assert_eq!(board.next_id, 2);
        assert_eq!(board.tasks.len(), 1);
    }

    #[test]
    fn task_id_matches_next_id_at_creation() {
        let mut board = Board::default();
        let task = make_task(board.next_id, "Task");
        assert_eq!(task.id, 1);
        board.next_id += 1;
        let task2 = make_task(board.next_id, "Task2");
        assert_eq!(task2.id, 2);
    }
}
