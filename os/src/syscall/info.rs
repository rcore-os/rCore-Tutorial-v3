use crate::sync::UPSafeCell;
use core::arch::asm;
use lazy_static::*;

pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_GETTASKINFO: usize = 100;

/// max number of syscalls
pub const SYSTEM_CALL_MAX_NUM: usize = 1024;
/// number of current syscalls
pub const SYSTEM_CALL_NUM: usize = 3;
/// array of current syscalls
pub const SYSTEM_CALL_ARRAY: [usize; SYSTEM_CALL_NUM] =
    [SYSCALL_WRITE, SYSCALL_EXIT, SYSCALL_GETTASKINFO];

// time frequency of QEMU
pub const TIME_FREQUENCY_NS: u64 = 10;
pub const TIME_FREQUENCY_US: u64 = 10_000;
pub const TIME_FREQUENCY_S: u64 = 10_000_000;

/// syscall info
pub struct SyscallInfo {
    id: usize,
    name: &'static str,
    count: usize,
}

impl SyscallInfo {
    fn new(id: usize, name: &'static str) -> Self {
        SyscallInfo { id, name, count: 0 }
    }

    fn add_one(&mut self) {
        self.count += 1;
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_count(&self) -> usize {
        self.count
    }
}

pub struct SyscallTable {
    data: [SyscallInfo; SYSTEM_CALL_NUM],
}

impl Default for SyscallTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SyscallTable {
    pub fn new() -> Self {
        SyscallTable {
            data: [
                SyscallInfo::new(SYSCALL_WRITE, "syscall_write"),
                SyscallInfo::new(SYSCALL_EXIT, "syscall_exit"),
                SyscallInfo::new(SYSCALL_GETTASKINFO, "syscall_gettaskinfo"),
            ],
        }
    }

    pub fn count(&mut self, syscall_id: usize) {
        if let Some(idx) = self.get_idx(syscall_id) {
            self.data[idx].add_one();
        } else {
            panic!("Could not get syscall index correctly");
        }
    }

    pub fn get_info(&self, syscall_id: usize) -> &SyscallInfo {
        if let Some(idx) = self.get_idx(syscall_id) {
            &self.data[idx]
        } else {
            panic!("Could not get syscall index correctly");
        }
    }

    fn get_idx(&self, syscall_id: usize) -> Option<usize> {
        self.data.iter().position(|info| info.id == syscall_id)
    }
}

lazy_static! {
    pub static ref SYSTALL_TABLE: UPSafeCell<SyscallTable> =
        unsafe { UPSafeCell::new(SyscallTable::new()) };
}

/// print statistics of syscall
pub fn print_statistics() {
    for syscall_id in SYSTEM_CALL_ARRAY {
        let syscall_table = SYSTALL_TABLE.access();
        let info = syscall_table.get_info(syscall_id);
        println!(
            "syscall info: [ id: {}, name: {}, count: {} ]",
            info.get_id(),
            info.get_name(),
            info.get_count()
        );
    }
}

/// ExecuteTime
pub struct ExecuteTime {
    start: u64,
    end: u64,
}

impl Default for ExecuteTime {
    fn default() -> Self {
        ExecuteTime::new()
    }
}

impl ExecuteTime {
    pub fn new() -> Self {
        ExecuteTime { start: 0, end: 0 }
    }

    pub fn start(&mut self) {
        self.start = rdtime();
    }

    pub fn end(&mut self) {
        self.end = rdtime();
    }

    pub fn interval_ns(&self) -> u64 {
        (self.end - self.start) / TIME_FREQUENCY_NS
    }

    pub fn interval_us(&self) -> u64 {
        (self.end - self.start) / TIME_FREQUENCY_US
    }

    pub fn interval_s(&mut self) -> u64 {
        (self.end - self.start) / TIME_FREQUENCY_S
    }
}

lazy_static! {
    pub static ref EXECUTE_TIME: UPSafeCell<ExecuteTime> =
        unsafe { UPSafeCell::new(ExecuteTime::new()) };
}

// lazy_static! {
//     pub static ref START: UPSafeCell<u64> = unsafe { UPSafeCell::new(0) };
// }
//
// lazy_static! {
//     pub static ref END: UPSafeCell<u64> = unsafe { UPSafeCell::new(0) };
// }
//

pub fn rdtime() -> u64 {
    let time;
    unsafe {
        asm!("rdtime {}", out(reg) time);
    }
    time
}
