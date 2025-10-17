use crate::{config::Config, filter};
use ratatui::{
    Frame,
    crossterm::style::Color,
    layout::{Constraint, Flex, Layout, Rect},
    style::{self, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Row, Table, Wrap},
};

use crate::{
    Task,
    app::{App, InputMode},
};
use style::palette::tailwind;
const HEADERS_LEN: usize = 6;
const DATE_CONSTRAINT: Constraint = Constraint::Length(10);

pub fn view(app: &mut App, f: &mut Frame) {
    match app.input_mode {
        InputMode::TableRow => table::view(app, f, f.area()),
        InputMode::FilterMode => {
            let [filter, table] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(f.area());
            filter::render_filter_column(app, f, filter);
            table::view(app, f, table);
        }
    }
    render_error_box(app, f);
}

fn render_error_box(app: &mut App, f: &mut Frame) {
    if let Some(error) = &app.error_popup {
        let area = center(
            f.area(),
            Constraint::Percentage(80),
            Constraint::Percentage(70),
        );
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(error.clone()).block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .padding(Padding::uniform(3))
                    .bg(Color::Black),
            ),
            area,
        );
    }
}
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
pub trait AsText {
    fn as_text(&self) -> Text<'_>;
}
pub mod table {
    use ratatui::{
        layout::Alignment,
        text::{Line, Span},
    };

    use super::*;
    pub fn get_headers() -> [Text<'static>; HEADERS_LEN] {
        [
            "Name".into(),
            "Bucket".into(),
            "Pro".into(),
            "Pri".into(),
            "Items".into(),
            "Created".into(),
        ]
    }
    pub fn view(app: &mut App, f: &mut Frame, area: Rect) {
        let headers = Row::new(get_headers());
        let rows = app
            .displayed_tasks
            .iter()
            .map(|task| task_to_row(task, &app.config));
        let cols: [Constraint; HEADERS_LEN] = [
            Constraint::Fill(1),
            Constraint::Length(15),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            DATE_CONSTRAINT,
        ];

        let table = Table::new(rows, cols)
            .header(headers)
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .block(Block::bordered().title("ms planner"));
        f.render_stateful_widget(table, area, &mut app.table_state);

        if let Some(i) = app.selected_task {
            task::view(app, f, area, i);
        }
    }
    fn task_to_row<'a>(task: &'a Task, config: &'a Config) -> Row<'a> {
        let name: Text = task.name.clone().into();
        let name = if config.filter.ids.contains(&task.id) {
            name.fg(tailwind::ORANGE.c300)
        } else {
            name
        };
        let cells: [Text; HEADERS_LEN] = [
            name,
            task.bucket.clone().into(),
            task.progress.as_text(),
            task.priority.as_text(),
            complete_items_text(task.items_completed),
            task.create_date.to_string().into(),
        ];
        Row::new(cells)
    }
    fn complete_items_text<'a>(items: Option<(usize, usize)>) -> Text<'a> {
        let text: Text = if let Some((completed, all)) = items {
            let span_completed = if completed == all {
                Span::from(completed.to_string()).light_green()
            } else if completed == 0 {
                Span::from(completed.to_string()).light_red()
            } else {
                Span::from(completed.to_string()).light_blue()
            };
            Line::from_iter([span_completed, format!("/{all}").into()]).into()
        } else {
            "".into()
        };
        text.alignment(Alignment::Center)
    }
}

pub mod task {
    use ratatui::{
        text::{Line, Span},
        widgets::List,
    };

    use super::*;
    pub fn view(app: &mut App, f: &mut Frame, area: Rect, i: usize) {
        let task = &app.displayed_tasks[i];
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        f.render_widget(Clear, area);
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title("Task");
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        let rows_needed = std::cmp::max(task.assigned_to.len(), task.labels.len());
        let [name_area, middle_area, description_area, items_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(rows_needed as u16),
            Constraint::Fill(1),
            Constraint::Length(task.items.len() as u16 + 2),
        ])
        .areas(inner_area);
        let [labels_area, assigned_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)]).areas(middle_area);
        f.render_widget(
            Paragraph::new(task.name.clone())
                .block(Block::bordered().title("name"))
                .wrap(Wrap::default()),
            name_area,
        );
        f.render_widget(Text::from_iter(task.assigned_to.clone()), assigned_area);
        f.render_widget(Text::from_iter(task.labels.clone()), labels_area);
        f.render_widget(
            Paragraph::new(task.description.clone())
                .block(Block::bordered().title("description"))
                .wrap(Wrap::default()),
            description_area,
        );
        let list = get_item_list(task);
        f.render_widget(list, items_area);
    }
    pub fn get_item_list<'a>(task: &'a Task) -> List<'a> {
        let title = format!(
            "Items: {}",
            task.items_completed
                .map(|(i, n)| format!("{i} / {n} "))
                .unwrap_or("_ / _ ".to_string())
        );
        let completed = task.items_completed.unwrap_or_default();
        let checked = Span::from(" [✓] ").fg(Color::Green);
        let unknown = Span::from(" [?] ").fg(Color::Blue);
        let unchecked = Span::from(" [x] ").fg(Color::Red);
        let symbol = if completed.0 == completed.1 {
            checked
        } else if completed.0 == 0 {
            unchecked
        } else {
            unknown
        };
        let items = task
            .items
            .iter()
            .map(|s| Line::from_iter([symbol.clone(), Span::from(s)]));

        List::new(items).block(Block::bordered().title(title))
    }
}
