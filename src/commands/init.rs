use crate::store::Board;
use clap::Args;

#[derive(Args)]
pub struct InitCommand {
    /// The new title for the board
    pub title: Option<String>,
}

impl InitCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut board = Board::load()?;

        if let Some(new_title) = &self.title {
            board.title = new_title.clone();
            tracing::info!("Board title set to: {}", new_title);
        } else {
            tracing::info!("Initialized board: {}", board.title);
        }

        board.save()?;
        crate::commands::list::ListCommand { columns: vec![] }.run()?;

        Ok(())
    }
}
