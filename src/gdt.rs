use x86_64::VirtAddr;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

// IST(interrupt stack table) は TSS(task state segment) の一部
// IST は、割り込みが発生した場合に使うスタックへのポインタを保持する
// 割り込みが発生すると、ハンドラ起動前にハードウェアによってスタックポインタが切り替えられる
// 特権レベルが変わったときにスタックを切り替える特権スタックテーブルというのもある

// IST は複数のスタックを持てる
// ダブルフォルト用に 0 番目のスタックを使うことにする
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // 可変なスタティック変数としてスタック領域を確保
            // ここはスワップアウトされないのか？
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;

            // スタックは下位アドレスに向けて伸びるので末尾を IST に登録
            stack_end
        };
        tss
    };
}

lazy_static! {
    // GDT はユーザ・カーネル空間の切り替えと TSS のロードに使う
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {code_selector, tss_selector})
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    // use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    // GDT を更新
    GDT.0.load();

    unsafe {
        // set_cs(GDT.1.code_selector);
        // コードセグメントと TSS も個別に更新しないといけない
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
