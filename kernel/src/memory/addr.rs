use spin::Lazy;

use crate::HHDM_REQUEST;

pub static HHDM_OFFSET: Lazy<VirtAddr> =
    Lazy::new(|| VirtAddr::new(HHDM_REQUEST.get_response().unwrap().offset()));

#[derive(Debug, Copy, Clone)]
pub enum AddrError {
    InvalidAlign,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn null() -> Self {
        Self(0)
    }

    pub fn as_hhdm_virt(self) -> VirtAddr {
        *HHDM_OFFSET + self.0
    }

    pub fn align_down(self, align: u64) -> Result<Self, AddrError> {
        if !align.is_power_of_two() {
            return Err(AddrError::InvalidAlign);
        }

        Ok(Self(self.0 & !(align - 1)))
    }
}

impl core::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:x}", self.0))
            .finish()
    }
}

impl core::iter::Step for PhysAddr {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        u64::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u64::forward_checked(start.0, count).map(PhysAddr)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u64::backward_checked(start.0, count).map(PhysAddr)
    }
}

impl core::ops::Add<u64> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::AddAssign<u64> for PhysAddr {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl From<u64> for PhysAddr {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<PhysAddr> for u64 {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl core::fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:x}", self.0))
            .finish()
    }
}

impl core::ops::Add<u64> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}
