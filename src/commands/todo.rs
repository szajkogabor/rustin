use crate::store::{Board, StatusTransition, TaskStatus};
use chrono::Utc;
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
            let from = task.status.clone();
            task.transitions.push(StatusTransition {
                from,
                to: TaskStatus::Todo,
                at: Utc::now(),
            });
            task.status = TaskStatus::Todo;
            board.save()?;
            tracing::info!("Task {} moved to Todo", self.id);
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
    fn moving_to_todo_sets_status() {
        let mut task = make_task(TaskStatus::Done);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from,
            to: TaskStatus::Todo,
            at: Utc::now(),
        });
        task.status = TaskStatus::Todo;
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn moving_to_todo_records_transition_from_done() {
        let mut task = make_task(TaskStatus::Done);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from: from.clone(),
            to: TaskStatus::Todo,
            at: Utc::now(),
        });
        task.status = TaskStatus::Todo;
        assert_eq!(task.transitions[0].from, TaskStatus::Done);
        assert_eq!(task.transitions[0].to, TaskStatus::Todo);
    }

    #[test]
    fn multiple_transitions_are_all_recorded() {
        let mut task = make_task(TaskStatus::Todo);
        for (from_s, to_s) in [
            (TaskStatus::Todo, TaskStatus::InProgress),
            (TaskStatus::InProgress, TaskStatus::Done),
            (TaskStatus::Done, TaskStatus::Todo),
        ] {
            task.transitions.push(StatusTransition {
                from: from_s,
                to: to_s,
                at: Utc::now(),
            });
        }
        assert_eq!(task.transitions.len(), 3);
    }
}
