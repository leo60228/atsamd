#![no_main]
#![no_std]

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::hal;
use xiao_m0 as bsp;

use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = bsp::Pins::new(peripherals.PORT);
    let mut led0 = pins.led0.into_open_drain_output(&mut pins.port);
    let mut led1 = pins.led1.into_open_drain_output(&mut pins.port);
    let mut led2 = pins.led2.into_open_drain_output(&mut pins.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut counter = 0u8;
    loop {
        counter = counter.wrapping_add(1);
        delay.delay_ms(100u8);
        if counter & (1 << 0) != 0 {
            led0.toggle();
        }
        if counter & (1 << 1) != 0 {
            led1.toggle();
        }
        if counter & (1 << 2) != 0 {
            led2.toggle();
        }
    }
}
