use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct SimLogger;

impl Log for SimLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool { true }
    fn log(&self, record: &Record) {
        let color = match record.level() {
            Level::Error => 31,
            Level::Warn => 93,
            Level::Info => 34,
            Level::Debug => 32,
            Level::Trace => 90
        };
        println!(
            "\x1b[{}m[{:>5}] {}\x1b0[m",
            color,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: SimLogger = SimLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG_LEV") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off
    });
}
