use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

use crate::{SimpleError, lang};

#[derive(Default, Debug, Clone)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub export_date: NaiveDate,
    pub tasks: Vec<Task>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Progress {
    #[default]
    NotStarted,
    Ongoing,
    Done,
}
impl Progress {
    pub fn items() -> Vec<Self> {
        vec![Self::NotStarted, Self::Ongoing, Self::Done]
    }
}
impl FromStr for Progress {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotStarted" => Ok(Self::NotStarted),
            "Ongoing" => Ok(Self::Ongoing),
            "Done" => Ok(Self::Done),
            _ => Err(format!("not a valid progress option: {s}").into()),
        }
    }
}
impl From<&str> for Progress {
    fn from(value: &str) -> Self {
        match value {
            lang::nl::PROGRESS_NOT_STARTED => Self::NotStarted,
            lang::nl::PROGRESS_ONGOING => Self::Ongoing,
            lang::nl::PROGRESS_DONE => Self::Done,
            _ => panic!("val: {value:?}"),
        }
    }
}
impl Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Urgent,
    Important,
    Mid,
    #[default]
    Low,
}
impl Priority {
    pub fn items() -> Vec<Self> {
        vec![Self::Urgent, Self::Important, Self::Mid, Self::Low]
    }
}
impl FromStr for Priority {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Urgent" => Ok(Self::Urgent),
            "Important" => Ok(Self::Important),
            "Mid" => Ok(Self::Mid),
            "Low" => Ok(Self::Low),
            _ => Err(format!("conversion not found: {s}").into()),
        }
    }
}
impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl From<&str> for Priority {
    fn from(value: &str) -> Self {
        match value {
            lang::nl::PRIO_IMPORTANT => Self::Important,
            lang::nl::PRIO_MID => Self::Mid,
            lang::nl::PRIO_LOW => Self::Low,
            lang::nl::PRIO_URGENT => Self::Urgent,
            _ => panic!("val: {value:?}"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    #[default]
    Name,
    Priority,
    CreateDate,
    StartDate,
    Deadline,
    CompleteDate,
    Progress,
    Bucket,
    Labels,
    AssignedTo,
    Description,
}

#[derive(Default, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub bucket: String,
    pub progress: Progress,
    pub priority: Priority,
    pub assigned_to: Vec<String>,
    pub created_by: String,
    pub create_date: NaiveDate,
    pub start_date: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub recurring: Option<String>,
    pub late: bool,
    pub complete_date: Option<NaiveDate>,
    pub completed_by: String,
    pub items_completed: Option<(usize, usize)>,
    pub items: Vec<String>,
    pub labels: Vec<String>,
    pub description: String,
}
