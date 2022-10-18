use bsp::hal::clock::ClockGenId;
use bsp::hal::clock::ClockSource;
use bsp::hal::clock::GenericClockController;
use bsp::hal::clock::I2S0Clock;
use bsp::hal::gpio::v2::Reset;
use bsp::hal::gpio::v2::AlternateG;
use bsp::hal::gpio::v2::PA07;
use bsp::hal::gpio::v2::PA10;
use bsp::hal::gpio::v2::PA11;
use bsp::hal::gpio::v2::Pin;
use bsp::pac::I2S;
use bsp::pac::PM;
use itsybitsy_m0 as bsp;

const SLOTS: u8 = 2;    // 2 channels (left & right)
const DIVIDE: u16 = 17; // 48,000,000 Hz / 44,100 samples / 32 bits / SLOTS


pub struct I2s {
    _d0: Pin<PA11, AlternateG>,
    _d1: Pin<PA10, AlternateG>,
    _d9: Pin<PA07, AlternateG>,
    _i2s_clk: I2S0Clock,
    i2s: I2S,
}

impl I2s {
    /// Configure pins and clocks for I2S.
    /// Currently only configures I2S for 32-bit Stereo at 44,100Hz
    pub fn init(
        pins: (Pin<PA11, Reset>, Pin<PA10, Reset>, Pin<PA07, Reset>),
        clocks: &mut GenericClockController,
        pm: &mut PM,
        i2s: I2S,
    ) -> Self {
        // set pins into G state (I2S)
        let d0: Pin<_, AlternateG> = pins.0.into(); // left/right select
        let d1: Pin<_, AlternateG> = pins.1.into(); // bit clock
        let d9: Pin<_, AlternateG> = pins.2.into(); // serial data

        // enable generic clock 3, pick its source and divisor
        let gclk3 = clocks.configure_gclk_divider_and_source(
            ClockGenId::GCLK3,
            DIVIDE,
            ClockSource::DFLL48M,
            true,
        ).unwrap();

        // enable the i2s0 clock and set its source to generic clock 3
        let i2s_clk = clocks.i2s0(&gclk3).unwrap();

        // turn on i2s in the power manager
        pm.apbcmask.write(|w| w.i2s_().set_bit());

        // disable the i2s, its clock, and its serializer
        i2s.ctrla.write(|w| {
            w.cken0().clear_bit();
            w.seren0().clear_bit();
            w.enable().clear_bit();
            w
        });

        // wait for sync
        while i2s.syncbusy.read().bits() != 0 {}

        // configure the i2s clock
        i2s.clkctrl[0].write(|w| {
            w.mcksel().gclk();
            w.scksel().mckdiv();
            w.fssel().sckdiv();
            w.bitdelay().i2s();
            unsafe { w.nbslots().bits(SLOTS - 1); }
            w.slotsize()._32();
            w
        });

        // configure the i2s serializer
        i2s.serctrl[0].write(|w| {
            w.dma().single();
            w.mono().stereo();
            w.bitrev().msbit();
            w.extend().zero();
            w.wordadj().right();
            w.datasize()._32();
            w.slotadj().right();
            w.clksel().clk0();
            w.sermode().tx(); // we will always be using TX mode
            w
        });

        // wait for settings to get applied
        while i2s.syncbusy.read().bits() != 0 {}

        Self {
            _d0: d0,
            _d1: d1,
            _d9: d9,
            _i2s_clk: i2s_clk,
            i2s,
        }
    }

    fn switch(&mut self, state: bool) {
        // enable or disable everything
        self.i2s.ctrla.write(|w| {
            w.seren0().bit(state);
            w.cken0().bit(state);
            w.enable().bit(state);
            w
        });
        // wait for settings to get applied
        while self.i2s.syncbusy.read().bits() != 0 {}
    }

    /// Enable I2S transmit only, transmit data by calling `I2s::write`.
    pub fn enable(&mut self) {
        self.switch(true);
    }

    /// Disable I2S transmit.
    pub fn disable(&mut self) {
        self.switch(false);
    }

    /// Write left and right channels.
    /// Will block until data can be sent before sending.
    pub fn write(&mut self, left: u32, right: u32) {
        // subsequent writes to register alternate between left and right channels
        for word in [left, right] {
            // wait while TX is not ready or while data bit is syncing
            while
                self.i2s.intflag.read().txrdy0().bit_is_clear() ||
                self.i2s.syncbusy.read().data0().bit_is_set()
            {}

            // clear any existing under-run flags
            self.i2s.intflag.write(|w| w.txur0().set_bit());

            // write our 16-bit data into the 32-bit register
            self.i2s.data[0].write(|w| unsafe {
                w.data().bits(word)
            });
        }
    }
}
