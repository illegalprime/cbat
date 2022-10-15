#![no_std]
#![no_main]

use bsp::hal::gpio::v2::AlternateG;
use bsp::hal::gpio::v2::PA07;
use bsp::hal::gpio::v2::PA10;
use bsp::hal::gpio::v2::PA11;
use bsp::hal::gpio::v2::Pin;
use bsp::pac::GCLK;
use bsp::pac::I2S;
use bsp::pac::PM;
use panic_halt as _;

use bsp::hal;
use bsp::pac;
use itsybitsy_m0 as bsp;

use bsp::entry;
use hal::clock::GenericClockController;
use pac::Peripherals;

const SLOTS: u8 = 2;

#[entry]
fn main() -> ! {
    let data = [100, 200, 300, 400, 500];

    // setup
    let mut peripherals = Peripherals::take().unwrap();
    let _clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);

    // ARDUINO BEGIN

    // clock pin
    let _d1: Pin<PA10, AlternateG> = pins.d1.into_alternate();

    // frame sync pin
    let _d0: Pin<PA11, AlternateG> = pins.d0.into_alternate();

    unsafe {
        while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}

        (*GCLK::ptr()).gendiv.write(|w| {
            w
                .id().bits(3) // gclk3
                .div().bits(17)
        });
    }

    unsafe {
        while (*GCLK::ptr()).status.read().syncbusy().bit_is_set() {}

        (*GCLK::ptr()).genctrl.write(|w| {
            w
                .id().bits(3) // gclk3
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

    let _d9: Pin<PA07, AlternateG> = pins.d9.into_alternate();

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

    // ARDUINO ENABLE TX

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

    let mut cycle = (0..data.len()).into_iter().cycle();

    loop {
        unsafe {
            while
                (*I2S::ptr()).intflag.read().txrdy0().bit_is_clear() ||
                (*I2S::ptr()).syncbusy.read().data0().bit_is_set()
            {}

            (*I2S::ptr()).intflag.write(|w| {
                w.txur0().set_bit()
            });

            (*I2S::ptr()).data[0].write(|w| {
                w.bits(data[cycle.next().unwrap()] as u32)
            });
        }
    }
}
