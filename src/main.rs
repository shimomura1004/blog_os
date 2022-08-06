#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
// 標準だとテスト用の main 関数を作るが、この場合 main は呼ばれないので困る
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello world{}", "!");

    blog_os::init();

    let valid_ptr = 0x204a23 as *mut u64;
    unsafe { let _x = *valid_ptr; }
    println!("read worked");

    let invalid_ptr = 0xdeadbeaf as *mut u64;
    // ここで範囲外アクセスが起こってページフォルトが発生する
    unsafe { *invalid_ptr = 42; }

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
