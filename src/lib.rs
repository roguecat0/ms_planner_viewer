pub mod app;
pub mod config;
pub mod lang;
pub mod ms_planner;
pub mod ui;

pub type AnyResult<T> = anyhow::Result<T>;
pub type SimpleResult<T> = Result<T, SimpleError>;
pub use common::SimpleError;
pub use config::Column;

pub const PLAN_PATH: &str = "./resources/plan.xlsx";
pub const CONFIG_PATH: &str = "./resources/config.toml";

pub mod common {
    #[derive(thiserror::Error, Debug)]
    #[error("{0}")]
    pub struct SimpleError(pub String);

    impl From<&str> for SimpleError {
        fn from(s: &str) -> Self {
            Self(s.to_owned())
        }
    }

    impl From<String> for SimpleError {
        fn from(s: String) -> Self {
            Self(s)
        }
    }
}
