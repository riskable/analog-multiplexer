#![no_std]
//! This crate provides an interface, `Multiplexer` that makes it trivially easy
//! to select channels on any given 74HC4051 or 74HC4067 series analog multiplexer.
//! Internally it keeps track of each multiplexer's state, allowing you to
//! check what channel is presently active or to enable/disable the multiplexer
//! at will.
//!
//! # Example using a 74HC4067 with a Blue Pill (stm32f104) board
//!
//! ```
//! // NOTE: This is pseudocode. It's just meant to get the concept across :)
//! use analog_multiplexer::Multiplexer; // Important part
//!
//! use stm32f1xx_hal::gpio::State;
//! // The pins we're using:
//! use stm32f1xx_hal::gpio::gpiob::{PB0, PB5, PB12, PB13, PB14, PB15};
//! use stm32f1xx_hal::{adc}; // So we can read an analog pin (PB0)
//!
//! fn main() {
//!     // stm32f1xx_hal boilerplate...
//!     let device = pac::Peripherals::take().unwrap();
//!     let mut flash = device.FLASH.constrain();
//!     let mut rcc = device.RCC.constrain();
//!     let mut _afio = device.AFIO.constrain(&mut rcc.apb2);
//!     let _clocks = rcc
//!         .cfgr
//!         .use_hse(8.mhz())
//!         .sysclk(72.mhz())
//!         .pclk1(36.mhz())
//!         .freeze(&mut flash.acr);
//!     // Setup ADC (we're using ADC1 for this example since we're reading PB0)
//!     let adc1 = adc::Adc::adc1(device.ADC1, &mut rcc.apb2, _clocks);
//!     // Setup GPIOB (so we can access the ADC via PB0)
//!     let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
//!     // Configure PB0 as an analog input (all channels lead to this analog input pin!)
//!     let analog_pin = gpiob.pb0.into_analog(&mut gpiob.crl);
//!     // Setup PB12-PB15 for accessing S0-S3 on the 74HC4067 multiplexer
//!     let s0 = gpiob
//!         .pb12
//!         .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
//!     let s1 = gpiob
//!         .pb13
//!         .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
//!     let s2 = gpiob
//!         .pb14
//!         .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
//!     let s3 = gpiob
//!         .pb15
//!         .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
//!     // NOTE: On some multiplexers the S0-S3 pins are labeled A, B, C, D
//!     // Enable pin...  If you want to be able to enable/disable the multiplexer on-the-fly
//!     let en = gpiob
//!         .pb5
//!         .into_push_pull_output_with_state(&mut gpiob.crl, State::Low);
//!     // TIP: Just run a wire from EN to GND to keep it enabled all the time
//!     // Multiplexer pins are given as a tuple in the order S0-S3 then enable pin (EN):
//!     let pins = (s0,s1,s2,s3,en); // For 16-channel
//!     // let pins = (s0,s1,s2,en); // For 8-channel
//!     let mut multiplexer = Multiplexer::new(pins); // The important part!
//!     multiplexer.enable(); // Make sure it's enabled (if using EN pin)
//!     loop {
//!         for chan in 0..multiplexer.num_channels {
//!             multiplexer.set_channel(chan); // Change the channel
//!             let data: u16 = adc1.read(&mut *analog_pin).unwrap();
//!             // Do something with the data here
//!         }
//!     }
//! }
//!
//! ```
//!
//! **NOTE:** There's a working Blue Pill/RTIC example in the `examples` directory.
//!

extern crate embedded_hal as hal;

use hal::digital::v2::OutputPin;

/// Provides an interface for setting the active channel
/// and enabling/disabling an 8-channel (74HC4051) or
/// 16-channel (74HC4067) analog multiplexer.  It also
/// keeps track of which channel is currently active
/// (`active_channel`) and provides a convenient
/// `num_channels` field that can be used to iterate
/// over all the multiplexer's channels.
pub struct Multiplexer<Pins> {
    pub pins: Pins,
    pub num_channels: u8,
    pub active_channel: u8,
    pub enabled: bool,
}

/// A trait so we can support both 8-channel and 16-channel
/// multiplexers simultaneously by merely instantiating them
/// with a 5 (16-channel) or 4 (8-channel) member tuple of
/// `OutputPin`s.
pub trait Output {
    fn set_channel(&mut self, channel: u8);
    fn enable(&mut self);
    fn disable(&mut self);
    fn num_channels(&self) -> u8;
}

/// A 5-pin implementation to support 16-channel multiplexers (e.g. 74HC4067)
impl<
        E,
        S0: OutputPin<Error = E>, // aka "A"
        S1: OutputPin<Error = E>, // aka "B"
        S2: OutputPin<Error = E>, // aka "C"
        S3: OutputPin<Error = E>, // aka "D"
        EN: OutputPin<Error = E>, // aka "Inhibit"
    > Output for (S0, S1, S2, S3, EN)
{
    /// Sets the current active channel on the multiplexer (0-15)
    fn set_channel(&mut self, channel: u8) {
        // NOTE: Figuring out the binary math on this was not fun.  Not fun at all!
        // Thanks to @grantm11235:matrix.org for showing me the way =)
        if channel & (1 << 0) == 0 {
            self.0.set_low().ok();
        } else {
            self.0.set_high().ok();
        }

        if channel & (1 << 1) == 0 {
            self.1.set_low().ok();
        } else {
            self.1.set_high().ok();
        }

        if channel & (1 << 2) == 0 {
            self.2.set_low().ok();
        } else {
            self.2.set_high().ok();
        }

        if channel & (1 << 3) == 0 {
            self.3.set_low().ok();
        } else {
            self.3.set_high().ok();
        }
    }

    /// Brings the `EN` pin low to enable the multiplexer
    fn enable(&mut self) {
        self.4.set_low().ok();
    }

    /// Brings the `EN` pin high to disable the multiplexer
    fn disable(&mut self) {
        self.4.set_high().ok();
    }

    /// Returns the number of channels supported by this multiplexer
    /// (so you can easily iterate over them).
    fn num_channels(&self) -> u8 {
        16
    }
}

/// A 4-pin implementation to support 8-channel multiplexers (e.g. 74HC4051)
impl<
        E,
        S0: OutputPin<Error = E>,
        S1: OutputPin<Error = E>,
        S2: OutputPin<Error = E>,
        EN: OutputPin<Error = E>,
    > Output for (S0, S1, S2, EN)
{
    /// Sets the current active channel on the multiplexer (0-7)
    fn set_channel(&mut self, channel: u8) {
        if channel & (1 << 0) == 0 {
            self.0.set_low().ok();
        } else {
            self.0.set_high().ok();
        }

        if channel & (1 << 1) == 0 {
            self.1.set_low().ok();
        } else {
            self.1.set_high().ok();
        }

        if channel & (1 << 2) == 0 {
            self.2.set_low().ok();
        } else {
            self.2.set_high().ok();
        }
    }

    /// Brings the `EN` pin low to enable the multiplexer
    fn enable(&mut self) {
        self.3.set_low().ok();
    }

    /// Brings the `EN` pin high to disable the multiplexer
    fn disable(&mut self) {
        self.3.set_high().ok();
    }

    /// Returns the number of channels supported by this multiplexer
    /// (so you can easily iterate over them).
    fn num_channels(&self) -> u8 {
        8
    }
}

impl<Pins: Output> Multiplexer<Pins> {
    /// Given a 5 or 4-member tuple, `(s0, s1, s2, s3, en)` or
    /// `(s0, s1, s2, en)` where every member is an `OutputPin`,
    /// returns a new instance of `Multiplexer` for a
    /// 16-channel or 8-channel analog multiplexer, respectively.
    ///
    /// **NOTE:** Some multiplexers label S0-S3 as A-D. They're
    /// the same thing.
    pub fn new(mut pins: Pins) -> Self {
        // Default to enabled on channel 0
        let active_channel = 0;
        let enabled = true;
        pins.enable();
        pins.set_channel(0);
        // For quick reference later:
        let num_channels = pins.num_channels();

        Self {
            pins,
            num_channels,
            active_channel,
            enabled,
        }
    }

    /// Sets the current active channel on the multiplexer
    /// (0 up to `num_channels`) and records that state in
    /// `self.active_channel`
    pub fn set_channel(&mut self, channel: u8) {
        self.pins.set_channel(channel);
        self.active_channel = channel;
    }

    /// Enables the multiplexer and sets `self.enabled = true`
    pub fn enable(&mut self) {
        self.pins.enable();
        self.enabled = true;
    }

    /// Disables the multiplexer and sets `self.enabled = false`
    pub fn disable(&mut self) {
        self.pins.enable();
        self.enabled = false;
    }
}
