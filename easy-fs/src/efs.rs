use crate::bitmap::Bitmap;
use crate::layout::{DiskInode, DiskInodeType, SuperBlock};
use crate::BlockCacheMgr;
use crate::BLOCK_SZ;
use alloc::vec::Vec;

const SUPER_BLOCK_ID: usize = 0;
const ROOT_INODE_ID: usize = 0;

/// The EasyFileSystem aggregates the underlying BlockCacheMgr and
/// describes the layout of the file system, including the positions
/// of different regions.
pub struct EasyFileSystem<const N: usize, RCacheMgr, RCache>
where
    RCacheMgr: lock_api::RawMutex,
    RCache: lock_api::RawMutex,
{
    /// The EasyFileSystem apply read/write operation on disk through the bcache_mgr.
    pub bcache_mgr: lock_api::Mutex<RCacheMgr, BlockCacheMgr<N, RCache>>,
    inode_bitmap: Bitmap,
    data_bitmap: Bitmap,
    inode_area_start_block_id: u32,
    data_area_start_block_id: u32,
}

type DataBlock = [u8; BLOCK_SZ];

impl<const N: usize, RCacheMgr, RCache> EasyFileSystem<N, RCacheMgr, RCache>
where
    RCacheMgr: lock_api::RawMutex,
    RCache: lock_api::RawMutex,
{
    fn new_bare(bcache_mgr: BlockCacheMgr<N, RCache>) -> Self {
        Self {
            bcache_mgr: lock_api::Mutex::new(bcache_mgr),
            inode_bitmap: Bitmap::default(),
            data_bitmap: Bitmap::default(),
            inode_area_start_block_id: 0,
            data_area_start_block_id: 0,
        }
    }

    /// Get inode by id
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inodes_per_block = (BLOCK_SZ / inode_size) as u32;
        let block_id = self.inode_area_start_block_id + inode_id / inodes_per_block;
        (
            block_id,
            (inode_id % inodes_per_block) as usize * inode_size,
        )
    }
    /// Get data block by id
    pub fn get_data_block_id(&self, data_block_id: u32) -> u32 {
        self.data_area_start_block_id + data_block_id
    }

    /// Allocate a new inode. This function should be called with BlockCacheMgr locked.
    fn alloc_inode(&self, bcache_mgr: &mut BlockCacheMgr<N, RCache>) -> u32 {
        self.inode_bitmap.alloc(bcache_mgr).unwrap() as u32
    }

    /// Allocate a data block
    pub fn alloc_data(&self, bcache_mgr: &mut BlockCacheMgr<N, RCache>) -> u32 {
        self.data_bitmap.alloc(bcache_mgr).unwrap() as u32 + self.data_area_start_block_id
    }
    /// Deallocate a data block
    pub fn dealloc_data(&self, bcache_mgr: &mut BlockCacheMgr<N, RCache>, block_id: u32) {
        bcache_mgr.write_block(block_id as usize, 0, |data_block: &mut DataBlock| {
            data_block.iter_mut().for_each(|p| {
                *p = 0;
            })
        });
        self.data_bitmap.dealloc(
            bcache_mgr,
            (block_id - self.data_area_start_block_id) as usize,
        )
    }

    /// Operate on the efs and sync the changes to block device.
    pub fn sync_transaction<T>(&mut self, op: impl FnOnce(&Self) -> T) -> T {
        let ret = op(self);
        self.bcache_mgr.lock().sync_all();
        ret
    }

    /// Create a new file system on a bcache_mgr.
    pub fn create(
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        bcache_mgr: BlockCacheMgr<N, RCache>,
    ) -> Self {
        let mut efs = Self::new_bare(bcache_mgr);
        // calculate block size of areas & create bitmaps
        // layout: SuperBlock | InodeBitmap | InodeArea | DataBitmap | DataArea
        efs.inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_num = efs.inode_bitmap.maximum();
        let inode_area_blocks =
            ((inode_num * core::mem::size_of::<DiskInode>() + BLOCK_SZ - 1) / BLOCK_SZ) as u32;
        let inode_total_blocks = inode_bitmap_blocks + inode_area_blocks;
        let data_total_blocks = total_blocks - 1 - inode_total_blocks;
        let data_bitmap_blocks = (data_total_blocks + 4096) / 4097;
        let data_area_blocks = data_total_blocks - data_bitmap_blocks;
        efs.data_bitmap = Bitmap::new(
            (1 + inode_bitmap_blocks + inode_area_blocks) as usize,
            data_bitmap_blocks as usize,
        );
        efs.inode_area_start_block_id = inode_bitmap_blocks + 1;
        efs.data_area_start_block_id = inode_total_blocks + data_bitmap_blocks + 1;

        efs.sync_transaction(|efs_self| {
            // clear all blocks
            let mut bcache_mgr = efs_self.bcache_mgr.lock();
            for i in 0..total_blocks {
                bcache_mgr.write_block(i as usize, 0, |data_block: &mut DataBlock| {
                    data_block.iter_mut().for_each(|byte| *byte = 0);
                });
            }
            // initialize SuperBlock
            bcache_mgr.write_block(SUPER_BLOCK_ID, 0, |super_block: &mut SuperBlock| {
                super_block.initialize(
                    total_blocks,
                    inode_bitmap_blocks,
                    inode_area_blocks,
                    data_bitmap_blocks,
                    data_area_blocks,
                );
            });
            assert_eq!(
                efs_self.inode_bitmap.alloc(&mut bcache_mgr),
                Some(ROOT_INODE_ID)
            );
            let (root_inode_block_id, root_inode_offset) =
                efs_self.get_disk_inode_pos(ROOT_INODE_ID as u32);
            bcache_mgr.write_block(
                root_inode_block_id as usize,
                root_inode_offset,
                |disk_inode: &mut DiskInode| {
                    disk_inode.initialize(DiskInodeType::Directory);
                },
            );
        });
        efs
    }

    /// Open a block device as a filesystem
    pub fn open(mut bcache_mgr: BlockCacheMgr<N, RCache>) -> Self {
        let mut super_block = SuperBlock::default();
        bcache_mgr.read_block(SUPER_BLOCK_ID, 0, |super_block_inner: &SuperBlock| {
            super_block = *super_block_inner;
        });
        assert!(super_block.is_valid(), "Error loading EFS!");
        let inode_total_blocks = super_block.inode_bitmap_blocks + super_block.inode_area_blocks;
        Self {
            bcache_mgr: lock_api::Mutex::new(bcache_mgr),
            inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_blocks as usize),
            data_bitmap: Bitmap::new(
                (1 + inode_total_blocks) as usize,
                super_block.data_bitmap_blocks as usize,
            ),
            inode_area_start_block_id: 1 + super_block.inode_bitmap_blocks,
            data_area_start_block_id: 1 + inode_total_blocks + super_block.data_bitmap_blocks,
        }
    }

    /// Create a new inode of the given inode type, return the inode id.
    ///
    /// This function should be called with bcache_mgr locked.
    pub fn new_inode_nolock(
        &self,
        bcache_mgr: &mut BlockCacheMgr<N, RCache>,
        inode_type: DiskInodeType,
    ) -> u32 {
        let new_inode_id = self.alloc_inode(bcache_mgr);
        let (new_inode_block_id, new_inode_block_offset) = self.get_disk_inode_pos(new_inode_id);
        bcache_mgr.write_block(
            new_inode_block_id as usize,
            new_inode_block_offset,
            |new_inode: &mut DiskInode| {
                new_inode.initialize(inode_type);
            },
        );
        new_inode_id
    }
    /// Increase the size of a disk inode.
    ///
    /// This function should be called with bcache_mgr locked.
    pub fn increase_size_nolock(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        bcache_mgr: &mut BlockCacheMgr<N, RCache>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(self.alloc_data(bcache_mgr));
        }
        disk_inode.increase_size(new_size, v, bcache_mgr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::sync::Arc;

    const EFS_BLK_NUM: usize = 2048;
    const BCACHE_NUM: usize = 256;

    #[test]
    pub fn test_efs_create_open() {
        let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<EFS_BLK_NUM>::new());
        {
            let bcache_mgr: BlockCacheMgr<BCACHE_NUM, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let _efs: EasyFileSystem<BCACHE_NUM, RawSpinlock, _> =
                EasyFileSystem::create(EFS_BLK_NUM as u32, 1, bcache_mgr);
        }
        {
            let bcache_mgr: BlockCacheMgr<BCACHE_NUM, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let efs: EasyFileSystem<BCACHE_NUM, RawSpinlock, _> = EasyFileSystem::open(bcache_mgr);
            // SuperBlock 1
            // InodeBitmap 1
            // InodeArea 1024
            // last=1022, DataBitmap 1
            assert_eq!(efs.inode_area_start_block_id, 2);
            assert_eq!(efs.data_area_start_block_id, 1027);
        }
    }
}
