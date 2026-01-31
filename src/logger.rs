use log::{self, error};

#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Silent = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
    Trace = 5
}

impl TryInto<log::Level> for LogLevel {
    type Error = ();

    fn try_into(self) -> Result<log::Level, ()> {
        match self {
            LogLevel::Silent => Err(()),
            LogLevel::Error => Ok(log::Level::Error),
            LogLevel::Warning => Ok(log::Level::Warn),
            LogLevel::Info => Ok(log::Level::Info),
            LogLevel::Debug => Ok(log::Level::Debug),
            LogLevel::Trace => Ok(log::Level::Trace)
        }
    }
}

pub(crate) static mut LOG_LEVEL: LogLevel = LogLevel::Info;

pub fn set_log_level(level: LogLevel) {
    unsafe {
        LOG_LEVEL = level;
    }
}

struct Logger;
static LOGGER: Logger = Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        unsafe { LOG_LEVEL }.try_into()
            .map(|l: log::Level| metadata.level() <= l)
            .unwrap_or(false)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let datetime = chrono::Local::now()
                .format("%H:%M:%S %d/%m");

            println!("{} [{}] {}", datetime, record.level(), record.args())
        }
    }

    fn flush(&self) { 
        todo!()
    }
}

pub(crate) fn init_logger() {
    let _ = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .map_err(|e| error!("{e}"));
}