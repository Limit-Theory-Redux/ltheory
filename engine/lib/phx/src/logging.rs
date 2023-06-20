use std::io::Write;

use regex::Regex;
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
use tracing_subscriber::{prelude::*, EnvFilter};

/// Clean messages from coloring ASCII instructions before writing to file.
struct MessageCleaner {
    appender: RollingFileAppender,
    re: Regex,
}

impl MessageCleaner {
    fn new(appender: RollingFileAppender) -> Self {
        Self {
            appender,
            re: Regex::new(r"\u{1b}\[[0-9;]*m").expect("Cannot compile regex"),
        }
    }
}

impl Write for MessageCleaner {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = String::from_utf8_lossy(buf);
        let clean_msg = self.re.replace_all(&msg, "").to_string();
        let bytes = clean_msg.as_bytes();
        let bytes_written = self.appender.write(bytes)?;
        let remaining_bytes = bytes.len() - bytes_written;

        Ok(buf.len() - remaining_bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.appender.flush()
    }
}

pub fn init_log(console_log: bool, log_dir: &str) -> Option<WorkerGuard> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .expect("Cannot create log env filter layer");
    let registry = tracing_subscriber::registry().with(filter_layer);

    let guard = if !log_dir.is_empty() {
        let file_appender = tracing_appender::rolling::daily(log_dir, "ltr");
        let message_cleaner = MessageCleaner::new(file_appender);
        let (non_blocking, guard) = tracing_appender::non_blocking(message_cleaner);

        let file_output_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(non_blocking);

        if console_log {
            let console_output_layer = tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false);

            registry
                .with(file_output_layer)
                .with(console_output_layer)
                .try_init()
                .expect("Cannot init log registry");
        } else {
            registry
                .with(file_output_layer)
                .try_init()
                .expect("Cannot init log registry");
        }

        Some(guard)
    } else {
        if console_log {
            let console_output_layer = tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false);

            registry
                .with(console_output_layer)
                .try_init()
                .expect("Cannot init log registry");
        } else {
            registry.try_init().expect("Cannot init log registry");
        }

        None
    };

    guard
}
