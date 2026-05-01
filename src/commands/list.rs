use crate::store::{Board, Task, TaskKind, TaskPriority, TaskStatus};
use clap::Args;
use std::cmp::Ordering;

#[derive(Args)]
pub struct ListCommand {}

impl ListCommand {
    pub fn run(&self) -> anyhow::Result<()> {
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
                    "{} {} [{}] {}",
                    priority_emoji(t.priority),
                    kind_emoji(t.kind),
                    t.id,
                    t.title
                )
            })
            .collect();
        let in_progress: Vec<String> = in_progress
            .iter()
            .map(|t| {
                format!(
                    "{} {} [{}] {}",
                    priority_emoji(t.priority),
                    kind_emoji(t.kind),
                    t.id,
                    t.title
                )
            })
            .collect();
        let done: Vec<String> = done
            .iter()
            .map(|t| {
                format!(
                    "{} {} [{}] {}",
                    priority_emoji(t.priority),
                    kind_emoji(t.kind),
                    t.id,
                    t.title
                )
            })
            .collect();

        let max_len = todos.len().max(in_progress.len()).max(done.len());

        println!("=== {} ===", board.title);

        if max_len == 0 {
            println!("The board is empty. Add a task with `rustin add \"Task title\"`");
        } else {
            print_columns(&todos, &in_progress, &done);
        }

        Ok(())
    }
}

fn print_columns(todo: &[String], in_progress: &[String], done: &[String]) {
    let todo_header = "Todo";
    let in_progress_header = "In Progress";
    let done_header = "Done";

    let todo_width = todo
        .iter()
        .map(|v| v.chars().count())
        .max()
        .unwrap_or(0)
        .max(todo_header.len());
    let in_progress_width = in_progress
        .iter()
        .map(|v| v.chars().count())
        .max()
        .unwrap_or(0)
        .max(in_progress_header.len());
    let done_width = done
        .iter()
        .map(|v| v.chars().count())
        .max()
        .unwrap_or(0)
        .max(done_header.len());

    println!(
        "{:todo_width$} | {:in_progress_width$} | {:done_width$}",
        todo_header, in_progress_header, done_header
    );

    let max_len = todo.len().max(in_progress.len()).max(done.len());
    for i in 0..max_len {
        let todo_value = todo.get(i).map_or("", String::as_str);
        let in_progress_value = in_progress.get(i).map_or("", String::as_str);
        let done_value = done.get(i).map_or("", String::as_str);

        println!(
            "{:todo_width$} | {:in_progress_width$} | {:done_width$}",
            todo_value, in_progress_value, done_value
        );
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
