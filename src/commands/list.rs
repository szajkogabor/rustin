use crate::store::{Board, Task, TaskPriority, TaskStatus};
use clap::Args;
use std::cmp::Ordering;
use tabled::{Table, Tabled, settings::Style};

#[derive(Args)]
pub struct ListCommand {}

#[derive(Tabled)]
struct BoardRow {
    #[tabled(rename = "Todo")]
    todo: String,
    #[tabled(rename = "In Progress")]
    in_progress: String,
    #[tabled(rename = "Done")]
    done: String,
}

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

        let max_len = todos.len().max(in_progress.len()).max(done.len());

        let mut rows = Vec::new();
        for i in 0..max_len {
            rows.push(BoardRow {
                todo: todos
                    .get(i)
                    .map(|t| format!("{} [{}] {}", priority_emoji(t.priority), t.id, t.title))
                    .unwrap_or_default(),
                in_progress: in_progress
                    .get(i)
                    .map(|t| format!("{} [{}] {}", priority_emoji(t.priority), t.id, t.title))
                    .unwrap_or_default(),
                done: done
                    .get(i)
                    .map(|t| format!("{} [{}] {}", priority_emoji(t.priority), t.id, t.title))
                    .unwrap_or_default(),
            });
        }

        println!("=== {} ===", board.title);

        if rows.is_empty() {
            println!("The board is empty. Add a task with `rustin add \"Task title\"`");
        } else {
            let mut table = Table::new(rows);
            table.with(Style::modern());
            println!("{}", table);
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::{priority_emoji, task_order};
    use crate::store::{Task, TaskPriority, TaskStatus};
    use chrono::{Duration, Utc};

    fn make_task(id: u32, priority: TaskPriority, created_at: chrono::DateTime<Utc>) -> Task {
        Task {
            id,
            title: format!("task-{id}"),
            priority,
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
}
