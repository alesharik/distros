use core::alloc::Layout;
use log::{debug, error, warn};
use talc::{OomHandler, Span, Talc, Talck};
use crate::arena::{Arena, arena_alloc, Error};

struct OomHandlerImpl;

impl OomHandler for OomHandlerImpl {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let mut arena = arena_alloc();
        match arena.allocate(layout.pad_to_align().size()) {
            Ok(arena) => {
                unsafe { talc.claim(arena.into())? };
                Ok(())
            }
            Err(e) => {
                error!("Failed to alloc arena: {:?}", e);
                Err(())
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, OomHandlerImpl> = Talc::new(OomHandlerImpl).lock();
