use crate::store::Board;
use anyhow::Context;
use anyhow::Result;
use clap::Args;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct InitCommand {
    /// The new title for the board
    pub title: Option<String>,
}

impl InitCommand {
    pub fn run(&self) -> Result<()> {
        let board_path = current_dir_board_path()?;
        let mut board = load_or_create_local_board(&board_path)?;

        if let Some(new_title) = &self.title {
            board.title = new_title.clone();
            tracing::info!("Board title set to: {}", new_title);
        } else {
            tracing::info!("Initialized board: {}", board.title);
        }

        save_local_board(&board_path, &mut board)?;
        maybe_offer_gitignore_entry()?;
        crate::commands::list::ListCommand { columns: vec![] }.run()?;

        Ok(())
    }
}

fn current_dir_board_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join(".rustin.json"))
}

fn current_version() -> String {
    option_env!("VERGEN_GIT_DESCRIBE")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string()
}

fn load_or_create_local_board(path: &Path) -> Result<Board> {
    if !path.exists() {
        let board = Board::default();
        return Ok(board);
    }

    let content = fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read board file at {}. Check that the file is readable.",
            path.display()
        )
    })?;
    let mut board: Board = serde_json::from_str(&content).with_context(|| {
        format!(
            "Failed to parse board file at {}. The file is not valid JSON. Fix the file or remove it and run `rustin init`.",
            path.display()
        )
    })?;
    board.version = current_version();

    Ok(board)
}

fn save_local_board(path: &Path, board: &mut Board) -> Result<()> {
    board.version = current_version();
    let content = serde_json::to_string_pretty(board)?;
    crate::store::save_atomically(path, &content)?;
    Ok(())
}

fn maybe_offer_gitignore_entry() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let gitignore_path = cwd.join(".gitignore");

    if !gitignore_path.exists() {
        return Ok(());
    }

    if gitignore_contains(&gitignore_path, ".rustin.json")? {
        return Ok(());
    }

    if !should_prompt_for_gitignore(io::stdin().is_terminal(), io::stdout().is_terminal()) {
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

fn should_prompt_for_gitignore(stdin_is_terminal: bool, stdout_is_terminal: bool) -> bool {
    stdin_is_terminal && stdout_is_terminal
}

fn gitignore_contains(path: &Path, entry: &str) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)?;
    Ok(content.lines().any(|line| line.trim() == entry))
}

fn append_gitignore_entry(path: &Path, entry: &str) -> Result<()> {
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
    use super::{
        append_gitignore_entry, current_version, gitignore_contains, load_or_create_local_board,
        should_prompt_for_gitignore,
    };
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
    fn load_or_create_local_board_uses_current_directory_title_for_new_board() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&nested).unwrap();

        let board = load_or_create_local_board(&nested.join(".rustin.json")).unwrap();

        std::env::set_current_dir(old_dir).unwrap();
        assert_eq!(board.title, "nested");
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

    #[test]
    fn should_prompt_for_gitignore_requires_interactive_streams() {
        assert!(should_prompt_for_gitignore(true, true));
        assert!(!should_prompt_for_gitignore(true, false));
        assert!(!should_prompt_for_gitignore(false, true));
        assert!(!should_prompt_for_gitignore(false, false));
    }

    #[test]
    fn load_or_create_local_board_migrates_existing_board_version() {
        let dir = TempDir::new().unwrap();
        let board_path = dir.path().join(".rustin.json");
        fs::write(
            &board_path,
            r#"{"version":"0.0.0","title":"Demo","next_id":1,"tasks":[]}"#,
        )
        .unwrap();

        let board = load_or_create_local_board(&board_path).unwrap();
        assert_eq!(board.title, "Demo");
        assert_eq!(board.version, current_version());
    }
}
