use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

fn default_board_title() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "My Board".to_string())
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_board_version() -> String {
    "0.0.0".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    #[serde(default = "default_board_version")]
    pub version: String,
    #[serde(default = "default_board_title")]
    pub title: String,
    pub next_id: u32,
    pub tasks: Vec<Task>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            version: current_version(),
            title: default_board_title(),
            next_id: 1,
            tasks: Vec::new(),
        }
    }
}

impl Board {
    fn find_project_root(start: PathBuf) -> Option<PathBuf> {
        let mut current_dir = start;

        loop {
            if current_dir.join("Cargo.toml").exists() || current_dir.join(".git").exists() {
                return Some(current_dir);
            }

            if !current_dir.pop() {
                return None;
            }
        }
    }

    fn get_file_path() -> anyhow::Result<PathBuf> {
        let mut current_dir = std::env::current_dir()?;
        loop {
            let potential_path = current_dir.join(".rustin.json");
            if potential_path.exists() {
                return Ok(potential_path);
            }
            if !current_dir.pop() {
                break;
            }
        }

        let cwd = std::env::current_dir()?;

        // If not found in any parent, prefer the detected project root and fall back to cwd.
        let base_dir = Self::find_project_root(cwd.clone()).unwrap_or(cwd);
        Ok(base_dir.join(".rustin.json"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::get_file_path()?;
        if path.exists() {
            tracing::debug!("Loading board from {:?}", path);
            let content = fs::read_to_string(&path)?;
            let mut board: Board = serde_json::from_str(&content)?;

            let cv = current_version();
            if board.version != cv {
                tracing::debug!("Migrating board from version {} to {}", board.version, cv);
                board.version = cv;
            }

            Ok(board)
        } else {
            tracing::debug!(
                "No existing board found at {:?}, initializing new default board",
                path
            );
            Ok(Board::default())
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.version = current_version();
        let path = Self::get_file_path()?;
        tracing::debug!("Saving board to {:?}", path);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn default_board_is_initialized_consistently() {
        let board = Board::default();

        assert_eq!(board.next_id, 1);
        assert!(board.tasks.is_empty());
        assert!(!board.title.trim().is_empty());
        assert_eq!(board.version, env!("CARGO_PKG_VERSION"));
    }
}
