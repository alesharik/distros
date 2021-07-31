//! System log manager
//!
//! This module setups `log` logger to write all kernel messages in internal rung buffer.
//! This buffer is exposed by `/dev/syslog` flow.
//! It does not use kblog and other vga facilities to not intervene with tty device.
use crate::driver::syslog::ring::{RingBufferIter, SYSLOG_RING_BUFFER};
use crate::flow::{FlowManager, Message, Provider, Subscription, AnyConsumer};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use log::{Log, Metadata, Record};
use spin::Mutex;

mod ring;
mod wait;

struct SysLog {}

impl SysLog {
    const fn new() -> Self {
        SysLog {}
    }
}

impl Log for SysLog {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let string = format!(
            "[{}][{}] {}",
            record.level(),
            record.target(),
            record.args()
        );
        SYSLOG_RING_BUFFER.add(&string);
    }

    fn flush(&self) {
        wait::wakeup();
    }
}

static LOG_INSTANCE: SysLog = SysLog::new();

pub struct SyslogMessage(String);

impl Message for SyslogMessage {}

impl Debug for SyslogMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct SyslogSubscription {
    id: u64,
    stop_flag: Arc<AtomicBool>,
}

impl Subscription for SyslogSubscription {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn cancel(self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

impl Drop for SyslogSubscription {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

struct SyslogProvider {
    id_counter: AtomicU64,
}

impl SyslogProvider {
    fn new() -> Self {
        SyslogProvider {
            id_counter: AtomicU64::new(0),
        }
    }

    async fn spawn_consumer(
        consumer: Box<dyn AnyConsumer>,
        stop_flag: Arc<AtomicBool>,
    ) {
        let mut iterator = RingBufferIter::new();
        while !stop_flag.load(Ordering::SeqCst) {
            if let Some(message) = iterator.next().map(SyslogMessage) {
                consumer.consume_msg(&message).await;
            } else {
                wait::wait_for_syslog().await;
            }
        }
    }
}

impl Provider for SyslogProvider {
    fn add_consumer(
        &mut self,
        consumer: Box<dyn AnyConsumer>,
    ) -> Box<dyn Subscription> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        spawn!(SyslogProvider::spawn_consumer(consumer, stop_flag.clone()));
        Box::new(SyslogSubscription {
            stop_flag,
            id: self.id_counter.fetch_add(1, Ordering::SeqCst),
        })
    }
}

pub fn init() {
    log::set_logger(&LOG_INSTANCE).expect("Cannot start syslog instance");
    FlowManager::register_endpoint::<SyslogMessage>(
        "/dev/syslog",
        Arc::new(Mutex::new(SyslogProvider::new())),
        None,
    ).unwrap();
}
