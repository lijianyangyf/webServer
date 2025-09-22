use crate::config::Config;
use anyhow::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging(config: &Config, log_directory: &str) -> Result<WorkerGuard> {
    let file_appender = tracing_appender::rolling::daily(log_directory, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_new(&config.rust_log)?;

    let stdout_layer = fmt::layer()
        .json()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(std::io::stdout);

    let file_layer = fmt::layer()
        .json()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init()?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_logging_creates_file() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        let config = Config {
            server_address: "".to_string(),
            database_url: "".to_string(),
            rust_log: "info".to_string(),
        };

        let guard = init_logging(&config, log_dir.to_str().unwrap());
        assert!(guard.is_ok());

        // Write a log message
        tracing::info!("this is a test log");

        // Drop the guard to ensure logs are flushed
        drop(guard);

        // Check if the log file was created
        let log_files: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.to_str().unwrap().contains("app.log"))
            .collect();
        
        assert!(!log_files.is_empty(), "Log file was not created.");
    }
}
