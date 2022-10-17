#![no_std]
#![no_main]

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

    loop {
        // get an iterator over the entire file that converts to 32-bit
        for word in wav.stream32() {
            // send our mono output to both left and right channels
            sound.write(word, word);
        }
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
