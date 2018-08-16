#![no_main]
#![no_std]
#![feature(proc_macro_gen)]
#![feature(use_extern_macros)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate stm32f103xx_rtc as rtc;
extern crate cortex_m_semihosting as sh;
extern crate heapless;

use rt::ExceptionFrame;
use hal::prelude::*;
use rtfm::{app, Threshold};
use core::fmt::Write;

entry!(main);

app! {
    device: hal::stm32f103xx,

    resources: {
        static RTC_DEV: rtc::Rtc;
    },

    tasks: {
        RTC: {
            path: handle_rtc,
            resources: [RTC_DEV],
        },
    },
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    let mut rcc = p.device.RCC.constrain();

    let mut rtc = rtc::Rtc::new(p.device.RTC, &mut rcc.apb1, &mut p.device.PWR);
    if rtc.get_cnt() < 100 {
        let today = rtc::datetime::DateTime {
            year: 2018,
            month: 8,
            day: 15,
            hour: 0,
            min: 45,
            sec: 0,
            day_of_week: rtc::datetime::DayOfWeek::Wednesday,
        };
        if let Some(epoch) = today.to_epoch() {
            rtc.set_cnt(epoch);
        }
    }
    rtc.enable_second_interrupt(&mut p.core.NVIC);
    init::LateResources {
        RTC_DEV: rtc,
    }
}

fn handle_rtc(_t: &mut rtfm::Threshold, mut r: RTC::Resources) {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let mut s = heapless::String::<heapless::consts::U32>::new();
    writeln!(s, "{}", rtc::datetime::DateTime::new(r.RTC_DEV.get_cnt())).unwrap();
    hstdout.write_str(&s).unwrap();
    r.RTC_DEV.clear_second_interrupt();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
