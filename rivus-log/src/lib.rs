//! # Rivus Log
//! 
//! 一个基于 `tracing` 生态系统的灵活可配置日志库。
//! 
//! 该库提供了一个简单的接口来配置日志输出到控制台和文件目标，
//! 支持自定义格式化和轮换选项。
//! 
//! ## 特性
//! 
//! - 支持控制台和文件日志记录
//! - 可配置的日志级别
//! - 文件输出的自动日志轮换
//! - 配置的 JSON 序列化支持
//! - 非阻塞文件 I/O 以提高性能
//! 
//! ## 示例
//! 
//! ```rust
//! use rivus_log::{LogOptions, LogFile, init};
//! 
//! let options = LogOptions {
//!     level: "info".to_string(),
//!     output: "console,file".to_string(),
//!     file: Some(LogFile {
//!         path: "/var/log/myapp".to_string(),
//!         prefix: "application".to_string(),
//!         max_size: Some(10 * 1024 * 1024), // 10MB
//!         max_age: Some(7), // 7 天
//!     }),
//! };
//! 
//! init(options);
//! 
//! // 现在可以使用 tracing 宏
//! tracing::info!("应用程序已启动");
//! tracing::error!("出现错误");
//! ```

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
pub use tracing;
use std::io::stdout;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::{DefaultFields, Format, Full};
use tracing_subscriber::fmt::Layer;

const TIMER_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";
static LOG_GUARD: OnceLock<Vec<tracing_appender::non_blocking::WorkerGuard>> = OnceLock::new();

/// 文件日志配置选项。
/// 
/// 定义基于文件的日志记录设置，包括路径、文件前缀以及可选的
/// 大小和年龄限制用于日志轮换。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFile {
    /// 存储日志文件的目录路径
    pub path: String,
    /// 日志文件名的前缀
    pub prefix: String,
    /// 轮换前的最大文件大小（字节，可选）
    pub max_size: Option<usize>,
    /// 日志轮换前的最大天数（可选）
    pub max_age: Option<usize>,
}

/// 初始化跟踪子系统的日志配置选项。
/// 
/// 该结构体定义了设置日志的配置参数，包括日志级别、输出目标
/// 和文件日志设置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogOptions {
    /// 日志级别过滤器（例如："debug"、"info"、"warn"、"error"）
    pub level: String,
    /// 逗号分隔的输出目标列表（"console"、"file" 或 "console,file"）
    pub output: String,
    /// 可选的文件日志配置
    pub file: Option<LogFile>,
}

/// 创建具有通用格式化选项的基础跟踪层。
/// 
/// 该函数设置一个标准化层，包含：
/// - 使用 ChronoLocal 的自定义时间戳格式
/// - 启用目标和级别信息
/// - 日志消息的完整格式化
fn create_base_layer<S>() -> Layer<S, DefaultFields, Format<Full, ChronoLocal>> {
    let timer = ChronoLocal::new(TIMER_FORMAT.into());
    fmt::layer()
        .with_timer(timer)
        .with_target(true)
        .with_level(true)
}

/// 使用指定的日志选项初始化全局跟踪订阅器。
/// 
/// 该函数根据提供的配置设置日志记录，支持多个输出目标
/// （控制台和/或文件）以及适当的格式化。
/// 
/// # 参数
/// 
/// * `log` - 日志配置选项
/// 
/// # 示例
/// 
/// ```rust
/// use rivus_log::{LogOptions, LogFile, init};
/// 
/// let options = LogOptions {
///     level: "info".to_string(),
///     output: "console,file".to_string(),
///     file: Some(LogFile {
///         path: "/var/log".to_string(),
///         prefix: "myapp".to_string(),
///         max_size: Some(10 * 1024 * 1024), // 10MB
///         max_age: Some(7), // 7 天
///     }),
/// };
/// 
/// init(options);
/// ```
pub fn init(log: LogOptions) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&log.level));
    let registry = Registry::default().with(filter);

    // 解析输出目标并收集层
    let outputs: Vec<&str> = log.output.split(',').map(|s| s.trim()).collect();
    let mut layers = Vec::new();
    let mut guards: Vec<WorkerGuard> = Vec::new();

    for output_target in outputs {
        match output_target {
            "console" => {
                let console_layer = create_base_layer()
                    .with_writer(stdout)
                    .boxed();
                layers.push(console_layer);
            }
            "file" => {
                if let Some(file_config) = &log.file {
                    let file_appender = rolling::daily(&file_config.path, &file_config.prefix);
                    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
                    guards.push(guard);

                    let file_layer = create_base_layer()
                        .with_writer(file_writer)
                        .with_ansi(false)
                        .boxed();
                    layers.push(file_layer);
                } else {
                    eprintln!("[警告] 指定了日志输出目标 'file'，但未找到文件配置。");
                }
            }
            unknown_target => {
                eprintln!("[警告] 未知的日志输出目标: '{}'", unknown_target);
            }
        }
    }

    // 初始化订阅器
    if !layers.is_empty() {
        let subscriber = registry.with(layers);
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("[错误] 设置全局默认订阅器失败: {}", e);
            return;
        }

        // 存储 guards 以防止过早释放
        if !guards.is_empty() {
            if LOG_GUARD.set(guards).is_err() {
                eprintln!("[错误] 无法设置 LOG_GUARD - 日志可能无法正常工作。");
            }
        }
    } else {
        // 如果没有配置有效输出，回退到控制台
        eprintln!("[错误] 未配置有效的日志输出。默认使用控制台。");
        let default_layer = create_base_layer().with_writer(stdout);
        let subscriber = registry.with(default_layer);
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("[错误] 设置回退控制台订阅器失败: {}", e);
        }
    }
}
