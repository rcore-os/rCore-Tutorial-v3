use crate::efs::EasyFileSystem;
use crate::layout::{DirEntry, DiskInode, DiskInodeType, DIRENT_SZ};
use crate::BlockCacheMgr;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::str::FromStr;

/// Virtual filesystem layer over easy-fs
pub struct Inode<REasyFS, const N: usize, RCacheMgr, RCache>
where
    REasyFS: lock_api::RawMutex,
    RCacheMgr: lock_api::RawMutex,
    RCache: lock_api::RawMutex,
{
    block_id: usize,
    block_offset: usize,
    fs: Arc<lock_api::Mutex<REasyFS, EasyFileSystem<N, RCacheMgr, RCache>>>,
}

impl<REasyFS, const N: usize, RCacheMgr, RCache> Inode<REasyFS, N, RCacheMgr, RCache>
where
    REasyFS: lock_api::RawMutex,
    RCacheMgr: lock_api::RawMutex,
    RCache: lock_api::RawMutex,
{
    /// Get the root inode of an EasyFileSystem.
    pub fn root_inode(
        efs: &Arc<lock_api::Mutex<REasyFS, EasyFileSystem<N, RCacheMgr, RCache>>>,
    ) -> Self {
        let (block_id, block_offset) = efs.lock().get_disk_inode_pos(0);
        Self {
            block_id: block_id as usize,
            block_offset,
            fs: Arc::clone(efs),
        }
    }

    /// Apply a read operation on the DiskInode the current Inode refers to.
    ///
    /// This requires the mutable reference of the bcache_mgr.
    pub fn read_disk_inode<V>(
        &self,
        bcache_mgr: &mut BlockCacheMgr<N, RCache>,
        op: impl FnOnce(&DiskInode) -> V,
    ) -> V {
        bcache_mgr.read_block(self.block_id, self.block_offset, op)
    }

    /// Apply a write operation on the DiskInode the current Inode refers to.
    ///
    /// This requires the mutable reference of the bcache_mgr.
    pub fn write_disk_inode<V>(
        &self,
        bcache_mgr: &mut BlockCacheMgr<N, RCache>,
        op: impl FnOnce(&mut DiskInode) -> V,
    ) -> V {
        bcache_mgr.write_block(self.block_id, self.block_offset, op)
    }

    /// Return true if the current Inode is a directory.
    pub fn is_dir(&self) -> bool {
        let fs_guard = self.fs.lock();
        let mut bcache_mgr = fs_guard.bcache_mgr.lock();
        bcache_mgr.read_block(
            self.block_id,
            self.block_offset,
            |disk_inode: &DiskInode| disk_inode.is_dir(),
        )
    }

    /// Return true if the current Inode is a file.
    pub fn is_file(&self) -> bool {
        let fs_guard = self.fs.lock();
        let mut bcache_mgr = fs_guard.bcache_mgr.lock();
        bcache_mgr.read_block(
            self.block_id,
            self.block_offset,
            |disk_inode: &DiskInode| disk_inode.is_file(),
        )
    }

    /// Return the size in bytes of the content the current Inode refers to.
    pub fn size(&self) -> u32 {
        let fs_guard = self.fs.lock();
        let mut bcache_mgr = fs_guard.bcache_mgr.lock();
        bcache_mgr.read_block(
            self.block_id,
            self.block_offset,
            |disk_inode: &DiskInode| disk_inode.size,
        )
    }

    /// Create a new file under the root directory.
    ///
    /// If the file with the given name exists, return None. Otherwise,
    /// return the created Inode.
    pub fn create(&self, name: &str) -> Option<Arc<Self>> {
        assert!(self.is_dir());
        if self.find(name).is_some() {
            return None;
        }
        let mut fs_guard = self.fs.lock();

        fs_guard.sync_transaction(|fs| {
            let mut bcache_mgr = fs.bcache_mgr.lock();

            let new_inode_id = fs.new_inode_nolock(&mut bcache_mgr, DiskInodeType::File);
            let (new_inode_blk_id, new_inode_blk_offset) = fs.get_disk_inode_pos(new_inode_id);
            let root_inode_blk = bcache_mgr.get_block_cache(self.block_id);
            root_inode_blk
                .lock()
                .write(self.block_offset, |root_inode: &mut DiskInode| {
                    // append file in the dirent
                    let file_count = (root_inode.size as usize) / DIRENT_SZ;
                    let new_size = (file_count + 1) * DIRENT_SZ;
                    // increase size
                    fs.increase_size_nolock(new_size as u32, root_inode, &mut bcache_mgr);
                    // write dirent
                    let dirent = DirEntry::new(name, new_inode_id);
                    root_inode.write_at(file_count * DIRENT_SZ, dirent.as_bytes(), &mut bcache_mgr);
                });
            Some(Arc::new(Self {
                block_id: new_inode_blk_id as usize,
                block_offset: new_inode_blk_offset,
                fs: Arc::clone(&self.fs),
            }))
        })
    }

    /// Search a file with the given name under the root directory and return the
    /// corresponding Inode.
    pub fn find(&self, name: &str) -> Option<Arc<Self>> {
        assert!(self.is_dir());
        let mut fs_guard = self.fs.lock();

        fs_guard.sync_transaction(|fs| {
            let mut bcache_mgr = fs.bcache_mgr.lock();
            let block_cache = bcache_mgr.get_block_cache(self.block_id);
            let disk_inode_op = |disk_inode: &DiskInode| {
                disk_inode
                    .inode_id_by_name(name, &mut bcache_mgr)
                    .map(|inode_id: u32| {
                        let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                        Arc::new(Self {
                            block_id: block_id as usize,
                            block_offset,
                            fs: self.fs.clone(),
                        })
                    })
            };
            let block_cache = block_cache.lock();
            block_cache.read(self.block_offset, disk_inode_op)
        })
    }

    /// Read the file's contents at the specified offset into the given buffer.
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let mut fs_guard = self.fs.lock();
        fs_guard.sync_transaction(|fs| {
            let mut bcache_mgr = fs.bcache_mgr.lock();
            let inode_blk = bcache_mgr.get_block_cache(self.block_id);
            let inode_blk_guard = inode_blk.lock();
            let disk_inode: &DiskInode = inode_blk_guard.value_ref_at_offset(self.block_offset);
            disk_inode.read_at(offset, buf, &mut bcache_mgr)
        })
    }

    /// Write the contents of the buffer to the file at the specified offset.
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs_guard = self.fs.lock();
        fs_guard.sync_transaction(|fs| {
            let mut bcache_mgr = fs.bcache_mgr.lock();
            let inode_blk = bcache_mgr.get_block_cache(self.block_id);
            let mut inode_blk_guard = inode_blk.lock();
            let disk_inode: &mut DiskInode = inode_blk_guard.value_mut_at_offset(self.block_offset);
            fs.increase_size_nolock((offset + buf.len()) as u32, disk_inode, &mut bcache_mgr);
            disk_inode.write_at(offset, buf, &mut bcache_mgr)
        })
    }

    /// Clear the size of the file to zero.
    pub fn clear(&self) {
        let mut fs_guard = self.fs.lock();
        fs_guard.sync_transaction(|fs| {
            let mut bcache_mgr = fs.bcache_mgr.lock();
            let inode_blk = bcache_mgr.get_block_cache(self.block_id);
            let mut inode_blk_guard = inode_blk.lock();
            let disk_inode: &mut DiskInode = inode_blk_guard.value_mut_at_offset(self.block_offset);
            let total_blks = disk_inode.total_blocks();
            let blocks_dealloc = disk_inode.clear_size(&mut bcache_mgr);
            assert_eq!(blocks_dealloc.len(), total_blks as usize);
            for block_id in blocks_dealloc.into_iter() {
                fs.dealloc_data(&mut bcache_mgr, block_id);
            }
        });
    }

    /// Return a vector contains all the file names as String under the directory.
    pub fn listdir(&self) -> Vec<String> {
        assert!(self.is_dir());
        let mut list: Vec<String> = Vec::new();
        assert_eq!(self.size() % (DIRENT_SZ as u32), 0);
        let mut dirent = DirEntry::empty();
        let mut offset = 0usize;
        while offset < self.size() as usize {
            self.read_at(offset, dirent.as_bytes_mut());
            list.push(String::from_str(dirent.name()).unwrap());
            offset += DIRENT_SZ;
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    const BLK_DEV_SZ: usize = 4096;
    const BCACHE_SZ: usize = 256;
    const TEST_FILENAME: &str = "test_file0";

    type EasyFileSystemType = EasyFileSystem<256, RawSpinlock, RawSpinlock>;

    fn test_prepare_efs() -> Arc<lock_api::Mutex<RawSpinlock, EasyFileSystemType>> {
        let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<BLK_DEV_SZ>::new());
        let bcache_mgr: BlockCacheMgr<BCACHE_SZ, RawSpinlock> = BlockCacheMgr::new(&block_dev);
        let efs: EasyFileSystem<BCACHE_SZ, RawSpinlock, _> =
            EasyFileSystem::create(BLK_DEV_SZ as u32, 1, bcache_mgr);
        let efs: lock_api::Mutex<RawSpinlock, EasyFileSystem<BCACHE_SZ, _, _>> =
            lock_api::Mutex::new(efs);
        Arc::new(efs)
    }

    #[test]
    pub fn test_file_create_and_find() {
        let efs = test_prepare_efs();
        let root_inode = Inode::root_inode(&efs);
        assert!(root_inode.is_dir());
        root_inode.create(TEST_FILENAME);
        let file_inode = root_inode.find(TEST_FILENAME);
        assert!(file_inode.is_some());
        let file_inode = file_inode.unwrap();
        assert!(file_inode.is_file());
        assert_eq!(root_inode.size(), DIRENT_SZ as u32);
        assert_eq!(file_inode.size(), 0);

        let mut dirent = DirEntry::empty();
        root_inode.read_at(0, dirent.as_bytes_mut());
        assert_eq!(dirent.name(), TEST_FILENAME);
    }

    #[test]
    pub fn test_listdir() {
        let efs = test_prepare_efs();
        let root_inode = Inode::root_inode(&efs);
        let filenames: Vec<String> = (0..1000).map(|id| format!("test_file{}", id)).collect();
        for filename in filenames.iter() {
            root_inode.create(filename.as_str());
        }
        let filenames_from_fs = root_inode.listdir();
        assert_eq!(filenames, filenames_from_fs);
    }

    #[test]
    pub fn test_file_read_write() {
        let efs = test_prepare_efs();
        let root_inode = Inode::root_inode(&efs);
        let file_inode = root_inode.create(TEST_FILENAME).unwrap();

        let file_size = 200 * BLOCK_SZ;
        let mut fake_file = FakeFile::new(file_size);
        for _ in 0..1000 {
            match FileOpGenerator::generate(file_size) {
                FileOp::FileRead { offset, len } => {
                    let mut buf: Vec<u8> = Vec::new();
                    buf.resize(len, 0);
                    let mut fake_buf: Vec<u8> = Vec::new();
                    fake_buf.resize(len, 0);
                    fake_file.read_at(offset, fake_buf.as_mut_slice());
                    file_inode.read_at(offset, buf.as_mut_slice());
                    assert_eq!(buf.as_slice(), fake_buf.as_slice());
                }
                FileOp::FileWrite { offset, data } => {
                    fake_file.write_at(offset, data.as_slice());
                    file_inode.write_at(offset, data.as_slice());
                }
            }
        }

        file_inode.clear();
        assert_eq!(file_inode.size(), 0);
    }
}
