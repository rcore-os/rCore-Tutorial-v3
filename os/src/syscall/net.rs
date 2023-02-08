use alloc::sync::Arc;
use crate::net::IPv4;
use crate::net::udp::UDP;
use crate::task::current_process;

// just support udp
pub fn sys_connect(raddr: u32, lport: u16, rport: u16) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let fd = inner.alloc_fd();
    let udp_node = UDP::new(IPv4::from_u32(raddr), lport, rport);
    inner.fd_table[fd] = Some(Arc::new(udp_node));
    fd as isize    
}