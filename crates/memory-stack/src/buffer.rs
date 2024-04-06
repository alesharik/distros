use crate::{KERNEL_STACK_BASE, KERNEL_STACK_BASE_GUARD, KERNEL_STACK_SIZE};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts;
use spin::RwLock;
use x86_64::VirtAddr;

#[derive(Clone)]
pub enum StackBuffer {
    KernelBase,
    Heap(Arc<Vec<u8>>),
}

impl StackBuffer {
    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            StackBuffer::KernelBase => KERNEL_STACK_SIZE as usize,
            StackBuffer::Heap(vec) => vec.capacity(),
        }
    }

    #[inline]
    pub fn start(&self) -> VirtAddr {
        match self {
            StackBuffer::KernelBase => KERNEL_STACK_BASE,
            StackBuffer::Heap(vec) => VirtAddr::from_ptr(vec.as_ptr()),
        }
    }
}

static BUFFERS: RwLock<BTreeMap<u64, StackBuffer>> = RwLock::new(BTreeMap::new());

pub struct StackBufferHandle(u64);

impl Drop for StackBufferHandle {
    fn drop(&mut self) {
        let mut buffers = BUFFERS.write();
        buffers.remove(&self.0);
    }
}

pub fn find_buffer<I: Into<u64>>(esp: VirtAddr, external_id: I) -> Option<StackBuffer> {
    if esp >= KERNEL_STACK_BASE_GUARD && esp < KERNEL_STACK_BASE {
        return None;
    }
    if esp >= KERNEL_STACK_BASE && esp <= (KERNEL_STACK_BASE + KERNEL_STACK_SIZE) {
        return Some(StackBuffer::KernelBase);
    }
    let buffers = BUFFERS.read();
    buffers.get(&external_id.into()).cloned()
}

pub fn new_buffer<I: Into<u64>>(external_id: I, size: usize) -> (StackBuffer, StackBufferHandle) {
    let buffer = StackBuffer::Heap(Arc::new(Vec::with_capacity(size)));
    let mut buffers = BUFFERS.write();
    let id = external_id.into();
    buffers.insert(id, buffer.clone());
    (buffer, StackBufferHandle(id))
}

pub fn make_copy<I: Into<u64>>(
    external_id: I,
    buffer: &StackBuffer,
) -> (StackBuffer, StackBufferHandle) {
    let mut buf_contents = vec![0u8; buffer.capacity()];
    match buffer {
        StackBuffer::KernelBase => {
            let slice = slice_from_raw_parts::<u8>(KERNEL_STACK_BASE.as_ptr(), buffer.capacity());
            unsafe { buf_contents.copy_from_slice(&*slice) }
        }
        StackBuffer::Heap(src) => {
            buf_contents.truncate(0);
            buf_contents.copy_from_slice(&src[0..buffer.capacity()]);
        }
    }

    let buffer = StackBuffer::Heap(Arc::new(buf_contents));
    let mut buffers = BUFFERS.write();
    let id = external_id.into();
    buffers.insert(id, buffer.clone());
    (buffer, StackBufferHandle(id))
}
