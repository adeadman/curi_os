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

    use curi_os::vesa_buffer::{Colour16Bit, draw_pixel, draw_line};
    unsafe{
        for i in 200..400 {
            let colour = match i%30 {
                0..9 => Colour16Bit::red(),
                10..19 => Colour16Bit::green(),
                20..29 => Colour16Bit::blue(),
                _ => Colour16Bit::white(),
            };
            draw_pixel(200, i, colour.as_u16());
        }

        draw_line(0, 0, 799, 599, Colour16Bit::red().as_u16());
        draw_line(500, 20, 20, 500, Colour16Bit::blue().as_u16());
        for i in 100..700 {
            let red: u8 = (32 * (700 - i) / 600) as u8;
            let green: u8 = (64 * (400 - i as i16).abs() / 300) as u8;
            let blue: u8 = (32 * (i - 100) / 600) as u8;
            let colour = Colour16Bit{red, green, blue};
            draw_line(i as usize, 50, i as usize, 150, colour.as_u16());
        }
    }

    #[cfg(test)]
    test_main();

    //println!("It didn't crash!");
    curi_os::hlt_loop();
}

