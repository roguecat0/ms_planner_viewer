const PLAN_PATH: &str = "./resources/plan.xlsx";
const CONFIG_PATH: &str = "./resources/config.toml";
use ms_planner_viewer::{config::Config, ms_planner};

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let plan = ms_planner::get_plan(PLAN_PATH)?;
    let config = if !std::fs::exists(CONFIG_PATH)? {
        let config = Config::default();
        config.to_file(CONFIG_PATH)?;
        config
    } else {
        Config::from_file(CONFIG_PATH)?
    };
    dbg!(&plan);
    dbg!(&config);
    Ok(())
}
