#![no_main]
#![no_std]

#[cfg(not(test))]
extern crate panic_semihosting;

use embedded_hal::digital::v1_compat::{OldInputPin, OldOutputPin};
use epd_waveshare::prelude::*;
use portable::datetime::DateTime;
use portable::{alarm, button, datetime, ui};
use pwm_speaker::songs::SO_WHAT;
use rtfm::app;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{delay, gpio, i2c, rtc, spi, stm32, timer};

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
    stm32f1xx_hal::spi::Spi2NoRemap,
    (
        gpio::gpiob::PB13<gpio::Alternate<gpio::PushPull>>,
        gpio::gpiob::PB14<gpio::Input<gpio::Floating>>,
        gpio::gpiob::PB15<gpio::Alternate<gpio::PushPull>>,
    ),
>;
type EPaperDisplay = epd_waveshare::epd2in9::EPD2in9<
    Spi,
    OldOutputPin<gpio::gpiob::PB12<gpio::Output<gpio::PushPull>>>, // cs/nss
    OldInputPin<gpio::gpioa::PA10<gpio::Input<gpio::Floating>>>,   // busy
    OldOutputPin<gpio::gpioa::PA8<gpio::Output<gpio::PushPull>>>,  // dc
    OldOutputPin<gpio::gpioa::PA9<gpio::Output<gpio::PushPull>>>,  // rst
>;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        rtc_dev: rtc::Rtc,
        bme280: bme280::BME280<I2C, delay::Delay>,
        alarm_manager: alarm::AlarmManager,
        sound: sound::Sound,
        button0: button::Button<Button0Pin>,
        button1: button::Button<Button1Pin>,
        button2: button::Button<Button2Pin>,
        button3: button::Button<Button3Pin>,
        display: EPaperDisplay,
        spi: Spi,
        ui: ui::Model,
        #[init(false)]
        full_update: bool,
        timer: timer::CountDownTimer<stm32::TIM3>,
        backup_domain: stm32f1xx_hal::backup_domain::BackupDomain,
    }

    #[init(spawn = [msg])]
    fn init(mut c: init::Context) -> init::LateResources {
        let mut flash = c.device.FLASH.constrain();
        let mut rcc = c.device.RCC.constrain();
        let mut afio = c.device.AFIO.constrain(&mut rcc.apb2);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let mut gpioa = c.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = c.device.GPIOB.split(&mut rcc.apb2);

        let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
        let c2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
        let c3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let c4 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);
        let mut pwm = timer::Timer::tim2(c.device.TIM2, &clocks, &mut rcc.apb1)
            .pwm::<timer::Tim2NoRemap, _, _, _>((c1, c2, c3, c4), &mut afio.mapr, 440.hz());
        pwm.0.enable();
        pwm.1.enable();
        let speaker = pwm_speaker::Speaker::new(pwm.0, clocks);

        let button0_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button1_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        let button2_pin = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
        let button3_pin = gpiob.pb1.into_pull_up_input(&mut gpiob.crl);

        let mut timer =
            timer::Timer::tim3(c.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.khz());
        timer.listen(timer::Event::Update);

        let mut backup_domain = rcc
            .bkp
            .constrain(c.device.BKP, &mut rcc.apb1, &mut c.device.PWR);
        let mut rtc_dev = rtc::Rtc::rtc(c.device.RTC, &mut backup_domain);
        if rtc_dev.current_time() < 100 {
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
                rtc_dev.set_time(epoch);
            }
        }
        rtc_dev.listen_seconds();

        let mut alarm_manager = alarm::AlarmManager::default();
        for i in 0..5 {
            let d0 = backup_domain.read_data_register_low(i * 2);
            let d1 = backup_domain.read_data_register_low(i * 2 + 1);
            if let Some(alarm) = alarm::Alarm::try_from(d0 as u32 | (d1 as u32) << 16) {
                alarm_manager.alarms[i] = alarm;
            }
        }

        let mut delay = delay::Delay::new(c.core.SYST, clocks);

        let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
        let miso = gpiob.pb14;
        let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
        let mut spi = spi::Spi::spi2(
            c.device.SPI2,
            (sck, miso, mosi),
            epd_waveshare::SPI_MODE,
            4.mhz(),
            clocks,
            &mut rcc.apb1,
        );
        let mut il3820 = epd_waveshare::epd2in9::EPD2in9::new(
            &mut spi,
            gpiob.pb12.into_push_pull_output(&mut gpiob.crh).into(),
            gpioa.pa10.into_floating_input(&mut gpioa.crh).into(),
            gpioa.pa8.into_push_pull_output(&mut gpioa.crh).into(),
            gpioa.pa9.into_push_pull_output(&mut gpioa.crh).into(),
            &mut delay,
        )
        .unwrap();
        il3820.set_lut(&mut spi, Some(RefreshLUT::QUICK)).unwrap();
        il3820.clear_frame(&mut spi).unwrap();

        c.core.DCB.enable_trace();
        c.core.DWT.enable_cycle_counter();
        let pb6 = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let pb7 = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
        let i2c = i2c::I2c::i2c1(
            c.device.I2C1,
            (pb6, pb7),
            &mut afio.mapr,
            i2c::Mode::Fast {
                frequency: 400.khz().into(),
                duty_cycle: i2c::DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
        );
        let i2c = i2c::blocking_i2c(i2c, clocks, 200, 10, 200, 200);
        let mut bme280 = bme280::BME280::new_primary(i2c, delay);
        bme280.init().expect("i2c init error");

        c.spawn
            .msg(ui::Msg::AlarmManager(alarm_manager.clone()))
            .unwrap();

        init::LateResources {
            rtc_dev,
            bme280,
            sound: sound::Sound::new(speaker),
            button0: button::Button::new(button0_pin),
            button1: button::Button::new(button1_pin),
            button2: button::Button::new(button2_pin),
            button3: button::Button::new(button3_pin),
            display: il3820,
            spi,
            ui: ui::Model::init(),
            alarm_manager,
            timer,
            backup_domain,
        }
    }

    #[task(binds = TIM3, priority = 4, spawn = [msg], resources = [button0, button1, button2, button3, sound, timer])]
    fn tick(c: tick::Context) {
        c.resources.timer.clear_update_interrupt_flag();

        if let button::Event::Pressed = c.resources.button0.poll() {
            c.resources.sound.stop();
            c.spawn.msg(ui::Msg::ButtonCancel).unwrap();
        }
        if let button::Event::Pressed = c.resources.button1.poll() {
            c.spawn.msg(ui::Msg::ButtonMinus).unwrap();
        }
        if let button::Event::Pressed = c.resources.button2.poll() {
            c.spawn.msg(ui::Msg::ButtonPlus).unwrap();
        }
        if let button::Event::Pressed = c.resources.button3.poll() {
            c.spawn.msg(ui::Msg::ButtonOk).unwrap();
        }
        c.resources.sound.poll();
    }

    #[task(binds = RTC, priority = 3, spawn = [msg], resources = [rtc_dev, bme280, alarm_manager, sound])]
    fn rtc_task(mut c: rtc_task::Context) {
        c.resources.rtc_dev.clear_second_flag();

        let datetime = DateTime::new(c.resources.rtc_dev.current_time());
        if datetime.sec == 0 && c.resources.alarm_manager.must_ring(&datetime) {
            c.resources
                .sound
                .lock(|alarm| alarm.play(&SO_WHAT, 10 * 60));
            let manager = c.resources.alarm_manager.clone();
            c.spawn.msg(ui::Msg::AlarmManager(manager)).unwrap();
        }
        c.spawn.msg(ui::Msg::DateTime(datetime)).unwrap();

        let msg = if let Ok(measurements) = c.resources.bme280.measure() {
            ui::Msg::Environment(crate::ui::Environment {
                pressure: measurements.pressure as u32,
                temperature: (measurements.temperature * 100.) as i16,
                humidity: measurements.humidity as u8,
            })
        } else {
            ui::Msg::FailEnvironment
        };
        c.spawn.msg(msg).unwrap();
    }

    #[task(priority = 2, capacity = 16, spawn = [msg], resources = [ui, rtc_dev, full_update, alarm_manager, backup_domain])]
    fn msg(mut c: msg::Context, msg: ui::Msg) {
        use crate::ui::Cmd::*;
        for cmd in c.resources.ui.update(msg) {
            match cmd {
                UpdateRtc(dt) => {
                    if let Some(epoch) = dt.to_epoch() {
                        c.resources.rtc_dev.lock(|rtc| {
                            let _ = rtc.set_time(epoch);
                        });
                        c.spawn.msg(ui::Msg::DateTime(dt)).unwrap();
                    }
                }
                UpdateAlarm(alarm, i) => {
                    let data = alarm.as_u32();
                    c.resources
                        .backup_domain
                        .write_data_register_low(i * 2, data as u16);
                    c.resources
                        .backup_domain
                        .write_data_register_low(i * 2 + 1, (data >> 16) as u16);
                    let manager = c.resources.alarm_manager.lock(|m| {
                        m.alarms[i] = alarm;
                        m.clone()
                    });
                    c.spawn.msg(ui::Msg::AlarmManager(manager)).unwrap();
                }
                FullUpdate => *c.resources.full_update = true,
            }
        }
        rtfm::pend(stm32::Interrupt::EXTI1);
    }

    #[task(binds = EXTI1, priority = 1, resources = [ui, display, spi, full_update])]
    fn render(mut c: render::Context) {
        let model = c.resources.ui.lock(|model| model.clone());
        let display = model.view();
        let full_update = c
            .resources
            .full_update
            .lock(|fu| core::mem::replace(&mut *fu, false));
        if full_update {
            c.resources
                .display
                .set_lut(&mut *c.resources.spi, Some(RefreshLUT::FULL))
                .unwrap();
        }

        c.resources
            .display
            .update_frame(&mut *c.resources.spi, &display.buffer())
            .unwrap();
        c.resources
            .display
            .display_frame(&mut *c.resources.spi)
            .unwrap();

        if full_update {
            // partial/quick refresh needs only be set when a full update was run before
            c.resources
                .display
                .set_lut(&mut *c.resources.spi, Some(RefreshLUT::QUICK))
                .unwrap();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI2();
    }
};
