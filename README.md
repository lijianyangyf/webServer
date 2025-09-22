# webServer 后端服务

这是一个使用 Rust 编写的高性能后端服务，内置了消息队列和任务调度器功能。

## 主要功能

*   **Web 服务**: 基于 [`axum`](https://github.com/tokio-rs/axum) 框架构建的异步 Web API 服务。
*   **数据库集成**: 使用 [`sqlx`](https://github.com/launchbadge/sqlx) 与 MySQL 数据库进行高效、安全的异步交互。
*   **消息队列**: 内置一个简单的内存优先级队列 (`PriorityQueue`)，用于管理待处理的任务。
*   **任务调度器**: 一个后台任务 (`scheduler`)，定期从队列中获取任务并执行。
*   **配置管理**: 通过 `.env` 文件加载应用配置，方便在不同环境中部署。
*   **结构化日志**: 集成 [`tracing`](https://github.com/tokio-rs/tracing) 库，提供结构化、可配置的日志输出。
*   **优雅停机**: 实现 `graceful shutdown`，确保在服务关闭时能够安全地完成正在处理的请求。

## 技术栈

*   **Web 框架**: `axum`
*   **异步运行时**: `tokio`
*   **数据库 ORM**: `sqlx` (MySQL)
*   **序列化/反序列化**: `serde` / `serde_json`
*   **日志**: `tracing`
*   **配置**: `dotenvy`
*   **错误处理**: `anyhow` / `thiserror`

## 项目结构

```
src
├── main.rs          # 应用主入口，负责初始化和启动服务
├── web.rs           # 定义 Web API 路由和处理逻辑
├── db.rs            # 数据库连接池和相关操作
├── queue.rs         # 优先级消息队列的实现
├── scheduler.rs     # 后台任务调度器的实现
├── config.rs        # 应用配置加载模块
├── error.rs         # 自定义错误类型
└── logging.rs       # 日志系统初始化
```

## 如何运行

1.  **环境准备**:
    *   安装 [Rust 工具链](https://www.rust-lang.org/tools/install)。
    *   准备一个 MySQL 数据库实例。

2.  **配置**:
    在项目根目录下创建一个 `.env` 文件，并参考以下内容配置数据库连接和服务器地址：
    ```env
    DATABASE_URL="mysql://user:password@host:port/database"
    SERVER_ADDRESS="127.0.0.1:3000"
    RUST_LOG="info"
    ```

3.  **安装依赖与运行**:
    ```bash
    # 编译并运行项目
    cargo run
    ```

4.  **访问服务**:
    服务启动后，将监听在 `.env` 文件中配置的 `SERVER_ADDRESS` 地址上。