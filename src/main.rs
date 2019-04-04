#![no_main]
#![no_std]

#[cfg(not(test))]
extern crate panic_semihosting;

use epd_waveshare::prelude::*;
use portable::datetime::DateTime;
use portable::{alarm, button, datetime, ui};
use pwm_speaker::songs::SO_WHAT;
use rtfm::app;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{delay, gpio, i2c, spi, stm32, timer};

mod sound;

type I2C = i2c::BlockingI2c<
    stm32::I2C1,
    (
        gpio::gpiob::PB6<gpio::Alternate<gpio::OpenDrain>>,
        gpio::gpiob::PB7<gpio::Alternate<gpio::OpenDrain>>,
    ),
>;
type Button0Pin = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
type Button1Pin = gpio::gpioa::PA7<gpio::Input<gpio::PullUp>>;
type Button2Pin = gpio::gpiob::PB0<gpio::Input<gpio::PullUp>>;
type Button3Pin = gpio::gpiob::PB1<gpio::Input<gpio::PullUp>>;
type Spi = spi::Spi<
    stm32::SPI2,
    (
        gpio::gpiob::PB13<gpio::Alternate<gpio::PushPull>>,
        gpio::gpiob::PB14<gpio::Input<gpio::Floating>>,
        gpio::gpiob::PB15<gpio::Alternate<gpio::PushPull>>,
    ),
>;
// type EPaperDisplay = il3820::Il3820<
//     Spi, //spi
//     gpio::gpiob::PB12<gpio::Output<gpio::PushPull>>, // cs/nss
//     gpio::gpioa::PA8<gpio::Output<gpio::PushPull>>, // dc
//     gpio::gpioa::PA9<gpio::Output<gpio::PushPull>>, // rst
//     gpio::gpioa::PA10<gpio::Input<gpio::Floating>>, // busy
// >;
type EPaperDisplay = epd_waveshare::epd2in9::EPD2in9<
    Spi,
    gpio::gpiob::PB12<gpio::Output<gpio::PushPull>>, // cs/nss
    gpio::gpioa::PA10<gpio::Input<gpio::Floating>>,  // busy
    gpio::gpioa::PA8<gpio::Output<gpio::PushPull>>,  // dc
    gpio::gpioa::PA9<gpio::Output<gpio::PushPull>>,  // rst
>;

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    static mut RTC_DEV: stm32f103xx_rtc::Rtc = ();
    static mut BME280: bme280::BME280<I2C, delay::Delay> = ();
    static mut ALARM_MANAGER: alarm::AlarmManager = ();
    static mut SOUND: sound::Sound = ();
    static mut BUTTON0: button::Button<Button0Pin> = ();
    static mut BUTTON1: button::Button<Button1Pin> = ();
    static mut BUTTON2: button::Button<Button2Pin> = ();
    static mut BUTTON3: button::Button<Button3Pin> = ();
    static mut DISPLAY: EPaperDisplay = ();
    static mut SPI: Spi = ();
    static mut UI: ui::Model = ();
    static mut FULL_UPDATE: bool = false;

    #[init(spawn = [msg])]
    fn init() -> init::LateResources {
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
        let c2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
        let c3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let c4 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);
        let mut pwm = device.TIM2.pwm(
            (c1, c2, c3, c4),
            &mut afio.mapr,
            440.hz(),
            clocks,
            &mut rcc.apb1,
        );
        pwm.0.enable();
        pwm.1.enable();
        let speaker = pwm_speaker::Speaker::new(pwm.0, clocks);

        let button0_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button1_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        let button2_pin = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
        let button3_pin = gpiob.pb1.into_pull_up_input(&mut gpiob.crl);

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        let mut rtc = stm32f103xx_rtc::Rtc::new(device.RTC, &mut rcc.apb1, &mut device.PWR);
        if rtc.get_cnt() < 100 {
            let today = DateTime {
                year: 2018,
                month: 9,
                day: 1,
                hour: 23,
                min: 59,
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

        let mut delay = delay::Delay::new(core.SYST, clocks);

        let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
        let miso = gpiob.pb14;
        let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
        let mut spi = spi::Spi::spi2(
            device.SPI2,
            (sck, miso, mosi),
            epd_waveshare::SPI_MODE,
            4.mhz(),
            clocks,
            &mut rcc.apb1,
        );
        let mut il3820 = epd_waveshare::epd2in9::EPD2in9::new(
            &mut spi,
            gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
            gpioa.pa10.into_floating_input(&mut gpioa.crh),
            gpioa.pa8.into_push_pull_output(&mut gpioa.crh),
            gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
            &mut delay,
        )
        .unwrap();
        il3820
            .set_lut(&mut spi, Some(RefreshLUT::QUICK))
            .unwrap();
        il3820.clear_frame(&mut spi).unwrap();

        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
        let pb6 = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let pb7 = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
        let i2c = i2c::I2c::i2c1(
            device.I2C1,
            (pb6, pb7),
            &mut afio.mapr,
            i2c::Mode::Fast {
                frequency: 400_000,
                duty_cycle: i2c::DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
        );
        let i2c = i2c::blocking_i2c(i2c, clocks, 200, 10, 200, 200);
        let mut bme280 = bme280::BME280::new_primary(i2c, delay);
        bme280.init().expect("i2c init error");

        spawn
            .msg(ui::Msg::AlarmManager(alarm_manager.clone()))
            .unwrap();

        init::LateResources {
            RTC_DEV: rtc,
            BME280: bme280,
            SOUND: sound::Sound::new(speaker),
            BUTTON0: button::Button::new(button0_pin),
            BUTTON1: button::Button::new(button1_pin),
            BUTTON2: button::Button::new(button2_pin),
            BUTTON3: button::Button::new(button3_pin),
            DISPLAY: il3820,
            SPI: spi,
            UI: ui::Model::init(),
            ALARM_MANAGER: alarm_manager,
        }
    }

    #[interrupt(priority = 4, spawn = [msg], resources = [BUTTON0, BUTTON1, BUTTON2, BUTTON3, SOUND])]
    fn TIM3() {
        unsafe { &*stm32::TIM3::ptr() }
            .sr
            .modify(|_, w| w.uif().clear_bit());

        if let button::Event::Pressed = resources.BUTTON0.poll() {
            resources.SOUND.stop();
            spawn.msg(ui::Msg::ButtonCancel).unwrap();
        }
        if let button::Event::Pressed = resources.BUTTON1.poll() {
            spawn.msg(ui::Msg::ButtonMinus).unwrap();
        }
        if let button::Event::Pressed = resources.BUTTON2.poll() {
            spawn.msg(ui::Msg::ButtonPlus).unwrap();
        }
        if let button::Event::Pressed = resources.BUTTON3.poll() {
            spawn.msg(ui::Msg::ButtonOk).unwrap();
        }
        resources.SOUND.poll();
    }

    #[interrupt(priority = 3, spawn = [msg], resources = [RTC_DEV, BME280, ALARM_MANAGER, SOUND])]
    fn RTC() {
        resources.RTC_DEV.clear_second_interrupt();
        resources.RTC_DEV.sync();

        let datetime = DateTime::new(resources.RTC_DEV.get_cnt());
        if datetime.sec == 0 && resources.ALARM_MANAGER.must_ring(&datetime) {
            resources.SOUND.lock(|alarm| alarm.play(&SO_WHAT, 10 * 60));
            let manager = resources.ALARM_MANAGER.clone();
            spawn.msg(ui::Msg::AlarmManager(manager)).unwrap();
        }
        spawn.msg(ui::Msg::DateTime(datetime)).unwrap();

        let msg = if let Ok(measurements) = resources.BME280.measure() {
            ui::Msg::Environment(crate::ui::Environment {
                pressure: measurements.pressure as u32,
                temperature: (measurements.temperature * 100.) as i16,
                humidity: measurements.humidity as u8,
            })
        } else {
            ui::Msg::FailEnvironment
        };
        spawn.msg(msg).unwrap();
    }

    #[task(priority = 2, capacity = 16, spawn = [msg], resources = [UI, RTC_DEV, FULL_UPDATE, ALARM_MANAGER])]
    fn msg(msg: ui::Msg) {
        use crate::ui::Cmd::*;
        for cmd in resources.UI.update(msg) {
            match cmd {
                UpdateRtc(dt) => {
                    if let Some(epoch) = dt.to_epoch() {
                        resources.RTC_DEV.lock(|rtc| {
                            let _ = rtc.set_cnt(epoch);
                        });
                        spawn.msg(ui::Msg::DateTime(dt)).unwrap();
                    }
                }
                UpdateAlarm(alarm, i) => {
                    let manager = resources.ALARM_MANAGER.lock(|m| {
                        m.alarms[i] = alarm;
                        m.clone()
                    });
                    spawn.msg(ui::Msg::AlarmManager(manager)).unwrap();
                }
                FullUpdate => *resources.FULL_UPDATE = true,
            }
        }
        rtfm::pend(stm32::Interrupt::EXTI1);
    }

    #[interrupt(priority = 1, resources = [UI, DISPLAY, SPI, FULL_UPDATE])]
    fn EXTI1() {
        while resources.DISPLAY.is_busy() {}
        let model = resources.UI.lock(|model| model.clone());
        let display = model.view();
        let full_update = resources
            .FULL_UPDATE
            .lock(|fu| core::mem::replace(&mut *fu, false));
        if full_update {
            resources
                .DISPLAY
                .set_lut(&mut *resources.SPI, Some(RefreshLUT::FULL))
                .unwrap(); //resources.DISPLAY.set_full();
        }
        resources
            .DISPLAY
            .update_frame(&mut *resources.SPI, &display.buffer())
            .unwrap();
        resources
            .DISPLAY
            .display_frame(&mut *resources.SPI)
            .unwrap();
        if full_update { // partial/quick refresh needs only be set when a full update was run before
            resources
                .DISPLAY
                .set_lut(&mut *resources.SPI, Some(RefreshLUT::QUICK))
                .unwrap();
        } //replacement of old: resources.DISPLAY.set_partial();        
        
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI2();
    }
};
