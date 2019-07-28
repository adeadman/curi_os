use x86_64::instructions::port::Port;

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub left_button: bool,
    pub middle_button: bool,
    pub right_button: bool,
    pub delta_x: i32,
    pub delta_y: i32,
}

enum MousePortStatus {
    Data,
    Signal,
}

/// ```text
/// █░░░░░░░░░░░░░░░
/// ██░░░░░░░░░░░░░░
/// ███░░░░░░░░░░░░░
/// ████░░░░░░░░░░░░
/// █████░░░░░░░░░░░
/// ██████░░░░░░░░░░
/// ███████░░░░░░░░░
/// ████████░░░░░░░░
/// █████████░░░░░░░
/// ██████████░░░░░░
/// ██████░░░░░░░░░░
/// ██░███░░░░░░░░░░
/// █░░░███░░░░░░░░░
/// ░░░░███░░░░░░░░░
/// ░░░░░███░░░░░░░░
/// ░░░░░███░░░░░░░░
/// ```
pub const CURSOR_BITMAP: [u16; 16] = [
    0b1000_0000_0000_0000,  // 1
    0b1100_0000_0000_0000,  // 2
    0b1110_0000_0000_0000,  // 3
    0b1111_0000_0000_0000,  // 4
    0b1111_1000_0000_0000,  // 5
    0b1111_1100_0000_0000,  // 6
    0b1111_1110_0000_0000,  // 7
    0b1111_1111_0000_0000,  // 8
    0b1111_1111_1000_0000,  // 9
    0b1111_1111_1100_0000,  // 10
    0b1111_1100_0000_0000,  // 11
    0b1101_1100_0000_0000,  // 12
    0b1000_1110_0000_0000,  // 13
    0b0000_1110_0000_0000,  // 14
    0b0000_0111_0000_0000,  // 15
    0b0000_0111_0000_0000,  // 16
];

fn mouse_wait_for_status(status: MousePortStatus) {
    let mut time_out: u32 = 100000;
    let mut port = Port::new(0x64_u16);

    match status {
        MousePortStatus::Data => {
            while time_out > 0 {
                let value: u8 = unsafe { port.read() };
                if (value & 1) == 1 {
                    return;
                }
                time_out -= 1;
            }
        },
        MousePortStatus::Signal => {
            while time_out > 0 {
                let value: u8 = unsafe { port.read() };
                if (value & 2) == 0 {
                    return;
                }
                time_out -= 1;
            }
        }
    }
}

pub fn mouse_write(data: u8) {
    let mut command_status_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);

    // wait until we can send a command
    mouse_wait_for_status(MousePortStatus::Signal);
    // tell mouse we are going to send a command
    unsafe { command_status_port.write(0xd4_u8); }
    // wait until we can write again
    mouse_wait_for_status(MousePortStatus::Signal);
    // write the data
    unsafe { data_port.write(data); }
}

pub fn mouse_read() -> u8 {
    let mut data_port = Port::new(0x60);
    mouse_wait_for_status(MousePortStatus::Data);
    unsafe { data_port.read() }
}

pub fn init_mouse() {
    let mut command_status_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);

    // enable the auxiliary mouse device
    mouse_wait_for_status(MousePortStatus::Signal);
    unsafe { command_status_port.write(0xa8_u8); }

    // enable the interrupts
    mouse_wait_for_status(MousePortStatus::Signal);
    unsafe { command_status_port.write(0x20_u8); }
    mouse_wait_for_status(MousePortStatus::Data);
    let mouse_status: u8 = unsafe { data_port.read() | 2 };
    mouse_wait_for_status(MousePortStatus::Signal);
    unsafe { command_status_port.write(0x60_u8); }
    mouse_wait_for_status(MousePortStatus::Signal);
    unsafe { data_port.write(mouse_status); }

    // tell the mouse to use default settings
    mouse_write(0xf6);
    mouse_read();  // acknowledge

    // enable the mouse
    mouse_write(0xf4);
    mouse_read();  // acknowledge
}
