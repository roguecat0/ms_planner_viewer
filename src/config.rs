use std::path::Path;

use crate::{
    AnyResult, Column, Priority, Progress,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskFilter {
    pub ids: Vec<String>,
    pub filter_ids: bool,
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
    pub fn reset_filter(&mut self, column: Column) {
        use Column as C;
        match column {
            C::Labels => self.labels = MultiTagFilter::default(),
            C::Bucket => self.bucket = TagFilter::default(),
            C::AssignedTo => self.assigned_to = MultiTagFilter::default(),
            C::Progress => self.progress = TagFilter::default(),
            C::Priority => self.priority = TagFilter::default(),
            C::Description => self.description = String::new(),
            C::Name => self.name = String::new(),
            C::StartDate | C::Deadline | C::CreateDate => (),
            C::CompleteDate => (),
        };
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
    pub column: Column,
    pub order: Order,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default, Copy)]
pub enum Order {
    Asc,
    #[default]
    Desc,
}
impl<T: PartialEq> TagFilter<T> {
    pub fn filter(&self, tag: &T) -> bool {
        if !self.not.is_empty() && self.not.contains(tag) {
            return false;
        }
        if !self.or.is_empty() && !self.or.contains(tag) {
            return false;
        }
        true
    }
    pub fn has_filter(&self) -> bool {
        !self.or.is_empty() || !self.not.is_empty()
    }
}
impl MultiTagFilter {
    pub fn filter(&self, tags: &[String]) -> bool {
        if !self.not.is_empty() && self.not.iter().any(|f| tags.contains(f)) {
            return false;
        }
        if !self.or.is_empty() && !self.or.iter().any(|f| tags.contains(f)) {
            return false;
        }
        if !self.and.is_empty() && !self.and.iter().all(|f| tags.contains(f)) {
            return false;
        }
        true
    }
    pub fn has_filter(&self) -> bool {
        !self.or.is_empty() || !self.not.is_empty() || !self.and.is_empty()
    }
}
pub fn no_case_contains(pattern: &str, text: &str) -> bool {
    if !pattern.is_empty() {
        text.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        true
    }
}
