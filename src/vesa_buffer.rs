use core::{fmt, slice};
use core::ptr::copy_nonoverlapping;
use crate::num_traits::float::FloatCore;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;

#[cfg(test)]
use crate::{serial_print, serial_println};

const BUFFER_HEIGHT: usize = 75;
const BUFFER_WIDTH: usize = 100;
pub const SCREEN_HEIGHT: usize = 600;
pub const SCREEN_WIDTH: usize = 800;
const VGA_BUFFER: *mut u16 = (500 * 512 * 4096) as *mut _;

pub struct Writer {
    xpos: usize,
    ypos: usize,
}

impl Writer {
    pub fn clear_screen(&mut self) {
        SCREEN.lock().clear_screen();
        self.xpos = 0;
        self.ypos = 0;
    }

    fn newline(&mut self) {
        self.ypos += 8;
        self.xpos = 0;
        if self.ypos >= SCREEN_HEIGHT {
            // self.clear_screen();
            SCREEN.lock().scroll_buffer_vertically(8);
            self.ypos -= 8;
        }
    }

    fn write_char(&mut self, c: char) {
        use font8x8::UnicodeFonts;

        if c == '\n' {
            self.newline();
            return;
        }

        self.xpos += 8;

        match c {
            ' '..='~' => {
                let rendered = font8x8::BASIC_FONTS
                    .get(c)
                    .expect("character not found in basic font");

                for (y, byte) in rendered.iter().enumerate() {
                    for (x, bit) in (0..8).enumerate() {
                        if *byte & (1 << bit) == 0 {
                            continue;
                        }
                        SCREEN.lock().plot_pixel(self.xpos + x, self.ypos + y, &WHITE);
                    }
                }
                SCREEN.lock().swap_buffers();
            }
            _ => panic!("unprintable character"),
        }

        if self.xpos + 8 >= SCREEN_WIDTH {
            self.newline();
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            if c.is_ascii() {
                self.write_char(c);
            } else {
                self.write_char(0xfe as char);
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        xpos: 0,
        ypos: 0,
    });
}

pub struct Screen {
    vga_buffer: &'static mut [u16],
    back_buffer: Vec<u16>,
}

impl Screen {
    pub fn swap_buffers(&mut self) {
        unsafe {
            let src_ptr = self.back_buffer.as_slice().as_ptr();
            let dst_ptr = self.vga_buffer.as_mut_ptr();
            copy_nonoverlapping(src_ptr, dst_ptr, SCREEN_WIDTH*SCREEN_HEIGHT);
        }
    }

    pub fn draw_red_square(&mut self) {
        let back_buffer = self.back_buffer.as_mut_slice();
        for y in 400..500 {
            for x in 400..500 {
                let pixel_offset: usize = y * SCREEN_WIDTH + x;
                back_buffer[pixel_offset] = RED.as_u16();
            }
        }
    }

    pub fn draw_sprite_to_fb(&mut self, x: usize, y: usize,
                             sprite: [u16; 16]) {
        for (i, byte) in sprite.iter().enumerate() {
            let mut sprite_line = Vec::new();
            for bit in 0..16 {
                if byte & (1 << (15 - bit)) == (1 << (15 - bit)) {
                    sprite_line.push(WHITE.as_u16());
                } else {
                    sprite_line.push(0x0);
                }
            }
            let offset = (y + i) * SCREEN_WIDTH + x;
            unsafe {
                let src_ptr = sprite_line.as_slice().as_ptr();
                let dst_ptr = self.vga_buffer.as_mut_ptr().offset(offset as isize);
                copy_nonoverlapping(src_ptr, dst_ptr, 16);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.clear_back_buffer();
        self.swap_buffers();
    }

    fn clear_back_buffer(&mut self) {
        let back_buffer = self.back_buffer.as_mut_slice();
        for pixel in 0..back_buffer.len() {
            back_buffer[pixel] = 0x0;
        }
    }

    pub fn scroll_buffer_vertically(&mut self, num_pixels: usize) {
        self.back_buffer.drain(0..SCREEN_WIDTH*num_pixels);
        self.back_buffer.extend(vec![0x0; SCREEN_WIDTH*num_pixels]);
        self.swap_buffers();
    }

    pub fn plot_pixel(&mut self, x: usize, y: usize, colour: &Colour16Bit) {
        let back_buffer = self.back_buffer.as_mut_slice();
        back_buffer[y * SCREEN_WIDTH + x] = colour.as_u16();
    }

    pub fn get_colour_at(&self, x: usize, y: usize) -> Colour16Bit {
        let back_buffer = self.back_buffer.as_slice();
        let pixel_offset = y * SCREEN_WIDTH + x;
        Colour16Bit::from_u16(back_buffer[pixel_offset])
    }
}

lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen {
        vga_buffer: unsafe { slice::from_raw_parts_mut(VGA_BUFFER, SCREEN_WIDTH * SCREEN_HEIGHT) },
        //back_buffer: Vec::<u16>with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT).as_mut_slice(),
        back_buffer: vec![0_u16; SCREEN_WIDTH * SCREEN_HEIGHT],
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vesa_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

    serial_println!("[ok]");
}

pub struct Colour16Bit {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Colour16Bit {
    pub fn red() -> Colour16Bit {
        Colour16Bit{red:0x1f, green:0x0, blue:0x0}
    }

    pub fn green() -> Colour16Bit {
        Colour16Bit{red:0x0, green:0x3f, blue:0x0}
    }

    pub fn blue() -> Colour16Bit {
        Colour16Bit{red:0x0, green:0x0, blue:0x1f}
    }

    pub fn white() -> Colour16Bit {
        Colour16Bit{red:0x1f, green:0x3f, blue:0x1f}
    }

    pub fn from_u16(rgb: u16) -> Colour16Bit {
        let red = ((rgb & 0xf800) >> 11) as u8;
        let green = ((rgb & 0x7e0) >> 5) as u8;
        let blue = (rgb & 0x1f) as u8;
        Colour16Bit{red, green, blue}
    }

    pub fn blend_colour(
        &self,
        other: &Colour16Bit,
        opacity: f64
    ) -> Colour16Bit {
        let red = scale_first_to_other(self.red, other.red, opacity);
        let green = scale_first_to_other(self.green, other.green, opacity);
        let blue = scale_first_to_other(self.blue, other.blue, opacity);
        Colour16Bit{red, green, blue}
    }

    pub fn as_u16(&self) -> u16 {
        ((self.red as u16) << 11) +
            ((self.green as u16) << 5) +
            self.blue as u16
    }
}

/// algorithm to find the value that is the mixed ratio between
/// the first value and the other value by the amount specified
///
/// e.g. first = 10, other = 20, amount = 0.2
/// should return 18
/// (20% of 10 and 80% of 20)
/// (20 * 0.8) + (10 * 0.2) = 16 + 2 = 18
fn scale_first_to_other(first: u8, other: u8, amount: f64) -> u8 {
    let first_f64 = first as f64;
    let other_f64 = other as f64;
    (other_f64 * (1.0 - amount) + first_f64 * amount) as u8
}

pub const RED: Colour16Bit = Colour16Bit { red:0x1f, green:0x0, blue:0x0 };
pub const GREEN: Colour16Bit  = Colour16Bit { red:0x0, green:0x3f, blue:0x0 };
pub const BLUE: Colour16Bit = Colour16Bit { red:0x0, green:0x0, blue:0x1f };
pub const WHITE: Colour16Bit = Colour16Bit{ red:0x1f, green:0x3f, blue:0x1f };
pub const BLACK: Colour16Bit = Colour16Bit{ red:0x0, green:0x0, blue:0x0 };

pub fn draw_pixel(x: usize, y: usize, colour: &Colour16Bit) {
    SCREEN.lock().plot_pixel(x, y, &colour);
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SCREEN.lock().clear_screen();
    });
}

/// Bresenham's Line Drawing Algorithm
pub fn draw_line(
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    colour: &Colour16Bit,
) {
    let deltax: f64 = (end_x as f64 - start_x as f64).abs();
    let sx: isize = if start_x < end_x { 1 } else { -1 };
    let deltay: f64 = -((end_y as f64 - start_y as f64).abs());
    let sy: isize = if start_y < end_y { 1 } else { -1 };

    let mut err = deltax + deltay;
    let mut done = false;
    let mut current_x = start_x;
    let mut current_y = start_y;
    while !done {
        if current_x == end_x && current_y == end_y {
            done = true;
        }
        draw_pixel(current_x, current_y, &colour);
        let e2 = 2.0 * err;
        if e2 >= deltay {
            err += deltay;
            current_x = (current_x as isize + sx) as usize;
        }
        if e2 <= deltax {
            err += deltax;
            current_y = (current_y as isize + sy) as usize;
        }
    }
}

pub fn draw_pixel_with_opacity(
    x: usize, y: usize,
    colour: &Colour16Bit,
    opacity: f64
) {
    assert!(opacity >= 0.0 && opacity <= 1.0,
            "draw_pixel_with_opacity: Bad opacity of {}", opacity);
    if opacity == 1.0 {
        draw_pixel(x, y, &colour);
    } else if opacity != 0.0 {
        let bg_colour = SCREEN.lock().get_colour_at(x, y);
        let blended_colour = colour.blend_colour(&bg_colour, opacity);
        SCREEN.lock().plot_pixel(x, y, &blended_colour);
    }
    // no need to draw anything if the opacity is 0.0
}

// Xiaolin Wu's Line Drawing Algorithm
pub fn draw_smooth_line(
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    colour: &Colour16Bit,
) {
    let deltax: f64 = (end_x as f64 - start_x as f64).abs();
    let deltay: f64 = (end_y as f64 - start_y as f64).abs();
    let steep = deltay > deltax;

    let mut x0 = start_x as f64;
    let mut x1 = end_x as f64;
    let mut y0 = start_y as f64;
    let mut y1 = end_y as f64;

    if steep == true {
        let tmp = y0;
        y0 = x0;
        x0 = tmp;

        let tmp = y1;
        y1 = x1;
        x1 = tmp;
    }
    if x0 > x1 {
        let tmp = x0;
        x0 = x1;
        x1 = tmp;

        let tmp = y0;
        y0 = y1;
        y1 = tmp;
    }

    let deltax = x1 - x0;
    let deltay = y1 - y0;
    let gradient = match deltax == 0.0 {
        true => 1.0,
        false => deltay / deltax,
    };

    // handle first endpoint
    let xend = hround(x0);
    let yend = y0 + gradient * (xend - x0);
    let xgap = rfract(x0 + 0.5);
    let x_pixel1 = xend as usize;
    let y_pixel1 = trunc(yend) as usize;
    if steep == true {
        draw_pixel_with_opacity(y_pixel1, x_pixel1, &colour, rfract(yend) * xgap);
        draw_pixel_with_opacity(y_pixel1 + 1, x_pixel1, &colour, fract(yend) * xgap);
    } else {
        draw_pixel_with_opacity(x_pixel1, y_pixel1, &colour, rfract(yend) * xgap);
        draw_pixel_with_opacity(x_pixel1, y_pixel1 + 1, &colour, fract(yend) * xgap);
    }

    let mut inter_y = yend + gradient;  // first y-intersection for the loop

    // handle second endpoint
    let xend = hround(x1);
    let yend = y1 + gradient * (xend - x1);
    let xgap = fract(x1 + 0.5);
    let x_pixel2 = xend as usize;
    let y_pixel2 = trunc(yend) as usize;
    if steep == true {
        draw_pixel_with_opacity(y_pixel2, x_pixel2, &colour, rfract(yend) * xgap);
        draw_pixel_with_opacity(y_pixel2 + 1, x_pixel2, &colour, fract(yend) * xgap);
    } else {
        draw_pixel_with_opacity(x_pixel2, y_pixel2, &colour, rfract(yend) * xgap);
        draw_pixel_with_opacity(x_pixel2, y_pixel2 + 1, &colour, fract(yend) * xgap);
    }

    // main draw loop
    if steep == true {
        for x in (x_pixel1 + 1)..x_pixel2 {
            draw_pixel_with_opacity(trunc(inter_y) as usize, x, &colour, rfract(inter_y));
            draw_pixel_with_opacity(trunc(inter_y) as usize + 1, x, &colour, fract(inter_y));
            inter_y += gradient;
        }
    } else {
        for x in (x_pixel1 + 1)..x_pixel2 {
            draw_pixel_with_opacity(x, trunc(inter_y) as usize, &colour, rfract(inter_y));
            draw_pixel_with_opacity(x, trunc(inter_y) as usize + 1, &colour, fract(inter_y));
            inter_y += gradient;
        }
    }
}

/// Super dumb implementation of a float fractional part calculation
fn fract(a: f64) -> f64 {
    let mut b = a;
    if b > 0.0 {
        while b >= 1.0 {
            b -= 1.0;
        }
    } else if b < 0.0 {
        while b <= -1.0 {
            b += 1.0;
        }
    }
    b
}

/// Super dumb implementation of a truncate function for floats
fn trunc(a: f64) -> f64 {
    a - fract(a)
}

fn hround(a: f64) -> f64 {
    trunc(a + 0.5)
}

fn rfract(a: f64) -> f64 {
    1.0 - fract(a)
}
