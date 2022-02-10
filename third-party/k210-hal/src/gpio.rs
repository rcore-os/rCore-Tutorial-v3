//! General Purpose Input/Output (GPIO)

use core::marker::PhantomData;
use crate::pac;
use crate::sysctl::{self, APB0};
use crate::fpioa::{IoPin, Pull, Mode};
use crate::bit_utils::{u32_set_bit, u32_toggle_bit, u32_bit_is_set, u32_bit_is_clear};
use embedded_hal::digital::{OutputPin, StatefulOutputPin, InputPin, ToggleableOutputPin};

/// Extension trait to split a GPIO peripheral into independent pins
pub trait GpioExt {
    /// Split the GPIO peripheral into parts
    fn split(self, apb0: &mut APB0) -> Parts;
}

macro_rules! def_gpio_pins {
    ($($GPIOX: ident: ($num: expr, $gpiox: ident, $func: ident);)+) => {
        
impl GpioExt for pac::GPIO {
    fn split(self, apb0: &mut APB0) -> Parts {
        // enable APB0 bus
        apb0.enable();
        // enable sysctl peripheral
        sysctl::clk_en_peri().modify(|_r, w| w.gpio_clk_en().set_bit());
        // return ownership
        Parts { 
            $( $gpiox: $GPIOX { _ownership: () }, )+
        }
    }
}

/// GPIO peripheral parts
pub struct Parts {
    $(
        /// GPIO pin
        pub $gpiox: $GPIOX,
    )+
}

pub use gpio_pins::*;

/// All GPIO pins
pub mod gpio_pins {
    use super::GpioIndex;
$(
    /// GPIO pin
    pub struct $GPIOX {
        pub(crate) _ownership: ()
    }

    impl GpioIndex for $GPIOX {
        type FUNC = crate::fpioa::functions::$func;
        const INDEX: u8 = $num;
    }
)+
}
    };
}

def_gpio_pins! {
    GPIO0: (0, gpio0, GPIO0);
    GPIO1: (1, gpio1, GPIO1);
    GPIO2: (2, gpio2, GPIO2);
    GPIO3: (3, gpio3, GPIO3);
    GPIO4: (4, gpio4, GPIO4);
    GPIO5: (5, gpio5, GPIO5);
    GPIO6: (6, gpio6, GPIO6);
    GPIO7: (7, gpio7, GPIO7);
}

/// GPIO Index
pub trait GpioIndex {
    type FUNC;
    const INDEX: u8;
}

/// Unknown mode (type state)
pub struct Unknown;

/// Input mode (type state)
pub struct Input<MODE>(MODE);

/// Floating input (type state)
pub struct Floating;

/// Pull down input (type state)
pub struct PullDown;

/// Pull up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output;

/// Marker trait for active states
pub trait Active {}

impl Active for Unknown {}

impl Active for Input<Floating> {}

impl Active for Input<PullUp> {}

impl Active for Input<PullDown> {}

impl Active for Output {}

/// GPIO wrapper struct
pub struct Gpio<GPIO, PIN, MODE> {
    gpio: GPIO,
    pin: PIN,
    _mode: PhantomData<MODE>,
}

impl<GPIO: GpioIndex, PIN: Mode<GPIO::FUNC>> Gpio<GPIO, PIN, Unknown> {
    pub fn new(gpio: GPIO, pin: PIN) -> Gpio<GPIO, PIN, Unknown> {
        Gpio { gpio, pin, _mode: PhantomData }
    }
}

impl<GPIO, PIN, MODE> Gpio<GPIO, PIN, MODE> {
    pub fn free(self) -> (GPIO, PIN) {
        (self.gpio, self.pin)
    }
}

impl<GPIO: GpioIndex, PIN: IoPin, MODE: Active> Gpio<GPIO, PIN, MODE> {
    pub fn into_floating_input(mut self) -> Gpio<GPIO, PIN, Input<Floating>> {
        self.pin.set_io_pull(Pull::None);
        self.direction_in();
        Gpio { gpio: self.gpio, pin: self.pin, _mode: PhantomData }
    }

    pub fn into_pull_up_input(mut self) -> Gpio<GPIO, PIN, Input<PullUp>> {
        self.pin.set_io_pull(Pull::Up);
        self.direction_in();
        Gpio { gpio: self.gpio, pin: self.pin, _mode: PhantomData }
    }

    pub fn into_pull_down_input(mut self) -> Gpio<GPIO, PIN, Input<PullDown>> {
        self.pin.set_io_pull(Pull::Down);
        self.direction_in();
        Gpio { gpio: self.gpio, pin: self.pin, _mode: PhantomData }
    }

    pub fn into_push_pull_output(mut self) -> Gpio<GPIO, PIN, Output> {
        self.pin.set_io_pull(Pull::Down);
        self.direction_out();
        Gpio { gpio: self.gpio, pin: self.pin, _mode: PhantomData }
    }

    #[inline]
    fn direction_in(&mut self) {
        unsafe { 
            let p = &(*pac::GPIO::ptr()).direction as *const _ as *mut _;
            u32_set_bit(p, false, GPIO::INDEX as usize);
        }
    }

    #[inline]
    fn direction_out(&mut self) {
        unsafe { 
            let p = &(*pac::GPIO::ptr()).direction as *const _ as *mut _;
            u32_set_bit(p, true, GPIO::INDEX as usize);
        }
    }
}

impl<GPIO: GpioIndex, PIN, MODE> InputPin for Gpio<GPIO, PIN, Input<MODE>> {
    type Error = core::convert::Infallible;

    fn try_is_high(&self) -> Result<bool, Self::Error> { 
        Ok(unsafe { 
            let p = &(*pac::GPIO::ptr()).data_input as *const _ as *const _;
            u32_bit_is_set(p, GPIO::INDEX as usize)
        })
    }

    fn try_is_low(&self) -> Result<bool, Self::Error> { 
        Ok(unsafe { 
            let p = &(*pac::GPIO::ptr()).data_input as *const _ as *const _;
            u32_bit_is_clear(p, GPIO::INDEX as usize)
        })
    }
}

impl<GPIO: GpioIndex, PIN> OutputPin for Gpio<GPIO, PIN, Output> {
    type Error = core::convert::Infallible;

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { 
            let p = &(*pac::GPIO::ptr()).data_output as *const _ as *mut _;
            u32_set_bit(p, true, GPIO::INDEX as usize);
        }
        Ok(())
    }

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { 
            let p = &(*pac::GPIO::ptr()).data_output as *const _ as *mut _;
            u32_set_bit(p, false, GPIO::INDEX as usize);
        }
        Ok(())
    }
}

impl<GPIO: GpioIndex, PIN> StatefulOutputPin for Gpio<GPIO, PIN, Output> {
    fn try_is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { 
            let p = &(*pac::GPIO::ptr()).data_output as *const _ as *const _;
            u32_bit_is_set(p, GPIO::INDEX as usize)
        })
    }

    fn try_is_set_low(&self) -> Result<bool, Self::Error> { 
        Ok(unsafe { 
            let p = &(*pac::GPIO::ptr()).data_output as *const _ as *const _;
            u32_bit_is_clear(p, GPIO::INDEX as usize)
        })
    }
}

impl<GPIO: GpioIndex, PIN> ToggleableOutputPin for Gpio<GPIO, PIN, Output> {
    type Error = core::convert::Infallible;

    fn try_toggle(&mut self) -> Result<(), Self::Error> { 
        unsafe { 
            let p = &(*pac::GPIO::ptr()).data_output as *const _ as *mut _;
            u32_toggle_bit(p, GPIO::INDEX as usize);
        }
        Ok(())
    }
}
