use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct SimLogger;

impl Log for SimLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool { true }
    fn log(&self, record: &Record) {
        let color = match record.level()
    }
}
