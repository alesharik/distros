use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use core::ops::Deref;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

const RING_BUFFER_SIZE: usize = 4096;

pub struct RingBuffer {
    write_cursor: AtomicUsize,
    buffer: [AtomicPtr<String>; RING_BUFFER_SIZE],
}

impl RingBuffer {
    fn new() -> Self {
        RingBuffer {
            write_cursor: AtomicUsize::new(0),
            buffer: [const { AtomicPtr::<String>::new(null_mut()) }; RING_BUFFER_SIZE],
        }
    }

    pub fn add(&self, string: &str) {
        let pos = self.write_cursor.fetch_add(1, Ordering::SeqCst) % RING_BUFFER_SIZE;
        let ptr = &self.buffer[pos];
        let str_ptr = Box::into_raw(Box::new(string.to_owned()));
        ptr.store(str_ptr, Ordering::SeqCst);
    }
}

lazy_static! {
    pub static ref SYSLOG_RING_BUFFER: RingBuffer = RingBuffer::new();
}

pub struct RingBufferIter {
    read_cursor: AtomicUsize,
}

impl RingBufferIter {
    pub fn new() -> Self {
        RingBufferIter {
            read_cursor: AtomicUsize::new(0),
        }
    }
}

impl Iterator for RingBufferIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = loop {
            let current_idx = self.read_cursor.load(Ordering::SeqCst);
            let write_cursor = SYSLOG_RING_BUFFER.write_cursor.load(Ordering::SeqCst);
            if current_idx >= write_cursor {
                return None;
            } else {
                if self
                    .read_cursor
                    .compare_exchange(
                        current_idx,
                        current_idx + 1,
                        Ordering::SeqCst,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    break current_idx;
                }
            }
        } % RING_BUFFER_SIZE;
        let ptr = &SYSLOG_RING_BUFFER.buffer[idx];
        unsafe {
            let b = Box::from_raw(ptr.load(Ordering::SeqCst));
            let str = b.deref().clone();
            core::mem::forget(b); // does not call destructor so it does not free ring string
            Some(str)
        }
    }
}
