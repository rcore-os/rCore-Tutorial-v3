pub use crate::{BlockDevice, BLOCK_SZ};
use core::mem::swap;
use lock_api::{GuardSend, RawMutex};
use rand::Rng;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub struct MockBlockDevice;

impl BlockDevice for MockBlockDevice {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) {}
    fn write_block(&self, _block_id: usize, _buf: &[u8]) {}
}

pub struct TestBlockDevice<const N: usize> {
    pub blocks: Box<Mutex<Vec<[u8; BLOCK_SZ]>>>,
}

impl<const N: usize> TestBlockDevice<N> {
    pub fn new() -> Self {
        let mut v: Vec<[u8; BLOCK_SZ]> = Vec::new();
        for _ in 0..N {
            v.push([0u8; BLOCK_SZ]);
        }
        Self {
            blocks: Box::new(Mutex::new(v)),
        }
    }
}

impl<const N: usize> BlockDevice for TestBlockDevice<N> {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        assert!(block_id < N);
        let blocks = self.blocks.as_ref().lock().unwrap();
        buf.copy_from_slice(&blocks[block_id]);
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(block_id < N);
        let mut blocks = self.blocks.as_ref().lock().unwrap();
        blocks[block_id].copy_from_slice(buf);
    }
}

/// Define our raw lock type
///
/// From [lock_api](https://docs.rs/lock_api/latest/lock_api/index.html)
pub struct RawSpinlock(AtomicBool);

// Implement RawMutex for this type
unsafe impl RawMutex for RawSpinlock {
    const INIT: RawSpinlock = RawSpinlock(AtomicBool::new(false));

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        // Note: This isn't the best way of implementing a spinlock, but it
        // suffices for the sake of this example.
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        self.0
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

pub struct StackAllocator<U> {
    next_id: U,
    free_list: VecDeque<U>,
}

impl StackAllocator<u32> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            free_list: VecDeque::new(),
        }
    }
    fn alloc_one(&mut self) -> u32 {
        if let Some(id) = self.free_list.pop_back() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }
    pub fn alloc(&mut self, n: usize) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..n {
            v.push(self.alloc_one());
        }
        v
    }
}

impl StackAllocator<usize> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            free_list: VecDeque::new(),
        }
    }
    fn alloc_one(&mut self) -> usize {
        if let Some(id) = self.free_list.pop_back() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }
    pub fn alloc(&mut self, n: usize) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::new();
        for _ in 0..n {
            v.push(self.alloc_one());
        }
        v
    }
}

impl<U> StackAllocator<U> {
    #[allow(unused)]
    fn free_one(&mut self, id: U) {
        self.free_list.push_back(id);
    }

    #[allow(unused)]
    pub fn free(&mut self, ids: Vec<U>) {
        for id in ids.into_iter() {
            self.free_one(id);
        }
    }
}

pub struct FakeFile {
    v: Vec<u8>,
}

impl FakeFile {
    pub fn new(size: usize) -> Self {
        let mut v: Vec<u8> = Vec::new();
        for _ in 0..size {
            v.push(0u8);
        }
        Self { v }
    }
    pub fn read_at(&self, offset: usize, dst: &mut [u8]) {
        dst.copy_from_slice(&self.v.as_slice()[offset..offset + dst.len()]);
    }
    pub fn write_at(&mut self, offset: usize, src: &[u8]) {
        let dst = &mut self.v.as_mut_slice()[offset..offset + src.len()];
        dst.copy_from_slice(src);
    }
}

pub struct FileOpGenerator;

#[derive(Debug)]
pub enum FileOp {
    FileRead { offset: usize, len: usize },
    FileWrite { offset: usize, data: Vec<u8> },
}

impl FileOpGenerator {
    pub fn generate(file_len: usize) -> FileOp {
        let offset: usize;
        let len: usize;
        let mut rng = rand::thread_rng();
        loop {
            let mut offset0 = rng.gen::<u64>() % (file_len as u64 - 1);
            let mut offset1 = rng.gen::<u64>() % (file_len as u64 - 1);
            if offset0 != offset1 {
                if offset0 > offset1 {
                    swap(&mut offset0, &mut offset1);
                }
                offset = offset0 as usize;
                len = (offset1 - offset0) as usize;
                break;
            }
        }
        if rand::random() {
            FileOp::FileRead { offset, len }
        } else {
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..len {
                data.push(rand::random::<u8>());
            }
            FileOp::FileWrite { offset, data }
        }
    }
}
