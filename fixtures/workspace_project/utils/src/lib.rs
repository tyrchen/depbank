use chrono::{DateTime, Utc};
use log::LevelFilter;

pub fn init_logger() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    log::info!("Logger initialized");
}

pub fn get_current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
