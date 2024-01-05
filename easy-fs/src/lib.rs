#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

//! EasyFileSystem

extern crate alloc;

mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod layout;
#[cfg(test)]
mod test_helper;
mod vfs;

/// Each block is of 512 bytes.
pub const BLOCK_SZ: usize = 512;
pub use block_cache::{BlockCache, BlockCacheMgr};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use vfs::Inode;
