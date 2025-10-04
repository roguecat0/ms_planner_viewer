use ratatui::{
    Frame,
    crossterm::style::Color,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Row, Table, Widget},
};

use crate::{
    app::App,
    ms_planner::{Priority, Progress, Task},
};
const HEADERS_LEN: usize = 5;
const DATE_CONSTRAINT: Constraint = Constraint::Length(10);

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
        "Created".into(),
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
        DATE_CONSTRAINT,
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
trait AsText {
    fn as_text(&self) -> Text {
        Text::from("lol")
    }
}
impl AsText for Priority {
    fn as_text(&self) -> Text {
        match self {
            Self::Low => Text::from(" Ⅰ ").fg(Color::Blue),
            Self::Mid => Text::from(" Ⅱ "),
            Self::Important => Text::from(" Ⅲ ").fg(Color::Yellow),
            Self::Urgent => Text::from(" Ⅳ ").fg(Color::Red),
        }
    }
}
impl AsText for Progress {
    fn as_text(&self) -> Text {
        match self {
            Self::Done => Text::from("[✓]"),
            Self::Ongoing => Text::from("[-]"),
            Self::NotStarted => Text::from("[ ]"),
        }
    }
}
