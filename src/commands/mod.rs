use crate::store::{Board, TaskStatus};
use anyhow::Result;

pub mod add;
pub mod display;
pub mod done;
pub mod edit;
pub mod init;
pub mod inprogress;
pub mod list;
pub mod remove;
pub mod show;
pub mod stat;
pub mod todo;
pub mod tui;

pub(crate) fn move_task_and_list(id: u32, to: TaskStatus, status_label: &str) -> Result<()> {
    let mut board = Board::load()?;

    if board.move_task(id, to) {
        board.save()?;
        tracing::info!("Task {} moved to {}", id, status_label);
    } else {
        tracing::warn!("Task {} not found", id);
    }

    crate::commands::list::ListCommand { columns: vec![] }.run()?;
    Ok(())
}
