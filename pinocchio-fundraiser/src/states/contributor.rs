use crate::utils::{impl_len, impl_load};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Contributor {
    pub amount: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl_len!(Contributor);
impl_load!(Contributor);
