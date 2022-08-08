use::x86_64::{
    structures::paging::PageTable,
    VirtAddr,
};

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
 -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    // ブートローダが Cr3 を作ってくれている
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    // Cr3 に書かれた物理アドレスにオフセットを足した仮想アドレスを作る
    let virt = physical_memory_offset + phys.as_u64();
    // 仮想アドレスをポインタにして返す
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
