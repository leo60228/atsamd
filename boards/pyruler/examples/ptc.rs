#![no_std]
#![no_main]

use bsp::hal;
use panic_probe as _;
use pyruler as bsp;
use rtt_target::{rprintln, rtt_init_print};

use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::gpio::v2::{Pin, B};
use hal::pac::{
    ptc::{convctrl::*, freqctrl::*, serres::*, yselect::*},
    CorePeripherals, Peripherals,
};
use hal::prelude::*;
use hal::thumbv6m::ptc::Ptc;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("initializing");
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);
    let mut red_led = Pin::from(pins.led5).into_push_pull_output();
    let mut micro_button = Pin::from(pins.cap1).into_alternate::<B>();

    rprintln!("getting ptc");
    let mut ptc = Ptc::ptc(peripherals.PTC, &mut peripherals.PM, &mut clocks);

    rprintln!("configuring ptc");
    ptc.yselect(YMUX_A::Y3);
    ptc.series_resistance(RESISTOR_A::RES50K);
    ptc.oversample(ADCACCUM_A::OVERSAMPLE4);
    ptc.sample_delay(SAMPLEDELAY_A::FREQHOP1);
    ptc.compcap(0x2000);
    ptc.intcap(0x3F);

    rprintln!("getting initial reading");
    let initial_value: u16 = ptc.read(&mut micro_button).unwrap_or(0xaaaa);
    let threshold = initial_value + 100;

    loop {
        rprintln!("measuring");
        let value: u16 = ptc.read(&mut micro_button).unwrap_or(0xaaaa);
        rprintln!("got {}", value);

        if value > threshold {
            red_led.set_high().unwrap();
        } else {
            red_led.set_low().unwrap();
        }

        delay.delay_ms(60u8);
    }
}
