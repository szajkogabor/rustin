use crate::commands::list::{kind_emoji, priority_emoji};
use crate::store::{Board, TaskStatus};
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    /// ID of the task to show
    pub id: u32,
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
    }
}

impl ShowCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let board = Board::load()?;

        let task = board
            .tasks
            .iter()
            .find(|t| t.id == self.id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", self.id))?;

        println!("ID:          {}", task.id);
        println!("Title:       {}", task.title);
        println!("Kind:        {:?} {}", task.kind, kind_emoji(task.kind));
        println!(
            "Priority:    {:?} {}",
            task.priority,
            priority_emoji(task.priority)
        );
        println!("Status:      {}", status_label(&task.status));
        println!(
            "Created:     {}",
            task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        if let Some(desc) = &task.description {
            println!("Description: {}", desc);
        }
        if !task.transitions.is_empty() {
            println!("History:");
            for t in &task.transitions {
                println!(
                    "  {} → {}  ({})",
                    status_label(&t.from),
                    status_label(&t.to),
                    t.at.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::status_label;
    use crate::store::TaskStatus;

    #[test]
    fn status_label_maps_all_statuses() {
        assert_eq!(status_label(&TaskStatus::Todo), "Todo");
        assert_eq!(status_label(&TaskStatus::InProgress), "In Progress");
        assert_eq!(status_label(&TaskStatus::Done), "Done");
    }
}
