#![no_main]
#![no_std]

extern crate bitflags;
extern crate bme280;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate embedded_graphics;
extern crate embedded_hal;
extern crate heapless;
extern crate il3820;
extern crate panic_semihosting;
extern crate pwm_speaker;
extern crate stm32f103xx_hal as hal;
extern crate stm32f103xx_rtc as rtc;

use hal::prelude::*;
use heapless::consts::*;
use heapless::Vec;
use pwm_speaker::songs::SO_WHAT;
use rt::{exception, ExceptionFrame};
use rtc::datetime::DateTime;
use rtfm::{app, Resource, Threshold};

mod alarm;
mod button;
mod msg_queue;
mod sound;
mod ui;

type I2C = hal::i2c::BlockingI2c<
    hal::stm32f103xx::I2C1,
    (
        hal::gpio::gpiob::PB6<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
        hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
    ),
>;
type Button0Pin = hal::gpio::gpioa::PA7<hal::gpio::Input<hal::gpio::PullUp>>;
type Button1Pin = hal::gpio::gpiob::PB0<hal::gpio::Input<hal::gpio::PullUp>>;
type Button2Pin = hal::gpio::gpiob::PB1<hal::gpio::Input<hal::gpio::PullUp>>;
type Spi = hal::spi::Spi<
    hal::stm32f103xx::SPI2,
    (
        hal::gpio::gpiob::PB13<hal::gpio::Alternate<hal::gpio::PushPull>>,
        hal::gpio::gpiob::PB14<hal::gpio::Input<hal::gpio::Floating>>,
        hal::gpio::gpiob::PB15<hal::gpio::Alternate<hal::gpio::PushPull>>,
    ),
>;
type EPaperDisplay = il3820::Il3820<
    Spi,
    hal::gpio::gpiob::PB12<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpioa::PA8<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpioa::PA9<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpioa::PA10<hal::gpio::Input<hal::gpio::Floating>>,
>;

app! {
    device: hal::stm32f103xx,

    resources: {
        static RTC_DEV: rtc::Rtc;
        static BME280: bme280::BME280<I2C, hal::delay::Delay>;
        static ALARM_MANAGER: alarm::AlarmManager;
        static SOUND: sound::Sound;
        static BUTTON0: button::Button<Button0Pin>;
        static BUTTON1: button::Button<Button1Pin>;
        static BUTTON2: button::Button<Button2Pin>;
        static DISPLAY: EPaperDisplay;
        static SPI: Spi;
        static UI: ui::Model;
        static FULL_UPDATE: bool;
        static MSG_QUEUE: msg_queue::MsgQueue;
    },

    tasks: {
        EXTI1: {
            path: render,
            resources: [UI, DISPLAY, SPI, FULL_UPDATE],
            priority: 1,
        },
        EXTI2: {
            path: msgs,
            resources: [UI, MSG_QUEUE, RTC_DEV, FULL_UPDATE],
            priority: 2,
        },
        RTC: {
            path: handle_rtc,
            resources: [RTC_DEV, BME280, ALARM_MANAGER, SOUND, MSG_QUEUE],
            priority: 3,
        },
        TIM3: {
            path: one_khz,
            resources: [BUTTON0, BUTTON1, BUTTON2, SOUND, MSG_QUEUE],
            priority: 4,
        },
    },
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let mut pwm = p
        .device
        .TIM2
        .pwm(c1, &mut afio.mapr, 440.hz(), clocks, &mut rcc.apb1);
    pwm.enable();
    let speaker = pwm_speaker::Speaker::new(pwm, clocks);

    let button0_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
    let button1_pin = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
    let button2_pin = gpiob.pb1.into_pull_up_input(&mut gpiob.crl);

    let mut timer = hal::timer::Timer::tim3(p.device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
    timer.listen(hal::timer::Event::Update);
    p.core.NVIC.enable(hal::stm32f103xx::Interrupt::TIM3);

    let mut rtc = rtc::Rtc::new(p.device.RTC, &mut rcc.apb1, &mut p.device.PWR);
    if rtc.get_cnt() < 100 {
        let today = DateTime {
            year: 2018,
            month: 9,
            day: 1,
            hour: 23,
            min: 15,
            sec: 40,
            day_of_week: rtc::datetime::DayOfWeek::Wednesday,
        };
        if let Some(epoch) = today.to_epoch() {
            rtc.set_cnt(epoch);
        }
    }
    rtc.enable_second_interrupt(&mut p.core.NVIC);

    use alarm::Mode;
    let mut alarm_manager = alarm::AlarmManager::default();
    alarm_manager.alarms[0].is_enable = true;
    alarm_manager.alarms[0].set_hour(7);
    alarm_manager.alarms[0].set_min(25);
    alarm_manager.alarms[0].mode = Mode::MONDAY | Mode::TUESDAY | Mode::THURSDAY | Mode::FRIDAY;
    alarm_manager.alarms[1].is_enable = true;
    alarm_manager.alarms[1].set_hour(8);
    alarm_manager.alarms[1].set_min(15);
    alarm_manager.alarms[1].mode = Mode::WEDNESDAY;

    let mut delay = hal::delay::Delay::new(p.core.SYST, clocks);

    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let mut spi = hal::spi::Spi::spi2(
        p.device.SPI2,
        (sck, miso, mosi),
        il3820::MODE,
        4.mhz(),
        clocks,
        &mut rcc.apb1,
    );
    let mut il3820 = il3820::Il3820::new(
        &mut spi,
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        gpioa.pa8.into_push_pull_output(&mut gpioa.crh),
        gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
        gpioa.pa10.into_floating_input(&mut gpioa.crh),
        &mut delay,
    );
    il3820.clear(&mut spi).unwrap();

    let pb6 = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let pb7 = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
    let i2c = hal::i2c::I2c::i2c1(
        p.device.I2C1,
        (pb6, pb7),
        &mut afio.mapr,
        hal::i2c::Mode::Fast {
            frequency: 400_000,
            duty_cycle: hal::i2c::DutyCycle::Ratio16to9,
        },
        clocks,
        &mut rcc.apb1,
    );
    let i2c = hal::i2c::blocking_i2c(i2c, clocks, 100, 100, 100, 100);
    let mut bme280 = bme280::BME280::new_primary(i2c, delay);
    bme280.init().unwrap();

    let mut msg_queue = msg_queue::MsgQueue::new();
    msg_queue.push(ui::Msg::AlarmManager(alarm_manager.clone()));

    init::LateResources {
        RTC_DEV: rtc,
        BME280: bme280,
        SOUND: sound::Sound::new(speaker),
        BUTTON0: button::Button::new(button0_pin),
        BUTTON1: button::Button::new(button1_pin),
        BUTTON2: button::Button::new(button2_pin),
        DISPLAY: il3820,
        SPI: spi,
        UI: ui::Model::init(),
        FULL_UPDATE: false,
        MSG_QUEUE: msg_queue,
        ALARM_MANAGER: alarm_manager,
    }
}

pub fn msgs(t: &mut rtfm::Threshold, mut r: EXTI2::Resources) {
    loop {
        let msgs = r.MSG_QUEUE.claim_mut(t, |q, _| q.get());
        if msgs.is_empty() {
            break;
        }
        let cmds: Vec<_, U16> = msgs.into_iter().flat_map(|msg| r.UI.update(msg)).collect();
        for cmd in cmds {
            use ui::Cmd::*;
            match cmd {
                UpdateRtc(dt) => if let Some(epoch) = dt.to_epoch() {
                    r.RTC_DEV.claim_mut(t, |rtc, _| {
                        let _ = rtc.set_cnt(epoch);
                    });
                    r.MSG_QUEUE
                        .claim_mut(t, |q, _| q.push(ui::Msg::DateTime(dt)));
                },
                FullUpdate => *r.FULL_UPDATE = true,
            }
        }
    }
    rtfm::set_pending(hal::stm32f103xx::Interrupt::EXTI1);
}

fn render(t: &mut rtfm::Threshold, mut r: EXTI1::Resources) {
    while r.DISPLAY.is_busy() {}
    let model = r.UI.claim(t, |model, _| model.clone());
    let display = model.view();
    let full_update = r.FULL_UPDATE.claim_mut(t, |fu, _| {
        let full_update = *fu;
        *fu = false;
        full_update
    });
    if full_update {
        r.DISPLAY.set_full();
    }
    r.DISPLAY.set_display(&mut *r.SPI, &display).unwrap();
    r.DISPLAY.update(&mut *r.SPI).unwrap();
    r.DISPLAY.set_partial();
}

fn handle_rtc(t: &mut rtfm::Threshold, mut r: RTC::Resources) {
    r.RTC_DEV.clear_second_interrupt();
    r.RTC_DEV.sync();

    let datetime = DateTime::new(r.RTC_DEV.get_cnt());
    if datetime.sec == 0 && r.ALARM_MANAGER.must_ring(&datetime) {
        r.SOUND
            .claim_mut(t, |alarm, _t| alarm.play(&SO_WHAT, 10 * 60));
        let manager = r.ALARM_MANAGER.clone();
        r.MSG_QUEUE
            .claim_mut(t, |q, _| q.push(ui::Msg::AlarmManager(manager)));
    }
    r.MSG_QUEUE
        .claim_mut(t, |q, _| q.push(ui::Msg::DateTime(datetime)));

    let measurements = r.BME280.measure().unwrap();
    r.MSG_QUEUE
        .claim_mut(t, |q, _| q.push(ui::Msg::Environment(measurements)));
}

fn one_khz(_t: &mut rtfm::Threshold, mut r: TIM3::Resources) {
    unsafe {
        (*hal::stm32f103xx::TIM3::ptr())
            .sr
            .modify(|_, w| w.uif().clear_bit());
    };

    if let button::Event::Pressed = r.BUTTON0.poll() {
        r.MSG_QUEUE.push(ui::Msg::ButtonMinus);
    }
    if let button::Event::Pressed = r.BUTTON1.poll() {
        r.SOUND.stop();
        r.MSG_QUEUE.push(ui::Msg::ButtonOk);
    }
    if let button::Event::Pressed = r.BUTTON2.poll() {
        r.MSG_QUEUE.push(ui::Msg::ButtonPlus);
    }
    r.SOUND.poll();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
