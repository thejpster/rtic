#[cfg(all(feature = "armv8r", not(feature = "armv8r-backend")))]
compile_error!("Building for Armv8-R, but 'armv8r-backend not selected'");

pub use aarch32_cpu::interrupt;

pub use arm_gic::IntId;

/// Get the highest prio pending interrupt, if any
pub fn get_and_process<F>(f: F) -> bool where F: FnOnce(IntId) {
    let Some(int_id) = arm_gic::gicv3::GicCpuInterface::get_and_acknowledge_interrupt(arm_gic::InterruptGroup::Group1) else {
        return false;
    };
    f(int_id);
    arm_gic::gicv3::GicCpuInterface::end_interrupt(int_id, arm_gic::InterruptGroup::Group1);
    true
}

/// Standard per-core peripherals for Armv8-R
pub struct Peripherals {
    pub virt_timer: aarch32_cpu::generic_timer::El1VirtualTimer,
    pub phys_timer: aarch32_cpu::generic_timer::El1PhysicalTimer,
    pub gic: Gic,
}

impl Peripherals {
    /// Unsafely create a set of peripherals
    ///
    /// # Safety
    /// 
    /// Must only be called once per core
    pub unsafe fn steal() -> Peripherals {
        Peripherals { 
            virt_timer: unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() },
            phys_timer: unsafe { aarch32_cpu::generic_timer::El1PhysicalTimer::new() },
            gic: unsafe { Gic::new() }
        }
    }
}

/// Represents our interrupt controller
pub struct Gic {
    inner: arm_gic::gicv3::GicV3<'static>
}

impl Gic {
    /// Create our ARM GIC driver
    /// 
    /// TODO: parmeterise GICD_BASE_OFFSET and GICR_BASE_OFFSET
    ///
    /// # Safety
    ///
    /// Only call this function once.
    pub unsafe fn new() -> Gic {
        /// Offset from PERIPHBASE for GIC Distributor
        const GICD_BASE_OFFSET: usize = 0x0000_0000usize;
    
        /// Offset from PERIPHBASE for the first GIC Redistributor
        const GICR_BASE_OFFSET: usize = 0x0010_0000usize;
    
        // Get the GIC address by reading CBAR
        let periphbase = aarch32_cpu::register::ImpCbar::read().periphbase();
        
        // Initialise the GIC.
        let gicd_base = periphbase.wrapping_byte_add(GICD_BASE_OFFSET);
        // SAFETY: `gicd_base` points to the valid GICD MMIO region as obtained from the
        // hardware CBAR register. This pointer is used exclusively by this GIC instance.
        let gicd_ptr = unsafe {
            arm_gic::UniqueMmioPointer::new(core::ptr::NonNull::new(gicd_base.cast()).unwrap())
        };

        let gicr_base = periphbase.wrapping_byte_add(GICR_BASE_OFFSET);
        let gicr_ptr = core::ptr::NonNull::new(gicr_base.cast()).unwrap();
        // SAFETY: The GICD and GICR base addresses point to valid GICv3 MMIO regions as
        // obtained from the hardware CBAR register. This function is only called once
        // (via Board::new()'s atomic guard), ensuring exclusive ownership of the GIC.
        let mut gic = unsafe {arm_gic::gicv3::GicV3::new(gicd_ptr, gicr_ptr, 1, false) };
        gic.setup(0);
        Gic {
            inner: gic
        }
    }

    /// Set the interrupt priority
    pub fn set_interrupt_priority(&mut self, int_id: IntId, priority: u8) {
        // we only support one core
        self.inner.set_group(int_id, Some(0), arm_gic::gicv3::Group::Group1NS).expect("set_group failed");
        self.inner.set_interrupt_priority(int_id, Some(0), priority).expect("set_interrupt_priority failed");
    }

    /// Enable/disable an interrupt
    pub fn enable_interrupt(&mut self, int_id: IntId, enable: bool) {
        self.inner.enable_interrupt(int_id, Some(0), enable).expect("enable_interrupt failed");
    }

    /// Set the interrupt priority mask
    pub fn set_priority_mask(priority: u8) {
        arm_gic::gicv3::GicCpuInterface::set_priority_mask(priority);
    }
}

#[inline(always)]
pub fn run<F>(_priority: u8, f: F)
where
    F: FnOnce(),
{
    use arm_gic::gicv3::GicCpuInterface;

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
    use arm_gic::gicv3::GicCpuInterface;

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
    GicCpuInterface::send_sgi(int, SgiTarget::All, SgiTargetGroup::CurrentGroup1).expect("bad IntId for send_sgi");
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
