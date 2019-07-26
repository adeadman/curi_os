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

    // Initialise the kernel library functions
    curi_os::init();

    // Create our memory mapper and allocator
    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // Initialise the heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialisation failed");

    use curi_os::vesa_buffer::clear_screen;
    clear_screen();

    println!("Hello World{}", "!");

    use curi_os::vesa_buffer::{Colour16Bit, draw_pixel, draw_line,
                               draw_pixel_with_opacity,
                               draw_smooth_line};
    use curi_os::vesa_buffer::{RED, GREEN, BLUE, WHITE};

    let blended_colour = RED.blend_colour(&WHITE, 0.5);
    assert_eq!(blended_colour.red, 31);
    assert_eq!(blended_colour.green, 31);
    assert_eq!(blended_colour.blue, 15);

    for i in 200..400 {
        let colour = match i%30 {
            0..10 => RED,
            10..20 => GREEN,
            20..30 => BLUE,
            _ => WHITE,
        };
        draw_pixel(200, i, &colour);
        draw_pixel_with_opacity(i, 530, &GREEN, (i - 200) as f64 / 200.0);
    }

    draw_line(0, 0, 799, 599, &RED);
    draw_line(500, 20, 20, 500, &BLUE);
    for i in 100..700 {
        let red: u8 = (32 * (700 - i) / 600) as u8;
        let green: u8 = (64 * (400 - i as i16).abs() / 300) as u8;
        let blue: u8 = (32 * (i - 100) / 600) as u8;
        let colour = Colour16Bit{red, green, blue};
        draw_line(i as usize, 50, i as usize, 150, &colour);
    }

    draw_smooth_line(250, 250, 380, 280, &WHITE);
    draw_smooth_line(250, 500, 480, 180, &Colour16Bit{red:31, green:63, blue:0});
    draw_smooth_line(699, 150, 100, 50,
                     &Colour16Bit{red:16, green:63, blue:16});

    use curi_os::vesa_buffer::SCREEN;
    SCREEN.lock().draw_red_square();
    SCREEN.lock().swap_buffers();
    #[cfg(test)]
    test_main();

    println!("It didn't crash!");
    curi_os::hlt_loop();
}

