use crate::store::Board;
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    /// ID of the task to show
    pub id: u32,
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
        println!("Kind:        {:?}", task.kind);
        println!("Priority:    {:?}", task.priority);
        println!("Status:      {:?}", task.status);
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
                    "  {:?} → {:?}  ({})",
                    t.from,
                    t.to,
                    t.at.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
        }

        Ok(())
    }
}
