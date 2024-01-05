use super::{BlockCacheMgr, BLOCK_SZ};
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Result};

/// Magic number for sanity check
const EFS_MAGIC: u32 = 0x3b800001;
/// The max number of direct inodes
const INODE_DIRECT_COUNT: usize = 28;
/// The max length of inode name
const NAME_LENGTH_LIMIT: usize = 27;
/// The max number of indirect1 inodes
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
/// The max number of indirect2 inodes
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
/// The upper bound of direct inode index
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
/// The upper bound of indirect1 inode index
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
/// The upper bound of indirect2 inode indexs
#[allow(unused)]
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;
/// Super block of a filesystem
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("SuperBlock")
            .field("total_blocks", &self.total_blocks)
            .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
            .field("inode_area_blocks", &self.inode_area_blocks)
            .field("data_bitmap_blocks", &self.data_bitmap_blocks)
            .field("data_area_blocks", &self.data_area_blocks)
            .finish()
    }
}

impl SuperBlock {
    /// Initialize a super block
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            magic: EFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }
    /// Check if a super block is valid using efs magic
    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}

/// Type of a disk inode
#[derive(PartialEq)]
#[repr(u32)]
pub enum DiskInodeType {
    File,
    Directory,
}

/// A indirect block
type IndirectBlock = [u32; BLOCK_SZ / 4];
/// A data block
type DataBlock = [u8; BLOCK_SZ];
/// A disk inode
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    /// Initialize a disk inode, as well as all direct inodes under it
    /// indirect1 and indirect2 block are allocated only when they are needed
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }
    /// Whether this inode is a directory
    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }
    /// Whether this inode is a file
    #[allow(unused)]
    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }
    /// Return block number correspond to size.
    pub fn data_blocks(&self) -> u32 {
        Self::data_blocks_inner(self.size)
    }
    fn data_blocks_inner(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }
    /// Return number of blocks needed include indirect1/2.
    pub fn total_blocks(&self) -> u32 {
        Self::total_blocks_inner(self.size)
    }
    fn total_blocks_inner(size: u32) -> u32 {
        let data_blocks = Self::data_blocks_inner(size) as usize;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            // sub indirect1
            total +=
                (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }
    /// Get the number of data blocks that have to be allocated given the new size of data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks_inner(new_size) - Self::total_blocks_inner(self.size)
    }
    /// Get id of block given inner id
    pub fn get_block_id<const N: usize, R>(
        &self,
        inner_id: u32,
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) -> u32
    where
        R: lock_api::RawMutex,
    {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            bcache_mgr.read_block(
                self.indirect1 as usize,
                0,
                |indirect_block: &IndirectBlock| indirect_block[inner_id - INODE_DIRECT_COUNT],
            )
        } else {
            let last = inner_id - INDIRECT1_BOUND;
            let indirect1 =
                bcache_mgr.read_block(self.indirect2 as usize, 0, |indirect2: &IndirectBlock| {
                    indirect2[last / INODE_INDIRECT1_COUNT]
                });
            bcache_mgr.read_block(indirect1 as usize, 0, |indirect1: &IndirectBlock| {
                indirect1[last % INODE_INDIRECT1_COUNT]
            })
        }
    }
    /// Inncrease the size of current disk inode
    pub fn increase_size<const N: usize, R>(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) where
        R: lock_api::RawMutex,
    {
        let mut current_blocks = self.data_blocks();
        self.size = new_size;
        let mut total_blocks = self.data_blocks();
        let mut new_blocks = new_blocks.into_iter();
        // fill direct
        while current_blocks < total_blocks.min(INODE_DIRECT_COUNT as u32) {
            self.direct[current_blocks as usize] = new_blocks.next().unwrap();
            current_blocks += 1;
        }
        // alloc indirect1
        if total_blocks > INODE_DIRECT_COUNT as u32 {
            if current_blocks == INODE_DIRECT_COUNT as u32 {
                self.indirect1 = new_blocks.next().unwrap();
            }
            current_blocks -= INODE_DIRECT_COUNT as u32;
            total_blocks -= INODE_DIRECT_COUNT as u32;
        } else {
            return;
        }
        // fill indirect1
        bcache_mgr.write_block(
            self.indirect1 as usize,
            0,
            |indirect1: &mut IndirectBlock| {
                while current_blocks < total_blocks.min(INODE_INDIRECT1_COUNT as u32) {
                    indirect1[current_blocks as usize] = new_blocks.next().unwrap();
                    current_blocks += 1;
                }
            },
        );
        // alloc indirect2
        if total_blocks > INODE_INDIRECT1_COUNT as u32 {
            if current_blocks == INODE_INDIRECT1_COUNT as u32 {
                self.indirect2 = new_blocks.next().unwrap();
            }
            current_blocks -= INODE_INDIRECT1_COUNT as u32;
            total_blocks -= INODE_INDIRECT1_COUNT as u32;
        } else {
            return;
        }
        // fill indirect2 from (a0, b0) -> (a1, b1)
        let mut a0 = current_blocks as usize / INODE_INDIRECT1_COUNT;
        let mut b0 = current_blocks as usize % INODE_INDIRECT1_COUNT;
        let a1 = total_blocks as usize / INODE_INDIRECT1_COUNT;
        let b1 = total_blocks as usize % INODE_INDIRECT1_COUNT;
        // alloc low-level indirect1
        let indirect2_blk = bcache_mgr.get_block_cache(self.indirect2 as usize);
        indirect2_blk
            .lock()
            .write(0, |indirect2: &mut IndirectBlock| {
                while (a0 < a1) || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        indirect2[a0] = new_blocks.next().unwrap();
                    }
                    // fill current
                    bcache_mgr.write_block(
                        indirect2[a0] as usize,
                        0,
                        |indirect1: &mut IndirectBlock| {
                            indirect1[b0] = new_blocks.next().unwrap();
                        },
                    );
                    // move to next
                    b0 += 1;
                    if b0 == INODE_INDIRECT1_COUNT {
                        b0 = 0;
                        a0 += 1;
                    }
                }
            });
    }

    /// Clear size to zero and return blocks that should be deallocated.
    /// We will clear the block contents to zero later.
    pub fn clear_size<const N: usize, R>(
        &mut self,
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) -> Vec<u32>
    where
        R: lock_api::RawMutex,
    {
        let mut v: Vec<u32> = Vec::new();
        let mut data_blocks = self.data_blocks() as usize;
        self.size = 0;
        let mut current_blocks = 0usize;
        // direct
        while current_blocks < data_blocks.min(INODE_DIRECT_COUNT) {
            v.push(self.direct[current_blocks]);
            self.direct[current_blocks] = 0;
            current_blocks += 1;
        }
        // indirect1 block
        if data_blocks > INODE_DIRECT_COUNT {
            v.push(self.indirect1);
            data_blocks -= INODE_DIRECT_COUNT;
            current_blocks = 0;
        } else {
            return v;
        }
        // indirect1
        bcache_mgr.write_block(
            self.indirect1 as usize,
            0,
            |indirect1: &mut IndirectBlock| {
                while current_blocks < data_blocks.min(INODE_INDIRECT1_COUNT) {
                    v.push(indirect1[current_blocks]);
                    indirect1[current_blocks] = 0;
                    current_blocks += 1;
                }
            },
        );
        self.indirect1 = 0;
        // indirect2 block
        if data_blocks > INODE_INDIRECT1_COUNT {
            v.push(self.indirect2);
            data_blocks -= INODE_INDIRECT1_COUNT;
        } else {
            return v;
        }
        // indirect2
        assert!(data_blocks <= INODE_INDIRECT2_COUNT);
        let a1 = data_blocks / INODE_INDIRECT1_COUNT;
        let b1 = data_blocks % INODE_INDIRECT1_COUNT;
        let indirect2_blk = bcache_mgr.get_block_cache(self.indirect2 as usize);

        indirect2_blk
            .lock()
            .write(0, |indirect2: &mut IndirectBlock| {
                // full indirect1 blocks
                for entry in indirect2.iter_mut().take(a1) {
                    v.push(*entry);
                    bcache_mgr.write_block(*entry as usize, 0, |indirect1: &mut IndirectBlock| {
                        for entry in indirect1.iter() {
                            v.push(*entry);
                        }
                    });
                }
                // last indirect1 block
                if b1 > 0 {
                    v.push(indirect2[a1]);
                    bcache_mgr.write_block(
                        indirect2[a1] as usize,
                        0,
                        |indirect1: &mut IndirectBlock| {
                            for entry in indirect1.iter().take(b1) {
                                v.push(*entry);
                            }
                        },
                    );
                    //indirect2[a1] = 0;
                }
            });
        self.indirect2 = 0;
        v
    }
    /// Read data from current disk inode
    pub fn read_at<const N: usize, R>(
        &self,
        offset: usize,
        buf: &mut [u8],
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) -> usize
    where
        R: lock_api::RawMutex,
    {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if start >= end {
            return 0;
        }
        let mut start_block = start / BLOCK_SZ;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];
            let block_id = self.get_block_id(start_block as u32, bcache_mgr);
            bcache_mgr.read_block(block_id as usize, 0, |data_block: &DataBlock| {
                let src = &data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_read_size];
                dst.copy_from_slice(src);
            });
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }
    /// Write data into current disk inode
    /// size must be adjusted properly beforehand
    pub fn write_at<const N: usize, R>(
        &mut self,
        offset: usize,
        buf: &[u8],
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) -> usize
    where
        R: lock_api::RawMutex,
    {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SZ;
        let mut write_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let block_write_size = end_current_block - start;
            let block_id = self.get_block_id(start_block as u32, bcache_mgr) as usize;
            bcache_mgr.write_block(block_id, 0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + block_write_size];
                let dst = &mut data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_write_size];
                dst.copy_from_slice(src);
            });
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }

    /// Find inode under a disk inode by name
    pub fn inode_id_by_name<const N: usize, R>(
        &self,
        name: &str,
        bcache_mgr: &mut BlockCacheMgr<N, R>,
    ) -> Option<u32>
    where
        R: lock_api::RawMutex,
    {
        // assert it is a directory
        assert!(self.is_dir());
        let file_count = (self.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                self.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), bcache_mgr),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }
}

#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT + 1],
    inode_number: u32,
}
/// Size of a directory entry
pub const DIRENT_SZ: usize = 32;

impl DirEntry {
    /// Create an empty directory entry
    pub fn empty() -> Self {
        Self {
            name: [0u8; NAME_LENGTH_LIMIT + 1],
            inode_number: 0,
        }
    }
    /// Create a directory entry from name and inode number
    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut bytes = [0u8; NAME_LENGTH_LIMIT + 1];
        let len = core::cmp::min(name.len(), NAME_LENGTH_LIMIT);
        bytes[..len].copy_from_slice(&name.as_bytes()[..len]);
        Self {
            name: bytes,
            inode_number,
        }
    }
    /// Serialize into bytes
    pub fn as_bytes(&self) -> &[u8] {
        // Safety: We can guarantee that size_of::<Self>() == DIRENT_SZ
        unsafe { core::mem::transmute::<&Self, &[u8; DIRENT_SZ]>(self) }
    }
    /// Serialize into mutable bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        // Safety: The same of Self::as_bytes
        unsafe { core::mem::transmute::<&mut Self, &mut [u8; DIRENT_SZ]>(self) }
    }
    /// Get name of the entry
    pub fn name(&self) -> &str {
        let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }
    /// Get inode number of the entry
    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::sync::Arc;

    #[test]
    pub fn test_static_size() {
        use core::mem::size_of;
        assert!(size_of::<SuperBlock>() <= BLOCK_SZ);
        assert_eq!(size_of::<DiskInode>(), BLOCK_SZ / 4);
        assert_eq!(size_of::<DirEntry>(), DIRENT_SZ);
    }

    pub mod disk_inode {
        use super::*;

        #[test]
        pub fn test_init() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<4096>::new());
            let mut bcache_mgr: BlockCacheMgr<256, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let mut blk_allocator = StackAllocator::<usize>::new();
            let inode_blk_id = blk_allocator.alloc(1)[0];

            bcache_mgr.write_block(inode_blk_id, 0, |disk_inode: &mut DiskInode| {
                disk_inode.initialize(DiskInodeType::File);
                assert_eq!(disk_inode.data_blocks(), 0);
                assert_eq!(disk_inode.total_blocks(), 0);
            });
        }

        #[test]
        pub fn test_increase_clear_size() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<4096>::new());
            let mut bcache_mgr: BlockCacheMgr<16, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let mut blk_allocator = StackAllocator::<u32>::new();
            let inode_blk_id = blk_allocator.alloc(1)[0] as usize;
            let mut allocated = 0usize;

            let inode_blk = bcache_mgr.get_block_cache(inode_blk_id);
            let mut inode_blk_guard = inode_blk.lock();
            let disk_inode = inode_blk_guard.value_mut_at_offset::<DiskInode>(0);

            disk_inode.initialize(DiskInodeType::File);
            let new_size = 20 * BLOCK_SZ;
            let needed_blk_num = disk_inode.blocks_num_needed(new_size as u32);
            allocated += needed_blk_num as usize;
            assert_eq!(needed_blk_num, 20);
            let new_blks = blk_allocator.alloc(needed_blk_num as usize);
            disk_inode.increase_size(new_size as u32, new_blks, &mut bcache_mgr);
            assert_eq!(disk_inode.size, new_size as u32);

            let new_size = INDIRECT1_BOUND * BLOCK_SZ;
            let needed_blk_num = disk_inode.blocks_num_needed(new_size as u32);
            allocated += needed_blk_num as usize;
            assert_eq!(needed_blk_num, INDIRECT1_BOUND as u32 - 20 + 1);
            let new_blks = blk_allocator.alloc(needed_blk_num as usize);
            disk_inode.increase_size(new_size as u32, new_blks, &mut bcache_mgr);
            assert_eq!(disk_inode.size, new_size as u32);

            let blks = disk_inode.clear_size(&mut bcache_mgr);
            assert_eq!(disk_inode.size, 0);
            assert_eq!(blks.len(), allocated);
        }

        #[test]
        pub fn test_read_write() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<4096>::new());
            let mut bcache_mgr: BlockCacheMgr<16, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let mut blk_allocator = StackAllocator::<u32>::new();
            let inode_blk_id = blk_allocator.alloc(1)[0] as usize;

            let inode_blk = bcache_mgr.get_block_cache(inode_blk_id);
            let mut inode_blk_guard = inode_blk.lock();
            let disk_inode = inode_blk_guard.value_mut_at_offset::<DiskInode>(0);
            let new_size = 200 * BLOCK_SZ as u32;
            let needed_blk_num = disk_inode.blocks_num_needed(new_size);
            let new_blks = blk_allocator.alloc(needed_blk_num as usize);
            disk_inode.increase_size(new_size, new_blks, &mut bcache_mgr);

            let test_ops = 1000;
            let mut fake_file = FakeFile::new(new_size as usize);
            for _ in 0..test_ops {
                let file_op = FileOpGenerator::generate(new_size as usize);
                match file_op {
                    FileOp::FileRead { offset, len } => {
                        let mut buf_fake: Vec<u8> = Vec::new();
                        let mut buf: Vec<u8> = Vec::new();
                        buf_fake.resize(len, 0);
                        buf.resize(len, 0);
                        disk_inode.read_at(offset, buf.as_mut_slice(), &mut bcache_mgr);
                        fake_file.read_at(offset, buf_fake.as_mut_slice());
                        assert_eq!(buf.as_slice(), buf_fake.as_slice());
                    }
                    FileOp::FileWrite { offset, data } => {
                        disk_inode.write_at(offset, data.as_slice(), &mut bcache_mgr);
                        fake_file.write_at(offset, data.as_slice());
                    }
                }
            }
        }
    }
}
