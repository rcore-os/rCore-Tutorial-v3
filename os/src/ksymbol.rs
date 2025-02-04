//! ksymbol

use alloc::string::{String, ToString};
use core::fmt::Debug;
use rustc_demangle::demangle;
use xmas_elf::{sections::SectionData::*, symbol_table::Entry};

core::arch::global_asm!(include_str!("ksymbol.S"));

extern "C" {
    fn _start_ksymbol_elf();
    fn _end_ksymbol_elf();
    fn stext();
    fn etext();
}

/// KernelFuncEntry describes a kernel func, which is especially useful
/// for dumping kernel symbols when backtracing the kernel stack.
pub struct KernelFuncEntry {
    func_name: &'static str,
    start_addr: usize,
    func_size: usize,
}

impl KernelFuncEntry {
    /// Mangled func name of the kernel func.
    fn func_name(&self) -> &'static str {
        self.func_name
    }

    /// Entry address of the kernel func.
    fn start_addr(&self) -> usize {
        self.start_addr
    }

    /// Size in bytes of the kernel func.
    fn func_size(&self) -> usize {
        self.func_size
    }
}

impl Debug for KernelFuncEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "KernelFuncEntry {{func_name: {}, entry_addr: {:#x}, func_size: {:#x} bytes}}",
            self.func_name(),
            self.start_addr(),
            self.func_size()
        )
    }
}

/// Return the [KernelFuncEntry] of a kernel func by which the input addr is covered.
///
/// If such kernel func cannot be found, return **None**.
pub fn kernel_func_by_addr(addr: usize) -> Option<KernelFuncEntry> {
    // SAFETY: The build process of os can guarantee that the kernel symbol elf
    // can be found between _start_ksymbol_elf and _end_ksymbol_elf. See os/src/build.rs
    // for more details.
    let elf_data = unsafe {
        core::slice::from_raw_parts(
            _start_ksymbol_elf as *const u8,
            _end_ksymbol_elf as usize - _start_ksymbol_elf as usize,
        )
    };
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let symtab = elf.find_section_by_name(".symtab").unwrap();
    let strtab = elf.find_section_by_name(".strtab").unwrap();
    let strtab_offset = strtab.offset() as usize;
    let strtab_size = strtab.size() as usize;
    let strtab_data = &elf_data[strtab_offset..strtab_offset + strtab_size];
    if let SymbolTable64(symtab_data) = symtab.get_data(&elf).unwrap() {
        symtab_data
            .iter()
            .find(|&entry| {
                if entry.size() == 0 {
                    return false;
                }
                if entry.value() < stext as u64 || entry.value() > etext as u64 {
                    return false;
                }
                let addr = addr as u64;
                addr >= entry.value() && addr < entry.value() + entry.size()
            })
            .map(|entry| {
                let func_name_len = strtab_data[entry.name() as usize..]
                    .iter()
                    .enumerate()
                    .find(|&pair| *pair.1 == 0)
                    .map(|pair| pair.0)
                    .unwrap();
                // SAFETY: We can guarantee that this is a valid str due to the correctness of
                // the ELF format.
                let func_name = unsafe {
                    core::str::from_raw_parts(&strtab_data[entry.name() as usize], func_name_len)
                };
                KernelFuncEntry {
                    func_name,
                    start_addr: entry.value() as usize,
                    func_size: entry.size() as usize,
                }
            })
    } else {
        None
    }
}

/// Return demangled func name as a String of a mangled func name.
pub fn demangled_func_name(func_name: &str) -> String {
    demangle(func_name).to_string()
}
