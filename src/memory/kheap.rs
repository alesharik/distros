use crate::interrupts;
use crate::kblog;
use core::alloc::{GlobalAlloc, Layout};
use core::ops::Deref;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use spin::Mutex;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{Page, Size2MiB, PhysFrame, PageTableFlags, FrameAllocator, PageSize, Size4KiB};
use x86_64::{VirtAddr, PhysAddr};
use core::fmt::{Display, Formatter};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use super::page_table;
use crate::flow::{FlowManager, Message, Provider, AnyConsumer, Subscription};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const HEAP_START: usize = 0x_5444_4444_0000;
pub const HEAP_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

pub struct LockedHeap(Mutex<Heap>);

impl LockedHeap {
    /// Creates an empty heap. All allocate calls will return `None`.
    pub const fn empty() -> LockedHeap {
        LockedHeap(Mutex::new(Heap::empty()))
    }

    /// Creates a new heap with the given `bottom` and `size`. The bottom address must be valid
    /// and the memory in the `[heap_bottom, heap_bottom + heap_size)` range must not be used for
    /// anything else. This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_bottom: usize, heap_size: usize) -> LockedHeap {
        LockedHeap(Mutex::new(Heap::new(heap_bottom, heap_size)))
    }
}

impl Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        interrupts::no_int(|| {
            self.0
                .lock()
                .allocate_first_fit(layout)
                .ok()
                .map_or(core::ptr::null_mut::<u8>(), |allocation| {
                    allocation.as_ptr()
                })
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        interrupts::no_int(|| {
            self.0
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout)
        })
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static TABLE_FRAME_SIZE: AtomicUsize = AtomicUsize::new(0);

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct KernelHeapInfo {
    pub size: usize,
    pub virt_start: u64,
    pub used: usize,
    /// How much bytes takes frame
    pub table_frame_size: usize,
}

impl Display for KernelHeapInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Kernel Heap at {:#x}: {} of {} used ({}%)", self.virt_start, self.used, self.size, self.used * 100 / self.size)
    }
}

impl Message for KernelHeapInfo {}

struct KHeapFrameAllocator<'a> {
    alloc_map: &'a mut [u64],
    map: &'static MemoryMap,
    allocated: usize,
}

unsafe impl<'a> FrameAllocator<Size4KiB> for KHeapFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for (idx, region) in self.map.iter().filter(|m| m.region_type == MemoryRegionType::Usable).enumerate() {
            if (region.range.end_frame_number - region.range.start_frame_number) * 4096 - self.alloc_map[idx] <= Size4KiB::SIZE {
                continue
            }
            self.alloc_map[idx] += Size4KiB::SIZE;
            self.allocated += Size4KiB::SIZE as usize;
            return Some(PhysFrame::containing_address(PhysAddr::new(region.range.start_frame_number * 4096 + self.alloc_map[idx] + 1)))
        }
        None
    }
}

pub fn init_kheap(map: &'static MemoryMap) -> Result<KernelHeapInfo, MapToError<Size2MiB>> {
    kblog!("KHeap", "Starting kernel heap");
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let heap_start_page = Page::<Size2MiB>::containing_address(heap_start);
    let heap_end_page = Page::<Size2MiB>::containing_address(heap_end);
    let mut alloc_map = [0u64; 16];
    let mut table_frame_size = 0usize;
    for page in Page::<Size2MiB>::range_inclusive(heap_start_page, heap_end_page) {
        for (idx, region) in map.iter().filter(|m| m.region_type == MemoryRegionType::Usable).enumerate() {
            if (region.range.end_frame_number - region.range.start_frame_number) * 4096 - alloc_map[idx] <= page.size() {
                continue
            }
            alloc_map[idx] += page.size();
            let phys_frame = PhysFrame::containing_address(PhysAddr::new(region.range.start_frame_number * 4096 + alloc_map[idx] + 1));
            let mut alloc = KHeapFrameAllocator { allocated: 0, map, alloc_map: &mut alloc_map };
            page_table::map_init(phys_frame, page, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, &mut alloc).unwrap();
            table_frame_size = alloc.allocated;
            break
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    kblog!(
        "KHeap",
        "Kernel heap started at pos {:#x} with size {} MiB",
        HEAP_START,
        HEAP_SIZE / 1024 / 1024
    );
    TABLE_FRAME_SIZE.store(table_frame_size, Ordering::SeqCst);
    Ok(KernelHeapInfo {
        size: HEAP_SIZE,
        virt_start: HEAP_START as u64,
        used: 0,
        table_frame_size,
    })
}

struct SubscriptionImpl {}

impl Subscription for SubscriptionImpl {
    fn get_id(&self) -> u64 {
        0
    }

    fn cancel(self) {
    }
}

struct ProviderImpl {}

impl ProviderImpl {
    async fn send(consumer: Box<dyn AnyConsumer>, info: KernelHeapInfo) {
        let sub = SubscriptionImpl { };
        consumer.consume_msg(&info).await;
        consumer.close_consumer(&sub).await;
    }
}

impl Provider for ProviderImpl {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let info = {
            let locked = ALLOCATOR.0.lock();
            KernelHeapInfo {
                table_frame_size: TABLE_FRAME_SIZE.load(Ordering::SeqCst),
                size: HEAP_SIZE,
                virt_start: HEAP_START as u64,
                used: locked.used(),
            }
        };
        crate::futures::spawn(ProviderImpl::send(consumer, info));
        let sub = SubscriptionImpl { };
        Box::new(sub)
    }
}

pub fn init_kheap_info() {
    FlowManager::register_endpoint::<KernelHeapInfo>(
        "/dev/kernel_heap/info",
        Arc::new(Mutex::new(ProviderImpl {  })),
        None,
    ).unwrap();
}