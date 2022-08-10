#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
// 標準だとテスト用の main 関数を作るが、この場合 main は呼ばれないので困る
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::frame;
use core::panic::PanicInfo;
use blog_os::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory;
    use blog_os::memory::BootInfoFrameAllocatior;
    use blog_os::allocator;
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("hello world{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocatior::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i)
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // // アドレス 0 のページを作り、VGA のメモリにマップさせる
    // // NULL ポインタアクセスをページフォルトにするためにマップしないのが普通なので
    // // アドレス 0 は未使用のはず、かつ、ブートローダが確実にメモリ先頭付近のL4~L1エントリを作っているはず
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // // VGA にマップできたか確認する
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // // 0 バイト目に書くとスクロールしてすぐ消えるので、少し後ろ(400)にする
    // // 書き込むバイト列は文字列
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    blog_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
