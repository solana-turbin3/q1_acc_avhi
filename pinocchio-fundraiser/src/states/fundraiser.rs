use crate::utils::{impl_len, impl_load};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
}

impl_len!(Fundraiser);
impl_load!(Fundraiser);
