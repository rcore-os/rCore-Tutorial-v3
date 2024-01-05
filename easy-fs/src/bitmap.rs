use super::{BlockCacheMgr, BLOCK_SZ};

/// A bitmap block
type BitmapBlock = [u64; 64];
/// Number of bits in a block
const BLOCK_BITS: usize = BLOCK_SZ * 8;
/// A bitmap
#[derive(Default)]
pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

/// Decompose bits into (block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}

impl Bitmap {
    /// A new bitmap from start block id and number of blocks
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }
    /// Allocate a new block from a block device
    pub fn alloc<const N: usize, R>(&self, bcache_mgr: &mut BlockCacheMgr<N, R>) -> Option<usize>
    where
        R: lock_api::RawMutex,
    {
        for block_id in 0..self.blocks {
            let real_block_id = block_id + self.start_block_id as usize;
            let pos = bcache_mgr.write_block(real_block_id, 0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // modify cache
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
                } else {
                    None
                }
            });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }
    /// Deallocate a block
    pub fn dealloc<const N: usize, R>(&self, bcache_mgr: &mut BlockCacheMgr<N, R>, bit: usize)
    where
        R: lock_api::RawMutex,
    {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        let real_block_id = block_pos + self.start_block_id;
        bcache_mgr.write_block(real_block_id, 0, |bitmap_block: &mut BitmapBlock| {
            assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
            bitmap_block[bits64_pos] -= 1u64 << inner_pos;
        });
    }
    /// Get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_helper::*, BlockCacheMgr};
    use std::sync::Arc;

    #[test]
    pub fn test_bitmap() {
        let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<16>::new());
        let mut bcache_mgr: BlockCacheMgr<4, RawSpinlock> = BlockCacheMgr::new(&block_dev);
        let bitmap = Bitmap::new(0, 16);
        assert_eq!(bitmap.maximum(), BLOCK_BITS * 16);

        let mut buf = [0u8; BLOCK_SZ];
        assert_eq!(bitmap.alloc(&mut bcache_mgr), Some(0));
        assert_eq!(bitmap.alloc(&mut bcache_mgr), Some(1));
        bcache_mgr.sync_all();
        block_dev.read_block(0, &mut buf);
        assert_eq!(buf[..8], 3u64.to_ne_bytes());
        assert!(buf[8..].iter().all(|byte| *byte == 0));

        bitmap.dealloc(&mut bcache_mgr, 0);
        bcache_mgr.sync_all();
        block_dev.read_block(0, &mut buf);
        assert_eq!(buf[..8], 2u64.to_ne_bytes());

        bitmap.dealloc(&mut bcache_mgr, 1);
        bcache_mgr.sync_all();
        block_dev.read_block(1, &mut buf);
        assert!(buf.iter().all(|byte| *byte == 0));
    }

    #[test]
    #[should_panic]
    pub fn test_bitmap_panic() {
        let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<16>::new());
        let mut bcache_mgr: BlockCacheMgr<4, RawSpinlock> = BlockCacheMgr::new(&block_dev);
        let bitmap = Bitmap::new(0, 1);
        assert_eq!(bitmap.alloc(&mut bcache_mgr), Some(0));
        bitmap.dealloc(&mut bcache_mgr, 1);
    }

    #[test]
    pub fn test_bitmap_large() {
        let block_dev: Arc<dyn BlockDevice> = Arc::new(TestBlockDevice::<16>::new());
        let mut bcache_mgr: BlockCacheMgr<4, RawSpinlock> = BlockCacheMgr::new(&block_dev);
        let bitmap = Bitmap::new(0, 16);
        let mut buf = [0u8; BLOCK_SZ];

        for i in 0..16 * BLOCK_BITS {
            assert_eq!(bitmap.alloc(&mut bcache_mgr), Some(i));
        }
        bcache_mgr.sync_all();
        for block_id in 0..16usize {
            block_dev.read_block(block_id, &mut buf);
            assert!(buf.iter().all(|byte| *byte == u8::MAX));
        }

        for i in 0..16 * BLOCK_BITS {
            bitmap.dealloc(&mut bcache_mgr, i);
        }
        bcache_mgr.sync_all();
        for block_id in 0..16usize {
            block_dev.read_block(block_id, &mut buf);
            assert!(buf.iter().all(|byte| *byte == 0));
        }
    }
}
