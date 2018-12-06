#![no_main]
#![no_std]

extern crate bme280;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate embedded_hal;
extern crate heapless;
extern crate il3820;
#[cfg(not(test))]
extern crate panic_semihosting;
extern crate portable;
extern crate pwm_speaker;
extern crate stm32f103xx_hal as hal;
extern crate stm32f103xx_rtc as rtc;

use crate::hal::prelude::*;
use crate::rt::{exception, ExceptionFrame};
use heapless::consts::*;
use heapless::Vec;
use portable::datetime::DateTime;
use portable::{alarm, button, datetime, ui};
use pwm_speaker::songs::SO_WHAT;
use rtfm::app;

mod msg_queue;
mod sound;

type I2C = hal::i2c::BlockingI2c<
    hal::device::I2C1,
    (
        hal::gpio::gpiob::PB6<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
        hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
    ),
>;
type Button0Pin = hal::gpio::gpioa::PA6<hal::gpio::Input<hal::gpio::PullUp>>;
type Button1Pin = hal::gpio::gpioa::PA7<hal::gpio::Input<hal::gpio::PullUp>>;
type Button2Pin = hal::gpio::gpiob::PB0<hal::gpio::Input<hal::gpio::PullUp>>;
type Button3Pin = hal::gpio::gpiob::PB1<hal::gpio::Input<hal::gpio::PullUp>>;
type Spi = hal::spi::Spi<
    hal::device::SPI2,
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

#[app(device = crate::hal::device)]
const APP: () = {
    static mut RTC_DEV: rtc::Rtc = ();
    static mut BME280: bme280::BME280<I2C, hal::delay::Delay> = ();
    static mut ALARM_MANAGER: alarm::AlarmManager = ();
    static mut SOUND: sound::Sound = ();
    static mut BUTTON0: button::Button<Button0Pin> = ();
    static mut BUTTON1: button::Button<Button1Pin> = ();
    static mut BUTTON2: button::Button<Button2Pin> = ();
    static mut BUTTON3: button::Button<Button3Pin> = ();
    static mut DISPLAY: EPaperDisplay = ();
    static mut SPI: Spi = ();
    static mut UI: ui::Model = ();
    static mut FULL_UPDATE: bool = ();
    static mut MSG_QUEUE: msg_queue::MsgQueue = ();

    #[init]
    fn init() {
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let mut afio = device.AFIO.constrain(&mut rcc.apb2);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

        let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
        let mut pwm = device
            .TIM2
            .pwm(c1, &mut afio.mapr, 440.hz(), clocks, &mut rcc.apb1);
        pwm.enable();
        let speaker = pwm_speaker::Speaker::new(pwm, clocks);

        let button0_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button1_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        let button2_pin = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
        let button3_pin = gpiob.pb1.into_pull_up_input(&mut gpiob.crl);

        let mut timer = hal::timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(hal::timer::Event::Update);

        let mut rtc = rtc::Rtc::new(device.RTC, &mut rcc.apb1, &mut device.PWR);
        if rtc.get_cnt() < 100 {
            let today = DateTime {
                year: 2018,
                month: 9,
                day: 1,
                hour: 23,
                min: 15,
                sec: 40,
                day_of_week: datetime::DayOfWeek::Wednesday,
            };
            if let Some(epoch) = today.to_epoch() {
                rtc.set_cnt(epoch);
            }
        }
        rtc.listen_second_interrupt();

        use crate::alarm::Mode;
        let mut alarm_manager = alarm::AlarmManager::default();
        alarm_manager.alarms[0].is_enable = true;
        alarm_manager.alarms[0].set_hour(7);
        alarm_manager.alarms[0].set_min(25);
        alarm_manager.alarms[0].mode = Mode::MONDAY | Mode::TUESDAY | Mode::THURSDAY | Mode::FRIDAY;
        alarm_manager.alarms[1].is_enable = true;
        alarm_manager.alarms[1].set_hour(8);
        alarm_manager.alarms[1].set_min(15);
        alarm_manager.alarms[1].mode = Mode::WEDNESDAY;
        alarm_manager.alarms[4].set_hour(13);
        alarm_manager.alarms[4].set_min(37);
        alarm_manager.alarms[4].mode = Mode::all() - Mode::ONE_TIME;

        let mut delay = hal::delay::Delay::new(core.SYST, clocks);

        let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
        let miso = gpiob.pb14;
        let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
        let mut spi = hal::spi::Spi::spi2(
            device.SPI2,
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

        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
        let pb6 = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let pb7 = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
        let i2c = hal::i2c::I2c::i2c1(
            device.I2C1,
            (pb6, pb7),
            &mut afio.mapr,
            hal::i2c::Mode::Fast {
                frequency: 400_000,
                duty_cycle: hal::i2c::DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
        );
        let i2c = hal::i2c::blocking_i2c(i2c, clocks, 100, 100, 100, 100);
        let mut bme280 = bme280::BME280::new_primary(i2c, delay);
        bme280.init().unwrap();

        let mut msg_queue = msg_queue::MsgQueue::new();
        msg_queue.push(ui::Msg::AlarmManager(alarm_manager.clone()));

        RTC_DEV = rtc;
        BME280 = bme280;
        SOUND = sound::Sound::new(speaker);
        BUTTON0 = button::Button::new(button0_pin);
        BUTTON1 = button::Button::new(button1_pin);
        BUTTON2 = button::Button::new(button2_pin);
        BUTTON3 = button::Button::new(button3_pin);
        DISPLAY = il3820;
        SPI = spi;
        UI = ui::Model::init();
        FULL_UPDATE = false;
        MSG_QUEUE = msg_queue;
        ALARM_MANAGER = alarm_manager;
    }

    #[interrupt(priority = 4, resources = [BUTTON0, BUTTON1, BUTTON2, BUTTON3, SOUND, MSG_QUEUE])]
    fn TIM3() {
        unsafe {
            (*hal::device::TIM3::ptr())
                .sr
                .modify(|_, w| w.uif().clear_bit());
        };

        if let button::Event::Pressed = resources.BUTTON0.poll() {
            resources.SOUND.stop();
            resources.MSG_QUEUE.push(ui::Msg::ButtonCancel);
        }
        if let button::Event::Pressed = resources.BUTTON1.poll() {
            resources.MSG_QUEUE.push(ui::Msg::ButtonMinus);
        }
        if let button::Event::Pressed = resources.BUTTON2.poll() {
            resources.MSG_QUEUE.push(ui::Msg::ButtonPlus);
        }
        if let button::Event::Pressed = resources.BUTTON3.poll() {
            resources.MSG_QUEUE.push(ui::Msg::ButtonOk);
        }
        resources.SOUND.poll();
    }

    #[interrupt(priority = 3, resources = [RTC_DEV, BME280, ALARM_MANAGER, SOUND, MSG_QUEUE])]
    fn RTC() {
        resources.RTC_DEV.clear_second_interrupt();
        resources.RTC_DEV.sync();

        let datetime = DateTime::new(resources.RTC_DEV.get_cnt());
        if datetime.sec == 0 && resources.ALARM_MANAGER.must_ring(&datetime) {
            resources.SOUND.lock(|alarm| alarm.play(&SO_WHAT, 10 * 60));
            let manager = resources.ALARM_MANAGER.clone();
            resources
                .MSG_QUEUE
                .lock(|q| q.push(ui::Msg::AlarmManager(manager)));
        }
        resources
            .MSG_QUEUE
            .lock(|q| q.push(ui::Msg::DateTime(datetime)));

        let measurements = if let Ok(measurements) = resources.BME280.measure() {
            crate::ui::Environment {
                pressure: measurements.pressure as u32,
                temperature: (measurements.temperature * 100.) as i16,
                humidity: measurements.humidity as u8,
            }
        } else {
            crate::ui::Environment {
                pressure: 0,
                temperature: 0,
                humidity: 0,
            }
        };
        resources
            .MSG_QUEUE
            .lock(|q| q.push(ui::Msg::Environment(measurements)));
    }

    #[interrupt(priority = 2, resources = [UI, MSG_QUEUE, RTC_DEV, FULL_UPDATE, ALARM_MANAGER])]
    fn EXTI2() {
        loop {
            let msgs = resources.MSG_QUEUE.lock(|q| q.get());
            if msgs.is_empty() {
                break;
            }
            let cmds: Vec<_, U16> = msgs
                .into_iter()
                .flat_map(|msg| resources.UI.update(msg))
                .collect();
            for cmd in cmds {
                use crate::ui::Cmd::*;
                match cmd {
                    UpdateRtc(dt) => {
                        if let Some(epoch) = dt.to_epoch() {
                            resources.RTC_DEV.lock(|rtc| {
                                let _ = rtc.set_cnt(epoch);
                            });
                            resources.MSG_QUEUE.lock(|q| q.push(ui::Msg::DateTime(dt)));
                        }
                    }
                    UpdateAlarm(alarm, i) => {
                        let manager = resources.ALARM_MANAGER.lock(|m| {
                            m.alarms[i] = alarm;
                            m.clone()
                        });
                        resources
                            .MSG_QUEUE
                            .lock(|q| q.push(ui::Msg::AlarmManager(manager)));
                    }
                    FullUpdate => *resources.FULL_UPDATE = true,
                }
            }
        }
        rtfm::pend(hal::device::Interrupt::EXTI1);
    }

    #[interrupt(priority = 1, resources = [UI, DISPLAY, SPI, FULL_UPDATE])]
    fn EXTI1() {
        while resources.DISPLAY.is_busy() {}
        let model = resources.UI.lock(|model| model.clone());
        let display = model.view();
        let full_update = resources.FULL_UPDATE.lock(|fu| {
            let full_update = *fu;
            *fu = false;
            full_update
        });
        if full_update {
            resources.DISPLAY.set_full();
        }
        resources
            .DISPLAY
            .set_display(&mut *resources.SPI, &display)
            .unwrap();
        resources.DISPLAY.update(&mut *resources.SPI).unwrap();
        resources.DISPLAY.set_partial();
    }
};

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
