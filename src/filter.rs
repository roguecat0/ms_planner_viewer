use ratatui::{
    Frame,
    text::{Span, Text},
    widgets::{Block, List, Paragraph},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Stylize},
};
use std::ops::IndexMut;
use std::str::FromStr;

use crate::{
    Column, Priority, Progress,
    app::{App, FilterViewMode},
    config::{MultiTagFilter, Order, TagFilter, TaskFilter, TaskSort},
    ui::AsText,
};

pub fn render_filter_column(app: &mut App, f: &mut Frame, area: Rect) {
    match &app.filter_view.filter_mode {
        FilterViewMode::TagFilter(ui_filter, _) => {
            let (list, title) = match ui_filter {
                UiTagFilter::Single(v) => (
                    List::new(v.iter().map(|(name, tag_state)| tag_state.as_text(&name))),
                    "Filter: Single Tag",
                ),
                UiTagFilter::Multi(v) => (
                    List::new(v.iter().map(|(name, tag_state)| tag_state.as_text(&name))),
                    "Filter: Multi Tag",
                ),
            };
            let list = list
                .block(Block::bordered().title(title))
                .highlight_symbol("|");
            f.render_stateful_widget(list, area, &mut app.filter_view.state);
        }
        FilterViewMode::Columns => {
            let list = List::new(
                UiColumn::all(&app.config.filter, &app.config.sort)
                    .into_iter()
                    .map(Into::<Text>::into),
            );
            let list = list
                .block(Block::bordered().title("Filter"))
                .highlight_symbol("|");
            f.render_stateful_widget(list, area, &mut app.filter_view.state);
        }
        FilterViewMode::TextFilter(input, column) => {
            let block = Block::bordered().title(format!("Text Filter: {column:?}"));
            let text = format!("search: {}", input.value());
            f.render_widget(Paragraph::new(text).block(block), area);
        }
    };
}

#[derive(Clone)]
pub enum UiTagFilter {
    Multi(Vec<(String, MultiTagState)>),
    Single(Vec<(String, TagState)>),
}
impl UiTagFilter {
    pub fn next_state(&mut self, index: usize) {
        match self {
            UiTagFilter::Single(v) => {
                v.index_mut(index).1.next();
            }
            UiTagFilter::Multi(v) => {
                v.index_mut(index).1.next();
            }
        }
    }
    pub fn and_state(&mut self, index: usize) {
        if let UiTagFilter::Multi(v) = self {
            v.index_mut(index).1 = MultiTagState::And;
        }
    }
    pub fn or_state(&mut self, index: usize) {
        match self {
            UiTagFilter::Single(v) => {
                v.index_mut(index).1 = TagState::Or;
            }
            UiTagFilter::Multi(v) => {
                v.index_mut(index).1 = MultiTagState::Or;
            }
        }
    }
    pub fn not_state(&mut self, index: usize) {
        match self {
            UiTagFilter::Single(v) => {
                v.index_mut(index).1 = TagState::Not;
            }
            UiTagFilter::Multi(v) => {
                v.index_mut(index).1 = MultiTagState::Not;
            }
        }
    }
    pub fn nil_state(&mut self, index: usize) {
        match self {
            UiTagFilter::Single(v) => {
                v.index_mut(index).1 = TagState::Nil;
            }
            UiTagFilter::Multi(v) => {
                v.index_mut(index).1 = MultiTagState::Nil;
            }
        }
    }
    pub fn from_column(c: Column, tf: &TaskFilter, uniques: &[String]) -> Self {
        use Column as C;
        match c {
            C::Labels => Self::from_multi_tag_filter(&tf.labels, uniques),
            C::Bucket => Self::from_tag_filter(&tf.bucket, uniques),
            C::Priority => Self::from_tag_filter(&tf.priority, uniques),
            C::Progress => Self::from_tag_filter(&tf.progress.clone(), uniques),
            C::AssignedTo => Self::from_multi_tag_filter(&tf.assigned_to.clone(), uniques),
            _ => todo!(),
        }
    }
    fn from_multi_tag_filter(tf: &MultiTagFilter, uniques: &[String]) -> Self {
        let filter = uniques
            .iter()
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
    fn from_tag_filter<T>(tf: &TagFilter<T>, uniques: &[String]) -> Self
    where
        T: FromStr + PartialEq,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let filter = uniques
            .iter()
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
    pub fn as_text(&self, value: &str) -> Text<'static> {
        use TagState as M;
        let symbol = match self {
            M::Or => "+",
            M::Not => "~",
            M::Nil => " ",
        };
        Text::from(format!("{symbol} {}", value))
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
    pub fn as_text(&self, value: &str) -> Text<'static> {
        use MultiTagState as M;
        let symbol = match self {
            M::Or => "+",
            M::And => "*",
            M::Not => "~",
            M::Nil => " ",
        };
        Text::from(format!("{symbol} {}", value))
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
#[derive(Clone, Copy)]
pub enum SortType {
    Sorted(Order),
    Unsorted,
    Nil,
}
impl SortType {
    pub fn new(c: Column, ts: &TaskSort) -> Self {
        use Column as C;
        match (c, ts.column == c) {
            (C::AssignedTo, _) => Self::Nil,
            (C::Labels, _) => Self::Nil,
            (_, false) => Self::Unsorted,
            (_, true) => Self::Sorted(ts.order),
        }
    }
}
#[derive(Clone, Copy)]
pub enum FilterType {
    Tag(bool),
    Text(bool),
    Nil,
}
impl FilterType {
    pub fn new(c: Column, tf: &TaskFilter) -> Self {
        use Column as C;
        match c {
            C::Labels => Self::Tag(tf.labels.has_filter()),
            C::Bucket => Self::Tag(tf.bucket.has_filter()),
            C::AssignedTo => Self::Tag(tf.assigned_to.has_filter()),
            C::Progress => Self::Tag(tf.progress.has_filter()),
            C::Priority => Self::Tag(tf.priority.has_filter()),
            C::Name => Self::Text(!tf.name.is_empty()),
            C::Description => Self::Text(!tf.description.is_empty()),
            C::Deadline => Self::Nil,
            C::CreateDate => Self::Nil,
            C::StartDate => Self::Nil,
            C::CompleteDate => Self::Nil,
        }
    }
}
#[derive(Clone)]
pub struct UiColumn {
    pub sort: SortType,
    pub filtered: FilterType,
    pub column: Column,
}
impl UiColumn {
    pub fn all(tf: &TaskFilter, ts: &TaskSort) -> Vec<UiColumn> {
        use Column as C;
        vec![
            C::Bucket,
            C::Progress,
            C::Priority,
            C::Labels,
            C::AssignedTo,
            C::Name,
            C::Deadline,
            C::CreateDate,
            C::StartDate,
            C::CompleteDate,
            C::Description,
        ]
        .into_iter()
        .map(|c| UiColumn {
            sort: SortType::new(c, ts),
            filtered: FilterType::new(c, tf),
            column: c,
        })
        .collect()
    }
}
impl From<UiColumn> for Text<'static> {
    fn from(value: UiColumn) -> Self {
        let sort = match value.sort {
            SortType::Sorted(Order::Asc) => Span::from("[↑]").green(),
            SortType::Sorted(Order::Desc) => Span::from("[↓]").red(),
            SortType::Unsorted => Span::from("[ ]"),
            _ => Span::from("   "),
        };
        let mut filtered = false;
        use FilterType as FT;
        let filter = match value.filtered {
            FT::Tag(true) | FT::Text(true) => {
                filtered = true;
                Span::from("[*]")
            }
            FT::Tag(false) | FT::Text(false) => Span::from("[ ]"),
            FT::Nil => Span::from("   "),
        };

        let mut text = Span::from(format!("{:?}", value.column));
        if filtered {
            text = text.add_modifier(Modifier::BOLD);
        }
        let span = sort + " ".into() + filter + Span::from(" ") + text;
        span.into()
    }
}

impl AsText for Priority {
    fn as_text(&self) -> Text<'static> {
        match self {
            Self::Low => Text::from(" Ⅰ ").fg(Color::Blue),
            Self::Mid => Text::from(" Ⅱ "),
            Self::Important => Text::from(" Ⅲ ").fg(Color::Yellow),
            Self::Urgent => Text::from(" Ⅳ ").fg(Color::Red),
        }
    }
}
impl AsText for Progress {
    fn as_text(&self) -> Text<'static> {
        match self {
            Self::Done => Text::from("[✓]"),
            Self::Ongoing => Text::from("[-]"),
            Self::NotStarted => Text::from("[ ]"),
        }
    }
}
