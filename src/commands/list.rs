use crate::commands::display::{
    TaskColumn, TaskColumns, build_task_table_rows, split_tasks, visible_task_columns,
};
use crate::store::Board;
use anyhow::Context;
use clap::Args;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Widget};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::io::{self, IsTerminal, Stdout};

#[derive(Args)]
pub struct ListCommand {
    /// Columns to display (default: all)
    #[arg(short, long, value_enum, num_args = 1..)]
    pub columns: Vec<TaskColumn>,
}

impl ListCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let board = Board::load()?;
        let view = ListView::from_board(&board, &self.columns);

        if io::stdout().is_terminal() {
            let mut terminal = InlineListTerminal::enter(view.viewport_height())?;
            terminal.draw(|frame| view.render(frame))?;
        } else {
            println!("{}", view.render_to_string(terminal_width()));
        }

        Ok(())
    }
}

struct InlineListTerminal {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl InlineListTerminal {
    fn enter(height: u16) -> anyhow::Result<Self> {
        enable_raw_mode().context("failed to enable raw mode for inline list rendering")?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(height),
            },
        )
        .context("failed to initialize inline list terminal")?;

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

impl Drop for InlineListTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Debug)]
struct ListView {
    title: String,
    columns: TaskColumns,
    visible_columns: Vec<TaskColumn>,
    is_empty: bool,
}

impl ListView {
    fn from_board(board: &Board, selected_columns: &[TaskColumn]) -> Self {
        Self {
            title: board.title.clone(),
            columns: split_tasks(&board.tasks),
            visible_columns: visible_task_columns(selected_columns),
            is_empty: board.tasks.is_empty(),
        }
    }

    fn viewport_height(&self) -> u16 {
        1 + self.content_height()
    }

    fn render(&self, frame: &mut Frame) {
        let [title_area, content_area] = split_layout(frame.area(), self.content_height());
        frame.render_widget(Paragraph::new(self.title_line()), title_area);

        if self.is_empty {
            frame.render_widget(self.empty_widget(), content_area);
        } else {
            frame.render_widget(self.table_widget(), content_area);
        }
    }

    fn render_to_string(&self, width: u16) -> String {
        let area = Rect::new(0, 0, width.max(20), self.viewport_height());
        let mut buffer = Buffer::empty(area);
        let [title_area, content_area] = split_layout(area, self.content_height());

        Paragraph::new(self.title_line()).render(title_area, &mut buffer);
        if self.is_empty {
            self.empty_widget().render(content_area, &mut buffer);
        } else {
            self.table_widget().render(content_area, &mut buffer);
        }

        buffer_to_string(&buffer)
    }

    fn content_height(&self) -> u16 {
        if self.is_empty {
            3
        } else {
            self.columns.max_rows(&self.visible_columns) as u16 + 3
        }
    }

    fn title_line(&self) -> String {
        format!("=== {} ===", self.title)
    }

    fn empty_widget(&self) -> Paragraph<'static> {
        Paragraph::new("The board is empty. Add a task with `rustin add \"Task title\"`")
            .block(Block::default().borders(Borders::ALL))
    }

    fn table_widget(&self) -> Table<'static> {
        let rows = build_task_table_rows(&self.columns, &self.visible_columns)
            .into_iter()
            .map(Row::new)
            .collect::<Vec<_>>();
        let widths = vec![
            Constraint::Ratio(1, self.visible_columns.len() as u32);
            self.visible_columns.len()
        ];
        let header = Row::new(
            self.visible_columns
                .iter()
                .map(|column| column.title())
                .collect::<Vec<_>>(),
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

        Table::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .block(Block::default().borders(Borders::ALL))
    }
}

fn split_layout(area: Rect, content_height: u16) -> [Rect; 2] {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(content_height)])
        .split(area);

    [sections[0], sections[1]]
}

fn terminal_width() -> u16 {
    size()
        .map(|(width, _)| width)
        .or_else(|_| {
            std::env::var("COLUMNS")
                .ok()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| std::io::Error::other("missing COLUMNS"))
        })
        .unwrap_or(120)
}

fn buffer_to_string(buffer: &Buffer) -> String {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::ListView;
    use crate::commands::display::TaskColumn;
    use crate::store::{Board, Task, TaskKind, TaskPriority, TaskStatus};
    use chrono::Utc;

    fn board_with_tasks(tasks: Vec<Task>) -> Board {
        Board {
            title: "Board".to_string(),
            next_id: tasks.len() as u32 + 1,
            tasks,
            ..Board::default()
        }
    }

    fn task(id: u32, title: &str, status: TaskStatus) -> Task {
        Task {
            id,
            title: title.to_string(),
            priority: TaskPriority::Medium,
            kind: TaskKind::Feature,
            description: None,
            status,
            created_at: Utc::now(),
            transitions: vec![],
        }
    }

    #[test]
    fn render_to_string_shows_board_title_and_table_headers() {
        let view = ListView::from_board(
            &board_with_tasks(vec![task(1, "Write tests", TaskStatus::Todo)]),
            &[],
        );

        let rendered = view.render_to_string(80);

        assert!(rendered.contains("=== Board ==="));
        assert!(rendered.contains("Todo"));
        assert!(rendered.contains("In Progress"));
        assert!(rendered.contains("Done"));
        assert!(rendered.contains("Write tests"));
    }

    #[test]
    fn render_to_string_honors_selected_columns() {
        let view = ListView::from_board(
            &board_with_tasks(vec![task(1, "Ship it", TaskStatus::Done)]),
            &[TaskColumn::Done],
        );

        let rendered = view.render_to_string(80);

        assert!(!rendered.contains("Todo │"));
        assert!(!rendered.contains("In Progress"));
        assert!(rendered.contains("Done"));
        assert!(rendered.contains("Ship it"));
    }

    #[test]
    fn render_to_string_shows_empty_message() {
        let view = ListView::from_board(&board_with_tasks(vec![]), &[]);

        let rendered = view.render_to_string(80);

        assert!(rendered.contains("=== Board ==="));
        assert!(rendered.contains("The board is empty"));
    }

    #[test]
    fn viewport_height_accounts_for_title_and_table_rows() {
        let view = ListView::from_board(
            &board_with_tasks(vec![
                task(1, "One", TaskStatus::Todo),
                task(2, "Two", TaskStatus::Done),
            ]),
            &[],
        );

        assert_eq!(view.viewport_height(), 5);
    }
}
