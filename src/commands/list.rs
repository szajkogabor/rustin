use crate::store::{Board, Task, TaskKind, TaskPriority, TaskStatus};
use clap::{Args, ValueEnum};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Column {
    Todo,
    Inprogress,
    Done,
}

#[derive(Args)]
pub struct ListCommand {
    /// Columns to display (default: all)
    #[arg(short, long, value_enum, num_args = 1..)]
    pub columns: Vec<Column>,
}

impl ListCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let show_all = self.columns.is_empty();
        let show = |col: Column| show_all || self.columns.contains(&col);

        let board = Board::load()?;

        let mut todos: Vec<_> = board
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect();
        let mut in_progress: Vec<_> = board
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();
        let mut done: Vec<_> = board
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .collect();

        todos.sort_by(task_order);
        in_progress.sort_by(task_order);
        done.sort_by(task_order);

        let todos: Vec<String> = todos
            .iter()
            .map(|t| {
                format!(
                    "{} [{}] {} {}",
                    priority_emoji(t.priority),
                    t.id,
                    t.title,
                    kind_emoji(t.kind)
                )
            })
            .collect();
        let in_progress: Vec<String> = in_progress
            .iter()
            .map(|t| {
                format!(
                    "{} [{}] {} {}",
                    priority_emoji(t.priority),
                    t.id,
                    t.title,
                    kind_emoji(t.kind)
                )
            })
            .collect();
        let done: Vec<String> = done
            .iter()
            .map(|t| {
                format!(
                    "{} [{}] {} {}",
                    priority_emoji(t.priority),
                    t.id,
                    t.title,
                    kind_emoji(t.kind)
                )
            })
            .collect();

        println!("=== {} ===", board.title);

        let todo_col = show(Column::Todo).then_some(todos.as_slice());
        let inprogress_col = show(Column::Inprogress).then_some(in_progress.as_slice());
        let done_col = show(Column::Done).then_some(done.as_slice());

        let total = todo_col.map_or(0, |c| c.len())
            + inprogress_col.map_or(0, |c| c.len())
            + done_col.map_or(0, |c| c.len());

        if total == 0 && board.tasks.is_empty() {
            println!("The board is empty. Add a task with `rustin add \"Task title\"`");
        } else {
            print_columns(todo_col, inprogress_col, done_col);
        }

        Ok(())
    }
}

fn print_columns(todo: Option<&[String]>, in_progress: Option<&[String]>, done: Option<&[String]>) {
    // Build the list of active columns: (header, rows)
    let mut cols: Vec<(&str, &[String])> = Vec::new();
    if let Some(rows) = todo {
        cols.push(("Todo", rows));
    }
    if let Some(rows) = in_progress {
        cols.push(("In Progress", rows));
    }
    if let Some(rows) = done {
        cols.push(("Done", rows));
    }

    if cols.is_empty() {
        return;
    }

    // Fit within terminal width
    let separators = " | ".len() * cols.len().saturating_sub(1);
    let term_width = terminal_width();
    let available = term_width.saturating_sub(separators);
    let col_cap = (available / cols.len()).max(10);

    let widths: Vec<usize> = cols
        .iter()
        .map(|(header, rows)| {
            rows.iter()
                .map(|v| v.chars().count())
                .max()
                .unwrap_or(0)
                .max(header.len())
                .min(col_cap)
        })
        .collect();

    // Header row
    let header_parts: Vec<String> = cols
        .iter()
        .zip(widths.iter())
        .map(|((header, _), w)| format!("{:w$}", header))
        .collect();
    println!("{}", header_parts.join(" | "));

    // Data rows
    let max_rows = cols.iter().map(|(_, rows)| rows.len()).max().unwrap_or(0);
    for i in 0..max_rows {
        let row_parts: Vec<String> = cols
            .iter()
            .zip(widths.iter())
            .map(|((_, rows), w)| {
                let cell = rows.get(i).map_or("", String::as_str);
                format!("{:w$}", truncate(cell, *w))
            })
            .collect();
        println!("{}", row_parts.join(" | "));
    }
}

fn terminal_width() -> usize {
    // COLUMNS env var is set by most shells; fall back to a safe default
    if let Ok(val) = std::env::var("COLUMNS")
        && let Ok(n) = val.parse::<usize>()
    {
        return n;
    }
    120
}

/// Truncate `s` to at most `max` visible chars, appending `…` if cut.
fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let cut = max.saturating_sub(1);
        chars[..cut].iter().collect::<String>() + "…"
    }
}

fn task_order(left: &&Task, right: &&Task) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| left.created_at.cmp(&right.created_at))
        .then_with(|| left.id.cmp(&right.id))
}

fn priority_emoji(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::High => "🔥",
        TaskPriority::Medium => "🌶️",
        TaskPriority::Low => "🧊",
    }
}

fn kind_emoji(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::Feature => "✨",
        TaskKind::Bug => "🐛",
        TaskKind::Chore => "🔧",
    }
}

#[cfg(test)]
mod tests {
    use super::{kind_emoji, priority_emoji, task_order};
    use crate::store::{Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::{Duration, Utc};

    fn make_task(id: u32, priority: TaskPriority, created_at: chrono::DateTime<Utc>) -> Task {
        Task {
            id,
            title: format!("task-{id}"),
            priority,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Todo,
            created_at,
            transitions: vec![],
        }
    }

    #[test]
    fn task_order_sorts_by_priority_then_date_then_id() {
        let now = Utc::now();
        let mut tasks = vec![
            make_task(3, TaskPriority::Medium, now - Duration::minutes(10)),
            make_task(2, TaskPriority::High, now - Duration::minutes(5)),
            make_task(1, TaskPriority::High, now - Duration::minutes(5)),
            make_task(4, TaskPriority::Low, now - Duration::minutes(20)),
        ];

        tasks.sort_by(|left, right| task_order(&left, &right));

        let ids: Vec<u32> = tasks.into_iter().map(|task| task.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
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
    }
}
