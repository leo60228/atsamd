//! Use a hardware timer to wiggle a pin by blocking until the timer is up
//! and then toggling a pin.

#![no_std]
#![no_main]

#[cfg(not(feature = "panic_led"))]
use panic_halt as _;
use pygamer::{self as hal, entry, pac, Pins};

use hal::clock::GenericClockController;
use hal::prelude::*;
use hal::timer::TimerCounter;
use pac::Peripherals;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = Pins::new(peripherals.PORT);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.MCLK);
    timer.start(250.khz());
    let mut d5 = pins.d5.into_push_pull_output(&mut pins.port);

    //50% duty cycle, so 500khz period
    loop {
        let _ = d5.set_high();
        let _ = nb::block!(timer.wait());
        let _ = d5.set_low();
        let _ = nb::block!(timer.wait());
    }
}
