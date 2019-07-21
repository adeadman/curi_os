use crate::{gdt, hlt_loop, print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};

#[cfg(test)]
use crate::{serial_print, serial_println};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Sys Interrupt 0 - Divide By Zero
        idt.device_not_available.set_handler_fn(divide_by_zero_handler);
        // Sys Interrupt 1 - Debug
        idt.device_not_available.set_handler_fn(debug_handler);
        // Sys Interrupt 2 - Non-Maskable Interrupt
        idt.device_not_available.set_handler_fn(non_maskable_interrupt_handler);
        // Sys Interrupt 3 - Breakpoint Interrupt
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // Sys Interrupt 4 - Overflow
        idt.device_not_available.set_handler_fn(overflow_handler);
        // Sys Interrupt 5 - Bound Range Exceeded
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        // Sys Interrupt 6 - Invalid Opcode
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        // Sys Interrupt 7 - Device Not Available
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        // Sys Interrupt 8 - Double Fault
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Sys Interrupt 10 - Invalid TSS
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        // Sys Interrupt 11 - Segment Not Present
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        // Sys Interrupt 12 - Stack Segment Fault
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        // Sys Interrupt 13 - General Protection Fault
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        // Sys Interrupt 14 - Page Fault
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Sys Interrupt 16 - X87 Floating Point Exception
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        // Sys Interrupt 17 - Alignment Check
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        // Sys Interrupt 18 - Machine Check
        idt.machine_check.set_handler_fn(machine_check_handler);
        // Sys Interrupt 19 - SIMD Floating Point
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        // Sys Interrupt 20 - Virtualization
        idt.virtualization.set_handler_fn(virtualization_handler);
        // Sys Interrupt 30 - Security Exception
        idt.security_exception.set_handler_fn(security_exception_handler);
        // PIC Interrupt 0 - Timer
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        // PIC Interrupt 1 - Keyboard
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

///////////////////////////////////////////////
/// Interrupt Handlers
///////////////////////////////////////////////
extern "x86-interrupt" fn divide_by_zero_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: DEVICE_NOT_AVAILABLE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: DOUBLEFAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: PageFaultErrorCode
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn x87_floating_point_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: ALIGNMENT CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn machine_check_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_floating_point_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn security_exception_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("EXCEPTION: SECURITY EXCEPTION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    //print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    use pc_keyboard::{Keyboard, ScancodeSet1, DecodedKey, layouts, HandleControl};
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::MapLettersToUnicode));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

///////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////
#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception...");
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}
