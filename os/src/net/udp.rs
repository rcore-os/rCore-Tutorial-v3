use alloc::vec;
use lose_net_stack::MacAddress;
use lose_net_stack::packets::udp::UDPPacket;
use lose_net_stack::IPv4;
use crate::fs::File;
use super::net_interrupt_handler;
use super::socket::{add_socket, remove_socket, pop_data};
use super::LOSE_NET_STACK;
use super::NET_DEVICE;

pub struct UDP{
    pub target: IPv4,
    pub sport: u16,
    pub dport: u16,
    pub socket_index: usize
}

impl UDP {
    pub fn new(target: IPv4, sport: u16, dport: u16) -> Self {
        let index = add_socket(target, sport, dport).expect("can't add socket");

        Self {
            target,
            sport,
            dport,
            socket_index: index
        }
    }
}

impl File for UDP {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
        loop {
            if let Some(data) = pop_data(self.socket_index) {
                let data_len = data.len();
                let mut left = 0;
                for i in 0..buf.buffers.len() {
                    let buffer_i_len = buf.buffers[i].len().min(data_len - left);
                    
                    buf.buffers[i][..buffer_i_len].copy_from_slice(&data[left..(left + buffer_i_len)]);
    
                    left += buffer_i_len;
                    if left == data_len {
                        break;
                    }
                }
                return left;
            } else {
                net_interrupt_handler();
            }
        }
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        let lose_net_stack = LOSE_NET_STACK.0.exclusive_access();

        let mut data = vec![0u8; buf.len()];
        
        let mut left = 0;
        for i in 0..buf.buffers.len() {
            data[left..(left + buf.buffers[i].len())].copy_from_slice(buf.buffers[i]);
            left += buf.buffers[i].len();
        }

        let len = data.len();

        let udp_packet = UDPPacket::new(
            lose_net_stack.ip, 
            lose_net_stack.mac, 
            self.sport, 
            self.target, 
            MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]), 
            self.dport, 
            len, 
            data.as_ref()
        );
        NET_DEVICE.transmit(&udp_packet.build_data());
        len
    }
}

impl Drop for UDP {
    fn drop(&mut self) {
        remove_socket(self.socket_index)
    }
}