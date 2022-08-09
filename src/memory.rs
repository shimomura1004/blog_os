use x86_64::{
    PhysAddr,
    VirtAddr,
    structures::paging::PageTable,
    structures::paging::OffsetPageTable,
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocatior {
    memory_map: &'static MemoryMap,
    next: usize,
}

// ブートローダから渡されたメモリマップを使って空きフレームを探すアロケータ
impl BootInfoFrameAllocatior {
    // 割り当てるフレームが本当に他で使われていないか(名前がつけられていないか)が
    // この関数側では保証できないので unsafe とする
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocatior {
            memory_map,
            next: 0,
        }
    }

    // 未使用のフレーム(物理領域)を順番に返す
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // メモリマップを走査して未使用フレームを抽出
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr() .. r.range.end_addr());

        // 各アドレス領域(開始から終了までの区間)それぞれに対し、ページ1つ分ごとでアドレスを抽出
        // 4KiB 以外のページがあることを考慮？
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocatior {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // 使用可能なフレームのうち最初のひとつを選び返す
        // 毎回イテレータを作っているので効率は悪い
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // 物理アドレス 0xb8000 を含むフレームを作成(ページテーブルはここを指すように修正される)
    // フレームが物理、ページは仮想
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    // 新たなマッピングを追加(page で渡される仮想アドレスを 0xb8000 にマッピング)
    // 戻り値の型の map_to は、追加したページ をTLB からクリアする flush メソッドを持っている
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

// pub struct EmptyFrameAllocator;

// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
//         None
//     }
// }

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
