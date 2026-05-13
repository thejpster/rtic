#![no_std]

pub const NVIC_PRIO_BITS: u8 = 4;

pub struct Peripherals {}

impl Peripherals {
    pub unsafe fn steal() -> Peripherals {
        Peripherals {}
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
#[allow(non_camel_case_types)]
pub enum interrupt {
    Sgi1,
    Sgi2,
    PhysTimer,
    VirtTimer,
}

impl interrupt {
    pub const fn as_intid(self) -> rtic::export::IntId {
        match self {
            Self::Sgi1 => rtic::export::IntId::sgi(1),
            Self::Sgi2 => rtic::export::IntId::sgi(2),
            Self::PhysTimer => rtic::export::IntId::ppi(14),
            Self::VirtTimer => rtic::export::IntId::ppi(11),
        }
    }
}

impl Into<rtic::export::IntId> for interrupt {
    fn into(self) -> rtic::export::IntId {
        self.as_intid()
    }
}

impl PartialEq<rtic::export::IntId> for interrupt {
    fn eq(&self, other: &rtic::export::IntId) -> bool {
        let self_int_id: rtic::export::IntId = (*self).into();
        self_int_id == *other
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _default_interrupt_handler() {
    unimplemented!()
}

static TIMER_QUEUE: rtic_time::timer_queue::TimerQueue<TimerBackend> =
    rtic_time::timer_queue::TimerQueue::new();

pub struct TimerBackend;

impl rtic_time::timer_queue::TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        aarch32_cpu::generic_timer::read_virtual_timer()
    }

    fn set_compare(instant: Self::Ticks) {
        defmt::debug!("Set compare to {}", instant);
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(instant);
    }

    fn clear_compare_flag() {
        defmt::debug!("clear compare");
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(u64::MAX);
    }

    fn pend_interrupt() {
        defmt::debug!("pend interrupt");
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(0);
    }

    fn timer_queue() -> &'static rtic_time::timer_queue::TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

pub struct Mono;

impl Mono {
    pub fn start(mut timer: aarch32_cpu::generic_timer::El1VirtualTimer) {
        use aarch32_cpu::generic_timer::GenericTimer;
        TIMER_QUEUE.initialize(TimerBackend {});
        timer.enable(true);
        timer.interrupt_mask(false);
    }

    pub fn handle_irq() {
        unsafe {
            TIMER_QUEUE.on_monotonic_interrupt();
        }
    }
}

impl rtic_time::monotonic::TimerQueueBasedMonotonic for Mono {
    type Backend = TimerBackend;

    type Instant = Instant;

    type Duration = Duration;
}

rtic_time::impl_embedded_hal_delay_fugit!(Mono);
rtic_time::impl_embedded_hal_async_delay_fugit!(Mono);

pub type Instant = fugit::Instant<u64, 1, 62_500_000>;
pub type Duration = fugit::Duration<u64, 1, 62_500_000>;
