use crate::store::{Task, TaskKind, TaskPriority, TaskStatus};
use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use console::measure_text_width;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TaskColumn {
    Todo,
    #[value(alias = "inprogress")]
    InProgress,
    Done,
}

impl TaskColumn {
    pub const ALL: [TaskColumn; 3] = [TaskColumn::Todo, TaskColumn::InProgress, TaskColumn::Done];

    pub fn title(self) -> &'static str {
        match self {
            TaskColumn::Todo => "Todo",
            TaskColumn::InProgress => "In Progress",
            TaskColumn::Done => "Done",
        }
    }

    pub fn next(self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::InProgress,
            TaskColumn::InProgress => TaskColumn::Done,
            TaskColumn::Done => TaskColumn::Todo,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::Done,
            TaskColumn::InProgress => TaskColumn::Todo,
            TaskColumn::Done => TaskColumn::InProgress,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskRow {
    pub(crate) id: u32,
    pub(crate) summary: String,
    pub(crate) title: String,
    pub(crate) priority: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) description: Option<String>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskColumns {
    pub(crate) todo: Vec<TaskRow>,
    pub(crate) in_progress: Vec<TaskRow>,
    pub(crate) done: Vec<TaskRow>,
}

impl TaskColumns {
    pub(crate) fn tasks(&self, column: TaskColumn) -> &[TaskRow] {
        match column {
            TaskColumn::Todo => &self.todo,
            TaskColumn::InProgress => &self.in_progress,
            TaskColumn::Done => &self.done,
        }
    }

    pub(crate) fn max_rows(&self, visible_columns: &[TaskColumn]) -> usize {
        visible_columns
            .iter()
            .map(|column| self.tasks(*column).len())
            .max()
            .unwrap_or(0)
    }
}

fn pad_to_width(text: &str, width: usize) -> String {
    let current = measure_text_width(text);
    let padding = width.saturating_sub(current);
    format!("{}{}", text, " ".repeat(padding))
}

pub(crate) fn format_task(task: &Task) -> String {
    let priority = priority_emoji(task.priority);
    let kind = kind_emoji(task.kind);
    format!("{priority}[{:2}]{kind} {}", task.id, task.title)
}

pub(crate) fn visible_task_columns(selected: &[TaskColumn]) -> Vec<TaskColumn> {
    if selected.is_empty() {
        TaskColumn::ALL.to_vec()
    } else {
        selected.to_vec()
    }
}

pub(crate) fn build_task_table_rows(
    columns: &TaskColumns,
    visible_columns: &[TaskColumn],
) -> Vec<Vec<String>> {
    let rows = columns.max_rows(visible_columns);

    (0..rows)
        .map(|row_index| {
            visible_columns
                .iter()
                .map(|column| {
                    columns
                        .tasks(*column)
                        .get(row_index)
                        .map(|task| task.summary.clone())
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect()
}

pub(crate) fn task_snapshot_lines(task: &TaskRow) -> Vec<String> {
    vec![
        format!("Task: [{}] {}", task.id, task.title),
        format!("Status: {}", task.status),
        format!("Priority: {}", task.priority),
        format!("Kind: {}", task.kind),
        format!(
            "Description: {}",
            task.description.as_deref().unwrap_or("(none)")
        ),
    ]
}

pub(crate) fn priority_emoji(priority: TaskPriority) -> String {
    let emoji = match priority {
        TaskPriority::High => "🔥",
        TaskPriority::Medium => "🌶️",
        TaskPriority::Low => "🧊",
    };
    pad_to_width(emoji, 2)
}

pub(crate) fn kind_emoji(kind: TaskKind) -> String {
    let emoji = match kind {
        TaskKind::Feature => "✨",
        TaskKind::Bug => "🐛",
        TaskKind::Chore => "🔧",
        TaskKind::Ci => "⚙️",
    };
    pad_to_width(emoji, 2)
}

pub(crate) fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
    }
}

pub(crate) fn task_order(left: &Task, right: &Task) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| left.created_at.cmp(&right.created_at))
        .then_with(|| left.id.cmp(&right.id))
}

pub(crate) fn split_tasks(tasks: &[Task]) -> TaskColumns {
    let mut ordered: Vec<&Task> = tasks.iter().filter(|t| t.deleted_at.is_none()).collect();
    ordered.sort_by(|left, right| task_order(left, right));

    let mut columns = TaskColumns::default();

    for task in ordered {
        let row = TaskRow {
            id: task.id,
            summary: format_task(task),
            title: task.title.clone(),
            priority: priority_emoji(task.priority),
            kind: kind_emoji(task.kind),
            status: status_label(&task.status).to_string(),
            description: task.description.clone(),
        };

        match task.status {
            TaskStatus::Todo => columns.todo.push(row),
            TaskStatus::InProgress => columns.in_progress.push(row),
            TaskStatus::Done => columns.done.push(row),
        }
    }

    columns
}

pub(crate) fn format_local_timestamp(utc: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = utc.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn task_detail_lines(task: &Task) -> Vec<String> {
    let mut lines = vec![
        format!("ID:          {}", task.id),
        format!("Title:       {}", task.title),
        format!("Kind:        {:?} {}", task.kind, kind_emoji(task.kind)),
        format!(
            "Priority:    {:?} {}",
            task.priority,
            priority_emoji(task.priority)
        ),
        format!("Status:      {}", status_label(&task.status)),
        format!("Created:     {}", format_local_timestamp(&task.created_at)),
    ];

    if let Some(desc) = &task.description {
        lines.push(format!("Description: {}", desc));
    } else {
        lines.push("Description: (none)".to_string());
    }
    if !task.transitions.is_empty() {
        lines.push("History:".to_string());
        for transition in &task.transitions {
            lines.push(format!(
                "  {} → {}  ({})",
                status_label(&transition.from),
                status_label(&transition.to),
                format_local_timestamp(&transition.at)
            ));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{
        TaskColumn, build_task_table_rows, format_local_timestamp, format_task, kind_emoji,
        priority_emoji, split_tasks, status_label, task_detail_lines, task_order,
        task_snapshot_lines, visible_task_columns,
    };
    use crate::store::{StatusTransition, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::{Duration, TimeZone, Utc};

    fn sample_task() -> Task {
        Task {
            id: 42,
            title: "Investigate formatter".to_string(),
            priority: TaskPriority::High,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
            deleted_at: None,
        }
    }

    fn sample_task_with(
        id: u32,
        priority: TaskPriority,
        created_at: chrono::DateTime<Utc>,
    ) -> Task {
        Task {
            id,
            title: format!("task-{id}"),
            priority,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at,
            transitions: vec![],
            deleted_at: None,
        }
    }

    #[test]
    fn priority_emoji_maps_all_priorities() {
        assert_eq!(priority_emoji(TaskPriority::High), "🔥");
        assert_eq!(priority_emoji(TaskPriority::Medium), "🌶️");
        assert_eq!(priority_emoji(TaskPriority::Low), "🧊");
    }

    #[test]
    fn kind_emoji_maps_all_kinds() {
        assert_eq!(kind_emoji(TaskKind::Feature), "✨");
        assert_eq!(kind_emoji(TaskKind::Bug), "🐛");
        assert_eq!(kind_emoji(TaskKind::Chore), "🔧");
        assert_eq!(kind_emoji(TaskKind::Ci), "⚙️");
    }

    #[test]
    fn format_task_uses_shared_marker_layout() {
        let output = format_task(&sample_task());
        assert!(output.contains("42"));
        assert!(output.contains("Investigate formatter"));
        assert!(output.contains("🔥"));
        assert!(output.contains("✨"));
    }

    #[test]
    fn visible_task_columns_defaults_to_all_columns() {
        assert_eq!(
            visible_task_columns(&[]),
            vec![TaskColumn::Todo, TaskColumn::InProgress, TaskColumn::Done]
        );
    }

    #[test]
    fn task_order_sorts_by_priority_then_date_then_id() {
        let now = Utc::now();
        let mut tasks = vec![
            sample_task_with(3, TaskPriority::Medium, now - Duration::minutes(10)),
            sample_task_with(2, TaskPriority::High, now - Duration::minutes(5)),
            sample_task_with(1, TaskPriority::High, now - Duration::minutes(5)),
            sample_task_with(4, TaskPriority::Low, now - Duration::minutes(20)),
        ];

        tasks.sort_by(task_order);

        let ids: Vec<u32> = tasks.into_iter().map(|task| task.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }

    #[test]
    fn split_tasks_groups_rows_and_reuses_shared_formatter() {
        let mut todo = sample_task();
        todo.status = TaskStatus::Todo;
        let mut done = sample_task();
        done.id = 7;
        done.kind = TaskKind::Bug;
        done.status = TaskStatus::Done;

        let columns = split_tasks(&[done.clone(), todo.clone()]);

        assert_eq!(columns.todo[0].summary, format_task(&todo));
        assert_eq!(columns.done[0].summary, format_task(&done));
    }

    #[test]
    fn build_task_table_rows_pads_shorter_columns() {
        let columns = split_tasks(&[
            Task {
                status: TaskStatus::Todo,
                ..sample_task()
            },
            Task {
                id: 7,
                title: "done task".to_string(),
                status: TaskStatus::Done,
                ..sample_task()
            },
        ]);

        let rows = build_task_table_rows(
            &columns,
            &[TaskColumn::Todo, TaskColumn::InProgress, TaskColumn::Done],
        );

        assert_eq!(rows.len(), 1);
        assert!(rows[0][0].contains("Investigate formatter"));
        assert_eq!(rows[0][1], "");
        assert!(rows[0][2].contains("done task"));
    }

    #[test]
    fn task_snapshot_lines_include_optional_description_placeholder() {
        let mut columns = split_tasks(&[sample_task()]);
        let row = columns.todo.remove(0);
        let lines = task_snapshot_lines(&row);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("Task: [42] Investigate formatter"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Description: (none)"))
        );
    }

    #[test]
    fn status_label_maps_all_statuses() {
        assert_eq!(status_label(&TaskStatus::Todo), "Todo");
        assert_eq!(status_label(&TaskStatus::InProgress), "In Progress");
        assert_eq!(status_label(&TaskStatus::Done), "Done");
    }

    #[test]
    fn task_detail_lines_include_description_and_history() {
        let created = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();
        let transitioned = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let task = Task {
            id: 7,
            title: "Investigate bug".to_string(),
            priority: TaskPriority::High,
            kind: TaskKind::Bug,
            description: Some("Collect logs".to_string()),
            status: TaskStatus::Done,
            created_at: created,
            transitions: vec![StatusTransition {
                from: TaskStatus::InProgress,
                to: TaskStatus::Done,
                at: transitioned,
            }],
            deleted_at: None,
        };

        let lines = task_detail_lines(&task);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Title:       Investigate bug"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Description: Collect logs"))
        );
        assert!(lines.iter().any(|line| line.contains("History:")));

        let expected_transition_time = format_local_timestamp(&transitioned);
        assert!(lines.iter().any(|line| line.contains(&format!(
            "In Progress → Done  ({})",
            expected_transition_time
        ))));

        let expected_created_time = format_local_timestamp(&created);
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!("Created:     {}", expected_created_time)))
        );
    }

    #[test]
    fn format_local_timestamp_converts_utc_to_local() {
        let utc = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let formatted = format_local_timestamp(&utc);

        // Must not contain "UTC" — it's local time now
        assert!(!formatted.contains("UTC"));
        // Must match YYYY-MM-DD HH:MM:SS pattern (19 chars)
        assert_eq!(formatted.len(), 19);

        // Verify round-trip: the local rendering matches chrono Local conversion
        use chrono::Local;
        let expected = utc
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        assert_eq!(formatted, expected);
    }

    #[test]
    fn task_detail_lines_show_none_placeholder_when_description_is_missing() {
        let task = Task {
            id: 1,
            title: "No desc".to_string(),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
            deleted_at: None,
        };

        let lines = task_detail_lines(&task);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Description: (none)"))
        );
    }
}
