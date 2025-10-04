use std::path::Path;

use crate::{
    AnyResult,
    ui::{UiColumn, UiTagFilter},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::ms_planner::{Priority, Progress};
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    pub filter: TaskFilter,
    pub sort: TaskSort,
}
impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> AnyResult<Self> {
        let s = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&s)?;
        Ok(config)
    }
    pub fn to_file(&self, path: impl AsRef<Path>) -> AnyResult<()> {
        let stdout = toml::to_string_pretty(self)?;
        std::fs::write(path, &stdout)?;
        Ok(())
    }
}

pub struct UniqueTaskKeys {
    pub buckets: Vec<String>,
    pub labels: Vec<String>,
    pub people: Vec<String>,
}

pub fn get_unique_strings<'a, I>(i: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a String>,
{
    i.into_iter()
        .map(|s| s.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskFilter {
    pub name: String,
    pub bucket: TagFilter<String>,
    pub progress: TagFilter<Progress>,
    pub priority: TagFilter<Priority>,
    pub labels: MultiTagFilter,
    pub assigned_to: MultiTagFilter,
    pub created_by: TagFilter<String>,
    pub description: String,
}
impl TaskFilter {
    pub fn get_ui_filter(&self, index: &UiColumn, unique_values: &[String]) -> UiTagFilter {
        use UiColumn as C;
        match index {
            C::Labels => (self.labels.clone(), unique_values).into(),
            C::Bucket => (self.bucket.clone(), unique_values).into(),
            C::Priority => (self.priority.clone(), unique_values).into(),
            C::Progress => (self.progress.clone(), unique_values).into(),
            C::AssignedTo => (self.assigned_to.clone(), unique_values).into(),
        }
    }
    pub fn get_ui_columns(&self) -> Vec<UiColumn> {
        use UiColumn as C;
        vec![
            C::Bucket,
            // C::Progress,
            // C::Priority,
            C::Labels,
            C::AssignedTo,
        ]
    }
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct MultiTagFilter {
    pub and: Vec<String>,
    pub or: Vec<String>,
    pub not: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TagFilter<T> {
    pub or: Vec<T>,
    pub not: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskSort {
    pub column: SortColumn,
    pub order: Order,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum Order {
    Asc,
    #[default]
    Desc,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum SortColumn {
    #[default]
    None,
    Priority,
    Name,
    CreateDate,
    StartDate,
    Deadline,
    CompleteDate,
    Progress,
}
impl<T: PartialEq> TagFilter<T> {
    pub fn filter(&self, tag: &T) -> bool {
        if !self.not.is_empty() {
            if self.not.contains(tag) {
                return false;
            }
        }
        if !self.or.is_empty() {
            if !self.or.contains(tag) {
                return false;
            }
        }
        true
    }
}
impl MultiTagFilter {
    pub fn filter(&self, tags: &[String]) -> bool {
        if !self.not.is_empty() {
            if self.not.iter().any(|f| tags.contains(f)) {
                return false;
            }
        }
        if !self.or.is_empty() {
            if !self.or.iter().any(|f| tags.contains(f)) {
                return false;
            }
        }
        if !self.and.is_empty() {
            if !self.and.iter().all(|f| tags.contains(f)) {
                return false;
            }
        }
        true
    }
}
pub fn no_case_contains(pattern: &str, text: &str) -> bool {
    if !pattern.is_empty() {
        text.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        true
    }
}

// id,
// name,
// bucket,
// progress,
// priority,
// assigned_to,
// created_by,
// create_date,
// start_date,
// deadline,
// recurring,
// late,
// complete_date,
// completed_by,
// items_completed,
// items,
// labels,
// description,
