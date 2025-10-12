use std::ops::IndexMut;

use crate::{
    AnyResult, Column,
    config::{self, Config, Order, UniqueTaskKeys},
    filter::{UiColumn, UiTagFilter},
    ms_planner::{Plan, Priority, Progress, Task},
    ui,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    widgets::{ListState, TableState},
};

pub struct App {
    pub plan: Plan,
    pub config: Config,
    pub table_state: TableState,
    pub displayed_tasks: Vec<Task>,
    pub error_popup: Option<String>,
    pub input_mode: InputMode,
    pub filter_view: FilterView,
    pub selected_task: Option<usize>,
}
pub struct FilterView {
    pub state: ListState,
    pub unique_task_keys: UniqueTaskKeys,
    pub filter_mode: FilterViewMode,
}
pub enum FilterViewMode {
    TagFilter(UiTagFilter, Column),
    Columns,
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
            input_mode: InputMode::TableRow,
            filter_view: FilterView {
                unique_task_keys,
                state: ListState::default().with_selected(Some(0)),
                filter_mode: FilterViewMode::Columns,
            },
            selected_task: None,
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
                    if let KeyCode::Esc = key.code {
                        self.error_popup = None
                    }
                    continue;
                } else if let KeyCode::Char('S') = key.code {
                    self.config.to_file(crate::CONFIG_PATH)?;
                    self.add_error_msg("config saved");
                } else if let KeyCode::Char('R') = key.code {
                    self.plan = crate::ms_planner::get_plan(crate::PLAN_PATH)?;
                    // self.selected_task = None;
                    // self.filter_view.ui_tag_filter = None;
                    self.add_error_msg("plan reloaded");
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
        match (key.code, self.selected_task) {
            (KeyCode::Char('j'), None) => self.table_state.select_next(),
            (KeyCode::Char('k'), None) => self.table_state.select_previous(),
            (KeyCode::Char('f'), None) => {
                self.input_mode = InputMode::FilterMode;
            }
            (KeyCode::Char(' '), None) => {
                if let Some(i) = self.table_state.selected() {
                    self.selected_task = Some(i)
                }
            }
            (KeyCode::Char('L'), Some(i)) => {
                let url = &self.displayed_tasks[i].to_url(&self.plan.id);
                webbrowser::open(url)?;
            }
            (KeyCode::Esc, Some(_)) => self.selected_task = None,
            _ => (),
        }
        Ok(())
    }
    pub fn run_filter_mode(&mut self, key: KeyEvent) -> AnyResult<()> {
        match self.filter_view.filter_mode {
            FilterViewMode::Columns => {
                let selected = self.filter_view.state.selected();
                let ui_col = selected.and_then(|i| {
                    Some(config::get_ui_columns(&self.config.filter, &self.config.sort)[i].clone())
                });
                self.run_columns_filter(key, ui_col)?
            }
            FilterViewMode::TagFilter(ref ui_tag_filter, c) => {
                self.run_tag_filter(key, ui_tag_filter.clone(), c)?
            }
        }
        Ok(())
    }
    pub fn run_tag_filter(
        &mut self,
        key: KeyEvent,
        mut ui_tag_filter: UiTagFilter,
        c: Column,
    ) -> AnyResult<()> {
        match (key.code, self.filter_view.state.selected()) {
            (KeyCode::Char('j'), _) => self.filter_view.state.select_next(),
            (KeyCode::Char('k'), _) => self.filter_view.state.select_previous(),
            (KeyCode::Esc, _) => {
                self.filter_view.filter_mode = FilterViewMode::Columns;
                self.filter_view.state.select_first();
            }
            (KeyCode::Char(' '), Some(i)) => {
                ui_tag_filter.next_state(i);
                self.update_task_filter(c, &ui_tag_filter)?;
            }
            _ => (),
        }
        Ok(())
    }
    fn update_task_filter(&mut self, c: Column, ui_tag_filter: &UiTagFilter) -> AnyResult<()> {
        match c {
            Column::Labels => self.config.filter.labels = ui_tag_filter.clone().try_into()?,
            Column::Bucket => self.config.filter.bucket = ui_tag_filter.clone().try_into()?,
            Column::AssignedTo => {
                self.config.filter.assigned_to = ui_tag_filter.clone().try_into()?
            }
            Column::Progress => self.config.filter.progress = ui_tag_filter.clone().try_into()?,
            Column::Priority => self.config.filter.priority = ui_tag_filter.clone().try_into()?,
            _ => todo!(),
        };
        Ok(())
    }
    pub fn run_columns_filter(
        &mut self,
        key: KeyEvent,
        ui_column: Option<UiColumn>,
    ) -> AnyResult<()> {
        match (key.code, ui_column) {
            (KeyCode::Char('j'), _) => self.filter_view.state.select_next(),
            (KeyCode::Char('k'), _) => self.filter_view.state.select_previous(),
            (KeyCode::Esc, _) => {
                self.input_mode = InputMode::TableRow;
                self.filter_view.state.select_first();
            }
            (KeyCode::Char('s'), Some(ui_col)) => {
                if let Some(sort) = ui_col.sort {
                    self.config.sort.order = match sort {
                        Order::Desc => Order::Asc,
                        Order::Asc => Order::Desc,
                    }
                } else {
                    self.config.sort.column = ui_col.column;
                }
            }
            (KeyCode::Char(' '), Some(ui_col)) => {
                let uniques = match ui_col.column {
                    Column::Labels => &self.filter_view.unique_task_keys.labels,
                    Column::Bucket => &self.filter_view.unique_task_keys.buckets,
                    Column::AssignedTo => &self.filter_view.unique_task_keys.people,
                    Column::Progress => {
                        &Progress::items().iter().map(ToString::to_string).collect()
                    }
                    Column::Priority => {
                        &Priority::items().iter().map(ToString::to_string).collect()
                    }
                    _ => todo!(),
                };
                self.filter_view.filter_mode = FilterViewMode::TagFilter(
                    self.config.filter.get_ui_filter(ui_col.column, uniques),
                    ui_col.column,
                );
                self.filter_view.state.select_first();
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
    use Column as C;
    match config.sort.column {
        C::Name => tasks.sort_by_key(|task| task.name.clone()),
        C::Deadline => tasks.sort_by_key(|task| task.deadline),
        C::CreateDate => tasks.sort_by_key(|task| task.create_date),
        C::StartDate => tasks.sort_by_key(|task| task.start_date),
        C::CompleteDate => tasks.sort_by_key(|task| task.complete_date),
        C::Priority => tasks.sort_by_key(|task| task.priority.clone()),
        C::Progress => tasks.sort_by_key(|task| task.progress.clone()),
        _ => todo!(),
    }
    if matches!(config.sort.order, config::Order::Asc) {
        tasks.reverse();
    }
}
