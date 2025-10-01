use calamine::{self, DataType, open_workbook_auto};
use calamine::{Data, Reader};
use chrono::NaiveDate;
use std::path::Path;

const DATA_FMT: &str = "%d-%m-%Y";

#[derive(Default, Debug, Clone)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub export_date: NaiveDate,
    pub tasks: Vec<Task>,
}

#[derive(Default, Debug, Clone)]
pub enum Progress {
    #[default]
    NotStarted,
    Ongoing,
    Done,
}
#[derive(Default, Debug, Clone)]
pub enum Priority {
    Urgent,
    Important,
    Medium,
    #[default]
    Low,
}

type AnyResult<T> = anyhow::Result<T>;

#[derive(Default, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub bucket: String,
    pub progress: String,
    pub priority: String,
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

impl Task {
    pub fn parse(data: &[Data]) -> AnyResult<Self> {
        let str_data: Result<Vec<String>, usize> = data
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                cell.as_string()
                    .or(cell.is_empty().then_some(String::new()))
                    .ok_or(i)
            })
            .collect();
        let str_data: Vec<String> =
            str_data.map_err(|i| anyhow::anyhow!("{:?} is not string", data[i]))?;
        let id = str_data[0].clone();
        let name = str_data[1].clone();
        let bucket = str_data[2].clone();
        let progress = str_data[3].clone();
        let priority = str_data[4].clone();
        let assigned_to = to_string_list(&str_data[5]);
        let created_by = str_data[6].clone();
        let create_date = NaiveDate::parse_from_str(&str_data[7], DATA_FMT)?;
        let start_date = to_option_date(&str_data[8])?;
        let deadline = to_option_date(&str_data[9])?;
        let recurring = (str_data[10] != "false").then_some(str_data[10].clone());
        let late = str_data[11] == "true";
        let complete_date = to_option_date(&str_data[12])?;
        let completed_by = str_data[13].clone();
        let items_completed = to_usizes(&str_data[14]);
        let items: Vec<String> = to_string_list(&str_data[15]);
        let labels: Vec<String> = to_string_list(&str_data[16]);
        let description = str_data[17].clone();
        Ok(Task {
            id,
            name,
            bucket,
            progress,
            priority,
            assigned_to,
            created_by,
            create_date,
            start_date,
            deadline,
            recurring,
            late,
            complete_date,
            completed_by,
            items_completed,
            items,
            labels,
            description,
        })
    }
    pub fn to_url(&self, plan_id: &str) -> String {
        format!(
            "https://planner.cloud.microsoft/webui/plan/{plan_id}/view/grid/task/{}",
            self.id
        )
    }
}
fn to_usizes(slice: &str) -> Option<(usize, usize)> {
    if slice.is_empty() {
        None
    } else {
        let mut iter = slice.split('/');
        let a: usize = iter.next().unwrap().parse().unwrap();
        let b: usize = iter.next().unwrap().parse().unwrap();
        Some((a, b))
    }
}
#[inline]
fn to_string_list(slice: &str) -> Vec<String> {
    if slice.is_empty() {
        Vec::new()
    } else {
        slice.split(';').map(str::to_string).collect()
    }
}
fn to_option_date(slice: &str) -> AnyResult<Option<NaiveDate>> {
    if !slice.is_empty() {
        Ok(Some(NaiveDate::parse_from_str(&slice, DATA_FMT)?))
    } else {
        Ok(None)
    }
}
pub fn get_plan(path: impl AsRef<Path>) -> AnyResult<Plan> {
    let mut workbook = open_workbook_auto(path).expect("lol");
    let info = workbook.worksheet_range("Plannaam ").unwrap();
    let range = workbook.worksheet_range("Taken").unwrap();
    let mut info = info.rows();
    let name = info.next().unwrap()[1].as_string().unwrap();
    let id = info.next().unwrap()[1].as_string().unwrap();
    let export_date = info.next().unwrap()[1].as_string().unwrap();
    let export_date = NaiveDate::parse_from_str(&export_date, DATA_FMT)?;
    let mut tasks: Vec<Task> = Vec::new();
    for (i, data) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        if i > 2 {
            break;
        }
        tasks.push(Task::parse(data)?);
    }
    dbg!(&tasks);
    Ok(Plan {
        id,
        name,
        export_date,
        tasks: tasks,
    })
}
