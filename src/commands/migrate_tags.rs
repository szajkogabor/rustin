use crate::store::Board;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct MigrateTagsCommand;

impl MigrateTagsCommand {
    pub fn run(&self) -> Result<()> {
        let mut board = Board::load()?;
        let count = board.migrate_tags();

        if count > 0 {
            board.save()?;
            tracing::info!("Migrated tags for {} task(s)", count);
        } else {
            tracing::info!("No bracketed tags found in task titles");
        }

        crate::commands::list::ListCommand {
            columns: vec![],
            tags: vec![],
        }
        .run()?;
        Ok(())
    }
}
