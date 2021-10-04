#![no_std]
#![no_main]

use bsp::hal;
use panic_halt as _;
use pyruler as bsp;

use bsp::entry;
use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU16, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::gpio::v2::{Pin, B};
use hal::pac::{
    interrupt,
    ptc::{convctrl::*, freqctrl::*, serres::*, yselect::*},
    CorePeripherals, Peripherals,
};
use hal::prelude::*;
use hal::thumbv6m::ptc::Ptc;
use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

struct NumberBuf {
    data: [u8; 16],
    len: usize,
}

impl NumberBuf {
    pub fn new() -> Self {
        Self {
            data: [0; 16],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl Write for NumberBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        let new_len = self.len + b.len();

        if new_len > 16 {
            return Err(core::fmt::Error);
        }

        let target = &mut self.data[self.len..][..b.len()];
        target.copy_from_slice(b);

        self.len = new_len;

        Ok(())
    }
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static USB_BUS: Mutex<RefCell<Option<UsbDevice<UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<SerialPort<UsbBus>>>> = Mutex::new(RefCell::new(None));
static READING: AtomicU16 = AtomicU16::new(0xbbbb);

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
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

    READING.store(0xdddd, Ordering::SeqCst);

    let mut ptc = Ptc::ptc(peripherals.PTC, &mut peripherals.PM, &mut clocks);

    ptc.oversample(ADCACCUM_A::OVERSAMPLE4);
    ptc.series_resistance(RESISTOR_A::RES0);
    ptc.compcap(0x2000);
    ptc.intcap(0x3F);
    ptc.sample_delay(SAMPLEDELAY_A::FREQHOP1);

    let bus_allocator = bsp::usb_allocator(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm,
        pins.usb_dp,
    );
    let bus_allocator: &'static UsbBusAllocator<UsbBus> =
        unsafe { USB_ALLOCATOR.insert(bus_allocator) };
    let usb_serial = SerialPort::new(bus_allocator);
    let usb_bus = UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    cortex_m::interrupt::free(move |cs| {
        *USB_BUS.borrow(cs).borrow_mut() = Some(usb_bus);
        *USB_SERIAL.borrow(cs).borrow_mut() = Some(usb_serial);
    });

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    let initial_value: u16 = ptc.read(&mut micro_button).unwrap_or(0xaaaa);
    let threshold = initial_value + 100;

    loop {
        let value: u16 = ptc.read(&mut micro_button).unwrap_or(0xaaaa);
        if value > threshold {
            red_led.set_high().unwrap();
        } else {
            red_led.set_low().unwrap();
        }

        READING.store(if value == 0 { 0xcccc } else { value }, Ordering::SeqCst);

        delay.delay_ms(60u8);
    }
}

#[interrupt]
fn USB() {
    let cs = unsafe { cortex_m::interrupt::CriticalSection::new() };
    let mut serial_ref = USB_SERIAL.borrow(&cs).borrow_mut();
    let mut bus_ref = USB_BUS.borrow(&cs).borrow_mut();
    if let (Some(usb_serial), Some(usb_bus)) = (serial_ref.as_mut(), bus_ref.as_mut()) {
        usb_bus.poll(&mut [usb_serial]);
        let mut buf = [0u8; 64];

        if let Ok(_count) = usb_serial.read(&mut buf) {
            let mut num = NumberBuf::new();
            let _ = num.write_fmt(format_args!("{}\n", READING.load(Ordering::SeqCst)));
            usb_serial.write(num.as_bytes()).unwrap();
        }
    }
}
