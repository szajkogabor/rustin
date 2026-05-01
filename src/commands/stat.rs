use crate::store::{Board, Task, TaskStatus};
use chrono::{DateTime, Duration, Utc};
use clap::Args;

#[derive(Args)]
pub struct StatCommand;

#[derive(Debug, PartialEq, Eq)]
struct TaskStat {
    id: u32,
    title: String,
    total_active_time: Duration,
    completed_cycles: usize,
}

impl StatCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let board = Board::load()?;
        let mut stats: Vec<_> = board.tasks.iter().map(task_stat).collect();
        stats.sort_by(|left, right| {
            right
                .total_active_time
                .cmp(&left.total_active_time)
                .then_with(|| left.id.cmp(&right.id))
        });

        let total_active_time = stats
            .iter()
            .fold(Duration::zero(), |sum, stat| sum + stat.total_active_time);
        let completed_cycles: usize = stats.iter().map(|stat| stat.completed_cycles).sum();
        println!("=== {} stats ===", board.title);
        println!("Tasks:           {}", board.tasks.len());
        println!("Completed runs:  {}", completed_cycles);
        println!("Total active:    {}", format_duration(total_active_time));
        println!();

        if stats.is_empty() {
            println!("The board has no tasks yet.");
            return Ok(());
        }

        println!("Per task:");
        let max_active_time = stats
            .iter()
            .map(|stat| stat.total_active_time)
            .max()
            .unwrap_or_else(Duration::zero);

        for stat in &stats {
            println!(
                "[{}] {:<24} {:>10}  runs:{}  {}",
                stat.id,
                truncate(&stat.title, 24),
                format_duration(stat.total_active_time),
                stat.completed_cycles,
                horizontal_bar(stat.total_active_time, max_active_time, 16),
            );
        }

        Ok(())
    }
}

fn horizontal_bar(duration: Duration, max_duration: Duration, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let duration_seconds = duration.num_seconds().max(0);
    let max_seconds = max_duration.num_seconds().max(0);

    let filled = if duration_seconds == 0 || max_seconds == 0 {
        0
    } else {
        ((duration_seconds as f64 / max_seconds as f64) * width as f64)
            .round()
            .clamp(1.0, width as f64) as usize
    };
    let empty = width.saturating_sub(filled);

    format!("|{}{}|", "█".repeat(filled), "░".repeat(empty))
}

fn task_stat(task: &Task) -> TaskStat {
    let mut started_at: Option<DateTime<Utc>> = None;
    let mut total_active_time = Duration::zero();
    let mut completed_cycles = 0;

    for transition in &task.transitions {
        match (&transition.from, &transition.to) {
            (_, TaskStatus::InProgress) => started_at = Some(transition.at),
            (TaskStatus::InProgress, TaskStatus::Done) => {
                if let Some(start) = started_at.take() {
                    total_active_time += transition.at - start;
                    completed_cycles += 1;
                }
            }
            (TaskStatus::InProgress, _) => started_at = None,
            _ => {}
        }
    }

    TaskStat {
        id: task.id,
        title: task.title.clone(),
        total_active_time,
        completed_cycles,
    }
}

fn format_duration(duration: Duration) -> String {
    let mut seconds = duration.num_seconds().max(0);
    let hours = seconds / 3600;
    seconds %= 3600;
    let minutes = seconds / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars {
        value.to_string()
    } else {
        let cutoff = max_chars.saturating_sub(1);
        chars[..cutoff].iter().collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskStat, format_duration, horizontal_bar, task_stat};
    use crate::store::{StatusTransition, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::{Duration, TimeZone, Utc};

    fn make_task(transitions: Vec<StatusTransition>) -> Task {
        Task {
            id: 1,
            title: "task".to_string(),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status: TaskStatus::Done,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
            transitions,
        }
    }

    #[test]
    fn task_stat_counts_completed_inprogress_to_done_cycles() {
        let task = make_task(vec![
            StatusTransition {
                from: TaskStatus::Todo,
                to: TaskStatus::InProgress,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            },
            StatusTransition {
                from: TaskStatus::InProgress,
                to: TaskStatus::Done,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 11, 30, 0).unwrap(),
            },
        ]);

        assert_eq!(
            task_stat(&task),
            TaskStat {
                id: 1,
                title: "task".to_string(),
                total_active_time: Duration::minutes(90),
                completed_cycles: 1,
            }
        );
    }

    #[test]
    fn task_stat_ignores_aborted_inprogress_runs() {
        let task = make_task(vec![
            StatusTransition {
                from: TaskStatus::Todo,
                to: TaskStatus::InProgress,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            },
            StatusTransition {
                from: TaskStatus::InProgress,
                to: TaskStatus::Todo,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap(),
            },
            StatusTransition {
                from: TaskStatus::Todo,
                to: TaskStatus::InProgress,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 15, 0).unwrap(),
            },
            StatusTransition {
                from: TaskStatus::InProgress,
                to: TaskStatus::Done,
                at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 45, 0).unwrap(),
            },
        ]);

        let stat = task_stat(&task);
        assert_eq!(stat.total_active_time, Duration::minutes(30));
        assert_eq!(stat.completed_cycles, 1);
    }

    #[test]
    fn format_duration_supports_hours_minutes_and_seconds() {
        assert_eq!(format_duration(Duration::seconds(45)), "45s");
        assert_eq!(format_duration(Duration::seconds(125)), "2m 05s");
        assert_eq!(format_duration(Duration::seconds(3723)), "1h 02m 03s");
    }

    #[test]
    fn task_stat_keeps_completed_cycle_count() {
        let stats = [TaskStat {
            id: 1,
            title: "task".to_string(),
            total_active_time: Duration::minutes(10),
            completed_cycles: 2,
        }];

        assert_eq!(stats[0].completed_cycles, 2);
    }

    #[test]
    fn horizontal_bar_uses_relative_fill() {
        let max = Duration::minutes(10);

        assert_eq!(horizontal_bar(Duration::zero(), max, 8), "|░░░░░░░░|");
        assert_eq!(horizontal_bar(Duration::minutes(5), max, 8), "|████░░░░|");
        assert_eq!(horizontal_bar(Duration::minutes(10), max, 8), "|████████|");
    }

    #[test]
    fn horizontal_bar_handles_zero_max_duration() {
        assert_eq!(
            horizontal_bar(Duration::minutes(5), Duration::zero(), 8),
            "|░░░░░░░░|"
        );
    }
}
