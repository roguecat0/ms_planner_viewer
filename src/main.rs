const PLAN_PATH: &str = "./resources/plan.xlsx";
const CONFIG_PATH: &str = "./resources/config.toml";

use ms_planner_viewer::{
    config::{self, Config},
    ms_planner,
};

fn main() -> anyhow::Result<()> {
    let plan = ms_planner::get_plan(PLAN_PATH)?;
    let config = if !std::fs::exists(CONFIG_PATH)? {
        let config = Config::default();
        config.to_file(CONFIG_PATH)?;
        config
    } else {
        Config::from_file(CONFIG_PATH)?
    };
    println!("url: {}", plan.tasks[0].to_url(&plan.id));
    let mut filtered_tasks = filter_tasks(&config, &plan.tasks);
    sort_tasks(&config, &mut filtered_tasks);
    println!(" ========= filtered ===================");
    if filtered_tasks.len() < 5 {
        dbg!(&filtered_tasks);
    }
    println!("tasks: len {}", plan.tasks.len());
    println!("filtered_tasks: len {}", filtered_tasks.len());
    Ok(())
}

fn filter_tasks(config: &Config, tasks: &[ms_planner::Task]) -> Vec<ms_planner::Task> {
    let tasks = tasks.iter();
    let tasks = tasks.filter(|task| config.filter.priority.filter(&task.priority));
    let tasks = tasks.filter(|task| config.filter.progress.filter(&task.progress));
    let tasks = tasks.filter(|task| config.filter.labels.filter(&task.labels));
    let tasks = tasks.filter(|task| config.filter.assigned_to.filter(&task.assigned_to));
    let tasks = tasks.filter(|task| config.filter.assigned_to.filter(&task.assigned_to));
    let tasks = tasks.filter(|task| config::no_case_contains(&config.filter.name, &task.name));
    let tasks = tasks
        .filter(|task| config::no_case_contains(&config.filter.description, &task.description));
    let tasks = tasks.cloned().collect();
    tasks
}
fn sort_tasks(config: &Config, tasks: &mut [ms_planner::Task]) {
    use config::SortColumn as SC;
    match config.sort.column {
        SC::None => (),
        SC::Name => tasks.sort_by_key(|task| task.name.clone()),
        SC::Deadline => tasks.sort_by_key(|task| task.deadline),
        SC::CreateDate => tasks.sort_by_key(|task| task.create_date),
        SC::StartDate => tasks.sort_by_key(|task| task.start_date),
        SC::CompleteDate => tasks.sort_by_key(|task| task.complete_date),
        SC::Priority => tasks.sort_by_key(|task| Priority::from(task.priority.as_str())),
        // _ => todo!(),
    }
    if matches!(config.sort.order, config::Order::Asc) {
        tasks.reverse();
    }
}
#[derive(PartialEq, PartialOrd, Ord, Eq)]
enum Priority {
    Urgent,
    Important,
    Mid,
    Low,
}
impl From<&str> for Priority {
    fn from(value: &str) -> Self {
        match value {
            "Belangrijk" => Self::Important,
            "Gemiddeld" => Self::Mid,
            "Laag" => Self::Low,
            "Urgent" => Self::Urgent,
            _ => todo!("val: {value:?}"),
        }
    }
}
