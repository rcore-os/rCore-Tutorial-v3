pub const CLOCK_FREQ: usize = 12500000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000), // core local interrupter (CLINT)
    (0xc000000, 0x210000), // VIRT_PLIC in virt machine
    (0x10000000, 0x9000),  // VIRT_UART0 with GPU  in virt machine
];

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;
pub type CharDeviceImpl = crate::drivers::chardev::NS16550a<VIRT_UART>;

pub const VIRT_PLIC: usize = 0xC00_0000;
pub const VIRT_UART: usize = 0x1000_0000;
#[allow(unused)]
pub const VIRTGPU_XRES: u32 = 1280;
#[allow(unused)]
pub const VIRTGPU_YRES: u32 = 800;

use crate::drivers::block::BLOCK_DEVICE;
use crate::drivers::chardev::{CharDevice, UART};
use crate::drivers::plic::{IntrTargetPriority, PLIC};
use crate::drivers::{KEYBOARD_DEVICE, MOUSE_DEVICE};

pub fn device_init() {
    use riscv::register::sie;
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let hart_id: usize = 0;
    let supervisor = IntrTargetPriority::Supervisor;
    let machine = IntrTargetPriority::Machine;
    plic.set_threshold(hart_id, supervisor, 0);
    plic.set_threshold(hart_id, machine, 1);
    //irq nums: 5 keyboard, 6 mouse, 8 block, 10 uart
    for intr_src_id in [5usize, 6, 8, 10] {
        plic.enable(hart_id, supervisor, intr_src_id);
        plic.set_priority(intr_src_id, 1);
    }
    unsafe {
        sie::set_sext();
    }
}

pub fn irq_handler() {
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
    match intr_src_id {
        5 => KEYBOARD_DEVICE.handle_irq(),
        6 => MOUSE_DEVICE.handle_irq(),
        8 => BLOCK_DEVICE.handle_irq(),
        10 => UART.handle_irq(),
        _ => panic!("unsupported IRQ {}", intr_src_id),
    }
    plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}


// core local interrupter (CLINT), which contains the timer
pub const CLINT: usize = 0x2000000;
// pub const fn clint_mtimecmp(hartid: usize) -> usize {
//     CLINT + 0x4000 + 8 * hartid
// }
pub const CLINT_MTIME: usize = CLINT + 0xBFF8; // Cycles since boot.
pub const CLINT_MTIMECMP: usize = CLINT + 0x4000;

#[naked]
#[repr(align(16))] // if miss this alignment, a load access fault will occur.
#[no_mangle]
pub unsafe extern "C" fn timervec() -> ! {
    // start.rs has set up the memory that mscratch points to:
    // scratch[0,8,16] : register save area.
    // scratch[24] : address of CLINT's MTIMECMP register.
    // scratch[32] : desired interval between interrupts.

    // Now, mscrach has a pointer to an additional scratch space.
    // to aboid overwriting the contents of the integer registers,
    // the prologue of an interrupts handler usually begins by swapping
    // an integer register(say a0) with mscratch CSR.
    // The interrupt handler stores the integer registers
    // used for processing in this scratch space.
    // a0 saved in mscrach, a1 ~ a3 saved in scratch space.
    //loop {}
    asm!(
        "csrrw a0, mscratch, a0",
        "sd a1, 0(a0)",
        "sd a2, 8(a0)",
        "sd a3, 16(a0)",
        // schedule the next timer interrupt
        // by adding interval to mtimecmp.
        "ld a1, 24(a0)", // CLINT_MTIMECMP(hartid) contents
        "ld a2, 32(a0)", // interval
        "ld a3, 0(a1)",
        "add a3, a3, a2",
        "sd a3, 0(a1)",
        // raise a supervisor software interrupt.
        "li a1, 2",
        "csrw sip, a1",
        // restore and return
        "ld a3, 16(a0)",
        "ld a2, 8(a0)",
        "ld a1, 0(a0)",
        "csrrw a0, mscratch, a0",
        "mret",
        options(noreturn)
    );
}
