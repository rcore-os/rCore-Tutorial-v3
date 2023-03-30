use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;

use crate::fs::File;
use crate::sync::UPIntrFreeCell;
use crate::task::TaskControlBlock;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
    pub receivable: bool,
    pub schedule: Option<Arc<TaskControlBlock>>,
}

lazy_static! {
    static ref LISTEN_TABLE: UPIntrFreeCell<Vec<Option<Port>>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub fn listen(port: u16) -> Option<usize> {
    let mut listen_table = LISTEN_TABLE.exclusive_access();
    let mut index = usize::MAX;
    for i in 0..listen_table.len() {
        if listen_table[i].is_none() {
            index = i;
            break;
        }
    }

    let listen_port = Port {
        port,
        receivable: false,
        schedule: None,
    };

    if index == usize::MAX {
        listen_table.push(Some(listen_port));
        Some(listen_table.len() - 1)
    } else {
        listen_table[index] = Some(listen_port);
        Some(index)
    }
}

// can accept request
pub fn accept(listen_index: usize, task: Arc<TaskControlBlock>) {
    let mut listen_table = LISTEN_TABLE.exclusive_access();
    assert!(listen_index < listen_table.len());
    let listen_port = listen_table[listen_index].as_mut();
    assert!(listen_port.is_some());
    let listen_port = listen_port.unwrap();
    listen_port.receivable = true;
    listen_port.schedule = Some(task);
}

pub fn port_acceptable(listen_index: usize) -> bool {
    let mut listen_table = LISTEN_TABLE.exclusive_access();
    assert!(listen_index < listen_table.len());

    let listen_port = listen_table[listen_index].as_mut();
    listen_port.map_or(false, |x| x.receivable)
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<()> {
    LISTEN_TABLE.exclusive_session(|listen_table| {
        let mut listen_ports: Vec<&mut Option<Port>> = listen_table
            .iter_mut()
            .filter(|x| match x {
                Some(t) => t.port == port && t.receivable == true,
                None => false,
            })
            .collect();
        if listen_ports.len() == 0 {
            None
        } else {
            let listen_port = listen_ports[0].as_mut().unwrap();
            let task = listen_port.schedule.clone().unwrap();
            // wakeup_task(Arc::clone(&listen_port.schedule.clone().unwrap()));
            listen_port.schedule = None;
            listen_port.receivable = false;

            accept_connection(port, tcp_packet, task);
            Some(())
        }
    })
}

pub fn accept_connection(_port: u16, tcp_packet: &TCPPacket, task: Arc<TaskControlBlock>) {
    let process = task.process.upgrade().unwrap();
    let mut inner = process.inner_exclusive_access();
    let fd = inner.alloc_fd();

    let tcp_socket = TCP::new(
        tcp_packet.source_ip,
        tcp_packet.dest_port,
        tcp_packet.source_port,
        tcp_packet.seq,
        tcp_packet.ack,
    );

    inner.fd_table[fd] = Some(Arc::new(tcp_socket));

    let cx = task.inner_exclusive_access().get_trap_cx();
    cx.x[10] = fd;
}

// store in the fd_table, delete the listen table when close the application.
pub struct PortFd(usize);

impl PortFd {
    pub fn new(port_index: usize) -> Self {
        PortFd(port_index)
    }
}

impl Drop for PortFd {
    fn drop(&mut self) {
        LISTEN_TABLE.exclusive_access()[self.0] = None
    }
}

impl File for PortFd {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, _buf: crate::mm::UserBuffer) -> usize {
        0
    }

    fn write(&self, _buf: crate::mm::UserBuffer) -> usize {
        0
    }
}
