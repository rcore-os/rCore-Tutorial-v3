use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::arch::global_asm;
use trace_lib::Symbol;


#[derive(Debug, Clone)]
pub struct SymbolEntry {
    addr: usize,
    symbol_str: String,
}


impl SymbolEntry {
    pub fn new(addr: usize, symbol_str: String) -> Self {
        Self { addr, symbol_str }
    }
}

impl Symbol for SymbolEntry  {
    fn addr(&self) -> usize {
        self.addr
    }
    fn name(&self) -> &str {
        &self.symbol_str
    }
}

extern "C" {
    fn symbol_num();
    fn symbol_address();
    fn symbol_index();
    fn symbol_name();
}

global_asm!(include_str!("kernel_symbol.S"));
pub fn init_kernel_trace() -> Vec<SymbolEntry> {
    let symbol_num_addr = symbol_num as usize as *const usize;
    let symbol_num = unsafe { symbol_num_addr.read_volatile() };
    let mut symbol_info = Vec::new();
    if symbol_num==0 {
        return symbol_info;
    }
    let symbol_addr = symbol_address as usize as *const usize; //符号地址存储区域
    let addr_data = unsafe { core::slice::from_raw_parts(symbol_addr, symbol_num) };
    let symbol_index = symbol_index as usize as *const usize; //符号字符串的起始位置
    let index_data = unsafe { core::slice::from_raw_parts(symbol_index, symbol_num) };
    let symbol_name = symbol_name as usize as *const u8; //符号字符串
    for i in 0..symbol_num - 1 {
        let name = unsafe {
            core::slice::from_raw_parts(
                symbol_name.add(index_data[i]),
                index_data[i + 1] - index_data[i],
            )
        };
        let name = core::str::from_utf8(name).unwrap();
        symbol_info.push(SymbolEntry::new(addr_data[i], name.to_string()));
    }
    let mut name: Vec<u8> = Vec::new();
    unsafe {
        for i in index_data[symbol_num - 1].. {
            let c = symbol_name.add(i);
            if *c == 0 {
                break;
            }
            name.push(*c);
        }
    }
    let name = core::str::from_utf8(&name).unwrap();
    symbol_info.push(SymbolEntry::new(
        addr_data[symbol_num - 1],
        name.to_string(),
    ));
    symbol_info
}
