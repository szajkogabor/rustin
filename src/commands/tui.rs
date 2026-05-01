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
                    KeyCode::Left => app.select_left(),
                    KeyCode::Right => app.select_right(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskColumn {
    Todo,
    InProgress,
    Done,
}

impl TaskColumn {
    const ALL: [TaskColumn; 3] = [TaskColumn::Todo, TaskColumn::InProgress, TaskColumn::Done];

    fn title(self) -> &'static str {
        match self {
            TaskColumn::Todo => "Todo",
            TaskColumn::InProgress => "In Progress",
            TaskColumn::Done => "Done",
        }
    }

    fn next(self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::InProgress,
            TaskColumn::InProgress => TaskColumn::Done,
            TaskColumn::Done => TaskColumn::Todo,
        }
    }

    fn previous(self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::Done,
            TaskColumn::InProgress => TaskColumn::Todo,
            TaskColumn::Done => TaskColumn::InProgress,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    column: TaskColumn,
    index: usize,
}

#[derive(Default)]
struct TaskColumns {
    todo: Vec<TaskRow>,
    in_progress: Vec<TaskRow>,
    done: Vec<TaskRow>,
}

impl TaskColumns {
    fn tasks(&self, column: TaskColumn) -> &[TaskRow] {
        match column {
            TaskColumn::Todo => &self.todo,
            TaskColumn::InProgress => &self.in_progress,
            TaskColumn::Done => &self.done,
        }
    }

    fn first_selection(&self) -> Option<Selection> {
        TaskColumn::ALL
            .into_iter()
            .find_map(|column| self.selection_for_column(column, 0))
    }

    fn find_selection(&self, task_id: u32) -> Option<Selection> {
        TaskColumn::ALL.into_iter().find_map(|column| {
            self.tasks(column)
                .iter()
                .position(|task| task.id == task_id)
                .map(|index| Selection { column, index })
        })
    }

    fn selection_for_column(
        &self,
        column: TaskColumn,
        preferred_index: usize,
    ) -> Option<Selection> {
        let tasks = self.tasks(column);
        if tasks.is_empty() {
            None
        } else {
            Some(Selection {
                column,
                index: preferred_index.min(tasks.len() - 1),
            })
        }
    }

    fn adjacent_selection(
        &self,
        current: TaskColumn,
        preferred_index: usize,
        move_right: bool,
    ) -> Option<Selection> {
        let mut column = current;
        for _ in 0..TaskColumn::ALL.len() - 1 {
            column = if move_right {
                column.next()
            } else {
                column.previous()
            };

            if let Some(selection) = self.selection_for_column(column, preferred_index) {
                return Some(selection);
            }
        }

        self.selection_for_column(current, preferred_index)
    }
}

struct App {
    title: String,
    columns: TaskColumns,
    selected: Option<Selection>,
    status_line: String,
}

impl App {
    fn load() -> anyhow::Result<Self> {
        Self::load_with_selected(None)
    }

    fn load_with_selected(selected_task_id: Option<u32>) -> anyhow::Result<Self> {
        let board = Board::load()?;
        let columns = split_tasks(&board.tasks);
        let selected = selected_task_id
            .and_then(|task_id| columns.find_selection(task_id))
            .or_else(|| columns.first_selection());
        let status_line = if selected.is_none() {
            "No tasks yet. Press q to quit.".to_string()
        } else {
            "Arrow keys move across the board. t/i/d change status. q quits.".to_string()
        };

        Ok(Self {
            title: board.title,
            columns,
            selected,
            status_line,
        })
    }

    fn selected_task_id(&self) -> Option<u32> {
        self.selected.and_then(|selection| {
            self.columns
                .tasks(selection.column)
                .get(selection.index)
                .map(|task| task.id)
        })
    }

    fn select_next(&mut self) {
        let Some(selection) = self.selected else {
            self.selected = self.columns.first_selection();
            return;
        };

        let tasks = self.columns.tasks(selection.column);
        if tasks.is_empty() {
            self.selected = self.columns.first_selection();
            return;
        }

        self.selected = Some(Selection {
            column: selection.column,
            index: (selection.index + 1) % tasks.len(),
        });
    }

    fn select_previous(&mut self) {
        let Some(selection) = self.selected else {
            self.selected = self.columns.first_selection();
            return;
        };

        let tasks = self.columns.tasks(selection.column);
        if tasks.is_empty() {
            self.selected = self.columns.first_selection();
            return;
        }

        self.selected = Some(Selection {
            column: selection.column,
            index: if selection.index == 0 {
                tasks.len() - 1
            } else {
                selection.index - 1
            },
        });
    }

    fn select_left(&mut self) {
        self.selected = self.selected.and_then(|selection| {
            self.columns
                .adjacent_selection(selection.column, selection.index, false)
        });
    }

    fn select_right(&mut self) {
        self.selected = self.selected.and_then(|selection| {
            self.columns
                .adjacent_selection(selection.column, selection.index, true)
        });
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
                Constraint::Min(8),
                Constraint::Length(6),
                Constraint::Length(2),
            ])
            .split(frame.area());

        let board_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(areas[0]);

        for (column, area) in [
            (TaskColumn::Todo, board_columns[0]),
            (TaskColumn::InProgress, board_columns[1]),
            (TaskColumn::Done, board_columns[2]),
        ] {
            self.render_column(frame, area, column);
        }

        let detail_lines = self
            .selected
            .and_then(|selection| self.columns.tasks(selection.column).get(selection.index))
            .map(|task| {
                vec![
                    Line::from(format!("Task: [{}] {}", task.id, task.title)),
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

        let details = Paragraph::new(detail_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} details", self.title)),
        );
        frame.render_widget(details, areas[1]);

        let footer =
            Paragraph::new(self.status_line.as_str()).style(Style::default().fg(Color::Yellow));
        frame.render_widget(footer, areas[2]);
    }

    fn render_column(&self, frame: &mut Frame, area: ratatui::layout::Rect, column: TaskColumn) {
        let items: Vec<ListItem> = self
            .columns
            .tasks(column)
            .iter()
            .map(|task| {
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} [{}] {}", task.priority, task.id, task.title)),
                    Span::styled(format!(" {}", task.kind), Style::default().fg(Color::Cyan)),
                ]))
            })
            .collect();

        let is_active = self
            .selected
            .is_some_and(|selection| selection.column == column);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(column.title())
                    .border_style(if is_active {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        if let Some(selection) = self.selected
            && selection.column == column
        {
            state.select(Some(selection.index));
        }

        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn split_tasks(tasks: &[Task]) -> TaskColumns {
    let mut ordered: Vec<&Task> = tasks.iter().collect();
    ordered.sort_by(task_order);

    let mut columns = TaskColumns::default();

    for task in ordered {
        let row = TaskRow {
            id: task.id,
            title: task.title.clone(),
            priority: priority_emoji(task.priority).to_string(),
            kind: kind_emoji(task.kind).to_string(),
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

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
    }
}

#[cfg(test)]
mod tests {
    use super::{App, Selection, TaskColumn, TaskColumns, TaskRow};

    fn row(id: u32, status: &str) -> TaskRow {
        TaskRow {
            id,
            title: format!("task-{id}"),
            priority: "🌶️".to_string(),
            kind: "✨".to_string(),
            status: status.to_string(),
            description: None,
        }
    }

    fn columns(todo: &[u32], in_progress: &[u32], done: &[u32]) -> TaskColumns {
        TaskColumns {
            todo: todo.iter().copied().map(|id| row(id, "Todo")).collect(),
            in_progress: in_progress
                .iter()
                .copied()
                .map(|id| row(id, "In Progress"))
                .collect(),
            done: done.iter().copied().map(|id| row(id, "Done")).collect(),
        }
    }

    fn app_with_columns(columns: TaskColumns, selected: Option<Selection>) -> App {
        App {
            title: "Board".to_string(),
            columns,
            selected,
            status_line: String::new(),
        }
    }

    #[test]
    fn first_selection_prefers_todo_then_in_progress_then_done() {
        let columns = columns(&[], &[20], &[30]);
        assert_eq!(
            columns.first_selection(),
            Some(Selection {
                column: TaskColumn::InProgress,
                index: 0
            })
        );
    }

    #[test]
    fn find_selection_returns_matching_column_and_index() {
        let columns = columns(&[10], &[20, 21], &[30]);
        assert_eq!(
            columns.find_selection(21),
            Some(Selection {
                column: TaskColumn::InProgress,
                index: 1
            })
        );
    }

    #[test]
    fn next_selection_wraps_within_the_current_column() {
        let mut app = app_with_columns(
            columns(&[1, 2, 3], &[], &[]),
            Some(Selection {
                column: TaskColumn::Todo,
                index: 2,
            }),
        );
        app.select_next();
        assert_eq!(
            app.selected,
            Some(Selection {
                column: TaskColumn::Todo,
                index: 0,
            })
        );
    }

    #[test]
    fn selecting_right_moves_to_the_next_non_empty_column() {
        let mut app = app_with_columns(
            columns(&[1, 2], &[], &[3, 4]),
            Some(Selection {
                column: TaskColumn::Todo,
                index: 1,
            }),
        );

        app.select_right();

        assert_eq!(
            app.selected,
            Some(Selection {
                column: TaskColumn::Done,
                index: 1,
            })
        );
    }

    #[test]
    fn selecting_left_wraps_to_previous_non_empty_column() {
        let mut app = app_with_columns(
            columns(&[1, 2], &[3], &[]),
            Some(Selection {
                column: TaskColumn::Todo,
                index: 1,
            }),
        );

        app.select_left();

        assert_eq!(
            app.selected,
            Some(Selection {
                column: TaskColumn::InProgress,
                index: 0,
            })
        );
    }

    #[test]
    fn moving_with_no_tasks_sets_status_message() {
        let mut app = app_with_columns(columns(&[], &[], &[]), None);
        app.move_selected(crate::store::TaskStatus::Done).unwrap();
        assert_eq!(app.status_line, "No task selected.");
    }
}
