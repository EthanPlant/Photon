use log::Level;

static LOGGER: KernelLogger = KernelLogger;

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // TODO: Make log level configurable at compile time
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            use crate::serial_print;

            let file = record.file().unwrap_or("unknown");
            let file = file.strip_prefix("src/").unwrap_or(file);

            let line = record.line().unwrap_or(0);

            let level = record.level();

            macro_rules! generic_log {
                ($level:ident, $($arg:tt)*) => {
                    let level = match level {
                        Level::Error => "\x1b[31m[ERROR]",
                        Level::Warn => "\x1b[33m[WARN]",
                        Level::Info => "\x1b[32m[INFO]",
                        Level::Debug => "\x1b[34m[DEBUG]",
                        Level::Trace => "\x1b[37m[TRACE]",
                    };
                    serial_print!("{}{}", level, format_args!($($arg)*));
                };
            }

            generic_log!(level, "\x1b[0m {}:{} - {}\n", file, line, record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .expect("Logger's already been initialized");
}
