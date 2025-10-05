use ms_planner_viewer::{CONFIG_PATH, PLAN_PATH, app::App, config::Config, ms_planner};
fn main() -> anyhow::Result<()> {
    let plan = ms_planner::get_plan(PLAN_PATH)?;
    let config = if !std::fs::exists(CONFIG_PATH)? {
        let config = Config::default();
        config.to_file(CONFIG_PATH)?;
        config
    } else {
        Config::from_file(CONFIG_PATH)?
    };
    // Ok(())

    let terminal = ratatui::init();
    let app_result = App::new(plan, config).run(terminal);
    ratatui::restore();
    app_result
}
