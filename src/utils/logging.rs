pub fn setup_logging() {
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();

    let level = match log_level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => {
            eprintln!(
                "Invalid LOG_LEVEL value: '{}', defaulting to 'info'",
                log_level
            );
            log::LevelFilter::Info
        }
    };

    env_logger::Builder::new().filter_level(level).init();
}
