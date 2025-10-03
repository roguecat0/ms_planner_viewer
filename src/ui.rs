use ratatui::{
    Frame,
    crossterm::style::Color,
    layout::{Constraint, Margin},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Row, Table, Widget},
};

use crate::{
    app::App,
    ms_planner::{Priority, Progress, Task},
};
const HEADERS_LEN: usize = 5;

pub fn view(app: &mut App, f: &mut Frame) {
    render_table(app, f);
    render_error_box(app, f);
}
pub fn get_headers() -> [Text<'static>; HEADERS_LEN] {
    [
        "Name".into(),
        "Bucket".into(),
        "Pro".into(),
        "Pri".into(),
        "hello".into(),
    ]
}
fn render_table(app: &mut App, f: &mut Frame) {
    let headers = Row::new(get_headers());
    let rows = app.displayed_tasks.iter().map(|task| task_to_row(task));
    let cols = [
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, cols)
        .header(headers)
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .block(Block::bordered().title("ms planner"));
    f.render_stateful_widget(table, f.area(), &mut app.table_state);
}
fn task_to_row(task: &Task) -> Row {
    let cells: [Text; HEADERS_LEN] = [
        task.name.clone().into(),
        task.bucket.clone().into(),
        task.progress.as_text(),
        task.priority.as_text(),
        task.create_date.to_string().into(),
    ];
    Row::new(cells)
}

fn render_error_box(app: &mut App, f: &mut Frame) {
    if let Some(error) = &app.error_popup {
        let area = f.area().inner(Margin::new(10, 10));
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
trait AsText {
    fn as_text(&self) -> Text {
        Text::from("lol")
    }
}
impl AsText for Priority {
    fn as_text(&self) -> Text {
        match self {
            Self::Low => Text::from("???").fg(Color::Blue),
            Self::Mid => Text::from("|||"),
            Self::Important => Text::from("!!!").fg(Color::Yellow),
            Self::Urgent => Text::from("$$$").fg(Color::Red),
        }
    }
}
impl AsText for Progress {
    fn as_text(&self) -> Text {
        match self {
            Self::Done => Text::from("[v]"),
            Self::Ongoing => Text::from("[-]"),
            Self::NotStarted => Text::from("[ ]"),
        }
    }
}
