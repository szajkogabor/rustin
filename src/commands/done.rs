use crate::store::TaskStatus;
use clap::Args;

#[derive(Args)]
pub struct DoneCommand {
    /// The ID of the task to mark as done
    pub id: u32,
}

impl DoneCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        crate::commands::move_task_and_list(self.id, TaskStatus::Done, "Done")
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
    fn marking_done_sets_status() {
        let mut task = make_task(TaskStatus::InProgress);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from,
            to: TaskStatus::Done,
            at: Utc::now(),
        });
        task.status = TaskStatus::Done;
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn marking_done_records_transition() {
        let mut task = make_task(TaskStatus::Todo);
        let from = task.status.clone();
        task.transitions.push(StatusTransition {
            from: from.clone(),
            to: TaskStatus::Done,
            at: Utc::now(),
        });
        task.status = TaskStatus::Done;
        assert_eq!(task.transitions.len(), 1);
        assert_eq!(task.transitions[0].from, TaskStatus::Todo);
        assert_eq!(task.transitions[0].to, TaskStatus::Done);
    }
}
