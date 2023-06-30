use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use good_memory_allocator::SpinLockedAllocator;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE  : usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        Page::range_inclusive(
            Page::containing_address(heap_start),
            Page::containing_address(heap_end)
        )
    };
    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

unsafe fn level_4_page_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let physical_addr = frame.start_address();
    let virtual_addr = physical_offset + physical_addr.as_u64();
    unsafe { &mut *virtual_addr.as_mut_ptr() }
}

pub unsafe fn init(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let page_table = level_4_page_table(physical_offset);
        OffsetPageTable::new(page_table, physical_offset)
    }
}

pub struct StandardFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize
}

impl StandardFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        Self { memory_regions, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions.iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| region.start..region.end)
            .flat_map(|range| range.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for StandardFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}