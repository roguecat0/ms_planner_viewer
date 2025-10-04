use std::str::FromStr;

use ratatui::{
    Frame,
    crossterm::style::Color,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, List, Padding, Paragraph, Row, Table},
};

use crate::{
    app::{App, InputMode},
    config::{MultiTagFilter, TagFilter},
    ms_planner::{Priority, Progress, Task},
};
const HEADERS_LEN: usize = 5;
const DATE_CONSTRAINT: Constraint = Constraint::Length(10);

pub fn view(app: &mut App, f: &mut Frame) {
    match app.input_mode {
        InputMode::TableRow => render_table(app, f),
        InputMode::FilterMode => render_filter_list(app, f),
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
fn render_filter_list(app: &mut App, f: &mut Frame) {
    let list = if let Some(ui_filter) = &app.filter_view.ui_tag_filter {
        let list = match ui_filter {
            UiTagFilter::Single(v) => List::new(v.iter().map(|(s, a)| s.as_str())),
            UiTagFilter::Multi(v) => List::new(v.iter().map(|(s, a)| s.as_str())),
        };
        list.block(Block::bordered().title("Filter"))
            .highlight_symbol("|")
    } else {
        let list = List::new(
            app.config
                .filter
                .get_ui_columns()
                .into_iter()
                .map(Into::<Text>::into),
        );
        list.block(Block::bordered().title("Filter"))
            .highlight_symbol("|")
    };
    f.render_stateful_widget(list, f.area(), &mut app.filter_view.state);
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
pub trait AsText {
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
pub enum UiTagFilter {
    Multi(Vec<(String, MultiTagState)>),
    Single(Vec<(String, TagState)>),
}
pub enum TagState {
    Or,
    Nil,
    Not,
}
pub enum MultiTagState {
    Or,
    And,
    Nil,
    Not,
}
impl<T: FromStr + PartialEq> From<(TagFilter<T>, &[String])> for UiTagFilter {
    fn from((tf, uniques): (TagFilter<T>, &[String])) -> Self {
        let filter = uniques
            .into_iter()
            .map(|s| {
                if let Ok(t) = T::from_str(s) {
                    if tf.or.contains(&t) {
                        (s.to_string(), TagState::Or)
                    } else if tf.not.contains(&t) {
                        (s.to_string(), TagState::Not)
                    } else {
                        (s.to_string(), TagState::Nil)
                    }
                } else {
                    (s.to_string(), TagState::Nil)
                }
            })
            .collect();
        Self::Single(filter)
    }
}
impl From<(MultiTagFilter, &[String])> for UiTagFilter {
    fn from((tf, uniques): (MultiTagFilter, &[String])) -> Self {
        let filter = uniques
            .into_iter()
            .map(|s| {
                if tf.or.contains(s) {
                    (s.to_string(), MultiTagState::Or)
                } else if tf.and.contains(s) {
                    (s.to_string(), MultiTagState::And)
                } else if tf.not.contains(s) {
                    (s.to_string(), MultiTagState::Not)
                } else {
                    (s.to_string(), MultiTagState::Nil)
                }
            })
            .collect();
        Self::Multi(filter)
    }
}
impl<T> TryFrom<UiTagFilter> for TagFilter<T>
where
    T: FromStr,
    T::Err: Sync + Send + std::error::Error + 'static,
{
    type Error = anyhow::Error;
    fn try_from(value: UiTagFilter) -> Result<Self, Self::Error> {
        if let UiTagFilter::Single(v) = value {
            let mut tf = TagFilter {
                or: vec![],
                not: vec![],
            };
            for (s, state) in v {
                match state {
                    TagState::Or => tf.or.push(T::from_str(&s)?),
                    TagState::Not => tf.not.push(T::from_str(&s)?),
                    TagState::Nil => (),
                }
            }
            Ok(tf)
        } else {
            anyhow::bail!("type conversion: UiTagFilter::Multi to TagFilter")
        }
    }
}
impl TryFrom<UiTagFilter> for MultiTagFilter {
    type Error = anyhow::Error;
    fn try_from(value: UiTagFilter) -> Result<Self, Self::Error> {
        if let UiTagFilter::Multi(v) = value {
            let mut tf = MultiTagFilter {
                or: vec![],
                and: vec![],
                not: vec![],
            };
            for (s, state) in v {
                match state {
                    MultiTagState::Or => tf.or.push(s),
                    MultiTagState::And => tf.and.push(s),
                    MultiTagState::Not => tf.not.push(s),
                    MultiTagState::Nil => (),
                }
            }
            Ok(tf)
        } else {
            anyhow::bail!("type conversion: UiTagFilter::Multi to TagFilter")
        }
    }
}
pub enum UiColumn {
    Bucket,
    Progress,
    Priority,
    Labels,
    AssignedTo,
}
impl From<UiColumn> for Text<'static> {
    fn from(value: UiColumn) -> Self {
        match value {
            UiColumn::AssignedTo => "assigned to".into(),
            UiColumn::Progress => "progress".into(),
            UiColumn::Priority => "prioity".into(),
            UiColumn::Labels => "labels".into(),
            UiColumn::Bucket => "bucket".into(),
        }
    }
}
