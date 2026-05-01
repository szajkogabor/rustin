use crate::commands::list::{kind_emoji, priority_emoji, task_order};
use crate::store::{Board, Task, TaskStatus};
use anyhow::Context;
use clap::Args;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};
use std::io::{self, Stdout};

#[derive(Args)]
pub struct TuiCommand;

impl TuiCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut terminal = TerminalSession::enter()?;
        let mut app = App::load()?;

        loop {
            terminal.draw(|frame| app.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Up => app.select_previous(),
                    KeyCode::Down => app.select_next(),
                    KeyCode::Char('t') => app.move_selected(TaskStatus::Todo)?,
                    KeyCode::Char('i') => app.move_selected(TaskStatus::InProgress)?,
                    KeyCode::Char('d') => app.move_selected(TaskStatus::Done)?,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("failed to initialize terminal")?;
        Ok(Self { terminal })
    }

    fn draw<F>(&mut self, render: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(render)?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Clone)]
struct TaskRow {
    id: u32,
    title: String,
    priority: String,
    kind: String,
    status: String,
    description: Option<String>,
}

struct App {
    title: String,
    tasks: Vec<TaskRow>,
    selected: Option<usize>,
    status_line: String,
}

impl App {
    fn load() -> anyhow::Result<Self> {
        Self::load_with_selected(None)
    }

    fn load_with_selected(selected_task_id: Option<u32>) -> anyhow::Result<Self> {
        let board = Board::load()?;
        let tasks = sorted_rows(&board.tasks);
        let selected = select_index(&tasks, selected_task_id);
        let status_line = if tasks.is_empty() {
            "No tasks yet. Press q to quit.".to_string()
        } else {
            "Arrow keys select. t/i/d change status. q quits.".to_string()
        };

        Ok(Self {
            title: board.title,
            tasks,
            selected,
            status_line,
        })
    }

    fn selected_task_id(&self) -> Option<u32> {
        self.selected
            .and_then(|index| self.tasks.get(index).map(|task| task.id))
    }

    fn select_next(&mut self) {
        if self.tasks.is_empty() {
            self.selected = None;
            return;
        }

        let next = match self.selected {
            Some(index) if index + 1 < self.tasks.len() => index + 1,
            _ => 0,
        };
        self.selected = Some(next);
    }

    fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            self.selected = None;
            return;
        }

        let previous = match self.selected {
            Some(index) if index > 0 => index - 1,
            _ => self.tasks.len() - 1,
        };
        self.selected = Some(previous);
    }

    fn move_selected(&mut self, status: TaskStatus) -> anyhow::Result<()> {
        let Some(task_id) = self.selected_task_id() else {
            self.status_line = "No task selected.".to_string();
            return Ok(());
        };

        let mut board = Board::load()?;
        if board.move_task(task_id, status.clone()) {
            board.save()?;
            let mut refreshed = Self::load_with_selected(Some(task_id))?;
            refreshed.status_line = format!("Task {task_id} moved to {}.", status_label(&status));
            *self = refreshed;
        } else {
            self.status_line = format!("Task {task_id} no longer exists.");
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(7),
                Constraint::Length(2),
            ])
            .split(frame.area());

        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let line = Line::from(vec![
                    Span::raw(format!("{} [{}] {}", task.priority, task.id, task.title)),
                    Span::styled(format!(" {}", task.kind), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("  {}", task.status),
                        Style::default().fg(status_color(&task.status)),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let tasks = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} tasks", self.title)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(self.selected);
        frame.render_stateful_widget(tasks, areas[0], &mut state);

        let detail_lines = self
            .selected
            .and_then(|index| self.tasks.get(index))
            .map(|task| {
                vec![
                    Line::from(format!("Status: {}", task.status)),
                    Line::from(format!("Priority: {}", task.priority)),
                    Line::from(format!("Kind: {}", task.kind)),
                    Line::from(format!(
                        "Description: {}",
                        task.description.as_deref().unwrap_or("(none)")
                    )),
                ]
            })
            .unwrap_or_else(|| vec![Line::from("No task selected.")]);

        let details = Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title("Details"));
        frame.render_widget(details, areas[1]);

        let footer =
            Paragraph::new(self.status_line.as_str()).style(Style::default().fg(Color::Yellow));
        frame.render_widget(footer, areas[2]);
    }
}

fn sorted_rows(tasks: &[Task]) -> Vec<TaskRow> {
    let mut ordered: Vec<&Task> = tasks.iter().collect();
    ordered.sort_by(task_order);

    ordered
        .into_iter()
        .map(|task| TaskRow {
            id: task.id,
            title: task.title.clone(),
            priority: priority_emoji(task.priority).to_string(),
            kind: kind_emoji(task.kind).to_string(),
            status: status_label(&task.status).to_string(),
            description: task.description.clone(),
        })
        .collect()
}

fn select_index(tasks: &[TaskRow], selected_task_id: Option<u32>) -> Option<usize> {
    if tasks.is_empty() {
        return None;
    }

    selected_task_id
        .and_then(|task_id| tasks.iter().position(|task| task.id == task_id))
        .or(Some(0))
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
    }
}

fn status_color(status: &str) -> Color {
    match status {
        "Todo" => Color::Blue,
        "In Progress" => Color::Yellow,
        "Done" => Color::Green,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::{App, TaskRow, select_index};

    fn row(id: u32) -> TaskRow {
        TaskRow {
            id,
            title: format!("task-{id}"),
            priority: "🌶️".to_string(),
            kind: "✨".to_string(),
            status: "Todo".to_string(),
            description: None,
        }
    }

    fn app_with_tasks(task_ids: &[u32], selected: Option<usize>) -> App {
        App {
            title: "Board".to_string(),
            tasks: task_ids.iter().copied().map(row).collect(),
            selected,
            status_line: String::new(),
        }
    }

    #[test]
    fn select_index_defaults_to_first_task() {
        let tasks = vec![row(10), row(20)];
        assert_eq!(select_index(&tasks, None), Some(0));
    }

    #[test]
    fn select_index_prefers_matching_task_id() {
        let tasks = vec![row(10), row(20), row(30)];
        assert_eq!(select_index(&tasks, Some(20)), Some(1));
    }

    #[test]
    fn next_selection_wraps_to_start() {
        let mut app = app_with_tasks(&[1, 2, 3], Some(2));
        app.select_next();
        assert_eq!(app.selected, Some(0));
    }

    #[test]
    fn previous_selection_wraps_to_end() {
        let mut app = app_with_tasks(&[1, 2, 3], Some(0));
        app.select_previous();
        assert_eq!(app.selected, Some(2));
    }

    #[test]
    fn moving_with_no_tasks_sets_status_message() {
        let mut app = app_with_tasks(&[], None);
        app.move_selected(crate::store::TaskStatus::Done).unwrap();
        assert_eq!(app.status_line, "No task selected.");
    }
}
