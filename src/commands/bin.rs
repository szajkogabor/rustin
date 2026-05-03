use crate::commands::display::{format_task, task_order};
use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct BinCommand;

impl BinCommand {
    pub fn run(&self) -> Result<()> {
        let board = Board::load()?;
        let mut deleted: Vec<_> = board.deleted_tasks();

        if deleted.is_empty() {
            println!("Bin is empty.");
            return Ok(());
        }

        deleted.sort_by(|a, b| task_order(a, b));

        println!("=== {} bin ===", board.title);
        for task in &deleted {
            println!("  {}", format_task(task));
        }

        Ok(())
    }
}
