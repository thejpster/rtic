#![no_std]

pub const NVIC_PRIO_BITS: u8 = 4;

pub struct Peripherals {
    dummy: u32
}

impl Peripherals {
    pub unsafe fn steal() -> Peripherals {
        Peripherals { dummy: 123 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum interrupt {
    SGI_1,
    SGI_2,
    PHYS_TIMER,
    VIRT_TIMER,
}


impl Into<rtic::export::IntId> for interrupt {
    fn into(self) -> rtic::export::IntId {
        match self {
            interrupt::SGI_1 => rtic::export::IntId::sgi(1),
            interrupt::SGI_2 => rtic::export::IntId::sgi(2),
            interrupt::PHYS_TIMER => rtic::export::IntId::ppi(14),
            interrupt::VIRT_TIMER => rtic::export::IntId::ppi(11),
        }
    } 
}

impl PartialEq<rtic::export::IntId> for interrupt {
    fn eq(&self, other: &rtic::export::IntId) -> bool {
        let self_int_id: rtic::export::IntId = (*self).into();
        self_int_id == *other
    }
}

/// Create the ARM GIC driver
///
/// # Safety
///
/// Only call this function once.
pub unsafe fn make_gic() -> rtic::export::arm_gic::gicv3::GicV3<'static> {
    /// Offset from PERIPHBASE for GIC Distributor
    const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

    /// Offset from PERIPHBASE for the first GIC Redistributor
    const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

    // Get the GIC address by reading CBAR
    let periphbase = aarch32_cpu::register::ImpCbar::read().periphbase();
    let gicd_base = periphbase.wrapping_byte_add(GICD_BASE_OFFSET);
    let gicr_base = periphbase.wrapping_byte_add(GICR_BASE_OFFSET);

    // Initialise the GIC.
    // SAFETY: `gicd_base` points to the valid GICD MMIO region as obtained from the
    // hardware CBAR register. This pointer is used exclusively by this GIC instance.
    let gicd = unsafe {
        rtic::export::arm_gic::UniqueMmioPointer::new(core::ptr::NonNull::new(gicd_base.cast()).unwrap())
    };
    let gicr_base = core::ptr::NonNull::new(gicr_base.cast()).unwrap();
    // SAFETY: The GICD and GICR base addresses point to valid GICv3 MMIO regions as
    // obtained from the hardware CBAR register. This function is only called once
    // (via Board::new()'s atomic guard), ensuring exclusive ownership of the GIC.
    let mut gic = unsafe { rtic::export::arm_gic::gicv3::GicV3::new(gicd, gicr_base, 1, false) };
    gic.setup(0);
    gic
}

#[unsafe(no_mangle)]
pub extern "C" fn _default_interrupt_handler() {
    unimplemented!()
}
