use x86_64::registers::control::Cr2;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame };
use crate::{println};

use lazy_static::lazy_static;

use crate::gdt;

use pic8259::ChainedPics;


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

		idt.breakpoint.set_handler_fn(breakpoint_handler);
		idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
		idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(crate::driver::keyboard::keyboard_handler);
		idt.page_fault.set_handler_fn(page_fault_handler);

		unsafe {
			idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
		}

		idt
	};
}

pub fn init_idt() {
	IDT.load();
}

extern "x86-interrupt" fn timer_handler(
		_stack_frame: InterruptStackFrame
) {
	unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    panic!("EXCEPTION: PAGE FAULT\nAttempted to access: {:?}\nError kind: {:?}\n{:#?}", Cr2::read(), error_code, stack_frame)
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
	)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
	stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
	panic!("FATAL DOUBLE FAULT ERR:\n{:#?}", stack_frame)
}
