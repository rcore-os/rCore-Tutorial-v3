pub mod port_table;
pub mod socket;
pub mod tcp;
pub mod udp;

pub use lose_net_stack::IPv4;

use alloc::{sync::Arc, vec};
use lose_net_stack::{results::Packet, LoseStack, MacAddress, TcpFlags};

use crate::{
    drivers::NET_DEVICE,
    net::socket::{get_socket, push_data},
    sync::UPIntrFreeCell,
};

use self::{port_table::check_accept, socket::set_s_a_by_index};

pub struct NetStack(UPIntrFreeCell<LoseStack>);

impl NetStack {
    pub fn new() -> Self {
        unsafe {
            NetStack(UPIntrFreeCell::new(LoseStack::new(
                IPv4::new(10, 0, 2, 15),
                MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
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

    let packet = LOSE_NET_STACK
        .0
        .exclusive_access()
        .analysis(&recv_buf[..len]);

    // println!("[kernel] receive a packet");
    // hexdump(&recv_buf[..len]);

    match packet {
        Packet::ARP(arp_packet) => {
            let lose_stack = LOSE_NET_STACK.0.exclusive_access();
            let reply_packet = arp_packet
                .reply_packet(lose_stack.ip, lose_stack.mac)
                .expect("can't build reply");
            let reply_data = reply_packet.build_data();
            NET_DEVICE.transmit(&reply_data)
        }

        Packet::UDP(udp_packet) => {
            let target = udp_packet.source_ip;
            let lport = udp_packet.dest_port;
            let rport = udp_packet.source_port;

            if let Some(socket_index) = get_socket(target, lport, rport) {
                push_data(socket_index, udp_packet.data.to_vec());
            }
        }

        Packet::TCP(tcp_packet) => {
            let target = tcp_packet.source_ip;
            let lport = tcp_packet.dest_port;
            let rport = tcp_packet.source_port;
            let flags = tcp_packet.flags;

            if flags.contains(TcpFlags::S) {
                // if it has a port to accept, then response the request
                if check_accept(lport, &tcp_packet).is_some() {
                    let mut reply_packet = tcp_packet.ack();
                    reply_packet.flags = TcpFlags::S | TcpFlags::A;
                    NET_DEVICE.transmit(&reply_packet.build_data());
                }
                return;
            } else if tcp_packet.flags.contains(TcpFlags::F) {
                // tcp disconnected
                let reply_packet = tcp_packet.ack();
                NET_DEVICE.transmit(&reply_packet.build_data());

                let mut end_packet = reply_packet.ack();
                end_packet.flags |= TcpFlags::F;
                NET_DEVICE.transmit(&end_packet.build_data());
            } else if tcp_packet.flags.contains(TcpFlags::A) && tcp_packet.data_len == 0 {
                return;
            }

            if let Some(socket_index) = get_socket(target, lport, rport) {
                push_data(socket_index, tcp_packet.data.to_vec());
                set_s_a_by_index(socket_index, tcp_packet.seq, tcp_packet.ack);
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
