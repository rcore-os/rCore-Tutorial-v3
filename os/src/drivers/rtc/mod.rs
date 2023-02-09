use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::any::Any;
use core::fmt::{Debug, Formatter};
use bit_field::BitField;
use lazy_static::lazy_static;
use time::macros::offset;

const RTC_TIME_LOW: usize = 0x00;
const RTC_TIME_HIGH: usize = 0x04;
const RTC_ALARM_LOW: usize = 0x08;
const RTC_ALARM_HIGH: usize = 0x0c;
const RTC_IRQ_ENABLED: usize = 0x10;
const RTC_CLEAR_ALARM: usize = 0x14;
const RTC_ALARM_STATUS: usize = 0x18;
const RTC_CLEAR_INTERRUPT: usize = 0x1c;


const RTC_ADDR: usize = 0x101000;
const RTC_IRQ: u32 = 11;


pub trait RtcDevice:Send+Sync+Any{
    fn read_time(&self) -> RtcTime;
    fn clear_alarm(&self);
    fn clear_interrupt(&self);
    fn set_alarm_with_next_second(&self,second:u64);
    fn enable_irq(&self);
    fn disable_irq(&self);
    fn handle_irq(&self);
}

#[derive(Debug)]
pub struct Rtc {
    base_addr: usize,
}

#[derive(Copy, Clone)]
pub struct RtcTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Debug for RtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl ToString for RtcTime {
    fn to_string(&self) -> String {
        format!(
            "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day,self.hour, self.minute, self.second,
        )
    }
}

lazy_static!{
    pub static ref RTC: Arc<dyn RtcDevice> = Arc::new(Rtc::new(RTC_ADDR));
}


impl Rtc {
    fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
    pub fn read_time(&self) -> RtcTime {
        let ns = self.get_timestamp();
        let t = time::OffsetDateTime::from_unix_timestamp_nanos(ns as i128).expect("invalid time");
        let t = t.to_offset(offset!(+8));
        RtcTime {
            year: t.year() as u32,
            month: t.month() as u8,
            day: t.day(),
            hour: t.hour(),
            minute: t.minute(),
            second: t.second(),
        }
    }
    pub fn get_timestamp(&self) -> u64 {
        let ns_low = unsafe { ((self.base_addr + RTC_TIME_LOW) as *const u32).read_volatile() };
        let ns_high = unsafe { ((self.base_addr + RTC_TIME_HIGH) as *const u32).read_volatile() };
        // 将ns转换为当前时间
        let ns = (ns_high as u64) << 32 | ns_low as u64;
        ns
    }
}

impl RtcDevice for Rtc {
    fn read_time(&self) -> RtcTime {
        self.read_time()
    }
    fn clear_alarm(&self) {
        unsafe {
            ((self.base_addr + RTC_CLEAR_ALARM) as *mut u32).write_volatile(1);
        }
    }
    fn clear_interrupt(&self) {
        unsafe {
            ((self.base_addr + RTC_CLEAR_INTERRUPT) as *mut u32).write_volatile(1);
        }
    }
    /// set alarm with next second
    fn set_alarm_with_next_second(&self, s: u64) {
        let ns = self.get_timestamp();
        let ns = 1e9 as u64 * s +  ns;
        unsafe {
            ((self.base_addr + RTC_ALARM_LOW) as *mut u32)
                .write_volatile((ns&0xffffffff) as u32);
            ((self.base_addr + RTC_ALARM_HIGH) as *mut u32).write_volatile((ns >> 32) as u32);
        }
    }
    /// enable interrupt
    fn enable_irq(&self) {
        unsafe {
            ((self.base_addr + RTC_IRQ_ENABLED) as *mut u32).write_volatile(1);
        }
    }
    /// disable interrupt
    fn disable_irq(&self) {
        unsafe {
            ((self.base_addr + RTC_IRQ_ENABLED) as *mut u32).write_volatile(0);
        }
    }
    fn handle_irq(&self) {
        self.clear_alarm();
        self.clear_interrupt();
        self.set_alarm_with_next_second(1);
    }
}

pub fn get_rtc_time() -> RtcTime {
    RTC.read_time()
}

pub fn set_rtc_alarm_with_next_second(s: u64) {
    RTC.set_alarm_with_next_second(s);
}

pub fn enable_rtc_irq() {
    RTC.enable_irq();
}

pub fn disable_rtc_irq() {
    RTC.disable_irq();
}