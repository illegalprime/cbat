use bsp::hal::gpio::v2::Reset;
use itsybitsy_m0 as bsp;
use bsp::hal::gpio::v2::AlternateG;
use bsp::hal::gpio::v2::PA07;
use bsp::hal::gpio::v2::PA10;
use bsp::hal::gpio::v2::PA11;
use bsp::hal::gpio::v2::Pin;
use bsp::pac::GCLK;
use bsp::pac::I2S;
use bsp::pac::PM;

const SLOTS: u8 = 2;
const GCLK3: u8 = 3;
const DIVIDE: u16 = 17; // 48_000_000 / (44_100 * 32 * 2)


pub struct I2s {
    _d0: Pin<PA11, AlternateG>,
    _d1: Pin<PA10, AlternateG>,
    _d9: Pin<PA07, AlternateG>,
}

impl I2s {
    /// Configure pins and clocks for I2S.
    /// Currently only configures I2S for 32-bit Stereo at 44,100Hz
    pub fn init(
        d0: Pin<PA11, Reset>,
        d1: Pin<PA10, Reset>,
        d9: Pin<PA07, Reset>,
    ) -> Self {
        // clock pin
        let d1: Pin<PA10, AlternateG> = d1.into_alternate();

        // frame sync pin
        let d0: Pin<PA11, AlternateG> = d0.into_alternate();

        unsafe {
            while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}

            (*GCLK::ptr()).gendiv.write(|w| {
                w
                    .id().bits(GCLK3)
                    .div().bits(DIVIDE)
            });
        }

        unsafe {
            while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}

            (*GCLK::ptr()).genctrl.write(|w| {
                w
                    .id().bits(GCLK3)
                    .src().dfll48m()
                    .idc().set_bit()
                    .genen().set_bit()
            });
        }

        unsafe {
            while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}

            (*GCLK::ptr()).clkctrl.write(|w| {
                w
                    .id().i2s_0()
                    .gen().gclk3()
                    .clken().set_bit()
            });
        }

        unsafe {
            while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}
        }

        let d9: Pin<PA07, AlternateG> = d9.into_alternate();

        unsafe {
            (*PM::ptr()).apbcmask.write(|w| {
                w.i2s_().set_bit()
            });
        }

        unsafe {
            (*I2S::ptr()).ctrla.write(|w| {
                w.enable().clear_bit()
            });

            while (*I2S::ptr()).syncbusy.read().enable().bit_is_set() {}
        }

        unsafe {
            (*I2S::ptr()).ctrla.write(|w| {
                w.cken0().clear_bit()
            });

            while (*I2S::ptr()).syncbusy.read().cken0().bit_is_set() {}
        }

        unsafe {
            (*I2S::ptr()).clkctrl[0].write(|w| {
                w
                    .mcksel().gclk()
                    .scksel().mckdiv()
                    .fssel().sckdiv()
                    .bitdelay().i2s()
                    .nbslots().bits(SLOTS - 1)
                    .slotsize()._32()
            });
        }

        unsafe {
            (*I2S::ptr()).ctrla.write(|w| {
                w.seren0().clear_bit()
            });

            while (*I2S::ptr()).syncbusy.read().seren0().bit_is_set() {}
        }

        unsafe {
            (*I2S::ptr()).serctrl[0].write(|w| {
                w
                    .dma().single()
                    .mono().stereo()
                    .bitrev().msbit()
                    .extend().zero()
                    .wordadj().right()
                    .datasize()._32()
                    .slotadj().right()
                    .clksel().clk0()
            });
        }

        Self {
            _d0: d0,
            _d1: d1,
            _d9: d9,
        }
    }

    /// Enable I2S Transmit only, transmit data by calling `I2s::write`
    pub fn enable(&mut self) {
        unsafe {
            (*I2S::ptr()).ctrla.write(|w| {
                w.enable().clear_bit()
            });

            while (*I2S::ptr()).syncbusy.read().enable().bit_is_set() {}
        }

        unsafe {
            (*I2S::ptr()).serctrl[0].write(|w| {
                w.sermode().tx()
            });
        }

        unsafe {
            (*I2S::ptr()).ctrla.write(|w| {
                w
                    .seren0().set_bit()
                    .cken0().set_bit()
                    .enable().set_bit()
            });

            while (*I2S::ptr()).syncbusy.read().bits() != 0 {} // all bits
        }
    }

    /// Write left and right channels.
    /// Will block until data can be sent before sending.
    pub fn write(&mut self, left: u32, right: u32) {
        for word in [left, right] {
            unsafe {
                // wait for it to be ready to accept more data
                while
                    (*I2S::ptr()).intflag.read().txrdy0().bit_is_clear() ||
                    (*I2S::ptr()).syncbusy.read().data0().bit_is_set()
                {}

                // clear any existing under-run flags
                (*I2S::ptr()).intflag.write(|w| {
                    w.txur0().set_bit()
                });

                // write our 16-bit data into the 32-bit register
                (*I2S::ptr()).data[0].write(|w| {
                    w.bits(word)
                });
            }
        }
    }
}
