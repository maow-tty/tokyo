use bitvec::{BitArr, bitarr};
use bitvec::order::Msb0;
use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use linked_list_allocator::LockedHeap;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, PhysFrame, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use crate::{mem, serial_println};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// FIXME: Kernel should dynamically allocate memory if it's running low and memory is available.
pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE  : usize = 5_000 * 1024; // 5,000 KiB

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    serial_println!("[tokyo] initializing heap");

    let page_range = mem::page_range(HEAP_START as u64, HEAP_SIZE as u64);
    mem::map(page_range, mapper, frame_allocator)?;

    serial_println!("[tokyo] initializing global allocator");

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

pub struct KernelFrameAllocator {
    region: &'static MemoryRegion,
    bitmap: BitArr!(for HEAP_SIZE, in u8, Msb0)
}

impl KernelFrameAllocator {
    pub unsafe fn new(regions: &'static MemoryRegions) -> Option<Self> {
        if let Some(region) = regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .find(|r| (r.end - r.start) as usize >= HEAP_SIZE)
        {
            return Some(Self {
                region,
                bitmap: bitarr![u8, Msb0; 0; HEAP_SIZE]
            });
        }
        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if let Some(i) = self.bitmap.first_zero() {
            let addr = PhysAddr::new(self.region.start + (i as u64 * 4096));
            self.bitmap.set(i, true);
            return Some(PhysFrame::containing_address(addr));
        }
        None
    }
}