use crate::commands::display::task_detail_lines;
use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    /// ID of the task to show
    pub id: u32,
}

impl ShowCommand {
    pub fn run(&self) -> Result<()> {
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
