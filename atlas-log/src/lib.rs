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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFile {
    pub path: String,
    pub prefix: String,
    pub max_size: Option<usize>,
    pub max_age: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub output: String,
    pub file: Option<LogFile>,
}

fn create_base_layer<S>() -> Layer<S, DefaultFields, Format<Full, ChronoLocal>> {
    let timer = ChronoLocal::new(TIMER_FORMAT.into());
    fmt::layer()
        .with_timer(timer)
        .with_target(true)
        .with_level(true)
}

pub fn init(log: LogConfig) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&log.level));
    let registry = Registry::default().with(filter);

    // --- 2. 解析输出目标并收集 Layers ---
    let outputs: Vec<&str> = log.output.split(',').map(|s| s.trim()).collect();
    let mut layers = Vec::new();
    let mut guards: Vec<WorkerGuard> = Vec::new();

    for output_target in outputs {
        // --- 【修改点 2】 ---

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
                    eprintln!("[WARN] Log output target 'file' specified, but no file configuration found.");
                }
            }
            unknown_target => {
                eprintln!("[WARN] Unknown log output target: '{}'", unknown_target);
            }
        }
    }

    // --- 3. 初始化 (循环外) ---
    if !layers.is_empty() {
        let subscriber = registry.with(layers);
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");

        // --- 【修改点 3】 ---
        // 现在 guards 是一个 Vec，LOG_GUARD 也是一个 OnceLock<Vec<...>>，类型匹配
        if !guards.is_empty() {
            if LOG_GUARD.set(guards).is_err() {
                eprintln!("[ERROR] Could not set a new LOG_GUARD, this might be a bug as it should only be called once.");
            }
        }
    } else {
        // 默认行为
        eprintln!("[ERROR] No valid log output configured. Defaulting to console.");
        let timer = ChronoLocal::new(TIMER_FORMAT.into());
        let default_layer = create_base_layer()
            .with_writer(stdout);
        let subscriber = registry.with(default_layer);
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    }
}
