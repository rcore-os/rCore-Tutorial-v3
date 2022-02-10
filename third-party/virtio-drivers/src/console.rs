use super::*;
use crate::queue::VirtQueue;
use bitflags::*;
use core::{fmt, hint::spin_loop};
use log::*;
use volatile::{ReadOnly, WriteOnly};

const QUEUE_RECEIVEQ_PORT_0: usize = 0;
const QUEUE_TRANSMITQ_PORT_0: usize = 1;

/// Virtio console. Only one single port is allowed since ``alloc'' is disabled.
/// Emergency and cols/rows unimplemented.
pub struct VirtIOConsole<'a> {
    header: &'static mut VirtIOHeader,
    receiveq: VirtQueue<'a>,
    transmitq: VirtQueue<'a>,
    queue_buf_dma: DMA,
    queue_buf_rx: &'a mut [u8],
    cursor: usize,
    pending_len: usize,
}

impl<'a> VirtIOConsole<'a> {
    /// Create a new VirtIO-Console driver.
    pub fn new(header: &'static mut VirtIOHeader) -> Result<Self> {
        header.begin_init(|features| {
            let features = Features::from_bits_truncate(features);
            info!("Device features {:?}", features);
            let supported_features = Features::empty();
            (features & supported_features).bits()
        });
        let config = unsafe { &mut *(header.config_space() as *mut Config) };
        info!("Config: {:?}", config);
        let receiveq = VirtQueue::new(header, QUEUE_RECEIVEQ_PORT_0, 2)?;
        let transmitq = VirtQueue::new(header, QUEUE_TRANSMITQ_PORT_0, 2)?;
        let queue_buf_dma = DMA::new(1)?;
        let queue_buf_rx = unsafe { &mut queue_buf_dma.as_buf()[0..] };
        header.finish_init();
        let mut console = VirtIOConsole {
            header,
            receiveq,
            transmitq,
            queue_buf_dma,
            queue_buf_rx,
            cursor: 0,
            pending_len: 0,
        };
        console.poll_retrieve()?;
        Ok(console)
    }
    fn poll_retrieve(&mut self) -> Result<()> {
        self.receiveq.add(&[], &[self.queue_buf_rx])?;
        Ok(())
    }
    /// Acknowledge interrupt.
    pub fn ack_interrupt(&mut self) -> Result<bool> {
        let ack = self.header.ack_interrupt();
        if !ack {
            return Ok(false);
        }
        let mut flag = false;
        while let Ok((_token, len)) = self.receiveq.pop_used() {
            assert_eq!(flag, false);
            flag = true;
            assert_ne!(len, 0);
            self.cursor = 0;
            self.pending_len = len as usize;
        }
        Ok(flag)
    }

    /// Try get char.
    pub fn recv(&mut self, pop: bool) -> Result<Option<u8>> {
        if self.cursor == self.pending_len {
            return Ok(None);
        }
        let ch = self.queue_buf_rx[self.cursor];
        if pop {
            self.cursor += 1;
            if self.cursor == self.pending_len {
                self.poll_retrieve()?;
            }
        }
        Ok(Some(ch))
    }
    /// Put a char onto the device.
    pub fn send(&mut self, chr: u8) -> Result<()> {
        let buf: [u8; 1] = [chr];
        self.transmitq.add(&[&buf], &[])?;
        self.header.notify(QUEUE_TRANSMITQ_PORT_0 as u32);
        while !self.transmitq.can_pop() {
            spin_loop();
        }
        self.transmitq.pop_used()?;
        Ok(())
    }
}

#[repr(C)]
struct Config {
    cols: ReadOnly<u16>,
    rows: ReadOnly<u16>,
    max_nr_ports: ReadOnly<u32>,
    emerg_wr: WriteOnly<u32>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("cols", &self.cols)
            .field("rows", &self.rows)
            .field("max_nr_ports", &self.max_nr_ports)
            .finish()
    }
}

bitflags! {
    struct Features: u64 {
        const SIZE                  = 1 << 0;
        const MULTIPORT             = 1 << 1;
        const EMERG_WRITE           = 1 << 2;

        // device independent
        const NOTIFY_ON_EMPTY       = 1 << 24; // legacy
        const ANY_LAYOUT            = 1 << 27; // legacy
        const RING_INDIRECT_DESC    = 1 << 28;
        const RING_EVENT_IDX        = 1 << 29;
        const UNUSED                = 1 << 30; // legacy
        const VERSION_1             = 1 << 32; // detect legacy

        // since virtio v1.1
        const ACCESS_PLATFORM       = 1 << 33;
        const RING_PACKED           = 1 << 34;
        const IN_ORDER              = 1 << 35;
        const ORDER_PLATFORM        = 1 << 36;
        const SR_IOV                = 1 << 37;
        const NOTIFICATION_DATA     = 1 << 38;
    }
}
