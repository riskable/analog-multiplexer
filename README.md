# analog-multiplexer
A platform agnostic driver for 4051 and 4067 series analog multiplexers targetting the Rust embedded-hal

This crate provides an interface, `Multiplexer` that makes it trivially easy
to select channels on any given 74HC4051 or 74HC4067 series analog multiplexer.
Internally it keeps track of each multiplexer's state, allowing you to
check what channel is presently active or to enable/disable the multiplexer
at will.

# Supported Hardware (Analog Multiplexers)

* 4067 series: [74HC4067](https://assets.nexperia.com/documents/data-sheet/74HC_HCT4067.pdf)
* 4051 series: [74HC4051](https://www.ti.com/lit/ds/symlink/cd74hc4051-ep.pdf)
* ...and any other similar IC that uses three or four channel select pins

# Usage

Here's an imaginary example using a 74HC4067 with a Blue Pill (stm32f104) board...

```rust
// NOTE: This is pseudocode. It's just meant to get the concept across :)
use analog_multiplexer::Multiplexer; // Important part

use stm32f1xx_hal::gpio::State;
// The pins we're using:
use stm32f1xx_hal::gpio::gpiob::{PB0, PB5, PB12, PB13, PB14, PB15};
use stm32f1xx_hal::{adc}; // So we can read an analog pin (PB0)

fn main() {
    // stm32f1xx_hal boilerplate...
    let device = pac::Peripherals::take().unwrap();
    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();
    let mut _afio = device.AFIO.constrain(&mut rcc.apb2);
    let _clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);
    // Setup ADC (we're using ADC1 for this example since we're reading PB0)
    let adc1 = adc::Adc::adc1(device.ADC1, &mut rcc.apb2, _clocks);
    // Setup GPIOB (so we can access the ADC via PB0)
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    // Configure PB0 as an analog input (all channels lead to this analog input pin!)
    let analog_pin = gpiob.pb0.into_analog(&mut gpiob.crl);
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
    let s3 = gpiob
        .pb15
        .into_push_pull_output_with_state(&mut gpiob.crh, State::Low);
    // NOTE: On some multiplexers the S0-S3 pins are labeled A, B, C, D
    // Enable pin...  If you want to be able to enable/disable the multiplexer on-the-fly
    let en = gpiob
        .pb5
        .into_push_pull_output_with_state(&mut gpiob.crl, State::Low);
    // TIP: Just run a wire from EN to GND to keep it enabled all the time
    // Multiplexer pins are given as a tuple in the order S0-S3 then enable pin (EN):
    let pins = (s0,s1,s2,s3,en); // For 16-channel
    // let pins = (s0,s1,s2,en); // For 8-channel
    let mut multiplexer = Multiplexer::new(pins); // The important part!
    multiplexer.enable(); // Make sure it's enabled (if using EN pin)
    loop {
        for chan in 0..multiplexer.num_channels {
            multiplexer.set_channel(chan); // Change the channel
            let data: u16 = adc1.read(&mut *analog_pin).unwrap();
            // Do something with the data here
        }
    }
}

```

# Working Example

There's a *proper* working example in the `examples` directory (`read_all`) that uses [RTIC](https://rtic.rs) and probe-rs to great effect.  It requires an ST-LINK programmer, a Blue Pill board, and [probe-run](https://crates.io/crates/probe-run).

Here's me using it to read a hall effect sensor on channel 15:

![Reading a hall effect sensor on channel 15](https://thumbs.gfycat.com/FlippantAptHadrosaurus-size_restricted.gif)

# License

Licensed under:

    * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
