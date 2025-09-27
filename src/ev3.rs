use core::{cell::Cell, future::Future, marker::PhantomData};

use crate::lcd::{Font, Lcd, LcdTopSide, LcdTopSideProvider, LcdTrait};
pub use ev3rt::LedColor;
use ev3rt::{
    analog_sensor_read, color_sensor_get_reflect, get_utime, gyro_sensor_get_rate, led_set_color,
    motor_config, motor_get_counts, motor_get_ticks, motor_reset_counts, motor_set_power,
    motor_stop, sensor_config, ultrasonic_sensor_get_distance_nxt, BtValue, MotorPort, MotorType,
    SensorPort, SensorType, BT, LCD_FRAMEBUFFER_ROWS, LCD_FRAMEBUFFER_ROW_BYTES,
};
use flagset::{flags, FlagSet};
use futures_micro::prelude::zip;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub us: u32,
}

impl Duration {
    pub const ZERO: Duration = Duration { us: 0 };
    pub const ONE_MS: Duration = Duration { us: 1000 };

    pub const fn from_us(micros: u32) -> Self {
        Duration { us: micros }
    }

    pub const fn from_ms(millis: u32) -> Self {
        Duration { us: millis * 1000 }
    }

    pub const fn from_secs(secs: u32) -> Self {
        Duration {
            us: secs * 1_000_000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Instant {
    pub us: u64,
}

impl Instant {
    pub const ZERO: Instant = Instant { us: 0 };

    pub fn absolute(us: u64) -> Self {
        Instant { us }
    }

    pub fn elapsed(&self, since: &Self) -> Duration {
        Duration {
            us: if self.us > since.us {
                (self.us - since.us) as u32
            } else {
                0
            },
        }
    }

    pub fn at(&self, duration: Duration) -> Self {
        Instant {
            us: self.us + duration.us as u64,
        }
    }

    pub fn is_after(&self, other: &Self) -> bool {
        self.us > other.us
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant {
            us: self.us + rhs.us as u64,
        }
    }
}

pub struct Time {
    start: Cell<Instant>,
    now: Cell<Instant>,
}

impl Time {
    pub fn new() -> Self {
        Time {
            start: Cell::new(Instant::ZERO),
            now: Cell::new(Instant::ZERO),
        }
    }

    pub fn reset(&self) {
        let now = get_utime();
        self.start.set(Instant::absolute(now));
        self.now.set(Instant::absolute(now));
    }

    pub fn update(&self) {
        let now = get_utime();
        self.now.set(Instant::absolute(now));
    }

    pub fn elapsed(&self) -> Duration {
        self.now.get().elapsed(&self.start.get())
    }

    pub fn start(&self) -> Instant {
        self.start.get()
    }

    pub fn now(&self) -> Instant {
        self.now.get()
    }

    pub fn timer_in<'a>(&'a self, at: Duration) -> Timer<'a> {
        Timer {
            time: &self,
            at: self.now().at(at),
        }
    }

    pub fn timer_ms<'a>(&'a self, ms: u32) -> Timer<'a> {
        self.timer_in(Duration::from_ms(ms as u32))
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Timer<'a> {
    time: &'a Time,
    at: Instant,
}

impl<'a> Timer<'a> {
    pub fn is_expired(&self) -> bool {
        self.time.now().is_after(&self.at)
    }

    pub fn map<TM>(
        self,
        mapper: &'a impl Fn(()) -> TM,
    ) -> MappedValue<'a, Self, impl Fn(()) -> TM> {
        MappedValue {
            original: self,
            mapper,
        }
    }

    pub fn filter<FILTER>(
        self,
        filter: &'a impl Fn(()) -> bool,
    ) -> FilteredValue<'a, Self, impl Fn(()) -> bool> {
        FilteredValue {
            original: self,
            filter,
        }
    }
}

impl Future for Timer<'_> {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        if self.is_expired() {
            core::task::Poll::Ready(())
        } else {
            core::task::Poll::Pending
        }
    }
}

pub struct ValueCell<T> {
    counter: Cell<usize>,
    value: Cell<T>,
}

impl<T: Copy + Default> ValueCell<T> {
    pub fn new() -> Self {
        ValueCell {
            counter: Cell::new(0),
            value: Cell::new(T::default()),
        }
    }

    pub fn get(&self) -> T {
        self.value.get()
    }

    pub fn update(&self, value: T) {
        self.counter.set(self.counter.get() + 1);
        self.value.set(value);
    }

    pub fn next<'a>(&'a self) -> NextValue<'a, T> {
        NextValue {
            sender: self,
            counter: self.counter.get() + 1,
        }
    }

    pub fn stream<'a>(&'a self) -> ValueStream<'a, T> {
        ValueStream {
            sender: self,
            counter: self.counter.get(),
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct NextValue<'a, T> {
    sender: &'a ValueCell<T>,
    counter: usize,
}

impl<'a, TO> NextValue<'a, TO> {
    pub fn map<TM>(
        self,
        mapper: &'a impl Fn(TO) -> TM,
    ) -> MappedValue<'a, Self, impl Fn(TO) -> TM> {
        MappedValue {
            original: self,
            mapper,
        }
    }

    pub fn filter(
        self,
        filter: &'a impl Fn(&TO) -> bool,
    ) -> FilteredValue<'a, Self, impl Fn(&TO) -> bool> {
        FilteredValue {
            original: self,
            filter,
        }
    }
}

impl<T: Copy + Default> Future for NextValue<'_, T> {
    type Output = T;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        if self.sender.counter.get() >= self.counter {
            core::task::Poll::Ready(self.sender.get())
        } else {
            core::task::Poll::Pending
        }
    }
}

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct MappedValue<'a, FUTURE, MAPPER> {
        #[pin]
        original: FUTURE,
        mapper: &'a MAPPER,
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, TM, MAPPER: Fn(TO) -> TM>
    MappedValue<'a, FUTURE, MAPPER>
{
    pub fn new(original: FUTURE, mapper: &'a MAPPER) -> Self {
        MappedValue { original, mapper }
    }

    pub fn filter<FILTER>(
        self,
        filter: &'a impl Fn(TO) -> bool,
    ) -> FilteredValue<'a, Self, impl Fn(TO) -> bool> {
        FilteredValue {
            original: self,
            filter,
        }
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, TM, MAPPER: Fn(TO) -> TM> Future
    for MappedValue<'a, FUTURE, MAPPER>
{
    type Output = TM;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match this.original.poll(cx) {
            core::task::Poll::Ready(value) => core::task::Poll::Ready((this.mapper)(value)),
            core::task::Poll::Pending => core::task::Poll::Pending,
        }
    }
}

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct FilteredValue<'a, FUTURE, FILTER> {
        #[pin]
        original: FUTURE,
        filter: &'a FILTER,
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, FILTER: Fn(&TO) -> bool>
    FilteredValue<'a, FUTURE, FILTER>
{
    pub fn new(original: FUTURE, filter: &'a FILTER) -> Self {
        FilteredValue { original, filter }
    }

    pub fn map<TM>(
        self,
        mapper: &'a impl Fn(TO) -> TM,
    ) -> MappedValue<'a, Self, impl Fn(TO) -> TM> {
        MappedValue {
            original: self,
            mapper,
        }
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, FILTER: Fn(&TO) -> bool> Future
    for FilteredValue<'a, FUTURE, FILTER>
{
    type Output = FUTURE::Output;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match this.original.poll(cx) {
            core::task::Poll::Ready(value) => {
                if (this.filter)(&value) {
                    core::task::Poll::Ready(value)
                } else {
                    core::task::Poll::Pending
                }
            }
            core::task::Poll::Pending => core::task::Poll::Pending,
        }
    }
}

pub struct ValueStream<'a, T: Copy + Default> {
    sender: &'a ValueCell<T>,
    counter: usize,
}

impl<T: Copy + Default> ValueStream<'_, T> {
    pub fn next<'a>(&'a mut self) -> NextValue<'a, T> {
        let counter = self.sender.counter.get().max(self.counter);
        self.counter = counter + 1;
        NextValue {
            sender: self.sender,
            counter,
        }
    }
}

pub struct Led {
    previous: Cell<LedColor>,
    required: Cell<LedColor>,
}

impl Led {
    pub fn new() -> Self {
        led_set_color(LedColor::OFF);
        Led {
            previous: Cell::new(LedColor::OFF),
            required: Cell::new(LedColor::OFF),
        }
    }

    pub fn set(&self, color: LedColor) {
        self.required.set(color);
    }

    pub fn apply(&self) {
        let required = self.required.get();
        if required != self.previous.get() {
            led_set_color(required);
            self.previous.set(required);
        }
    }
}

flags! {
    pub enum Button: u8 {
        UP,
        DOWN,
        LEFT,
        RIGHT,
        ENTER,
        BACK,
    }
}
pub type ButtonFlags = FlagSet<Button>;

fn transpose_buttons<TOP: LcdTopSideProvider>(buttons: ButtonFlags) -> ButtonFlags {
    let mut result = buttons & (Button::ENTER | Button::BACK);
    match TOP::TOP_SIDE {
        LcdTopSide::Up => {
            if buttons.contains(Button::UP) {
                result |= Button::UP;
            }
            if buttons.contains(Button::RIGHT) {
                result |= Button::RIGHT;
            }
            if buttons.contains(Button::DOWN) {
                result |= Button::DOWN;
            }
            if buttons.contains(Button::LEFT) {
                result |= Button::LEFT;
            }
        }
        LcdTopSide::Right => {
            if buttons.contains(Button::RIGHT) {
                result |= Button::UP;
            }
            if buttons.contains(Button::DOWN) {
                result |= Button::RIGHT;
            }
            if buttons.contains(Button::LEFT) {
                result |= Button::DOWN;
            }
            if buttons.contains(Button::UP) {
                result |= Button::LEFT;
            }
        }
        LcdTopSide::Down => {
            if buttons.contains(Button::DOWN) {
                result |= Button::UP;
            }
            if buttons.contains(Button::LEFT) {
                result |= Button::RIGHT;
            }
            if buttons.contains(Button::UP) {
                result |= Button::DOWN;
            }
            if buttons.contains(Button::RIGHT) {
                result |= Button::LEFT;
            }
        }
        LcdTopSide::Left => {
            if buttons.contains(Button::LEFT) {
                result |= Button::UP;
            }
            if buttons.contains(Button::UP) {
                result |= Button::RIGHT;
            }
            if buttons.contains(Button::RIGHT) {
                result |= Button::DOWN;
            }
            if buttons.contains(Button::DOWN) {
                result |= Button::LEFT;
            }
        }
    }
    result
}

pub struct ButtonsController<TOP: LcdTopSideProvider> {
    top: PhantomData<TOP>,
    previous: Cell<ButtonFlags>,
    pub down: ValueCell<ButtonFlags>,
    pub pressed_event: ValueCell<ButtonFlags>,
    pub released_event: ValueCell<ButtonFlags>,
    pub flltered_pressed_event: ValueCell<()>,
}

pub trait ButtonsControllerTrait {
    fn down(&self) -> &ValueCell<ButtonFlags>;
    fn pressed_event(&self) -> &ValueCell<ButtonFlags>;
    fn released_event(&self) -> &ValueCell<ButtonFlags>;
    fn flltered_pressed_event(&self) -> &ValueCell<()>;
}

impl<TOP: LcdTopSideProvider> ButtonsControllerTrait for ButtonsController<TOP> {
    fn down(&self) -> &ValueCell<ButtonFlags> {
        &self.down
    }

    fn pressed_event(&self) -> &ValueCell<ButtonFlags> {
        &self.pressed_event
    }

    fn released_event(&self) -> &ValueCell<ButtonFlags> {
        &self.released_event
    }

    fn flltered_pressed_event(&self) -> &ValueCell<()> {
        &self.flltered_pressed_event
    }
}

impl<TOP: LcdTopSideProvider> ButtonsController<TOP> {
    pub fn new() -> Self {
        ButtonsController {
            top: PhantomData,
            previous: Cell::new(ButtonFlags::default()),
            down: ValueCell::new(),
            pressed_event: ValueCell::new(),
            released_event: ValueCell::new(),
            flltered_pressed_event: ValueCell::new(),
        }
    }

    pub fn update(&self) {
        let previous = self.previous.get();
        let mut current = ButtonFlags::default();
        if ev3rt::button_is_pressed(ev3rt::Button::UP) {
            current |= Button::UP;
        }
        if ev3rt::button_is_pressed(ev3rt::Button::DOWN) {
            current |= Button::DOWN;
        }
        if ev3rt::button_is_pressed(ev3rt::Button::LEFT) {
            current |= Button::LEFT;
        }
        if ev3rt::button_is_pressed(ev3rt::Button::RIGHT) {
            current |= Button::RIGHT;
        }
        if ev3rt::button_is_pressed(ev3rt::Button::ENTER) {
            current |= Button::ENTER;
        }
        if ev3rt::button_is_pressed(ev3rt::Button::BACK) {
            current |= Button::BACK;
        }
        let current = transpose_buttons::<TOP>(current);

        self.down.update(current);
        self.pressed_event.update(current & !previous);
        self.released_event.update(previous & !current);
        if current != previous {
            let pressed = current & !previous;
            if pressed != ButtonFlags::from(None) {
                self.flltered_pressed_event.update(());
            }
        }
    }
}

pub trait SensorPortProvider {
    const ID: SensorPort;
}
pub struct SensorPort1 {}
impl SensorPortProvider for SensorPort1 {
    const ID: SensorPort = SensorPort::S1;
}
pub struct SensorPort2 {}
impl SensorPortProvider for SensorPort2 {
    const ID: SensorPort = SensorPort::S2;
}
pub struct SensorPort3 {}
impl SensorPortProvider for SensorPort3 {
    const ID: SensorPort = SensorPort::S3;
}
pub struct SensorPort4 {}
impl SensorPortProvider for SensorPort4 {
    const ID: SensorPort = SensorPort::S4;
}

pub trait SensorConfig {
    const PERIOD: Duration;
    const KIND: SensorType;
}
pub struct SensorNone {}
impl SensorConfig for SensorNone {
    const PERIOD: Duration = Duration::ONE_MS;
    const KIND: SensorType = SensorType::NONE;
}
pub struct SensorColor {}
impl SensorConfig for SensorColor {
    const PERIOD: Duration = Duration::from_us(500);
    const KIND: SensorType = SensorType::COLOR;
}
pub struct SensorNxtUltrasonic {}
impl SensorConfig for SensorNxtUltrasonic {
    const PERIOD: Duration = Duration::from_ms(20);
    const KIND: SensorType = SensorType::NxtULTRASONIC;
}
pub struct SensorTouchAnalog {}
impl SensorConfig for SensorTouchAnalog {
    const PERIOD: Duration = Duration::from_us(500);
    const KIND: SensorType = SensorType::TOUCH;
}
pub struct SensorGyro {}
impl SensorConfig for SensorGyro {
    const PERIOD: Duration = Duration::from_us(500);
    const KIND: SensorType = SensorType::GYRO;
}

#[derive(Clone, Copy)]
pub struct SensorValue {
    pub dt: Duration,
    pub value: i32,
}

impl SensorValue {
    pub(crate) fn check_actual_read(&self) -> bool {
        self.value != 100 && self.value != 255 && self.value != 0
    }
}

impl Default for SensorValue {
    fn default() -> Self {
        SensorValue {
            dt: Duration::ONE_MS,
            value: 0,
        }
    }
}

pub struct Sensor<CFG: SensorConfig, PORT: SensorPortProvider> {
    config: PhantomData<CFG>,
    port: PhantomData<PORT>,
    last_read: Cell<Instant>,
    next_read: Cell<Instant>,
    pub value: ValueCell<SensorValue>,
}

pub trait SensorTrait {
    fn value(&self) -> &ValueCell<SensorValue>;
    fn is_configured(&self) -> bool;
}

impl<CFG: SensorConfig, PORT: SensorPortProvider> SensorTrait for Sensor<CFG, PORT> {
    fn value(&self) -> &ValueCell<SensorValue> {
        &self.value
    }

    fn is_configured(&self) -> bool {
        self.is_configured()
    }
}

impl<CFG: SensorConfig, PORT: SensorPortProvider> Sensor<CFG, PORT> {
    pub fn new() -> Self {
        Sensor {
            config: PhantomData,
            port: PhantomData,
            last_read: Cell::new(Instant::ZERO),
            next_read: Cell::new(Instant::ZERO + CFG::PERIOD),
            value: ValueCell::new(),
        }
    }

    fn init(&self) {
        sensor_config(PORT::ID, CFG::KIND);
    }

    fn update(&self, now: Instant) {
        if now.is_after(&self.next_read.get()) {
            let dt = now.elapsed(&self.last_read.get());
            let next_read = now + CFG::PERIOD;
            let value = {
                match CFG::KIND {
                    SensorType::COLOR => color_sensor_get_reflect(PORT::ID) as i32,
                    SensorType::NxtULTRASONIC => {
                        ultrasonic_sensor_get_distance_nxt(PORT::ID) as i32
                    }
                    SensorType::TOUCH => analog_sensor_read(PORT::ID) as i32,
                    SensorType::GYRO => gyro_sensor_get_rate(PORT::ID) as i32,
                    _ => 0,
                }
            };
            self.last_read.set(now);
            self.next_read.set(next_read);
            self.value.update(SensorValue { dt, value });
        }
    }

    pub fn is_configured(&self) -> bool {
        match CFG::KIND {
            SensorType::NONE => false,
            _ => true,
        }
    }
}

pub trait MotorPortProvider {
    const ID: MotorPort;
}
pub struct MotorPortA {}
impl MotorPortProvider for MotorPortA {
    const ID: MotorPort = MotorPort::A;
}
pub struct MotorPortB {}
impl MotorPortProvider for MotorPortB {
    const ID: MotorPort = MotorPort::B;
}
pub struct MotorPortC {}
impl MotorPortProvider for MotorPortC {
    const ID: MotorPort = MotorPort::C;
}
pub struct MotorPortD {}
impl MotorPortProvider for MotorPortD {
    const ID: MotorPort = MotorPort::D;
}

pub trait MotorConfig {
    const KIND: MotorType;
    const BRAKE: bool;
}

pub struct MotorNone {}
impl MotorConfig for MotorNone {
    const KIND: MotorType = MotorType::NONE;
    const BRAKE: bool = false;
}
pub struct MotorLarge {}
impl MotorConfig for MotorLarge {
    const KIND: MotorType = MotorType::LARGE;
    const BRAKE: bool = false;
}
pub struct MotorLargeBrake {}
impl MotorConfig for MotorLargeBrake {
    const KIND: MotorType = MotorType::LARGE;
    const BRAKE: bool = true;
}
pub struct MotorMedium {}
impl MotorConfig for MotorMedium {
    const KIND: MotorType = MotorType::MEDIUM;
    const BRAKE: bool = false;
}
pub struct MotorMediumBrake {}
impl MotorConfig for MotorMediumBrake {
    const KIND: MotorType = MotorType::MEDIUM;
    const BRAKE: bool = true;
}
pub struct MotorUnregulated {}
impl MotorConfig for MotorUnregulated {
    const KIND: MotorType = MotorType::UNDEGULATED;
    const BRAKE: bool = false;
}
pub struct MotorUnregulatedBrake {}
impl MotorConfig for MotorUnregulatedBrake {
    const KIND: MotorType = MotorType::UNDEGULATED;
    const BRAKE: bool = true;
}

#[derive(Clone, Copy)]
pub struct MotorCounterValue {
    pub dt: Duration,
    pub ticks: u32,
    pub value: i32,
}

impl Default for MotorCounterValue {
    fn default() -> Self {
        MotorCounterValue {
            dt: Duration::ONE_MS,
            ticks: 0,
            value: 0,
        }
    }
}

pub struct Motor<CFG: MotorConfig, PORT: MotorPortProvider> {
    cfg: PhantomData<CFG>,
    port: PhantomData<PORT>,
    power: Cell<i32>,
    brake: Cell<bool>,
    reset: Cell<bool>,
    last_read: Cell<Instant>,
    pub counter: ValueCell<MotorCounterValue>,
}

pub trait MotorTrait {
    fn counter(&self) -> &ValueCell<MotorCounterValue>;
    fn is_configured(&self) -> bool;
    fn stop(&self);
    fn get_power(&self) -> i32;
    fn set_power(&self, power: i32);
    fn set_brake(&mut self, brake: bool);
    fn reset(&mut self);
}

impl<CFG: MotorConfig, PORT: MotorPortProvider> MotorTrait for Motor<CFG, PORT> {
    fn counter(&self) -> &ValueCell<MotorCounterValue> {
        &self.counter
    }

    fn is_configured(&self) -> bool {
        self.is_configured()
    }

    fn stop(&self) {
        self.stop()
    }

    fn get_power(&self) -> i32 {
        self.get_power()
    }

    fn set_power(&self, power: i32) {
        self.set_power(power);
    }

    fn set_brake(&mut self, brake: bool) {
        self.set_brake(brake);
    }

    fn reset(&mut self) {
        self.reset();
    }
}

impl<CFG: MotorConfig, PORT: MotorPortProvider> Motor<CFG, PORT> {
    pub fn new(now: Instant) -> Self {
        Self {
            cfg: PhantomData,
            port: PhantomData,
            power: Cell::new(0),
            brake: Cell::new(CFG::BRAKE),
            reset: Cell::new(false),
            last_read: Cell::new(now),
            counter: ValueCell::new(),
        }
    }

    pub fn is_configured(&self) -> bool {
        match CFG::KIND {
            MotorType::NONE => false,
            _ => true,
        }
    }

    pub fn init(&self) {
        motor_config(PORT::ID, CFG::KIND);
        self.power.set(0);
        self.brake.set(CFG::BRAKE);
        self.reset.set(true);
        self.apply();
    }

    pub fn stop(&self) {
        self.set_power(0);
    }

    pub fn get_power(&self) -> i32 {
        self.power.get()
    }

    pub fn set_power(&self, power: i32) {
        self.power.set(power);
        self.brake.set(if power == 0 { CFG::BRAKE } else { false });
    }

    pub fn set_brake(&mut self, brake: bool) {
        self.brake.set(brake);
    }

    pub fn reset(&mut self) {
        self.reset.set(true);
    }

    pub fn update(&self, now: Instant) {
        let dt = now.elapsed(&self.last_read.get());
        let (counter, ticks) = {
            let port = PORT::ID;
            let kind = CFG::KIND;
            match kind {
                MotorType::MEDIUM | MotorType::LARGE => {
                    (motor_get_counts(port), motor_get_ticks(port))
                }
                MotorType::NONE | MotorType::UNDEGULATED => (0, 0),
            }
        };
        let value = MotorCounterValue {
            dt,
            ticks,
            value: counter,
        };
        self.last_read.set(now);
        self.counter.update(value);
    }

    pub fn apply(&self) {
        let power = self.power.get();
        let brake = self.brake.get();
        let reset = self.reset.get();
        motor_set_power(PORT::ID, power);
        if power == 0 {
            motor_stop(PORT::ID, brake);
        }
        if reset {
            motor_reset_counts(PORT::ID);
            self.reset.set(false);
        }
    }
}

pub struct BaseEv3<
    S1: SensorConfig,
    S2: SensorConfig,
    S3: SensorConfig,
    S4: SensorConfig,
    MA: MotorConfig,
    MB: MotorConfig,
    MC: MotorConfig,
    MD: MotorConfig,
    TOP: LcdTopSideProvider + 'static,
> {
    top: PhantomData<TOP>,
    pub time: Time,
    pub s1: Sensor<S1, SensorPort1>,
    pub s2: Sensor<S2, SensorPort2>,
    pub s3: Sensor<S3, SensorPort3>,
    pub s4: Sensor<S4, SensorPort4>,
    pub ma: Motor<MA, MotorPortA>,
    pub mb: Motor<MB, MotorPortB>,
    pub mc: Motor<MC, MotorPortC>,
    pub md: Motor<MD, MotorPortD>,
    pub lcd: Lcd<TOP>,
    lcd_refresh_rate: Duration,
    next_lcd_refresh: Cell<Instant>,
    pub buttons: ButtonsController<TOP>,
    pub led: Led,
    bt: Cell<Option<BT>>,
    bt_value: Cell<Option<BtValue>>,
}

pub type PinBoxed<T> = core::pin::Pin<alloc::boxed::Box<T>>;
pub fn pin_boxed<T>(t: T) -> PinBoxed<T> {
    alloc::boxed::Box::pin(t)
}

pub type RcPinBoxed<T> = core::pin::Pin<alloc::rc::Rc<T>>;
pub fn rc_pin_boxed<T>(t: T) -> RcPinBoxed<T> {
    alloc::rc::Rc::pin(t)
}

pub struct PinnedEv3<EV3: Ev3Brick + Ev3BrickUpdater> {
    ev3: RcPinBoxed<EV3>,
}

impl<EV3: Ev3Brick + Ev3BrickUpdater> Clone for PinnedEv3<EV3> {
    fn clone(&self) -> Self {
        PinnedEv3 {
            ev3: self.ev3.clone(),
        }
    }
}

impl<EV3: Ev3Brick + Ev3BrickUpdater> PinnedEv3<EV3> {
    pub fn run(self, future: PinBoxed<impl Future<Output = ()>>, diagnostic: bool) {
        let mut future = future;
        let waker = noop_waker();
        let mut context = core::task::Context::from_waker(&waker);

        let mut tick = 0;
        loop {
            self.ev3.update();
            if future.as_mut().poll(&mut context) == core::task::Poll::Ready(()) {
                break;
            }
            self.ev3.apply();

            if diagnostic {
                tick += 1;
                if tick % 100 == 0 {
                    let bars = tick / 100;
                    for r in 1..6 {
                        let start = LCD_FRAMEBUFFER_ROW_BYTES * (LCD_FRAMEBUFFER_ROWS - r);
                        let middle = start + (LCD_FRAMEBUFFER_ROW_BYTES * bars / 10);
                        let end = start + LCD_FRAMEBUFFER_ROW_BYTES;
                        ev3rt::lcd_apply(|pixels| (&mut pixels[start..middle]).fill(0xff));
                        ev3rt::lcd_apply(|pixels| (&mut pixels[middle..end]).fill(0x00));
                    }
                }
                tick %= 1000;
            }
            ev3rt::msleep(1);
        }
        ev3rt::reset(false);
    }
}

impl<EV3: Ev3Brick + Ev3BrickUpdater> Ev3Brick for PinnedEv3<EV3> {
    fn init(&self) {
        self.ev3.init();
    }

    fn reset_time(&self) {
        self.ev3.time().reset();
    }

    async fn setup(&self) {
        self.ev3.setup().await;
    }

    fn time(&self) -> &Time {
        &self.ev3.time()
    }

    fn timer_in<'a>(&'a self, at: Duration) -> Timer<'a> {
        self.ev3.timer_in(at)
    }

    fn timer_ms<'a>(&'a self, ms: u32) -> Timer<'a> {
        self.ev3.timer_ms(ms)
    }

    fn tick<'a>(&'a self) -> Timer<'a> {
        self.ev3.tick()
    }

    fn s1(&self) -> &impl SensorTrait {
        self.ev3.s1()
    }

    fn s2(&self) -> &impl SensorTrait {
        self.ev3.s2()
    }

    fn s3(&self) -> &impl SensorTrait {
        self.ev3.s3()
    }

    fn s4(&self) -> &impl SensorTrait {
        self.ev3.s4()
    }

    fn ma(&self) -> &impl MotorTrait {
        self.ev3.ma()
    }

    fn mb(&self) -> &impl MotorTrait {
        self.ev3.mb()
    }

    fn mc(&self) -> &impl MotorTrait {
        self.ev3.mc()
    }

    fn md(&self) -> &impl MotorTrait {
        self.ev3.md()
    }

    fn lcd(&self) -> &impl LcdTrait {
        self.ev3.lcd()
    }

    fn buttons(&self) -> &impl ButtonsControllerTrait {
        self.ev3.buttons()
    }

    fn led(&self) -> &Led {
        self.ev3.led()
    }

    fn set_bt_connection(&self, bt: BT) {
        self.ev3.set_bt_connection(bt);
    }

    fn last_bt_value(&self) -> BtValue {
        self.ev3.last_bt_value()
    }

    fn set_bt_value(&self, value: BtValue) {
        self.ev3.set_bt_value(value);
    }

    fn mut_bt_value(&self, f: impl FnOnce(&mut BtValue)) {
        self.ev3.mut_bt_value(f);
    }

    fn bt_is_connected(&self) -> bool {
        self.ev3.bt_is_connected()
    }

    fn bt_read_count(&self) -> usize {
        self.ev3.bt_read_count()
    }

    fn bt_read_value(&self) -> BtValue {
        self.ev3.bt_read_value()
    }

    async fn bt_connect_to_device(&self, device: &[u8; 6], pin: &[u8; 4], read_period_ms: u32) {
        self.ev3
            .bt_connect_to_device(device, pin, read_period_ms)
            .await;
    }

    async fn bt_connect_to_host(&self, read_period_ms: u32) {
        self.ev3.bt_connect_to_host(read_period_ms).await;
    }
}

impl<
        S1: SensorConfig,
        S2: SensorConfig,
        S3: SensorConfig,
        S4: SensorConfig,
        MA: MotorConfig,
        MB: MotorConfig,
        MC: MotorConfig,
        MD: MotorConfig,
        TOP: LcdTopSideProvider + 'static,
    > BaseEv3<S1, S2, S3, S4, MA, MB, MC, MD, TOP>
{
    pub fn new(font: &'static Font<TOP>, lcd_refresh_rate: Duration) -> PinnedEv3<Self> {
        let time = Time::new();
        let now = time.now();
        let next_lcd_refresh = Cell::new(now + lcd_refresh_rate);
        PinnedEv3 {
            ev3: rc_pin_boxed(BaseEv3 {
                top: PhantomData,
                time,
                s1: Sensor::new(),
                s2: Sensor::new(),
                s3: Sensor::new(),
                s4: Sensor::new(),
                ma: Motor::new(now),
                mb: Motor::new(now),
                mc: Motor::new(now),
                md: Motor::new(now),
                lcd: Lcd::new(font),
                lcd_refresh_rate,
                next_lcd_refresh,
                buttons: ButtonsController::new(),
                led: Led::new(),
                bt: Cell::new(None),
                bt_value: Cell::new(None),
            }),
        }
    }

    pub fn set_bt_connection(&self, bt: BT) {
        self.bt.set(Some(bt));
    }
    pub fn last_bt_value(&self) -> BtValue {
        self.bt_value.get().unwrap_or(0.into())
    }
    pub fn set_bt_value(&self, value: BtValue) {
        self.bt_value.set(Some(value));
    }
    pub fn mut_bt_value(&self, f: impl FnOnce(&mut BtValue)) {
        let mut value = self.bt_value.get().unwrap_or(0.into());
        f(&mut value);
        self.bt_value.set(Some(value));
    }
    pub fn bt_is_connected(&self) -> bool {
        self.bt.get().map(|bt| bt.is_connected()).unwrap_or(false)
    }
    pub fn bt_read_count(&self) -> usize {
        self.bt.get().map(|bt| bt.read_count()).unwrap_or(0)
    }
    pub fn bt_read_value(&self) -> BtValue {
        self.bt.get().map(|bt| bt.read()).unwrap_or(0.into())
    }

    pub async fn bt_connect_to_device(&self, device: &[u8; 6], pin: &[u8; 4], read_period_ms: u32) {
        self.lcd.clear();
        self.lcd.print(1, "  BT   ");
        self.lcd.print(2, " CONN n");
        self.led.set(LedColor::RED);
        self.tick().await;

        let mut attempts = 0;
        let bt = loop {
            attempts += 1;
            self.lcd.print_value(3, 1, 5, attempts);
            if let Some(bt) = BT::new_master(device, pin, read_period_ms) {
                break bt;
            }
            self.timer_ms(25).await;
        };

        self.set_bt_connection(bt);
        self.lcd.clear();
        self.lcd.print(1, "  BT   ");
        self.lcd.print(2, " CONN y");
        self.led.set(LedColor::GREEN);
        self.timer_in(Duration::from_secs(1)).await;

        self.lcd.clear();
        self.led.set(LedColor::OFF);
        self.tick().await;
    }

    pub async fn bt_connect_to_host(&self, read_period_ms: u32) {
        self.lcd.clear();
        self.lcd.print(1, "  BT   ");
        self.lcd.print(2, " CONN n");
        self.led.set(LedColor::RED);
        self.tick().await;

        let mut attempts = 0;
        let bt = loop {
            attempts += 1;
            self.lcd.print_value(3, 1, 5, attempts);
            if let Some(bt) = BT::new_slave(read_period_ms) {
                break bt;
            }
            self.timer_ms(25).await;
        };

        self.set_bt_connection(bt);
        self.lcd.clear();
        self.lcd.print(1, "  BT   ");
        self.lcd.print(2, " CONN y");
        self.led.set(LedColor::GREEN);
        self.tick().await;

        self.timer_in(Duration::from_secs(1)).await;
        self.lcd.clear();
        self.led.set(LedColor::OFF);
        self.tick().await;
    }

    pub fn init(&self) {
        self.time.reset();
        self.s1.init();
        self.s2.init();
        self.s3.init();
        self.s4.init();
        self.ma.init();
        self.mb.init();
        self.mc.init();
        self.md.init();
        self.lcd.clear();
        self.time.update();
    }

    pub fn update(&self) {
        self.time.update();
        let now = self.time.now();
        self.s1.update(now);
        self.s2.update(now);
        self.s3.update(now);
        self.s4.update(now);
        self.ma.update(now);
        self.mb.update(now);
        self.mc.update(now);
        self.md.update(now);
        self.buttons.update();
        if let (Some(bt), Some(value)) = (self.bt.get(), self.bt_value.get()) {
            bt.write(value);
            self.bt_value.set(None);
        }
    }

    pub fn apply(&self) {
        self.ma.apply();
        self.mb.apply();
        self.mc.apply();
        self.md.apply();
        self.led.apply();

        if self.time.now().is_after(&self.next_lcd_refresh.get()) {
            self.lcd.refresh();
            self.next_lcd_refresh
                .set(self.time.now() + self.lcd_refresh_rate);
        }
    }

    pub fn timer_in<'a>(&'a self, at: Duration) -> Timer<'a> {
        self.time.timer_in(at)
    }

    pub fn timer_ms<'a>(&'a self, ms: u32) -> Timer<'a> {
        self.time.timer_ms(ms)
    }

    pub fn tick<'a>(&'a self) -> Timer<'a> {
        self.time.timer_in(Duration::from_us(500))
    }

    pub async fn setup(&self) {
        self.lcd.clear();
        self.lcd.print(0, "  CFG  ");
        self.lcd.print(1, " SETUP ");
        self.lcd.print(3, "- - - -");
        self.lcd.print(4, "1 2 3 4");
        self.lcd.print(5, "- - - -");
        self.lcd.print(6, "A B C D");
        zip(self.check_sensors(), self.check_motors()).await;
        self.timer_in(Duration::from_secs(1)).await;
    }

    async fn check_sensor_1(&self) {
        const ROW: usize = 3;
        const COLUMN: usize = 0;

        if S1::KIND == SensorType::NONE {
            return;
        }

        let mut values = self.s1.value().stream();
        loop {
            let value = values.next().await;
            if value.check_actual_read() {
                self.lcd.print_char(COLUMN, ROW, 'y');
                return;
            } else {
                self.lcd.print_char(COLUMN, ROW, 'n');
            }
        }
    }

    async fn check_sensor_2(&self) {
        const ROW: usize = 3;
        const COLUMN: usize = 2;

        if S1::KIND == SensorType::NONE {
            return;
        }

        let mut values = self.s2.value().stream();
        loop {
            let value = values.next().await;
            if value.check_actual_read() {
                self.lcd.print_char(COLUMN, ROW, 'y');
                return;
            } else {
                self.lcd.print_char(COLUMN, ROW, 'n');
            }
        }
    }

    async fn check_sensor_3(&self) {
        const ROW: usize = 3;
        const COLUMN: usize = 4;

        if S1::KIND == SensorType::NONE {
            return;
        }

        let mut values = self.s3.value().stream();
        loop {
            let value = values.next().await;
            if value.check_actual_read() {
                self.lcd.print_char(COLUMN, ROW, 'y');
                return;
            } else {
                self.lcd.print_char(COLUMN, ROW, 'n');
            }
        }
    }

    async fn check_sensor_4(&self) {
        const ROW: usize = 3;
        const COLUMN: usize = 6;

        if S1::KIND == SensorType::NONE {
            return;
        }

        let mut values = self.s4.value().stream();
        loop {
            let value = values.next().await;
            if value.check_actual_read() {
                self.lcd.print_char(COLUMN, ROW, 'y');
                return;
            } else {
                self.lcd.print_char(COLUMN, ROW, 'n');
            }
        }
    }

    async fn check_sensors(&self) {
        zip!(
            self.check_sensor_1(),
            self.check_sensor_2(),
            self.check_sensor_3(),
            self.check_sensor_4(),
        )
        .await;
    }

    async fn check_motors(&self) {
        const MOVEMENT_DURATION: Duration = Duration::from_ms(125);
        const MOVEMENT_POWER: i32 = 50;
        const Y: usize = 5;
        const XA: usize = 0;
        const XB: usize = 2;
        const XC: usize = 4;
        const XD: usize = 6;

        if MA::KIND != MotorType::NONE {
            let m = &self.ma;
            self.lcd.print_char(XA, Y, 'n');
            for i in 0..8 {
                m.set_power(MOVEMENT_POWER * (i % 2 * 2 - 1));
                self.timer_in(MOVEMENT_DURATION).await;
            }
            m.stop();
            self.lcd.print_char(XA, Y, 'y');
        }

        if MB::KIND != MotorType::NONE {
            let m = &self.mb;
            self.lcd.print_char(XB, Y, 'n');
            for i in 0..8 {
                m.set_power(MOVEMENT_POWER * (i % 2 * 2 - 1));
                self.timer_in(MOVEMENT_DURATION).await;
            }
            m.stop();
            self.lcd.print_char(XB, Y, 'y');
        }

        if MC::KIND != MotorType::NONE {
            let m = &self.mc;
            self.lcd.print_char(XC, Y, 'n');
            for i in 0..8 {
                m.set_power(MOVEMENT_POWER * (i % 2 * 2 - 1));
                self.timer_in(MOVEMENT_DURATION).await;
            }
            m.stop();
            self.lcd.print_char(XC, Y, 'y');
        }

        if MD::KIND != MotorType::NONE {
            let m = &self.md;
            self.lcd.print_char(XD, Y, 'n');
            for i in 0..8 {
                m.set_power(MOVEMENT_POWER * (i % 2 * 2 - 1));
                self.timer_in(MOVEMENT_DURATION).await;
            }
            m.stop();
            self.lcd.print_char(XD, Y, 'y');
        }
    }
}

fn no_op(_: *const ()) {}
fn no_op_clone(_: *const ()) -> core::task::RawWaker {
    noop_raw_waker()
}
static RWVT: core::task::RawWakerVTable =
    core::task::RawWakerVTable::new(no_op_clone, no_op, no_op, no_op);

#[inline]
fn noop_raw_waker() -> core::task::RawWaker {
    core::task::RawWaker::new(core::ptr::null(), &RWVT)
}

#[inline]
fn noop_waker() -> core::task::Waker {
    unsafe { core::task::Waker::from_raw(noop_raw_waker()) }
}

pub trait Ev3BrickUpdater {
    fn update(&self);
    fn apply(&self);
}

pub trait Ev3Brick {
    fn init(&self);
    fn setup(&self) -> impl Future<Output = ()>;
    fn time(&self) -> &Time;
    fn reset_time(&self);
    fn timer_in<'a>(&'a self, at: Duration) -> Timer<'a>;
    fn timer_ms<'a>(&'a self, ms: u32) -> Timer<'a>;
    fn tick<'a>(&'a self) -> Timer<'a>;
    fn s1(&self) -> &impl SensorTrait;
    fn s2(&self) -> &impl SensorTrait;
    fn s3(&self) -> &impl SensorTrait;
    fn s4(&self) -> &impl SensorTrait;
    fn ma(&self) -> &impl MotorTrait;
    fn mb(&self) -> &impl MotorTrait;
    fn mc(&self) -> &impl MotorTrait;
    fn md(&self) -> &impl MotorTrait;
    fn lcd(&self) -> &impl LcdTrait;
    fn buttons(&self) -> &impl ButtonsControllerTrait;
    fn led(&self) -> &Led;

    fn set_bt_connection(&self, bt: BT);
    fn last_bt_value(&self) -> BtValue;
    fn set_bt_value(&self, value: BtValue);
    fn mut_bt_value(&self, f: impl FnOnce(&mut BtValue));
    fn bt_is_connected(&self) -> bool;
    fn bt_read_count(&self) -> usize;
    fn bt_read_value(&self) -> BtValue;
    #[allow(async_fn_in_trait)]
    async fn bt_connect_to_device(&self, device: &[u8; 6], pin: &[u8; 4], read_period_ms: u32);
    #[allow(async_fn_in_trait)]
    async fn bt_connect_to_host(&self, read_period_ms: u32);
}

impl<
        S1: SensorConfig,
        S2: SensorConfig,
        S3: SensorConfig,
        S4: SensorConfig,
        MA: MotorConfig,
        MB: MotorConfig,
        MC: MotorConfig,
        MD: MotorConfig,
        TOP: LcdTopSideProvider + 'static,
    > Ev3BrickUpdater for BaseEv3<S1, S2, S3, S4, MA, MB, MC, MD, TOP>
{
    fn update(&self) {
        self.update();
    }

    fn apply(&self) {
        self.apply();
    }
}

impl<
        S1: SensorConfig,
        S2: SensorConfig,
        S3: SensorConfig,
        S4: SensorConfig,
        MA: MotorConfig,
        MB: MotorConfig,
        MC: MotorConfig,
        MD: MotorConfig,
        TOP: LcdTopSideProvider + 'static,
    > Ev3Brick for BaseEv3<S1, S2, S3, S4, MA, MB, MC, MD, TOP>
{
    fn init(&self) {
        self.init();
    }

    async fn setup(&self) {
        self.setup().await;
    }

    fn time(&self) -> &Time {
        &self.time
    }

    fn reset_time(&self) {
        self.time.reset();
    }

    fn timer_in<'a>(&'a self, at: Duration) -> Timer<'a> {
        self.timer_in(at)
    }

    fn timer_ms<'a>(&'a self, ms: u32) -> Timer<'a> {
        self.timer_ms(ms)
    }

    fn tick<'a>(&'a self) -> Timer<'a> {
        self.tick()
    }

    fn s1(&self) -> &impl SensorTrait {
        &self.s1
    }

    fn s2(&self) -> &impl SensorTrait {
        &self.s2
    }

    fn s3(&self) -> &impl SensorTrait {
        &self.s3
    }

    fn s4(&self) -> &impl SensorTrait {
        &self.s4
    }

    fn ma(&self) -> &impl MotorTrait {
        &self.ma
    }

    fn mb(&self) -> &impl MotorTrait {
        &self.mb
    }

    fn mc(&self) -> &impl MotorTrait {
        &self.mc
    }

    fn md(&self) -> &impl MotorTrait {
        &self.md
    }

    fn lcd(&self) -> &impl LcdTrait {
        &self.lcd
    }

    fn buttons(&self) -> &impl ButtonsControllerTrait {
        &self.buttons
    }

    fn led(&self) -> &Led {
        &self.led
    }

    fn set_bt_connection(&self, bt: BT) {
        self.set_bt_connection(bt);
    }

    fn last_bt_value(&self) -> BtValue {
        self.last_bt_value()
    }

    fn set_bt_value(&self, value: BtValue) {
        self.set_bt_value(value);
    }

    fn mut_bt_value(&self, f: impl FnOnce(&mut BtValue)) {
        self.mut_bt_value(f);
    }

    fn bt_is_connected(&self) -> bool {
        self.bt_is_connected()
    }

    fn bt_read_count(&self) -> usize {
        self.bt_read_count()
    }

    fn bt_read_value(&self) -> BtValue {
        self.bt_read_value()
    }

    async fn bt_connect_to_device(&self, device: &[u8; 6], pin: &[u8; 4], read_period_ms: u32) {
        self.bt_connect_to_device(device, pin, read_period_ms).await;
    }

    async fn bt_connect_to_host(&self, read_period_ms: u32) {
        self.bt_connect_to_host(read_period_ms).await;
    }
}
