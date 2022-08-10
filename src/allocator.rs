use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, frame,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

// pub struct Dummy;

// // 自作 OS では標準のアロケータが使えないので自前で定義する必要がある
// // 必ずアロケーションに失敗するダミーのアロケータ定義
// unsafe impl GlobalAlloc for Dummy {
//     unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
//         null_mut()
//     }

//     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
//         panic!("dealloc shoud never be called")
//     }
// }

// #[global_allocator]
// static ALLOCATOR: Dummy = Dummy;

// 名前が locked なのは排他制御にスピンロックを使っているから
// デッドロックする可能性があるので割込みハンドラ内でアロケートしてはいけない
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// メモリアロケートに失敗した場合に呼ばれるハンドラ
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// ヒープに使う領域(仮想アドレス)を定義しておく
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE : usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // 指定した場所(仮想アドレス)とサイズに対応するページ情報を作る
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // ヒープ用に使うページを、順番に物理フレームのどこかに割当てていく
    for page in page_range {
        // フレームアロケータを使って未使用の物理フレームを確保
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            // マップを更新して TLB をクリアする
            // ここでもフレームアロケータを渡しているのは、L2 以上のページテーブルを更新する可能性があるから
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    // アロケータに確保したメモリ領域の情報を伝えて初期化する
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
