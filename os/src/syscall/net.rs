use crate::net::port_table::{accept, listen, port_acceptable, PortFd};
use crate::net::udp::UDP;
use crate::net::{net_interrupt_handler, IPv4};
use crate::task::{current_process, current_task, current_trap_cx};
use alloc::sync::Arc;

// just support udp
pub fn sys_connect(raddr: u32, lport: u16, rport: u16) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let fd = inner.alloc_fd();
    let udp_node = UDP::new(IPv4::from_u32(raddr), lport, rport);
    inner.fd_table[fd] = Some(Arc::new(udp_node));
    fd as isize
}

// listen a port
pub fn sys_listen(port: u16) -> isize {
    match listen(port) {
        Some(port_index) => {
            let process = current_process();
            let mut inner = process.inner_exclusive_access();
            let fd = inner.alloc_fd();
            let port_fd = PortFd::new(port_index);
            inner.fd_table[fd] = Some(Arc::new(port_fd));

            // NOTICE: this return the port index, not the fd
            port_index as isize
        }
        None => -1,
    }
}

// accept a tcp connection
pub fn sys_accept(port_index: usize) -> isize {
    println!("accepting port {}", port_index);

    let task = current_task().unwrap();
    accept(port_index, task);
    // block_current_and_run_next();

    // NOTICE: There does not have interrupt handler, just call it munually.
    loop {
        net_interrupt_handler();

        if !port_acceptable(port_index) {
            break;
        }
    }

    let cx = current_trap_cx();
    cx.x[10] as isize
}
