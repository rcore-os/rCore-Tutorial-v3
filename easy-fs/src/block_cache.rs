use crate::{BlockDevice, BLOCK_SZ};

use alloc::sync::Arc;
use core::ops::{Deref, DerefMut};

use lock_api;

#[derive(Copy, Clone)]
#[repr(align(512))]
pub struct CacheBlock([u8; BLOCK_SZ]);
impl Default for CacheBlock {
    fn default() -> Self {
        Self([0u8; BLOCK_SZ])
    }
}
impl Deref for CacheBlock {
    type Target = [u8; BLOCK_SZ];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for CacheBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Cached block inside memory
pub struct BlockCache {
    /// cached block data, provided by BlockCacheMgr
    /// We use 'static lifetime here since:
    /// 1. We do not want BlockCacheMgr to be a self-referential struct.
    /// 2. We can guarantee that the BlockCacheMgr outlives BlockCache.
    /// 3. We can guarantee that only one BlockCache can access this region
    /// at a time.
    cache: &'static mut CacheBlock,
    /// underlying block id
    block_id: usize,
    /// underlying block device
    block_device: Arc<dyn BlockDevice>,
    /// whether the block is dirty
    pub dirty: bool,
    /// The timestamp of the last access of the block.
    /// It is provided by the BlockCacheMgr.
    last_access_time: usize,
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(
        block_id: usize,
        block_device: &Arc<dyn BlockDevice>,
        cache: &mut CacheBlock,
    ) -> Self {
        block_device.read_block(block_id, &mut cache.0);
        // Safety: See the definition of BlockCache.
        let cache: &'static mut CacheBlock = unsafe {
            let cache_ptr: *mut CacheBlock = core::mem::transmute(cache.as_mut_ptr());
            &mut *(cache_ptr)
        };
        Self {
            cache,
            block_id,
            block_device: Arc::clone(block_device),
            dirty: false,
            last_access_time: 0,
        }
    }

    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache.0[offset] as *const _ as usize
    }

    fn check_value_validity<T>(&self, offset: usize) {
        let start_addr = self.addr_of_offset(offset);
        // We guarantee that the input arguments are correct. Thus,
        // if we find that the arguments are not valid, just panic.
        //
        assert_eq!(start_addr % core::mem::align_of::<T>(), 0);
        assert!(offset + core::mem::size_of::<T>() <= BLOCK_SZ);
    }

    /// Get an reference to a value of generic type T at a given
    /// offset on the block cache.
    pub fn value_ref_at_offset<T>(&self, offset: usize) -> &T {
        self.check_value_validity::<T>(offset);
        // Safety:
        // we have checked that the address at the given offset is
        // aligned to type T and the value of type T is in the boundary
        // of the cache array.
        unsafe { core::mem::transmute::<&u8, &T>(&self.cache.0[offset]) }
    }

    /// Get an mutable reference to a value of generic type T at a given
    /// offset on the block cache.
    pub fn value_mut_at_offset<T>(&mut self, offset: usize) -> &mut T {
        self.check_value_validity::<T>(offset);
        self.dirty = true;
        // Safety: See value_ref_at_offset.
        unsafe { core::mem::transmute::<&mut u8, &mut T>(&mut self.cache.0[offset]) }
    }

    /// Apply a read operation on this BlockCache at the given offset.
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.value_ref_at_offset::<T>(offset))
    }

    /// Apply a write operation on this BlockCache at the given offset.
    pub fn write<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.value_mut_at_offset::<T>(offset))
    }

    /// Sync the changes on this block cache in memory to the underlying
    /// block device.
    pub fn sync_to_device(&mut self) {
        if self.dirty {
            self.dirty = false;
            self.block_device.write_block(self.block_id, &self.cache.0);
        }
    }

    /// Update the access time used for LRU when this block is
    /// accessed.
    pub fn update_access_time(&mut self, new_time: usize) {
        self.last_access_time = new_time;
    }

    /// Get the timestamp of the last access of this block.
    pub fn access_time(&self) -> usize {
        self.last_access_time
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync_to_device();
    }
}

/// The block cache manager.
///
/// The user can control the sync primitive used by each block cache via
/// R which implements the `lock_api::RawMutex` trait. At the same time, user can
/// control the capacity of the block cache pool via the const generic parameter N.
pub struct BlockCacheMgr<const N: usize, R>
where
    R: lock_api::RawMutex,
{
    cache_block_arr: [CacheBlock; N],
    block_id_arr: [Option<usize>; N],
    cache_arr: [Option<Arc<lock_api::Mutex<R, BlockCache>>>; N],
    block_dev: Arc<dyn BlockDevice>,
    current_time: usize,
}

impl<const N: usize, R> BlockCacheMgr<N, R>
where
    R: lock_api::RawMutex,
{
    /// Initialize a block cache manager.
    pub fn new(block_dev: &Arc<dyn BlockDevice>) -> Self {
        let mut cache_arr: [Option<Arc<lock_api::Mutex<R, BlockCache>>>; N] =
            unsafe { core::mem::zeroed() };
        for cache in cache_arr.iter_mut() {
            *cache = None;
        }
        Self {
            cache_block_arr: [CacheBlock::default(); N],
            block_id_arr: [None; N],
            cache_arr,
            block_dev: Arc::clone(block_dev),
            current_time: 0,
        }
    }

    fn get_slot_id(&self, block_id: usize) -> Option<usize> {
        (0..N).find(|slot_id| {
            if let Some(block_id_t) = self.block_id_arr[*slot_id].as_ref() {
                *block_id_t == block_id
            } else {
                false
            }
        })
    }

    fn get_empty_slot_id(&self) -> Option<usize> {
        (0..N).find(|slot_id| self.block_id_arr[*slot_id].is_none())
    }

    /// Get a block cache from block cache manager. Caller need to provide the block id.
    ///
    /// # Panics
    ///
    /// This function will panic if all caches cannot be swapped out since they are
    /// still referenced.
    pub fn get_block_cache(&mut self, block_id: usize) -> Arc<lock_api::Mutex<R, BlockCache>> {
        let slot_id = if let Some(slot_id) = self.get_slot_id(block_id) {
            slot_id
        } else if let Some(slot_id) = self.get_empty_slot_id() {
            self.block_id_arr[slot_id] = Some(block_id);
            self.cache_arr[slot_id] = Some(Arc::new(lock_api::Mutex::new(BlockCache::new(
                block_id,
                &self.block_dev,
                &mut self.cache_block_arr[slot_id],
            ))));
            slot_id
        } else {
            // select a block:
            // 1. with minimum last access time
            // 2. it is not referenced by any thread
            let invalid_slot_id = usize::MAX;
            let (slot_id, _) = (0..N).fold(
                (invalid_slot_id, usize::MAX),
                |(target_slot_id, min_last_time), slot_id| {
                    let cache = self.cache_arr[slot_id].as_ref().unwrap();
                    if Arc::strong_count(cache) > 1 {
                        return (target_slot_id, min_last_time);
                    }
                    // Only acquire the cache lock if it has not been acquired(through the refcnt)
                    // This can avoid the AA deadlock.
                    let last_time = cache.lock().access_time();
                    if last_time < min_last_time {
                        (slot_id, last_time)
                    } else {
                        (target_slot_id, min_last_time)
                    }
                },
            );
            // If all caches are still being used, just panic
            assert_ne!(slot_id, invalid_slot_id);
            assert_eq!(
                Arc::strong_count(self.cache_arr[slot_id].as_ref().unwrap()),
                1
            );
            // manually substitute it with a cache of a new block
            // the drop of the older value is later than the init of the new value, thus
            // we cannot put them in one line
            self.cache_arr[slot_id] = None;
            self.block_id_arr[slot_id] = Some(block_id);
            self.cache_arr[slot_id] = Some(Arc::new(lock_api::Mutex::new(BlockCache::new(
                block_id,
                &self.block_dev,
                &mut self.cache_block_arr[slot_id],
            ))));
            slot_id
        };
        // update the access time
        self.current_time += 1;
        let cache = self.cache_arr[slot_id].as_ref().unwrap();
        cache.lock().update_access_time(self.current_time);

        Arc::clone(cache)
    }

    /// Sync the changes on all block caches of this block cache manager to the
    /// block device.
    pub fn sync_all(&self) {
        for cache in self.cache_arr.iter() {
            if let Some(cache) = cache.as_ref() {
                cache.lock().sync_to_device();
            }
        }
    }

    /// Apply a read operation on a block at the given offset.
    pub fn read_block<T, V>(
        &mut self,
        block_id: usize,
        offset: usize,
        op: impl FnOnce(&T) -> V,
    ) -> V {
        let bcache = self.get_block_cache(block_id);
        let bcache_guard = bcache.lock();
        bcache_guard.read(offset, op)
    }

    /// Apply a write operation on a block at the given offset.
    pub fn write_block<T, V>(
        &mut self,
        block_id: usize,
        offset: usize,
        op: impl FnOnce(&mut T) -> V,
    ) -> V {
        let bcache = self.get_block_cache(block_id);
        let mut bcache_guard = bcache.lock();
        bcache_guard.write(offset, op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::sync::Arc;
    mod block_cache {
        use super::*;

        #[test]
        pub fn test_init() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(MockBlockDevice);
            let mut cache_block: CacheBlock = Default::default();
            let block_cache = BlockCache::new(0, &block_dev, &mut cache_block);
            assert_eq!(block_cache.dirty, false);
        }

        #[test]
        pub fn test_read_write() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(MockBlockDevice);
            let mut cache_block: CacheBlock = Default::default();
            let mut block_cache = BlockCache::new(0, &block_dev, &mut cache_block);
            let test_value: u32 = 0x11223344;

            {
                let u32_mut = block_cache.value_mut_at_offset::<u32>(0);
                *u32_mut = test_value;
            }

            let u32_ref = block_cache.value_ref_at_offset::<u32>(0);
            assert_eq!(*u32_ref, test_value);
        }

        #[test]
        pub fn test_sync() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<4>::new());
            let mut cache_block: CacheBlock = Default::default();
            let mut block_cache = BlockCache::new(0, &block_dev, &mut cache_block);
            let mut buf = [0xffu8; BLOCK_SZ];
            block_dev.read_block(0, &mut buf);
            assert!(buf.iter().all(|byte| *byte == 0));

            // write and sync manually
            let test_value: u32 = 0x11223344;
            {
                let u32_mut = block_cache.value_mut_at_offset::<u32>(0);
                *u32_mut = test_value;
            }
            block_cache.sync_to_device();
            block_dev.read_block(0, &mut buf);
            assert!(&buf[..4] == test_value.to_ne_bytes());
            assert!(&buf[4..].iter().all(|byte| *byte == 0));

            // write and sync automatically after block_cache is dropped
            let test_value2: u32 = 0x55667788;
            {
                let u32_mut = block_cache.value_mut_at_offset::<u32>(4);
                *u32_mut = test_value2;
            }
            drop(block_cache);
            block_dev.read_block(0, &mut buf);
            assert!(&buf[..4] == test_value.to_ne_bytes());
            assert!(&buf[4..8] == test_value2.to_ne_bytes());
            assert!(&buf[8..].iter().all(|byte| *byte == 0));
        }
    }

    mod block_cache_mgr {
        use super::*;

        #[test]
        pub fn test_init() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<4>::new());
            let mut block_cache_mgr: BlockCacheMgr<16, RawSpinlock> =
                BlockCacheMgr::new(&block_dev);
            let block_cache = block_cache_mgr.get_block_cache(0);

            // write a value
            let test_value: u32 = 0x11223344;
            let mut locked_cache = block_cache.lock();
            locked_cache.write(0, |v: &mut u32| {
                *v = test_value;
            });
            // check block_dev, it won't be changed since we've not sync the changes yet
            let mut buf = [0u8; BLOCK_SZ];
            block_dev.read_block(0, &mut buf);
            assert!(buf.iter().all(|byte| *byte == 0));
            // sync changes and check again
            locked_cache.sync_to_device();
            block_dev.read_block(0, &mut buf);
            assert_eq!(buf[..4], test_value.to_ne_bytes());
            assert!(buf[4..].iter().all(|byte| *byte == 0));
        }

        #[test]
        pub fn test_swap_and_sync() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<16>::new());
            let mut block_cache_mgr: BlockCacheMgr<4, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let block_cache_list: Vec<_> = (0..4)
                .map(|block_id: usize| block_cache_mgr.get_block_cache(block_id))
                .collect();
            let test_value: u32 = 0x11223344;
            {
                // write the block of id 0
                let block_cache = &block_cache_list[0];
                let mut locked_cache = block_cache.lock();
                locked_cache.write(0, |v: &mut u32| {
                    *v = test_value;
                });
            }
            // Access block caches other than block 0.
            // In this way, the block 0 will be swapped out when a new block comes in.
            for block_id in 1..4usize {
                let _ = block_cache_mgr.get_block_cache(block_id);
            }
            // all block caches will not be referenced
            drop(block_cache_list);
            // access a new block
            let _ = block_cache_mgr.get_block_cache(5);
            // block 0 should be swapped out, thus the changes on it should be synced to device
            let mut buf = [0u8; BLOCK_SZ];
            block_dev.read_block(0, &mut buf);
            assert_eq!(&buf[0..4], test_value.to_ne_bytes());
            assert!(&buf[4..].iter().all(|byte| *byte == 0));
        }

        #[test]
        #[should_panic]
        pub fn test_all_caches_referenced() {
            let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<16>::new());
            let mut block_cache_mgr: BlockCacheMgr<4, RawSpinlock> = BlockCacheMgr::new(&block_dev);
            let _block_cache_list: Vec<_> = (0..4)
                .map(|block_id| block_cache_mgr.get_block_cache(block_id))
                .collect();
            let _ = block_cache_mgr.get_block_cache(5);
        }
    }
}
