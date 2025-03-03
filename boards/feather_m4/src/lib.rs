#![no_std]
#![recursion_limit = "1024"]

pub use atsamd_hal as hal;

#[cfg(feature = "rt")]
use cortex_m_rt;
#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

use hal::prelude::*;
use hal::*;

pub use hal::common::*;
pub use hal::pac;
pub use hal::samd51::*;

use gpio::{Floating, Input, Port};
use hal::clock::GenericClockController;
use hal::sercom::{
    v2::{
        uart::{self, BaudMode, Oversampling},
        IoSet1, Sercom5,
    },
    I2CMaster2, PadPin, SPIMaster1,
};
use hal::time::Hertz;

use gpio::v2::{AlternateC, AnyPin, Pin, C, PB16, PB17};

#[cfg(feature = "usb")]
use gpio::v2::{PA24, PA25};
#[cfg(feature = "usb")]
use hal::usb::usb_device::bus::UsbBusAllocator;
#[cfg(feature = "usb")]
pub use hal::usb::UsbBus;

define_pins!(
    /// Maps the pins to their arduino names and
    /// the numbers printed on the board.
    struct Pins,
    pac: pac,

    /// Analog pin 0.  Can act as a true analog output
    /// as it has a DAC (which is not currently supported
    /// by this hal) as well as input.
    pin a0 = a2,

    /// Analog Pin 1
    pin a1 = a5,
    /// Analog Pin 2
    pin a2 = b8,
    /// Analog Pin 3
    pin a3 = b9,
    /// Analog Pin 4
    pin a4 = a4,
    /// Analog Pin 5
    pin a5 = a6,

    /// Pin 0, rx
    pin d0 = b17,
    /// Pin 1, tx
    pin d1 = b16,
    /// Pin 4, PWM capable
    pin d4 = a14,

    /// Pin 5, PWM capable
    pin d5 = a16,
    /// Pin 6, PWM capable
    pin d6 = a18,
    /// Neopixel Pin
    pin neopixel = b3,
    /// Pin 9, PWM capable.  Also analog input (A7)
    pin d9 = a19,
    /// Pin 10, PWM capable
    pin d10 = a20,
    /// Pin 11, PWM capable
    pin d11 = a21,
    /// Pin 12, PWM capable
    pin d12 = a22,
    /// Pin 13, which is also attached to
    /// the red LED.  PWM capable.
    pin d13 = a23,

    /// The I2C data line
    pin sda = a12,
    /// The I2C clock line
    pin scl = a13,

    /// The SPI SCK
    pin sck = a17,
    /// The SPI MOSI
    pin mosi = b23,
    /// The SPI MISO
    pin miso = b22,

    /// The USB D- pad
    pin usb_dm = a24,
    /// The USB D+ pad
    pin usb_dp = a25,
);

/// Convenience for setting up the labelled SPI peripheral.
/// This powers up SERCOM1 and configures it for use as an
/// SPI Master in SPI Mode 0.
pub fn spi_master<F: Into<Hertz>>(
    clocks: &mut GenericClockController,
    bus_speed: F,
    sercom1: pac::SERCOM1,
    mclk: &mut pac::MCLK,
    sck: gpio::Pa17<Input<Floating>>,
    mosi: gpio::Pb23<Input<Floating>>,
    miso: gpio::Pb22<Input<Floating>>,
    port: &mut Port,
) -> SPIMaster1<
    hal::sercom::Sercom1Pad2<gpio::Pb22<gpio::PfC>>,
    hal::sercom::Sercom1Pad3<gpio::Pb23<gpio::PfC>>,
    hal::sercom::Sercom1Pad1<gpio::Pa17<gpio::PfC>>,
> {
    let gclk0 = clocks.gclk0();
    SPIMaster1::new(
        &clocks.sercom1_core(&gclk0).unwrap(),
        bus_speed.into(),
        hal::hal::spi::Mode {
            phase: hal::hal::spi::Phase::CaptureOnFirstTransition,
            polarity: hal::hal::spi::Polarity::IdleLow,
        },
        sercom1,
        mclk,
        (miso.into_pad(port), mosi.into_pad(port), sck.into_pad(port)),
    )
}

/// Convenience for setting up the labelled SDA, SCL pins to
/// operate as an I2C master running at the specified frequency.
pub fn i2c_master<F: Into<Hertz>>(
    clocks: &mut GenericClockController,
    bus_speed: F,
    sercom2: pac::SERCOM2,
    mclk: &mut pac::MCLK,
    sda: gpio::Pa12<Input<Floating>>,
    scl: gpio::Pa13<Input<Floating>>,
    port: &mut Port,
) -> I2CMaster2<
    hal::sercom::Sercom2Pad0<gpio::Pa12<gpio::PfC>>,
    hal::sercom::Sercom2Pad1<gpio::Pa13<gpio::PfC>>,
> {
    let gclk0 = clocks.gclk0();
    I2CMaster2::new(
        &clocks.sercom2_core(&gclk0).unwrap(),
        bus_speed.into(),
        sercom2,
        mclk,
        sda.into_pad(port),
        scl.into_pad(port),
    )
}

pub type UartRx = Pin<PB17, AlternateC>;
pub type UartTx = Pin<PB16, AlternateC>;
pub type UartPads = uart::Pads<Sercom5, IoSet1, UartRx, UartTx>;

/// UART device for the labelled RX & TX pins
pub type Uart = uart::Uart<uart::Config<UartPads>, uart::Duplex>;

/// Convenience for setting up the labelled RX, TX pins to
/// operate as a UART device running at the specified baud.
pub fn uart(
    clocks: &mut GenericClockController,
    baud: impl Into<Hertz>,
    sercom5: Sercom5,
    mclk: &mut pac::MCLK,
    rx: impl AnyPin<Id = PB17>,
    tx: impl AnyPin<Id = PB16>,
) -> Uart {
    let gclk0 = clocks.gclk0();

    let clock = &clocks.sercom5_core(&gclk0).unwrap();
    let baud = baud.into();
    let pads = uart::Pads::default()
        .rx(rx.into().into_alternate::<C>())
        .tx(tx.into().into_alternate::<C>());
    uart::Config::new(mclk, sercom5, pads, clock.freq())
        .baud(baud, BaudMode::Fractional(Oversampling::Bits16))
        .enable()
}

#[cfg(feature = "usb")]
pub fn usb_allocator(
    dm: impl AnyPin<Id = PA24>,
    dp: impl AnyPin<Id = PA25>,
    usb: pac::USB,
    clocks: &mut GenericClockController,
    mclk: &mut pac::MCLK,
) -> UsbBusAllocator<UsbBus> {
    use pac::gclk::{genctrl::SRC_A, pchctrl::GEN_A};

    clocks.configure_gclk_divider_and_source(GEN_A::GCLK2, 1, SRC_A::DFLL, false);
    let usb_gclk = clocks.get_gclk(GEN_A::GCLK2).unwrap();
    let usb_clock = &clocks.usb(&usb_gclk).unwrap();

    UsbBusAllocator::new(UsbBus::new(usb_clock, mclk, dm, dp, usb))
}
