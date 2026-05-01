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

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{StatusTransition, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::Utc;

    fn make_task(status: TaskStatus) -> Task {
        Task {
            id: 1,
            title: "Task".to_string(),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status,
            created_at: Utc::now(),
            transitions: vec![],
        }
    }

    #[test]
    fn marking_inprogress_sets_status() {
        let mut task = make_task(TaskStatus::Todo);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from,
            to: TaskStatus::InProgress,
            at: Utc::now(),
        });
        task.status = TaskStatus::InProgress;
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn marking_inprogress_records_transition_from_todo() {
        let mut task = make_task(TaskStatus::Todo);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from: from.clone(),
            to: TaskStatus::InProgress,
            at: Utc::now(),
        });
        task.status = TaskStatus::InProgress;
        assert_eq!(task.transitions[0].from, TaskStatus::Todo);
        assert_eq!(task.transitions[0].to, TaskStatus::InProgress);
    }
}
