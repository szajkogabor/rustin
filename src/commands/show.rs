use crate::commands::display::{kind_emoji, priority_emoji, status_label};
use crate::store::{Board, Task};
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    /// ID of the task to show
    pub id: u32,
}

pub(crate) fn task_detail_lines(task: &Task) -> Vec<String> {
    let mut lines = vec![
        format!("ID:          {}", task.id),
        format!("Title:       {}", task.title),
        format!("Kind:        {:?} {}", task.kind, kind_emoji(task.kind)),
        format!(
            "Priority:    {:?} {}",
            task.priority,
            priority_emoji(task.priority)
        ),
        format!("Status:      {}", status_label(&task.status)),
        format!(
            "Created:     {}",
            task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ),
    ];

    if let Some(desc) = &task.description {
        lines.push(format!("Description: {}", desc));
    }
    if !task.transitions.is_empty() {
        lines.push("History:".to_string());
        for transition in &task.transitions {
            lines.push(format!(
                "  {} → {}  ({})",
                status_label(&transition.from),
                status_label(&transition.to),
                transition.at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }
    }

    lines
}

impl ShowCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let board = Board::load()?;

        let task = board
            .tasks
            .iter()
            .find(|t| t.id == self.id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", self.id))?;

        for line in task_detail_lines(task) {
            println!("{line}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::task_detail_lines;
    use crate::store::{StatusTransition, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::{TimeZone, Utc};

    fn sample_task() -> Task {
        Task {
            id: 7,
            title: "Investigate bug".to_string(),
            priority: TaskPriority::High,
            kind: TaskKind::Bug,
            description: Some("Collect logs".to_string()),
            status: TaskStatus::Done,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
            transitions: vec![StatusTransition {
                from: TaskStatus::InProgress,
                to: TaskStatus::Done,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            }],
        }
    }

    #[test]
    fn task_detail_lines_include_description_and_history() {
        let lines = task_detail_lines(&sample_task());
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Title:       Investigate bug"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Description: Collect logs"))
        );
        assert!(lines.iter().any(|line| line.contains("History:")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("In Progress → Done  (2024-01-01 10:00:00 UTC)"))
        );
    }
}
