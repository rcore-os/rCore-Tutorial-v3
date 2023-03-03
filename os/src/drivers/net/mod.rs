use core::any::Any;

use crate::drivers::virtio::VirtioHal;
use crate::sync::UPIntrFreeCell;
use alloc::sync::Arc;
use lazy_static::*;
use virtio_drivers::{VirtIOHeader, VirtIONet};

const VIRTIO8: usize = 0x10004000;

lazy_static! {
    pub static ref NET_DEVICE: Arc<dyn NetDevice> = Arc::new(VirtIONetWrapper::new());
}

pub trait NetDevice: Send + Sync + Any {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
}

pub struct VirtIONetWrapper(UPIntrFreeCell<VirtIONet<'static, VirtioHal>>);

impl NetDevice for VirtIONetWrapper {
    fn transmit(&self, data: &[u8]) {
        self.0
            .exclusive_access()
            .send(data)
            .expect("can't send data")
    }

    fn receive(&self, data: &mut [u8]) -> usize {
        self.0
            .exclusive_access()
            .recv(data)
            .expect("can't receive data")
    }
}

impl VirtIONetWrapper {
    pub fn new() -> Self {
        unsafe {
            let virtio = VirtIONet::<VirtioHal>::new(&mut *(VIRTIO8 as *mut VirtIOHeader))
                .expect("can't create net device by virtio");
            VirtIONetWrapper(UPIntrFreeCell::new(virtio))
        }
    }
}
