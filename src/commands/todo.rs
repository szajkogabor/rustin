use crate::store::TaskStatus;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct TodoCommand {
    /// The ID of the task to move to todo
    pub id: u32,
}

impl TodoCommand {
    pub fn run(&self) -> Result<()> {
        crate::commands::move_task_and_list(self.id, TaskStatus::Todo, "Todo")
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
            deleted_at: None,
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
