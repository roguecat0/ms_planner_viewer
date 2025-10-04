use std::ops::IndexMut;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    widgets::{ListState, TableState},
};

use crate::{
    AnyResult,
    config::{self, Config, UniqueTaskKeys},
    ms_planner::{Plan, Task},
    ui::{self, UiColumn, UiTagFilter},
};

pub struct App {
    pub plan: Plan,
    pub config: Config,
    pub table_state: TableState,
    pub displayed_tasks: Vec<Task>,
    pub error_popup: Option<String>,
    pub input_mode: InputMode,
    pub filter_view: FilterView,
}
pub struct FilterView {
    pub state: ListState,
    pub unique_task_keys: UniqueTaskKeys,
    pub ui_tag_filter: Option<(UiTagFilter, UiColumn)>,
}
pub enum InputMode {
    TableRow,
    FilterMode,
}

impl App {
    pub fn new(plan: Plan, config: Config) -> Self {
        let buckets = plan.tasks.iter().map(|t| &t.bucket);
        let labels = plan.tasks.iter().map(|t| &t.labels).flatten();
        let people = plan.tasks.iter().map(|t| &t.assigned_to).flatten();
        let unique_task_keys = UniqueTaskKeys {
            buckets: config::get_unique_strings(buckets),
            labels: config::get_unique_strings(labels),
            people: config::get_unique_strings(people),
        };
        let mut app = App {
            plan,
            config,
            displayed_tasks: vec![],
            error_popup: None,
            table_state: TableState::new().with_selected(0),
            input_mode: InputMode::FilterMode,
            filter_view: FilterView {
                unique_task_keys,
                state: ListState::default().with_selected(Some(0)),
                ui_tag_filter: None,
            },
        };
        app.set_filterd_tasks();
        app
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) -> AnyResult<()> {
        loop {
            terminal.draw(|frame| ui::view(&mut self, frame))?;
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                } else if let Some(_) = self.error_popup {
                    if let KeyCode::Char('e') = key.code {
                        self.error_popup = None
                    }
                    continue;
                }
                match &self.input_mode {
                    InputMode::TableRow => self.run_table_row_mode(key),
                    InputMode::FilterMode => self.run_filter_mode(key),
                }?;
                self.set_filterd_tasks();
            }
        }
        Ok(())
    }
    pub fn run_table_row_mode(&mut self, key: KeyEvent) -> AnyResult<()> {
        match key.code {
            KeyCode::Char('j') => self.table_state.select_next(),
            KeyCode::Char('k') => self.table_state.select_previous(),
            KeyCode::Char('f') => {
                self.input_mode = InputMode::FilterMode;
                // self.table_state.select_first();
            }
            _ => (),
        }
        Ok(())
    }
    pub fn run_filter_mode(&mut self, key: KeyEvent) -> AnyResult<()> {
        match key.code {
            KeyCode::Char('j') => self.filter_view.state.select_next(),
            KeyCode::Char('k') => self.filter_view.state.select_previous(),
            KeyCode::Esc => {
                if let Some(_) = self.filter_view.ui_tag_filter {
                    self.filter_view.ui_tag_filter = None;
                } else {
                    self.input_mode = InputMode::TableRow;
                }
                self.filter_view.state.select_first();
            }
            KeyCode::Char(' ') => {
                if let Some(i) = self.filter_view.state.selected() {
                    if let Some((filter_tags, column)) = &mut self.filter_view.ui_tag_filter {
                        match filter_tags {
                            UiTagFilter::Single(v) => {
                                let (_, state) = v.index_mut(i);
                                state.next();
                            }
                            UiTagFilter::Multi(v) => {
                                let (_, state) = v.index_mut(i);
                                state.next();
                            }
                        }
                        match column {
                            UiColumn::Labels => {
                                self.config.filter.labels = filter_tags.clone().try_into()?
                            }
                            UiColumn::Bucket => {
                                self.config.filter.bucket = filter_tags.clone().try_into()?
                            }
                            UiColumn::AssignedTo => {
                                self.config.filter.assigned_to = filter_tags.clone().try_into()?
                            }
                            _ => todo!(),
                        };
                    } else {
                        let column = &self.config.filter.get_ui_columns()[i];
                        let uniques = match column {
                            UiColumn::Labels => &self.filter_view.unique_task_keys.labels,
                            UiColumn::Bucket => &self.filter_view.unique_task_keys.buckets,
                            UiColumn::AssignedTo => &self.filter_view.unique_task_keys.people,
                            _ => todo!(),
                        };
                        self.filter_view.ui_tag_filter = Some((
                            self.config.filter.get_ui_filter(column, uniques),
                            column.clone(),
                        ));
                        self.filter_view.state.select_first();
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub fn set_filterd_tasks(&mut self) {
        let mut filtered_tasks = filter_tasks(&self.config, &self.plan.tasks);
        sort_tasks(&self.config, &mut filtered_tasks);
        // filtered_tasks = filtered_tasks.into_iter().take(3).collect();
        self.displayed_tasks = filtered_tasks;
        // self.add_error_msg(&format!(
        //     "all tasks{}\ntasks found: {}",
        //     self.plan.tasks.len(),
        //     self.displayed_tasks.len()
        // ));
    }
    pub fn add_error_msg(&mut self, s: &str) {
        let text = if let Some(text) = &self.error_popup {
            text.to_owned() + "\n" + s
        } else {
            s.to_string()
        };
        self.error_popup = Some(text);
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
