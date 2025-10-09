use ratatui::{
    Frame,
    text::Text,
    widgets::{Block, List},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Stylize},
};
use std::str::FromStr;

use crate::{
    Column,
    app::App,
    config::{self, MultiTagFilter, Order, TagFilter, TaskFilter},
    ms_planner::{Priority, Progress},
    ui::AsText,
};

pub fn render_filter_list(app: &mut App, f: &mut Frame, area: Rect) {
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
pub enum FilterType {
    Tag(bool),
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
            C::Name => Self::Nil,
            C::Deadline => Self::Nil,
            C::CreateDate => Self::Nil,
            C::StartDate => Self::Nil,
            C::CompleteDate => Self::Nil,
        }
    }
}
pub struct UiColumn {
    pub sort: Option<Order>,
    pub filtered: FilterType,
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
        match value.filtered {
            FilterType::Tag(true) => Text::from(format!("[*] {s}")).add_modifier(Modifier::BOLD),
            FilterType::Tag(false) => Text::from(format!("[ ] {s}")).add_modifier(Modifier::BOLD),
            FilterType::Nil => Text::from(format!("    {s}")).add_modifier(Modifier::BOLD),
        }
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
