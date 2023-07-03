pub mod heap;

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::VirtAddr;

pub unsafe fn mapper(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    let (frame, _) = Cr3::read();

    let phys = frame.start_address();
    let virt = physical_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    let page_table = unsafe { &mut *page_table_ptr };
    unsafe { OffsetPageTable::new(page_table, physical_offset) }
}

pub fn page_range(start: u64, size: u64) -> PageRangeInclusive {
    let start = VirtAddr::new(start);
    let end = start + (size - 1);
    Page::range_inclusive(
        Page::containing_address(start),
        Page::containing_address(end)
    )
}

pub fn map(page_range: PageRangeInclusive,
           mapper: &mut impl Mapper<Size4KiB>,
           frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    Ok(())
}