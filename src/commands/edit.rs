use crate::store::{Board, TaskKind, TaskPriority};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct EditCommand {
    /// ID of the task to edit
    pub id: u32,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New priority
    #[arg(short, long, value_enum)]
    pub priority: Option<TaskPriority>,

    /// New kind
    #[arg(short, long, value_enum)]
    pub kind: Option<TaskKind>,

    /// New description (pass empty string to clear)
    #[arg(short, long)]
    pub description: Option<String>,
}

impl EditCommand {
    pub fn run(&self) -> Result<()> {
        let mut board = Board::load()?;

        let task = board
            .tasks
            .iter_mut()
            .find(|t| t.id == self.id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", self.id))?;

        if let Some(title) = &self.title {
            task.title = title.clone();
        }
        if let Some(priority) = self.priority {
            task.priority = priority;
        }
        if let Some(kind) = self.kind {
            task.kind = kind;
        }
        if let Some(description) = &self.description {
            task.description = if description.is_empty() {
                None
            } else {
                Some(description.clone())
            };
        }

        board.save()?;

        crate::commands::list::ListCommand { columns: vec![] }.run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{TaskKind, TaskPriority};

    #[test]
    fn empty_description_string_becomes_none() {
        let input = "".to_string();
        let result: Option<String> = if input.is_empty() { None } else { Some(input) };
        assert!(result.is_none());
    }

    #[test]
    fn nonempty_description_is_preserved() {
        let input = "My description".to_string();
        let result: Option<String> = if input.is_empty() {
            None
        } else {
            Some(input.clone())
        };
        assert_eq!(result, Some("My description".to_string()));
    }

    #[test]
    fn priority_values_are_all_distinct() {
        assert_ne!(TaskPriority::High, TaskPriority::Medium);
        assert_ne!(TaskPriority::Medium, TaskPriority::Low);
        assert_ne!(TaskPriority::High, TaskPriority::Low);
    }

    #[test]
    fn kind_values_are_all_distinct() {
        assert_ne!(TaskKind::Feature, TaskKind::Bug);
        assert_ne!(TaskKind::Bug, TaskKind::Chore);
        assert_ne!(TaskKind::Feature, TaskKind::Chore);
        assert_ne!(TaskKind::Feature, TaskKind::Ci);
        assert_ne!(TaskKind::Ci, TaskKind::Bug);
    }
}
