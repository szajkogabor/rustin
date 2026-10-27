use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Task {
    /// Title prefixed with the task's tags, e.g. `[UI,BUG] Fix button`.
    pub fn display_title(&self) -> String {
        if self.tags.is_empty() {
            self.title.clone()
        } else {
            format!("[{}] {}", self.tags.join(","), self.title)
        }
    }

    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter()
            .any(|tag| self.tags.contains(&normalize_tag(tag)))
    }

    /// Adds tags (normalized to uppercase), skipping empty ones and duplicates.
    pub fn add_tags<I, S>(&mut self, tags: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for tag in tags {
            let tag = normalize_tag(tag.as_ref());
            if !tag.is_empty() && !self.tags.contains(&tag) {
                self.tags.push(tag);
            }
        }
    }

    /// Moves every `[a,b]` group in the title into tags. Returns true if anything changed.
    pub fn migrate_title_tags(&mut self) -> bool {
        let mut title = String::new();
        let mut found = Vec::new();
        let mut rest = self.title.as_str();

        while let Some(start) = rest.find('[') {
            let Some(len) = rest[start..].find(']') else {
                break;
            };
            title.push_str(&rest[..start]);
            found.extend(rest[start + 1..start + len].split(',').map(str::to_string));
            rest = &rest[start + len + 1..];
        }
        title.push_str(rest);

        if found.is_empty() {
            return false;
        }

        self.title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        self.add_tags(found);
        true
    }
}

pub fn normalize_tag(tag: &str) -> String {
    tag.trim().to_uppercase()
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
    Ci,
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

    fn get_file_path() -> Result<PathBuf> {
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

    pub fn load() -> Result<Self> {
        let path = Self::get_file_path()?;
        if path.exists() {
            tracing::debug!("Loading board from {:?}", path);
            let content = fs::read_to_string(&path).with_context(|| {
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

    pub fn save(&mut self) -> Result<()> {
        self.version = current_version();
        let path = Self::get_file_path()?;
        tracing::debug!("Saving board to {:?}", path);
        let content = serde_json::to_string_pretty(self)?;
        save_atomically(&path, &content)?;
        Ok(())
    }

    pub fn active_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.deleted_at.is_none())
            .collect()
    }

    pub fn deleted_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.deleted_at.is_some())
            .collect()
    }

    pub fn soft_delete(&mut self, id: u32) -> bool {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id && t.deleted_at.is_none())
        {
            task.deleted_at = Some(chrono::Utc::now());
            true
        } else {
            false
        }
    }

    pub fn undelete(&mut self, id: u32) -> bool {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id && t.deleted_at.is_some())
        {
            task.deleted_at = None;
            true
        } else {
            false
        }
    }

    pub fn archive_done(&mut self) -> usize {
        let now = chrono::Utc::now();
        let mut count = 0;
        for task in &mut self.tasks {
            if task.status == TaskStatus::Done && task.deleted_at.is_none() {
                task.deleted_at = Some(now);
                count += 1;
            }
        }
        count
    }

    pub fn migrate_tags(&mut self) -> usize {
        self.tasks
            .iter_mut()
            .map(|task| task.migrate_title_tags())
            .filter(|changed| *changed)
            .count()
    }

    pub fn move_task(&mut self, id: u32, to: TaskStatus) -> bool {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|task| task.id == id && task.deleted_at.is_none())
        {
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

pub(crate) fn save_atomically(path: &Path, content: &str) -> Result<()> {
    let temp_path = atomic_temp_path(path)?;
    let mut temp_file = fs::File::create(&temp_path).with_context(|| {
        format!(
            "Failed to create temporary board file at {}.",
            temp_path.display()
        )
    })?;
    temp_file.write_all(content.as_bytes()).with_context(|| {
        format!(
            "Failed to write temporary board file at {}.",
            temp_path.display()
        )
    })?;
    temp_file.sync_all().with_context(|| {
        format!(
            "Failed to flush temporary board file at {}.",
            temp_path.display()
        )
    })?;
    drop(temp_file);

    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to replace board file at {} with temporary file. If another process or tool is using the file, close it and retry.",
            path.display()
        )
    })?;

    Ok(())
}

fn atomic_temp_path(path: &Path) -> Result<PathBuf> {
    let file_name = path.file_name().with_context(|| {
        format!(
            "Failed to determine board file name for atomic save at {}.",
            path.display()
        )
    })?;
    let temp_name = format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id()
    );
    Ok(path.with_file_name(temp_name))
}

#[cfg(test)]
mod tests {
    use super::{
        Board, Task, TaskKind, TaskPriority, TaskStatus, atomic_temp_path, current_version,
        save_atomically,
    };
    use chrono::Utc;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

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
            deleted_at: None,
            tags: vec![],
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
    fn task_deserializes_missing_deleted_at_as_none() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert!(task.deleted_at.is_none());
    }

    #[test]
    fn task_deserializes_missing_tags_as_empty() {
        let json = r#"{"id":1,"title":"t","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert!(task.tags.is_empty());
    }

    #[test]
    fn add_tags_uppercases_and_deduplicates() {
        let mut task = make_task(1, TaskStatus::Todo);
        task.add_tags(["ui", " Bug ", "UI", ""]);
        assert_eq!(task.tags, vec!["UI", "BUG"]);
    }

    #[test]
    fn display_title_prefixes_tags() {
        let mut task = make_task(1, TaskStatus::Todo);
        assert_eq!(task.display_title(), "task-1");
        task.add_tags(["ui", "bug"]);
        assert_eq!(task.display_title(), "[UI,BUG] task-1");
    }

    #[test]
    fn has_any_tag_matches_case_insensitively() {
        let mut task = make_task(1, TaskStatus::Todo);
        task.add_tags(["ui"]);
        assert!(task.has_any_tag(&["Ui".to_string()]));
        assert!(task.has_any_tag(&["cli".to_string(), "ui".to_string()]));
        assert!(!task.has_any_tag(&["cli".to_string()]));
    }

    #[test]
    fn migrate_title_tags_extracts_bracketed_groups() {
        let mut task = make_task(1, TaskStatus::Todo);
        task.title = "[ui, tui] Fix layout [bug]".to_string();
        assert!(task.migrate_title_tags());
        assert_eq!(task.title, "Fix layout");
        assert_eq!(task.tags, vec!["UI", "TUI", "BUG"]);
    }

    #[test]
    fn migrate_title_tags_leaves_plain_titles_untouched() {
        let mut task = make_task(1, TaskStatus::Todo);
        task.title = "No tags [here".to_string();
        assert!(!task.migrate_title_tags());
        assert_eq!(task.title, "No tags [here");
        assert!(task.tags.is_empty());
    }

    #[test]
    fn migrate_tags_counts_changed_tasks() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 3,
            tasks: vec![
                make_task(1, TaskStatus::Todo),
                make_task(2, TaskStatus::Done),
            ],
        };
        board.tasks[1].title = "[cli] Something".to_string();

        assert_eq!(board.migrate_tags(), 1);
        assert_eq!(board.tasks[1].tags, vec!["CLI"]);
        assert_eq!(board.tasks[1].title, "Something");
    }

    #[test]
    fn board_deserializes_missing_version_with_default() {
        let json = r#"{"title":"MyBoard","next_id":1,"tasks":[]}"#;
        let board: Board = serde_json::from_str(json).unwrap();
        assert_eq!(board.version, "0.0.0");
    }

    #[test]
    fn task_deserializes_ci_kind() {
        let json = r#"{"id":1,"title":"t","kind":"ci","status":"todo","created_at":"2024-01-01T00:00:00Z"}"#;
        let task: super::Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.kind, TaskKind::Ci);
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

    #[test]
    fn soft_delete_sets_deleted_at() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        assert!(board.soft_delete(1));
        assert!(board.tasks[0].deleted_at.is_some());
    }

    #[test]
    fn soft_delete_returns_false_for_already_deleted() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        assert!(board.soft_delete(1));
        assert!(!board.soft_delete(1));
    }

    #[test]
    fn undelete_clears_deleted_at() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        board.soft_delete(1);
        assert!(board.undelete(1));
        assert!(board.tasks[0].deleted_at.is_none());
    }

    #[test]
    fn undelete_returns_false_for_active_task() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        assert!(!board.undelete(1));
    }

    #[test]
    fn archive_done_soft_deletes_done_tasks_only() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 4,
            tasks: vec![
                make_task(1, TaskStatus::Todo),
                make_task(2, TaskStatus::Done),
                make_task(3, TaskStatus::Done),
            ],
        };

        let count = board.archive_done();
        assert_eq!(count, 2);
        assert!(board.tasks[0].deleted_at.is_none());
        assert!(board.tasks[1].deleted_at.is_some());
        assert!(board.tasks[2].deleted_at.is_some());
    }

    #[test]
    fn active_tasks_excludes_deleted() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 3,
            tasks: vec![
                make_task(1, TaskStatus::Todo),
                make_task(2, TaskStatus::Todo),
            ],
        };

        board.soft_delete(1);
        assert_eq!(board.active_tasks().len(), 1);
        assert_eq!(board.active_tasks()[0].id, 2);
    }

    #[test]
    fn deleted_tasks_returns_only_deleted() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 3,
            tasks: vec![
                make_task(1, TaskStatus::Todo),
                make_task(2, TaskStatus::Todo),
            ],
        };

        board.soft_delete(1);
        assert_eq!(board.deleted_tasks().len(), 1);
        assert_eq!(board.deleted_tasks()[0].id, 1);
    }

    #[test]
    fn move_task_skips_deleted_tasks() {
        let mut board = Board {
            version: "0.0.0".to_string(),
            title: "Board".to_string(),
            next_id: 2,
            tasks: vec![make_task(1, TaskStatus::Todo)],
        };

        board.soft_delete(1);
        assert!(!board.move_task(1, TaskStatus::Done));
    }

    #[test]
    fn save_atomically_persists_content_and_removes_temp_file() {
        let dir = TempDir::new().unwrap();
        let board_path = dir.path().join(".rustin.json");

        save_atomically(&board_path, "{\"title\":\"Board\"}").unwrap();

        assert_eq!(
            fs::read_to_string(&board_path).unwrap(),
            "{\"title\":\"Board\"}"
        );
        assert!(!atomic_temp_path(&board_path).unwrap().exists());
    }

    #[test]
    fn atomic_temp_path_uses_hidden_sibling_file() {
        let path = PathBuf::from("/tmp/.rustin.json");
        let temp_path = atomic_temp_path(&path).unwrap();

        assert_eq!(temp_path.parent(), path.parent());
        assert!(
            temp_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("..rustin.json.")
        );
        assert!(
            temp_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".tmp")
        );
    }
}
