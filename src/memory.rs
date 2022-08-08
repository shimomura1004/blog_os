use x86_64::{
    VirtAddr,
    structures::paging::PageTable,
    structures::paging::OffsetPageTable,
};

// mut な参照を返す関数なので、何回も呼ばれて複数の名前で同一のメモリを参照すると危険
// init からのみ呼び出すようにする
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
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

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    // OffsetPageTable は固定オフセットで全物理メモリをマップする場合に使えるライブラリ関数
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
//  -> Option<PhysAddr>
// {
//     translate_addr_inner(addr, physical_memory_offset)
// }

// fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
//  -> Option<PhysAddr>
// {
//     use x86_64::structures::paging::page_table::FrameError;
//     use x86_64::registers::control::Cr3;

//     let (level4_table_frame, _) = Cr3::read();

//     let table_indexes = [
//         addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
//     ];
//     // この変数が指す先を書き換えながら各レベルのフレームを参照していく
//     let mut frame = level4_table_frame;

//     for &index in &table_indexes {
//         // エントリに書かれているのは物理アドレスなので、仮想アドレスに変換する
//         let virt = physical_memory_offset + frame.start_address().as_u64();
//         let table_ptr: *const PageTable = virt.as_ptr();
//         let table = unsafe { &*table_ptr };

//         let entry = &table[index];
//         // L1 のインデックスまで調べると、実際のページの物理アドレスが得られる
//         frame = match entry.frame() {
//             Ok(frame) => frame,
//             Err(FrameError::FrameNotPresent) => return None,
//             Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
//         };
//     }

//     // オフセットを足して物理アドレスを仮想アドレスにしてから返す
//     Some(frame.start_address() + u64::from(addr.page_offset()))
// }
