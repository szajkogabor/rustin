use crate::commands::display::{
    TaskColumn, TaskColumns, split_tasks, status_label, task_detail_lines, task_snapshot_lines,
};
use crate::store::{Board, TaskStatus};
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
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
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

                if app.showing_details() {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => app.close_details(),
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Up => app.select_previous(),
                    KeyCode::Down => app.select_next(),
                    KeyCode::Left => app.select_left(),
                    KeyCode::Right => app.select_right(),
                    KeyCode::Enter => app.open_details()?,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    column: TaskColumn,
    index: usize,
}

impl TaskColumns {
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
    detail_lines: Option<Vec<String>>,
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
            "Arrow keys move across the board. Enter shows details. t/i/d change status. q quits."
                .to_string()
        };

        Ok(Self {
            title: board.title,
            columns,
            selected,
            detail_lines: None,
            status_line,
        })
    }

    fn showing_details(&self) -> bool {
        self.detail_lines.is_some()
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

    fn open_details(&mut self) -> anyhow::Result<()> {
        let Some(task_id) = self.selected_task_id() else {
            self.status_line = "No task selected.".to_string();
            return Ok(());
        };

        let board = Board::load()?;
        let Some(task) = board.tasks.iter().find(|task| task.id == task_id) else {
            self.status_line = format!("Task {task_id} no longer exists.");
            return Ok(());
        };

        self.detail_lines = Some(task_detail_lines(task));
        self.status_line = format!("Viewing task {task_id}. Press Enter or Esc to close.");
        Ok(())
    }

    fn close_details(&mut self) {
        self.detail_lines = None;
        self.status_line = if self.selected.is_some() {
            "Arrow keys move across the board. Enter shows details. t/i/d change status. q quits."
                .to_string()
        } else {
            "No tasks yet. Press q to quit.".to_string()
        };
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
            .map(task_snapshot_lines)
            .map(|lines| lines.into_iter().map(Line::from).collect())
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

        if let Some(lines) = &self.detail_lines {
            self.render_detail_overlay(frame, lines);
        }
    }

    fn render_column(&self, frame: &mut Frame, area: ratatui::layout::Rect, column: TaskColumn) {
        let items: Vec<ListItem> = self
            .columns
            .tasks(column)
            .iter()
            .map(|task| ListItem::new(task.summary.clone()))
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

    fn render_detail_overlay(&self, frame: &mut Frame, lines: &[String]) {
        let popup_area = centered_rect(75, 70, frame.area());
        let content = lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect::<Vec<_>>();
        let popup = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Task details"));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }
}

fn centered_rect(
    horizontal_percent: u16,
    vertical_percent: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - vertical_percent) / 2),
            Constraint::Percentage(vertical_percent),
            Constraint::Percentage((100 - vertical_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - horizontal_percent) / 2),
            Constraint::Percentage(horizontal_percent),
            Constraint::Percentage((100 - horizontal_percent) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::{App, Selection};
    use crate::commands::display::{TaskColumn, TaskColumns, TaskRow, format_task, split_tasks};
    use crate::store::{Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::Utc;

    fn row(id: u32, status: &str) -> TaskRow {
        TaskRow {
            id,
            summary: format!("[{id:>2}] sample"),
            title: format!("task-{id}"),
            priority: "~".to_string(),
            kind: "+".to_string(),
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
            detail_lines: None,
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
    fn close_details_clears_overlay_and_restores_status_message() {
        let mut app = app_with_columns(
            columns(&[1], &[], &[]),
            Some(Selection {
                column: TaskColumn::Todo,
                index: 0,
            }),
        );
        app.detail_lines = Some(vec!["ID: 1".to_string()]);

        app.close_details();

        assert!(app.detail_lines.is_none());
        assert!(app.status_line.contains("Enter shows details"));
    }

    #[test]
    fn moving_with_no_tasks_sets_status_message() {
        let mut app = app_with_columns(columns(&[], &[], &[]), None);
        app.move_selected(crate::store::TaskStatus::Done).unwrap();
        assert_eq!(app.status_line, "No task selected.");
    }

    #[test]
    fn split_tasks_uses_shared_list_formatter_for_summary() {
        let task = Task {
            id: 7,
            title: "reuse formatter".to_string(),
            priority: TaskPriority::High,
            kind: TaskKind::Bug,
            description: Some("details".to_string()),
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            transitions: vec![],
        };

        let columns = split_tasks(std::slice::from_ref(&task));

        assert_eq!(columns.todo[0].summary, format_task(&task));
        assert_eq!(columns.todo[0].priority, "🔥");
        assert_eq!(columns.todo[0].kind, "🐛");
    }
}
