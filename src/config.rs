use std::path::Path;

use serde::{Deserialize, Serialize};
type AnyResult<T> = anyhow::Result<T>;
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    filter: TaskFilter,
    sort: TaskSort,
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
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskFilter {
    name: String,
    bucket: TagFilter,
    progress: TagFilter,
    priority: TagFilter,
    assigned_to: TagFilter,
    created_by: TagFilter,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TagFilter {
    and: Vec<String>,
    or: Vec<String>,
    not: Vec<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskSort {
    column: SortColumn,
    order: Order,
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
