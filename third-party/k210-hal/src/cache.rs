//! (TODO) Bypass cache 
//! 
//! Todo: verify this module!

use core::slice;
use core::str;

/// Convert a buffer or a pointer into ones with uncached address.
/// 
/// Section 3.4.1, Kendryte K210 Datasheet
pub fn uncached<T: Uncache>(buf: T) -> T {
    buf.uncache()
}

/// Uncacheable buffer or pointer.
pub trait Uncache {
    /// Convert this buffer or pointer to uncached addressed ones
    fn uncache(self) -> Self;
}

impl<T> Uncache for &T {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self as *const T as usize;
        assert_addr_cached(addr);
        // note(unsafe): safe for source address is safe
        unsafe { &*((addr - 0x4000_0000) as *const T) }
    }
}

impl<T> Uncache for &mut T {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self as *mut T as usize;
        assert_addr_cached(addr);
        // note(unsafe): safe for source address is safe
        unsafe { &mut *((addr - 0x4000_0000) as *mut T) }
    }
}

impl<T> Uncache for &[T] {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self.as_ptr() as usize;
        assert_addr_cached(addr);
        let new_ptr = (addr - 0x4000_0000) as *const T;
        // note(unsafe): source address is safe; passing ownership
        unsafe { slice::from_raw_parts(new_ptr, self.len()) }
    }
}

impl<T> Uncache for &mut [T] {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self.as_ptr() as usize;
        assert_addr_cached(addr);
        let new_ptr = (addr - 0x4000_0000) as *mut T;
        // note(unsafe): source address is safe; passing ownership
        unsafe { slice::from_raw_parts_mut(new_ptr, self.len()) }
    }
}

impl Uncache for &str {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self.as_ptr() as usize;
        assert_addr_cached(addr);
        let new_ptr = (addr - 0x4000_0000) as *const u8;
        // note(unsafe): source address is safe; passing ownership
        let slice = unsafe { slice::from_raw_parts(new_ptr, self.len()) };
        // note(unsafe): source slice is guaranteed valid in UTF-8
        unsafe { str::from_utf8_unchecked(slice) }
    }
}

impl Uncache for &mut str {
    #[inline]
    fn uncache(self) -> Self {
        let addr = self.as_ptr() as usize;
        assert_addr_cached(addr);
        let new_ptr = (addr - 0x4000_0000) as *mut u8;
        // note(unsafe): source address is safe; passing ownership
        let slice = unsafe { slice::from_raw_parts_mut(new_ptr, self.len()) };
        // note(unsafe): source slice is guaranteed valid in UTF-8
        unsafe { str::from_utf8_unchecked_mut(slice) }
    }
}

impl<T> Uncache for *const T {
    #[inline]
    fn uncache(self) -> Self {
        assert_addr_cached(self as usize);
        (self as usize - 0x4000_0000) as *const T
    }
}

impl<T> Uncache for *mut T {
    #[inline]
    fn uncache(self) -> Self {
        assert_addr_cached(self as usize);
        (self as usize - 0x4000_0000) as *mut T
    }
}

#[inline]
fn assert_addr_cached(addr: usize) {
    /*
        SRAM memory: 0x8000_0000 to 0x805F_FFFF
        AI memory:   0x8060_0000 to 0x807F_FFFF
        SRAM memory includes two parts: MEM0 and MEM1
        MEM0 memory: 0x8000_0000 to 0x803F_FFFF
        MEM1 memory: 0x8040_0000 to 0x805F_FFFF
    */
    // assure that memory is within SRAM or AI 
    // todo: should we exclude AI?
    assert!(addr <= 0x807F_FFFF && addr >= 0x8000_0000);
}
