#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(exclusive_range_pattern)]
#![test_runner(curi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo,  entry_point};
use core::panic::PanicInfo;
use curi_os::println;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    curi_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    curi_os::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use curi_os::allocator;
    use curi_os::memory;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{Page};

    use curi_os::vesa_buffer::clear_screen;
    clear_screen();

    println!("Hello World{}", "!");
    curi_os::init();

    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialisation failed");

    use curi_os::vesa_buffer::{draw_pixel, draw_line};
    unsafe{
        for i in 200..400 {
            let colour: u16 = match i%30 {
                0..9 => 0xf800,
                10..19 => 0x07e0,
                20..29 => 0x001f,
                _ => 0xffff,
            };
            draw_pixel(200, i, colour);
        }

        draw_line(0, 0, 799, 599, 0xf800);
        draw_line(500, 20, 20, 500, 0x001f);
    }

    #[cfg(test)]
    test_main();

    //println!("It didn't crash!");
    curi_os::hlt_loop();
}

