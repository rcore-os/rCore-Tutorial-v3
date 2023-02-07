pub mod udp;
pub mod socket;

pub use lose_net_stack::IPv4;

use alloc::{vec, sync::Arc};
use lose_net_stack::{LoseStack, MacAddress, results::Packet};

use crate::{drivers::NET_DEVICE, sync::UPIntrFreeCell, net::socket::{get_socket, push_data}};

pub struct NetStack(UPIntrFreeCell<LoseStack>);

impl NetStack {
    pub fn new() -> Self {
        unsafe {
            NetStack(UPIntrFreeCell::new(LoseStack::new(
                IPv4::new(10, 0, 2, 15),
                MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]) 
            )))
        }
    }
}

lazy_static::lazy_static! {
    static ref LOSE_NET_STACK: Arc<NetStack> = Arc::new(NetStack::new());
}


pub fn net_interrupt_handler() {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);
        
    let packet = LOSE_NET_STACK.0.exclusive_access().analysis(&recv_buf[..len]);
    
    // println!("[kernel] receive a packet");
    // hexdump(&recv_buf[..len]);

    match packet {
        Packet::ARP(arp_packet) => {
            let lose_stack = LOSE_NET_STACK.0.exclusive_access();
            let reply_packet = arp_packet.reply_packet(lose_stack.ip, lose_stack.mac).expect("can't build reply");
            let reply_data = reply_packet.build_data();
            NET_DEVICE.transmit(&reply_data)
        },

        Packet::UDP(udp_packet) => {
            let target = udp_packet.source_ip;
            let lport = udp_packet.dest_port;
            let rport = udp_packet.source_port;
            
            if let Some(socket_index) = get_socket(target, lport, rport) {
                push_data(socket_index, udp_packet.data.to_vec());
            }
        }
        _ => {}
    }
}

#[allow(unused)]
pub fn hexdump(data: &[u8]) {
    const PRELAND_WIDTH: usize = 70;
    println!("[kernel] {:-^1$}", " hexdump ", PRELAND_WIDTH);
    for offset in (0..data.len()).step_by(16) {
        print!("[kernel] ");
        for i in 0..16 {
            if offset + i < data.len() {
                print!("{:02x} ", data[offset + i]);
            } else {
                print!("{:02} ", "");
            }
        }

        print!("{:>6}", ' ');

        for i in 0..16 {
            if offset + i < data.len() {
                let c = data[offset + i];
                if c >= 0x20 && c <= 0x7e {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            } else {
                print!("{:02} ", "");
            }
        }
        
        println!("");
    }
    println!("[kernel] {:-^1$}", " hexdump end ", PRELAND_WIDTH);
}