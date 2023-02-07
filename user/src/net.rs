use super::*;

pub fn connect(ip: u32, sport: u16, dport: u16) -> isize {
    sys_connect(ip, sport, dport)
}