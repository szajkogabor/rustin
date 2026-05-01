use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusTransition {
    pub from: TaskStatus,
    pub to: TaskStatus,
    pub at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    #[serde(default = "default_task_priority")]
    pub priority: TaskPriority,
    #[serde(default)]
    pub kind: TaskKind,
    #[serde(default)]
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub transitions: Vec<StatusTransition>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

fn default_task_priority() -> TaskPriority {
    TaskPriority::Medium
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TaskKind {
    #[default]
    Feature,
    Bug,
    Chore,
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
        .map(|cwd| Board::find_project_root(cwd.clone()).unwrap_or(cwd))
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "My Board".to_string())
}

fn current_version() -> String {
    option_env!("VERGEN_GIT_DESCRIBE")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string()
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

    pub fn move_task(&mut self, id: u32, to: TaskStatus) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            let from = task.status.clone();
            task.transitions.push(StatusTransition {
                from,
                to: to.clone(),
                at: chrono::Utc::now(),
            });
            task.status = to;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Task, TaskKind, TaskPriority, TaskStatus, current_version};
    use chrono::Utc;

    fn make_task(id: u32, status: TaskStatus) -> Task {
        Task {
            id,
            title: format!("task-{id}"),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status,
            created_at: Utc::now(),
            transitions: vec![],
        }
    }

    #[test]
    fn default_board_is_initialized_consistently() {
        let board = Board::default();
        assert_eq!(board.next_id, 1);
        assert!(board.tasks.is_empty());
        assert!(!board.title.trim().is_empty());
        assert_eq!(board.version, current_version());
    }

    #[test]
    fn task_priority_ordering_high_greater_than_medium_greater_than_low() {
        assert!(TaskPriority::High > TaskPriority::Medium);
        assert!(TaskPriority::Medium > TaskPriority::Low);
        assert!(TaskPriority::High > TaskPriority::Low);
    }

    #[test]
    fn task_kind_default_is_feature() {
        let kind = TaskKind::default();
        assert_eq!(kind, TaskKind::Feature);
    }

    #[test]
    fn task_status_equality() {
        assert_eq!(TaskStatus::Todo, TaskStatus::Todo);
        assert_ne!(TaskStatus::Todo, TaskStatus::Done);
        assert_ne!(TaskStatus::InProgress, TaskStatus::Done);
    }

    #[test]
    fn task_deserializes_missing_priority_as_medium() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, TaskPriority::Medium);
    }

    #[test]
    fn task_deserializes_missing_kind_as_feature() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.kind, TaskKind::Feature);
    }

    #[test]
    fn task_deserializes_missing_description_as_none() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert!(task.description.is_none());
    }

    #[test]
    fn task_deserializes_missing_transitions_as_empty() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert!(task.transitions.is_empty());
    }

    #[test]
    fn board_deserializes_missing_version_with_default() {
        let json = r#"{"title":"MyBoard","next_id":1,"tasks":[]}"#;
        let board: Board = serde_json::from_str(json).unwrap();
        assert_eq!(board.version, "0.0.0");
    }

    #[test]
    fn move_task_updates_status_and_history() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        assert!(board.move_task(1, TaskStatus::Done));

        let task = &board.tasks[0];
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.transitions.len(), 1);
        assert_eq!(task.transitions[0].from, TaskStatus::Todo);
        assert_eq!(task.transitions[0].to, TaskStatus::Done);
    }

    #[test]
    fn move_task_returns_false_when_missing() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 1,
            tasks: vec![],
        };

        assert!(!board.move_task(99, TaskStatus::Done));
    }
}
