use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RemoveCommand {
    /// The ID of the task to remove
    pub id: u32,
}

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use crate::store::{Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::Utc;

    fn make_task(id: u32) -> Task {
        Task {
            id,
            title: format!("task-{id}"),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
        }
    }

    #[test]
    fn retain_removes_task_by_id() {
        let mut tasks = vec![make_task(1), make_task(2), make_task(3)];
        tasks.retain(|t| t.id != 2);
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|t| t.id != 2));
    }

    #[test]
    fn retain_nonexistent_id_leaves_tasks_unchanged() {
        let mut tasks = vec![make_task(1), make_task(2)];
        let before = tasks.len();
        tasks.retain(|t| t.id != 999);
        assert_eq!(tasks.len(), before);
    }

    #[test]
    fn retain_all_other_tasks_are_preserved() {
        let mut tasks = vec![make_task(1), make_task(2), make_task(3)];
        tasks.retain(|t| t.id != 1);
        let ids: Vec<u32> = tasks.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![2, 3]);
    }
}
