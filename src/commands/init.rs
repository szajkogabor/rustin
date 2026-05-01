use crate::store::Board;
use clap::Args;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
        maybe_offer_gitignore_entry()?;
        crate::commands::list::ListCommand { columns: vec![] }.run()?;

        Ok(())
    }
}

fn maybe_offer_gitignore_entry() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let Some(git_root) = find_git_root(&cwd) else {
        return Ok(());
    };

    let gitignore_path = git_root.join(".gitignore");
    if gitignore_contains(&gitignore_path, ".rustin.json")? {
        return Ok(());
    }

    print!("Add .rustin.json to .gitignore? [y/N]: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    if matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        append_gitignore_entry(&gitignore_path, ".rustin.json")?;
        tracing::info!("Added .rustin.json to {}", gitignore_path.display());
    }

    Ok(())
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        if current.join(".git").exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

fn gitignore_contains(path: &Path, entry: &str) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)?;
    Ok(content.lines().any(|line| line.trim() == entry))
}

fn append_gitignore_entry(path: &Path, entry: &str) -> anyhow::Result<()> {
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(entry);
    content.push('\n');
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{append_gitignore_entry, find_git_root, gitignore_contains};
    use crate::store::Board;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn setting_title_updates_board_title() {
        let mut board = Board::default();
        let new_title = "My Project".to_string();
        board.title = new_title.clone();
        assert_eq!(board.title, "My Project");
    }

    #[test]
    fn not_setting_title_leaves_board_title_unchanged() {
        let mut board = Board::default();
        let original = board.title.clone();
        let new_title: Option<String> = None;
        if let Some(t) = new_title {
            board.title = t;
        }
        assert_eq!(board.title, original);
    }

    #[test]
    fn find_git_root_walks_up_to_repository_root() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        let nested = project.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir(project.join(".git")).unwrap();

        assert_eq!(find_git_root(&nested), Some(project));
    }

    #[test]
    fn append_gitignore_entry_adds_line_with_newline() {
        let dir = TempDir::new().unwrap();
        let gitignore = dir.path().join(".gitignore");
        fs::write(&gitignore, "target").unwrap();

        append_gitignore_entry(&gitignore, ".rustin.json").unwrap();

        assert_eq!(
            fs::read_to_string(&gitignore).unwrap(),
            "target\n.rustin.json\n"
        );
    }

    #[test]
    fn gitignore_contains_checks_exact_line_match() {
        let dir = TempDir::new().unwrap();
        let gitignore = dir.path().join(".gitignore");
        fs::write(&gitignore, "target\n.rustin.json\n").unwrap();

        assert!(gitignore_contains(&gitignore, ".rustin.json").unwrap());
        assert!(!gitignore_contains(&gitignore, "rustin").unwrap());
    }
}
