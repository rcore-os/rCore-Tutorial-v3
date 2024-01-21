pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
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
