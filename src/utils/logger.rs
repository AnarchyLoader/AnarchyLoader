use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Local;
use log::{Metadata, Record};
use simplelog::{CombinedLogger, LevelFilter, SharedLogger, TermLogger, WriteLogger};

use crate::LOGGER;

#[derive(Clone)]
pub struct MyLogger {
    pub buffer: Arc<Mutex<String>>,
}

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = self.buffer.lock().unwrap();
            *buffer += &format!(
                "[{}] {} - {}\n",
                record.level(),
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

impl MyLogger {
    pub fn init() -> &'static MyLogger {
        LOGGER.get_or_init(|| {
            let log_buffer = Arc::new(Mutex::new(String::new()));
            let logger = MyLogger { buffer: log_buffer };

            let loggers: Vec<Box<dyn SharedLogger>> = vec![
                TermLogger::new(
                    LevelFilter::Trace,
                    simplelog::Config::default(),
                    simplelog::TerminalMode::Mixed,
                    simplelog::ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Trace,
                    simplelog::Config::default(),
                    File::create(
                        dirs::config_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join("anarchyloader")
                            .join("anarchyloader.log"),
                    )
                    .expect("Failed to create log file"),
                ),
                Box::new(logger.clone()),
            ];

            CombinedLogger::init(loggers).expect("Failed to initialize logger");
            logger
        })
    }

    pub fn set_level(&self, level: LevelFilter) {
        log::set_max_level(level);
        let mut buffer = self.buffer.lock().unwrap();
        *buffer += &format!(
            "{} - Logger level set to: {:?}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            level
        );
    }
}

impl SharedLogger for MyLogger {
    fn level(&self) -> LevelFilter {
        log::max_level()
    }

    fn config(&self) -> Option<&simplelog::Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn log::Log + 'static> {
        Box::new(*self)
    }
}
