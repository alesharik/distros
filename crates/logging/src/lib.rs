#![no_std]

use log::{LevelFilter, Log, Metadata, Record};
use core::fmt::Write;
use core::mem;
use core::panic::PanicInfo;
use spin::Mutex;

pub struct Logger<T> {
    level: LevelFilter,
    writer: Mutex<T>,
}

impl<T: Write + Send + Sync + 'static> Logger<T> {
    pub fn new(target: T) -> Logger<T> {
        Logger {
            level: LevelFilter::Info,
            writer: Mutex::new(target),
        }
    }

    pub fn set_max_level(mut self, level: LevelFilter) -> Logger<T> {
        self.level = level;
        self
    }

    pub fn init(self) {
        log::set_max_level(self.level);
        unsafe {
            log::set_logger_racy(mem::transmute(&self as &dyn Log)).unwrap();
            mem::forget(self);
        }
    }

    pub fn panic(info: &PanicInfo) {
        let logger: &Logger<T> = unsafe { &*(log::logger() as *const _ as *const Logger<T>) };
        if logger.writer.is_locked() {
            unsafe { logger.writer.force_unlock(); }
        }
        let mut w = logger.writer.lock();
        write!(w, "PANIC! {}", info).unwrap();
    }
}

impl<T: Write + Send + Sync> Log for Logger<T> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let mut writer = self.writer.lock();
        write!(writer, "[{}] {} {}\n", record.level(),target, record.args()).unwrap();
    }

    fn flush(&self) {}
}
