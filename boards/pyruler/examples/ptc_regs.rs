#![no_std]
#![no_main]

use bsp::hal;
use panic_probe as _;
use pyruler as bsp;
use rtt_target::{rprintln, rtt_init_print};

use bsp::entry;
use hal::clock::GenericClockController;
use hal::gpio::v2::{Pin, B};
use hal::pac::{Peripherals, PTC};
use hal::prelude::*;
use hal::thumbv6m::clock::{ClockGenId, ClockSource};

fn measure(ptc: &PTC) -> u16 {
    rprintln!("selecting");
    ptc.yselect.write(|w| w.ymux().y3());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("setting resistor");
    ptc.serres.write(|w| w.resistor().res50k());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("setting oversampling");
    ptc.convctrl.modify(|_, w| w.adcaccum().oversample4());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("setting freqctrl");
    ptc.freqctrl
        .modify(|_, w| w.freqspreaden().clear_bit().sampledelay().freqhop1());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("setting capacitors");
    ptc.compcap.write(|w| unsafe { w.value().bits(0x3000) });
    while ptc.ctrlb.read().syncflag().bit() {}
    ptc.intcap.write(|w| unsafe { w.value().bits(0x3F) });
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("setting burstmode");
    ptc.burstmode.write(|w| unsafe { w.bits(0xA4) });
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("starting conversion");
    ptc.convctrl.modify(|_, w| w.convert().set_bit());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("started");
    while ptc.convctrl.read().convert().bit() {}
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("done");

    let conversion = ptc.result.read().result().bits();

    conversion / 4
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("initializing clocks");

    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    rprintln!("configuring gclk3");
    let gclk3 = clocks
        .configure_gclk_divider_and_source(ClockGenId::GCLK3, 1, ClockSource::OSC8M, false)
        .unwrap();
    rprintln!("configuring ptc clock");
    clocks.ptc(&gclk3).unwrap();

    rprintln!("enabling ptc power");
    peripherals.PM.apbcmask.modify(|_, w| w.ptc_().set_bit());

    let pins = bsp::Pins::new(peripherals.PORT);

    let mut yellow_led = Pin::from(pins.led5).into_push_pull_output();

    let _cap1 = Pin::from(pins.cap1).into_alternate::<B>();

    let ptc = peripherals.PTC;

    rprintln!("disabling ptc");
    while ptc.ctrlb.read().syncflag().bit() {}
    ptc.ctrla.modify(|_, w| w.enable().clear_bit());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("initializing ptc");
    ptc.unk4c04
        .modify(|r, w| unsafe { w.bits(r.bits() & 0xF7) });
    ptc.unk4c04
        .modify(|r, w| unsafe { w.bits(r.bits() & 0xFB) });
    ptc.unk4c04
        .modify(|r, w| unsafe { w.bits(r.bits() & 0xFC) });
    while ptc.ctrlb.read().syncflag().bit() {}
    ptc.freqctrl
        .write(|w| w.freqspreaden().clear_bit().sampledelay().freqhop1());
    ptc.ctrlc.modify(|_, w| w.init().set_bit());
    ptc.ctrla.modify(|_, w| w.runstdby().set_bit());
    while ptc.ctrlb.read().syncflag().bit() {}
    ptc.intenclr.modify(|_, w| w.wco().set_bit());
    while ptc.ctrlb.read().syncflag().bit() {}
    ptc.intenclr.modify(|_, w| w.eoc().set_bit());
    while ptc.ctrlb.read().syncflag().bit() {}

    ptc.yselecten.write(|w| unsafe { w.bits(0xFFFF) });
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("enabling ptc");
    ptc.ctrla.modify(|_, w| w.enable().set_bit());
    while ptc.ctrlb.read().syncflag().bit() {}

    rprintln!("measuring");
    let threshold = measure(&ptc) + 200;

    loop {
        if measure(&ptc) > threshold {
            yellow_led.set_high().unwrap();
        } else {
            yellow_led.set_low().unwrap();
        }
    }
}
