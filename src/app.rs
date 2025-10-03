use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    widgets::TableState,
};

use crate::{
    config::{self, Config},
    ms_planner::{Plan, Priority, Task},
    ui,
};
type AnyResult<T> = anyhow::Result<T>;

pub struct App {
    pub plan: Plan,
    pub config: Config,
    pub table_state: TableState,
    pub displayed_tasks: Vec<Task>,
    pub error_popup: Option<String>,
}

impl App {
    pub fn new(plan: Plan, config: Config) -> Self {
        let mut app = App {
            plan,
            config,
            displayed_tasks: vec![],
            error_popup: None,
            table_state: TableState::new().with_selected(0),
        };
        app.set_filterd_tasks();
        app
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) -> AnyResult<()> {
        loop {
            terminal.draw(|frame| ui::view(&mut self, frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('e') => self.error_popup = None,
                    KeyCode::Char('j') => self.table_state.select_next(),
                    KeyCode::Char('k') => self.table_state.select_previous(),
                    _ => (),
                }
            }
        }
        Ok(())
    }
    pub fn set_filterd_tasks(&mut self) {
        let mut filtered_tasks = filter_tasks(&self.config, &self.plan.tasks);
        sort_tasks(&self.config, &mut filtered_tasks);
        // filtered_tasks = filtered_tasks.into_iter().take(3).collect();
        self.displayed_tasks = filtered_tasks;
        self.error_popup = Some(format!(
            "all tasks{}\ntasks found: {}",
            self.plan.tasks.len(),
            self.displayed_tasks.len()
        ))
    }
}

fn filter_tasks(config: &Config, tasks: &[Task]) -> Vec<Task> {
    let tasks = tasks.iter();
    let tasks = tasks.filter(|task| config.filter.bucket.filter(&task.bucket));
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
fn sort_tasks(config: &Config, tasks: &mut [Task]) {
    use config::SortColumn as SC;
    match config.sort.column {
        SC::None => (),
        SC::Name => tasks.sort_by_key(|task| task.name.clone()),
        SC::Deadline => tasks.sort_by_key(|task| task.deadline),
        SC::CreateDate => tasks.sort_by_key(|task| task.create_date),
        SC::StartDate => tasks.sort_by_key(|task| task.start_date),
        SC::CompleteDate => tasks.sort_by_key(|task| task.complete_date),
        SC::Priority => tasks.sort_by_key(|task| task.priority.clone()),
        SC::Progress => tasks.sort_by_key(|task| task.progress.clone()),
    }
    if matches!(config.sort.order, config::Order::Asc) {
        tasks.reverse();
    }
}
