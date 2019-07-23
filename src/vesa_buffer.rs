use core::{fmt, slice};
use crate::num_traits::float::FloatCore;
use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(test)]
use crate::{serial_print, serial_println};

const BUFFER_HEIGHT: usize = 75;
const BUFFER_WIDTH: usize = 100;
const SCREEN_HEIGHT: usize = 600;
const SCREEN_WIDTH: usize = 800;
const VGA_BUFFER: *mut u16 = (500 * 512 * 4096) as *mut _;

pub struct Writer {
    xpos: usize,
    ypos: usize,
    vga_buffer: &'static mut [u16],
}

impl Writer {
    pub fn clear_screen(&mut self) {
        for pixel in 0..self.vga_buffer.len() {
            self.vga_buffer[pixel] = 0x0;  // 66;
        }
        self.xpos = 0;
        self.ypos = 0;
    }
    fn newline(&mut self) {
        self.ypos += 8;
        self.xpos = 0;
        if self.ypos >= SCREEN_HEIGHT {
            self.clear_screen();
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
                        let color = 0xffff;
                        self.vga_buffer[(self.ypos + y) * SCREEN_WIDTH + self.xpos + x] = color;
                    }
                }
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
        vga_buffer: unsafe { slice::from_raw_parts_mut(VGA_BUFFER, SCREEN_WIDTH * SCREEN_HEIGHT) }
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

    pub fn as_u16(&self) -> u16 {
        ((self.red as u16) << 11) +
            ((self.green as u16) << 5) +
            self.blue as u16
    }
}

pub const RED: Colour16Bit = Colour16Bit { red:0x1f, green:0x0, blue:0x0 };
pub const GREEN: Colour16Bit  = Colour16Bit { red:0x0, green:0x3f, blue:0x0 };
pub const BLUE: Colour16Bit = Colour16Bit { red:0x0, green:0x0, blue:0x1f };
pub const WHITE: Colour16Bit = Colour16Bit{ red:0x1f, green:0x3f, blue:0x1f };
pub const BLACK: Colour16Bit = Colour16Bit{ red:0x0, green:0x0, blue:0x0 };

pub fn draw_pixel(x: usize, y: usize, colour: &Colour16Bit) {
    let vga_buffer: &mut [u16] = unsafe {
        slice::from_raw_parts_mut(VGA_BUFFER, SCREEN_WIDTH * SCREEN_HEIGHT)
    };
    let pixel_offset: usize = y * 800 + x;
    vga_buffer[pixel_offset] = colour.as_u16();
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
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

pub fn draw_pixel_with_brightness(
    x: usize, y: usize,
    colour: &Colour16Bit,
    brightness: f64
) {
    assert!(brightness >= 0.0 && brightness <= 1.0,
            "draw_pixel_with_brightness: Bad brightness of {}", brightness);
    match brightness {
        1.0 => draw_pixel(x, y, &colour),
        0.0 => draw_pixel(x, y, &BLACK),
        _ => draw_pixel(x, y, &Colour16Bit{
            red: (colour.red as f64 * brightness) as u8,
            green: (colour.green as f64 * brightness) as u8,
            blue: (colour.blue as f64 * brightness) as u8,
        }),
    }
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
        draw_pixel_with_brightness(y_pixel1, x_pixel1, &colour, rfract(yend) * xgap);
        draw_pixel_with_brightness(y_pixel1 + 1, x_pixel1, &colour, fract(yend) * xgap);
    } else {
        draw_pixel_with_brightness(x_pixel1, y_pixel1, &colour, rfract(yend) * xgap);
        draw_pixel_with_brightness(x_pixel1, y_pixel1 + 1, &colour, fract(yend) * xgap);
    }

    let mut inter_y = yend + gradient;  // first y-intersection for the loop

    // handle second endpoint
    let xend = hround(x1);
    let yend = y1 + gradient * (xend - x1);
    let xgap = fract(x1 + 0.5);
    let x_pixel2 = xend as usize;
    let y_pixel2 = trunc(yend) as usize;
    if steep == true {
        draw_pixel_with_brightness(y_pixel2, x_pixel2, &colour, rfract(yend) * xgap);
        draw_pixel_with_brightness(y_pixel2 + 1, x_pixel2, &colour, fract(yend) * xgap);
    } else {
        draw_pixel_with_brightness(x_pixel2, y_pixel2, &colour, rfract(yend) * xgap);
        draw_pixel_with_brightness(x_pixel2, y_pixel2 + 1, &colour, fract(yend) * xgap);
    }

    // main draw loop
    if steep == true {
        for x in (x_pixel1 + 1)..x_pixel2 {
            draw_pixel_with_brightness(trunc(inter_y) as usize, x, &colour, rfract(inter_y));
            draw_pixel_with_brightness(trunc(inter_y) as usize + 1, x, &colour, fract(inter_y));
            inter_y += gradient;
        }
    } else {
        for x in (x_pixel1 + 1)..x_pixel2 {
            draw_pixel_with_brightness(x, trunc(inter_y) as usize, &colour, rfract(inter_y));
            draw_pixel_with_brightness(x, trunc(inter_y) as usize + 1, &colour, fract(inter_y));
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
