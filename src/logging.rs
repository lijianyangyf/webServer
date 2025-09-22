use crate::config::Config;
use anyhow::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// 初始化日志系统。
///
/// 这个函数配置了 `tracing` subscriber，用于将日志输出到两个地方：
/// 1. 标准输出 (stdout)，格式为 JSON。
/// 2. 滚动日志文件，每天创建一个新文件，格式为 JSON。
///
/// # Arguments
/// * `config` - 应用的配置，主要用于获取 `RUST_LOG` 日志级别。
/// * `log_directory` - 存放日志文件的目录。
///
/// # Returns
/// 返回一个 `WorkerGuard`。这个 guard 必须在应用的整个生命周期内保持存活。
/// 当 `guard`被 drop 时，它会确保所有缓冲的日志都被刷新到文件中。
pub fn init_logging(config: &Config, log_directory: &str) -> Result<WorkerGuard> {
    // 配置滚动文件 appender，日志会写入到 `log_directory` 下，文件名格式为 `app.log.YYYY-MM-DD`
    let file_appender = tracing_appender::rolling::daily(log_directory, "app.log");
    // 使用 `non_blocking` writer 来避免日志写入操作阻塞应用主线程
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 从配置中创建 EnvFilter，用于根据 `RUST_LOG` 环境变量的值来过滤日志
    let env_filter = EnvFilter::try_new(&config.rust_log)?;

    // 配置标准输出层 (layer)
    let stdout_layer = fmt::layer()
        .json() // 使用 JSON 格式输出
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE) // 在 span 创建和关闭时记录事件
        .with_writer(std::io::stdout); // 写入到标准输出

    // 配置文件输出层 (layer)
    let file_layer = fmt::layer()
        .json() // 使用 JSON 格式输出
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE) // 在 span 创建和关闭时记录事件
        .with_writer(non_blocking); // 写入到非阻塞的文件 appender

    // 使用 `tracing_subscriber::registry` 组合多个层
    tracing_subscriber::registry()
        .with(env_filter) // 添加环境过滤器
        .with(stdout_layer) // 添加标准输出层
        .with(file_layer) // 添加文件输出层
        .try_init()?; // 初始化 subscriber 并设置为全局默认

    // 返回 guard，调用者需要负责保持它
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// 测试 `init_logging` 是否能成功创建日志文件。
    #[test]
    fn test_init_logging_creates_file() {
        // 创建一个临时目录用于测试
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // 创建一个临时的测试配置
        let config = Config {
            server_address: "".to_string(),
            database_url: "".to_string(),
            rust_log: "info".to_string(),
        };

        // 初始化日志
        let guard = init_logging(&config, log_dir.to_str().unwrap());
        assert!(guard.is_ok());

        // 写入一条测试日志
        tracing::info!("这是一条测试日志");

        // 显式地 drop guard 来确保日志被刷新到文件
        drop(guard);

        // 检查日志文件是否已创建
        let log_files: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.to_str().unwrap().contains("app.log"))
            .collect();

        assert!(!log_files.is_empty(), "日志文件未被创建。");
    }
}
