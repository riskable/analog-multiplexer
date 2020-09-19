#![deny(unsafe_code)]
#![no_main]
#![no_std]

//! **NOTE:** For this example to work you need a Blue Pill board connected
//! to an ST-LINK debugger and [probe-run](1).  A `.cargo/config` file is
//! included and configured with `runner = "probe-run --chip STM32F103C8"`.
//!
//! This example reads the state of all channels on a 16 or 8-channel
//! multiplexer every 100ms and pretty-prints their values using
//! probe-rs (friends don't let friends use semihosting!).  By default it's
//! configured for a 16-channel multiplexer (74HC4067) so if you want to use
//! an 8-channel one you'll need to comment-swap a few lines (search for
//! "4051" to find them).
//!
//! You'll also need to change the pins to whatever pins you're using with
//! your own Blue Pill board.  The default configuration uses:
//!
//! * `PB0`: Connected to `Z` (analog output on the multilpexer)
//! * `PB5`: Connected to `EN` (enable pin)
//! * `PB12, PB13, PB14, PB15`: Connected to `S0, S1, S2, S3` (channel select pins)
//!
//! **NOTE::** You can just run `EN` to ground to keep it always enabled.
//!
//! [1]: https://github.com/knurling-rs/probe-run
//!

// The part that matters:
extern crate analog_multiplexer;
use analog_multiplexer::Multiplexer;
// This is just a convenient container for keeping track of channel data and pretty-printing it:
mod channels; // Look at channels/mod.rs if you're curious how it works

extern crate panic_halt;
use rtt_target::{rtt_init_print, rprintln};
use embedded_hal::digital::v2::OutputPin;
use cortex_m;
use rtic::app;
use rtic::cyccnt::U32Ext as _;
use stm32f1xx_hal::gpio::{gpioc::PC13, Output, PushPull, State, Analog};
// NOTE: Change these to the pins you plan to use for S0, S1, etc:
use stm32f1xx_hal::gpio::gpiob::{PB0, PB5, PB12, PB13, PB14, PB15};
// ...you'll also need to change them in the `type` declarations below...
use stm32f1xx_hal::{adc};
use stm32f1xx_hal::pac::{ADC1};
use stm32f1xx_hal::prelude::*;

// Define which pins go to where on your analog multiplexer (for RTIC's `Resources`)
type S0 = PB12<Output<PushPull>>; // These just make things easier to read/reason about
type S1 = PB13<Output<PushPull>>; // aka "very expressive"
type S2 = PB14<Output<PushPull>>;
type S3 = PB15<Output<PushPull>>; // You can comment this out if using 8-channel (74HC4051)
type EN = PB5<Output<PushPull>>; // NOTE: You can use an unused pin if not using this feature
// NOTE: embedded_hal really needs a DummyPin feature for things like unused driver pins!
// You can swap which line is commented below to use an 8-channel instead of 16:
type Multiplex = Multiplexer<(S0, S1, S2, S3, EN)>; // If using 16-channel (74HC4067)
// type Multiplex = Multiplexer<(S0, S1, S2, EN)>; // If using 8-channel (74HC4051)
// NOTE: If you swapped above you'll also need to swap the `let pins...` line below

const PERIOD: u32 = 10_000_000; // Update state (and blink LED 1/2) every 100ms

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f1xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init (RTIC stuff)
    struct Resources {
        led: PC13<Output<PushPull>>,
        multiplexer: Multiplex, // If we didn't use the types above this would be very messy
        ch_states: channels::ChannelValues,
        adc1: adc::Adc<ADC1>,
        analog_pin: PB0<Analog>,
    }

    // This is needed for probe-rs to work with RTIC at present (20200919).  Won't be needed forever (they're working on it =)
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {cortex_m::asm::nop();} // Don't do anything at all
    }

    // NOTE: Most of this is rtic and cortex-m boilerplate.
    // The multiplexer-specific stuff is noted via comments below...
    #[init(schedule = [readall])]
    fn init(cx: init::Context) -> init::LateResources {
        // Enable rtt-target/probe-rs debugging stuff
        rtt_init_print!();
        rprintln!("init multiplexer example()");

        // Enable cycle counter (so we can schedule things)
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let device: stm32f1xx_hal::stm32::Peripherals = cx.device;

        // Setup clocks (stm32f1xx_hal boilerplate)
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let mut _afio = device.AFIO.constrain(&mut rcc.apb2);
        let _clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        // Setup ADC (we're using ADC1 for this example since we're reding PB0)
        let adc1 = adc::Adc::adc1(device.ADC1, &mut rcc.apb2, _clocks);

        // Setup GPIOB (so we can access the ADC via PB0)
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

        // Configure PB0 as an analog input (all channels lead to this analog input pin!)
        let analog_pin = gpiob.pb0.into_analog(&mut gpiob.crl);

        // Setup the Blue Pill's built-in LED for blinkage (blink-rage?)
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);

        // Setup PB12-PB15 for accessing S0-S3 on the 74HC4067 multiplexer
        let s0 = gpiob
            .pb12
            .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
        let s1 = gpiob
            .pb13
            .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
        let s2 = gpiob
            .pb14
            .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
        // You can comment this one out if using an 8-channel (74HC4051):
        let s3 = gpiob
            .pb15
            .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);

        // NOTE: We need something like a DummyPin option in embedded_hal!
        // Enable pin...  If you want to be able to enable/disable the multiplexer on-the-fly
        let en = gpiob
            .pb5
            .into_push_pull_output_with_state(&mut gpiob.crl, State::Low);
            // Just run a wire from EN to GND to keep it enabled all the time

        // ** ANALOG MULTIPLEXER STUFF **
        // Setup the Multiplexer with our configured pins (swap comments below for 8 channel)
        let pins = (s0,s1,s2,s3,en); // For 16-channel (74HC4067)
        // let pins = (s0,s1,s2,en); // For 8-channel (74HC4051)
        let mut multiplexer = Multiplexer::new(pins);
        multiplexer.enable(); // Make sure it's enabled

        // Keep track of channel states/values (for pretty printing)
        let ch_states: channels::ChannelValues = Default::default();

        // Schedule the reading/blinking task
        cx.schedule.readall(cx.start + PERIOD.cycles()).unwrap();

        init::LateResources {
            led: led,
            multiplexer: multiplexer,
            ch_states: ch_states,
            analog_pin: analog_pin, // NOTE: Wish we didn't need BOTH
            adc1: adc1              // the pin and the ADC to read it
        }
    }

    #[task(resources = [led, multiplexer, ch_states, analog_pin, adc1], schedule = [readall])]
    fn readall(cx: readall::Context) {
        // Use the safe local `static mut` of RTIC
        static mut LED_STATE: bool = false; // RTIC's blink scheduler boilerplate
        // These are just here to keep the code nice and concise:
        let multiplexer = cx.resources.multiplexer;
        let ch_states = cx.resources.ch_states;
        let adc1 = cx.resources.adc1;
        let led = cx.resources.led;

        // ** ANALOG MULTIPLEXER STUFF **
        // Cycle through all channels and record the value of each in ChannelValues (ch_states)
        for chan in 0..multiplexer.num_channels {
            multiplexer.set_channel(chan); // NOTE: Changing channels takes at most 7ns
            let data: u16 = adc1.read(&mut *cx.resources.analog_pin).unwrap();
            ch_states.update_by_index(chan as u8, data);
        }
        rprintln!("{}", ch_states); // probe-rs goodness

        if *LED_STATE {
            led.set_high().unwrap();
            *LED_STATE = false;
        } else {
            led.set_low().unwrap();
            *LED_STATE = true;
        }
        // See you in the next PERIOD!
        cx.schedule.readall(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
