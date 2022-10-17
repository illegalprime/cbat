#![no_std]
#![no_main]

use bsp::hal::gpio::v2::Pin;
use bsp::hal::gpio::v2::PullUpInput;
use bsp::hal::prelude::*;
use panic_halt as _;

use bsp::hal;
use bsp::pac;
use itsybitsy_m0 as bsp;

use bsp::entry;
use hal::clock::GenericClockController;
use pac::Peripherals;

mod i2s;
mod wav;

const WAV_DATA: &'static [u8] = include_bytes!("../res/sine-a1s.wav");


#[entry]
fn main() -> ! {
    // include all our data in the binary
    let wav = wav::Wav16::new(WAV_DATA);

    // setup pins
    let pins = setup();

    // set up our custom i2s implementation
    let mut sound = i2s::I2s::init(pins.d0, pins.d1, pins.d9);
    sound.enable();

    // our button input
    let btn: Pin<_, PullUpInput> = pins.d10.into_pull_up_input();

    // debug
    let mut red_led: bsp::RedLed = pins.d13.into();

    loop {
        // check if we pressed the button
        if btn.is_low().unwrap() {
            // signal that we're writing sound
            red_led.set_high().unwrap();
            // get an iterator over the entire file that converts to 32-bit
            for word in wav.stream32() {
                // send our mono output to both left and right channels
                sound.write(word, word);
            }
        }
        red_led.set_low().unwrap();
    }
}


fn setup() -> bsp::Pins {
    let mut peripherals = Peripherals::take().unwrap();
    let _clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    bsp::Pins::new(peripherals.PORT)
}
