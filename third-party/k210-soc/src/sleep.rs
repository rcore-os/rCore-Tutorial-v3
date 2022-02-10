use super::sysctl;
use riscv::register::time;

pub fn time_sleep(n: usize) {
    let start = time::read();
    while time::read() < start + n {}
}

pub fn usleep(n: usize) {
    let freq = sysctl::clock_get_freq(sysctl::clock::CPU) as usize / 62;
    time_sleep(freq * n / 1000000);
}
