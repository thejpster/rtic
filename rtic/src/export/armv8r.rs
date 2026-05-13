#[cfg(all(feature = "armv8r", not(feature = "armv8r-backend")))]
compile_error!("Building for Armv8-R, but 'armv8r-backend not selected'");

use arm_gic::gicv3::GicCpuInterface;
pub use arm_gic::IntId;
// pub use aarch32_cpu::interrupt;

pub use arm_gic;

pub mod interrupt {
    pub unsafe fn enable() {
        defmt::println!("Interrupts on!");
        unsafe {
            aarch32_cpu::interrupt::enable();
        }
    }

    pub fn disable() {
        defmt::println!("Interrupts off!");
        aarch32_cpu::interrupt::disable();
    }
}

/// Standard per-core peripherals for Armv8-R
pub struct Peripherals {
    pub timer: aarch32_cpu::generic_timer::El1VirtualTimer,
}

impl Peripherals {
    /// Unsafely create a set of peripherals
    ///
    /// # Safety
    /// 
    /// Must only be called once per core
    pub unsafe fn steal() -> Peripherals {
        Peripherals { 
            timer: unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() }
        }
    }
}

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{

    // TODO: who sets the priority of my interrupts?

    // if priority == 1 {
    //     f();
    //     GicCpuInterface::set_priority_mask(1);
    // } else {
        let initial = GicCpuInterface::get_priority_mask();
        f();
        GicCpuInterface::set_priority_mask(initial);
    // }
}

/// Lock implementation using threshold and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the threshold to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    unsafe {
        let current = GicCpuInterface::get_priority_mask();

        GicCpuInterface::set_priority_mask(ceiling);

        let r = f(&mut *ptr);

        GicCpuInterface::set_priority_mask(current);

        r
    }
}

/// Sets the given software interrupt as pending
#[inline(always)]
pub fn pend(int: impl Into<IntId>) {
    use arm_gic::gicv3::{SgiTargetGroup, SgiTarget, GicCpuInterface};
    let int: IntId = int.into();
    assert!(int.is_sgi());
    GicCpuInterface::send_sgi(int, SgiTarget::All, SgiTargetGroup::CurrentGroup1);
}

// // Sets the given software interrupt as not pending
// pub fn unpend(int: impl Into<IntId>) {
//     // TODO
//     // let arm_gic = arm_gic::gicv3::GicCpuInterface::send_sgi(intid, target, group);
//     let int: IntId = int.into();
//     todo!("unpend");
// }

// pub fn enable(int: impl Into<IntId>, prio: u8, cpu_int_id: u8) {
//     // TODO
//     let int: IntId = int.into();
//     todo!("enable");
// }
