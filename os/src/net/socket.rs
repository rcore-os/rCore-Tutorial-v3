use alloc::collections::VecDeque;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use lose_net_stack::IPv4;

use crate::sync::UPIntrFreeCell;

// TODO: specify the protocol, TCP or UDP
pub struct Socket {
    pub raddr: IPv4,                // remote address
    pub lport: u16,                 // local port
    pub rport: u16,                 // rempote port
    pub buffers: VecDeque<Vec<u8>>, // datas
    pub seq: u32,
    pub ack: u32,
}

lazy_static! {
    static ref SOCKET_TABLE: UPIntrFreeCell<Vec<Option<Socket>>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

/// get the seq and ack by socket index
pub fn get_s_a_by_index(index: usize) -> Option<(u32, u32)> {
    let socket_table = SOCKET_TABLE.exclusive_access();

    assert!(index < socket_table.len());

    socket_table.get(index).map_or(None, |x| match x {
        Some(x) => Some((x.seq, x.ack)),
        None => None,
    })
}

pub fn set_s_a_by_index(index: usize, seq: u32, ack: u32) {
    let mut socket_table = SOCKET_TABLE.exclusive_access();

    assert!(socket_table.len() > index);
    assert!(socket_table[index].is_some());

    let sock = socket_table[index].as_mut().unwrap();

    sock.ack = ack;
    sock.seq = seq;
}

pub fn get_socket(raddr: IPv4, lport: u16, rport: u16) -> Option<usize> {
    let socket_table = SOCKET_TABLE.exclusive_access();
    for i in 0..socket_table.len() {
        let sock = &socket_table[i];
        if sock.is_none() {
            continue;
        }

        let sock = sock.as_ref().unwrap();
        if sock.raddr == raddr && sock.lport == lport && sock.rport == rport {
            return Some(i);
        }
    }
    None
}

pub fn add_socket(raddr: IPv4, lport: u16, rport: u16) -> Option<usize> {
    if get_socket(raddr, lport, rport).is_some() {
        return None;
    }

    let mut socket_table = SOCKET_TABLE.exclusive_access();
    let mut index = usize::MAX;
    for i in 0..socket_table.len() {
        if socket_table[i].is_none() {
            index = i;
            break;
        }
    }

    let socket = Socket {
        raddr,
        lport,
        rport,
        buffers: VecDeque::new(),
        seq: 0,
        ack: 0,
    };

    if index == usize::MAX {
        socket_table.push(Some(socket));
        Some(socket_table.len() - 1)
    } else {
        socket_table[index] = Some(socket);
        Some(index)
    }
}

pub fn remove_socket(index: usize) {
    let mut socket_table = SOCKET_TABLE.exclusive_access();

    assert!(socket_table.len() > index);

    socket_table[index] = None;
}

pub fn push_data(index: usize, data: Vec<u8>) {
    let mut socket_table = SOCKET_TABLE.exclusive_access();

    assert!(socket_table.len() > index);
    assert!(socket_table[index].is_some());

    socket_table[index]
        .as_mut()
        .unwrap()
        .buffers
        .push_back(data);
}

pub fn pop_data(index: usize) -> Option<Vec<u8>> {
    let mut socket_table = SOCKET_TABLE.exclusive_access();

    assert!(socket_table.len() > index);
    assert!(socket_table[index].is_some());

    socket_table[index].as_mut().unwrap().buffers.pop_front()
}
