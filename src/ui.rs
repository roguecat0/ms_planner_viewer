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
    app::{App, InputMode},
    ms_planner::Task,
};
use style::palette::tailwind;
const HEADERS_LEN: usize = 5;
const DATE_CONSTRAINT: Constraint = Constraint::Length(10);

pub fn view(app: &mut App, f: &mut Frame) {
    match app.input_mode {
        InputMode::TableRow => render_table(app, f, f.area()),
        InputMode::FilterMode => {
            let [filter, table] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(f.area());
            filter::render_filter_column(app, f, filter);
            render_table(app, f, table);
        }
    }
    render_error_box(app, f);
}
pub fn get_headers() -> [Text<'static>; HEADERS_LEN] {
    [
        "Name".into(),
        "Bucket".into(),
        "Pro".into(),
        "Pri".into(),
        "Created".into(),
    ]
}
fn render_table(app: &mut App, f: &mut Frame, area: Rect) {
    let headers = Row::new(get_headers());
    let rows = app
        .displayed_tasks
        .iter()
        .map(|task| task_to_row(task, &app.config));
    let cols = [
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Length(3),
        Constraint::Length(3),
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
pub mod task {
    use super::*;
    pub fn view(app: &mut App, f: &mut Frame, area: Rect, i: usize) {
        let task = &app.displayed_tasks[i];
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        f.render_widget(Clear, area.clone());
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title("Task");
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        let rows_needed = std::cmp::max(task.assigned_to.len(), task.labels.len());
        let [name_area, middle_area, description_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(rows_needed as u16),
            Constraint::Fill(1),
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
        task.create_date.to_string().into(),
    ];
    Row::new(cells)
}

fn render_error_box(app: &mut App, f: &mut Frame) {
    if let Some(error) = &app.error_popup {
        let area = center(
            f.area(),
            Constraint::Percentage(80),
            Constraint::Percentage(70),
        );
        f.render_widget(Clear, area.clone());
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
