//! get task info
pub fn sys_get_taskinfo(id: usize, buf: *const u8, len: usize) -> isize {
    let slice = unsafe { core::slice::from_raw_parts(buf, len) };
    let name = core::str::from_utf8(slice).unwrap();
    println!("current task: [id: {}, name: {}]", id, name);
    0
}
