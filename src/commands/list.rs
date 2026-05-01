use crate::store::{Board, Task, TaskStatus};
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
                    .map(|t| format!("[{}] {}", t.id, t.title))
                    .unwrap_or_default(),
                in_progress: in_progress
                    .get(i)
                    .map(|t| format!("[{}] {}", t.id, t.title))
                    .unwrap_or_default(),
                done: done
                    .get(i)
                    .map(|t| format!("[{}] {}", t.id, t.title))
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
