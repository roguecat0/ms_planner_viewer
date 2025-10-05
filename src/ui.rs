use std::str::FromStr;

use crate::{Column, config};
use ratatui::{
    Frame,
    crossterm::style::Color,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, List, Padding, Paragraph, Row, Table, Wrap},
};

use crate::{
    app::{App, InputMode},
    config::{MultiTagFilter, Order, TagFilter},
    ms_planner::{Priority, Progress, Task},
};
const HEADERS_LEN: usize = 5;
const DATE_CONSTRAINT: Constraint = Constraint::Length(10);

pub fn view(app: &mut App, f: &mut Frame) {
    match app.input_mode {
        InputMode::TableRow => render_table(app, f, f.area()),
        InputMode::FilterMode => {
            let [filter, table] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(f.area());
            render_filter_list(app, f, filter);
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
fn render_filter_list(app: &mut App, f: &mut Frame, area: Rect) {
    let list = if let Some((ui_filter, _)) = &app.filter_view.ui_tag_filter {
        let list = match ui_filter {
            UiTagFilter::Single(v) => List::new(v.iter().map(|u| u.as_text())),
            UiTagFilter::Multi(v) => List::new(v.iter().map(|u| u.as_text())),
        };
        list.block(Block::bordered().title("Filter"))
            .highlight_symbol("|")
    } else {
        let list = List::new(
            config::get_ui_columns(&app.config.filter, &app.config.sort)
                .into_iter()
                .map(Into::<Text>::into),
        );
        list.block(Block::bordered().title("Filter"))
            .highlight_symbol("|")
    };
    f.render_stateful_widget(list, area, &mut app.filter_view.state);
}
fn render_table(app: &mut App, f: &mut Frame, area: Rect) {
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
    f.render_stateful_widget(table, area, &mut app.table_state);

    if let Some(i) = app.selected_task {
        let task = &app.displayed_tasks[i];
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        f.render_widget(Clear, area.clone());
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title("Task");
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        let [name_area, description_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(inner_area);
        f.render_widget(
            Paragraph::new(task.name.clone())
                .block(Block::bordered().title("name"))
                .wrap(Wrap::default()),
            name_area,
        );
        f.render_widget(
            Paragraph::new(task.description.clone())
                .block(Block::bordered().title("description"))
                .wrap(Wrap::default()),
            description_area,
        );
    }
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
    fn as_text(&self) -> Text;
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
#[derive(Clone)]
pub enum UiTagFilter {
    Multi(Vec<(String, MultiTagState)>),
    Single(Vec<(String, TagState)>),
}
#[derive(Clone)]
pub enum TagState {
    Or,
    Nil,
    Not,
}
#[derive(Clone)]
pub enum MultiTagState {
    Or,
    And,
    Nil,
    Not,
}
impl TagState {
    pub fn next(&mut self) {
        let next = match self {
            Self::Nil => Self::Or,
            Self::Or => Self::Not,
            Self::Not => Self::Nil,
        };
        let _ = std::mem::replace(self, next);
    }
}
impl MultiTagState {
    pub fn next(&mut self) {
        let next = match self {
            Self::Nil => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Not,
            Self::Not => Self::Nil,
        };
        let _ = std::mem::replace(self, next);
    }
}
impl AsText for (String, MultiTagState) {
    fn as_text(&self) -> Text {
        use MultiTagState as M;
        let symbol = match self.1 {
            M::Or => "+",
            M::And => "*",
            M::Not => "-",
            M::Nil => " ",
        };
        Text::from(format!("{symbol} {}", self.0))
    }
}
impl AsText for (String, TagState) {
    fn as_text(&self) -> Text {
        use TagState as M;
        let symbol = match self.1 {
            M::Or => "+",
            M::Not => "-",
            M::Nil => " ",
        };
        Text::from(format!("{symbol} {}", self.0))
    }
}
impl<T> From<(TagFilter<T>, &[String])> for UiTagFilter
where
    T: FromStr + PartialEq,
    T::Err: std::error::Error + Send + Sync + 'static,
{
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
pub struct UiColumn {
    pub sort: Option<Order>,
    pub filtered: bool,
    pub column: Column,
}
impl From<UiColumn> for Text<'static> {
    fn from(value: UiColumn) -> Self {
        use Column as C;
        let mut s = match value.sort {
            Some(Order::Asc) => "A ",
            Some(Order::Desc) => "V ",
            None => "  ",
        }
        .to_string();
        s += match value.column {
            C::AssignedTo => "assigned to",
            C::Progress => "progress",
            C::Priority => "prioity",
            C::Labels => "labels",
            C::Bucket => "bucket",
            _ => todo!(),
        };
        if value.filtered {
            Text::from(format!("[*] {s}")).add_modifier(Modifier::BOLD)
        } else {
            Text::from(format!("    {s}"))
        }
    }
}
