use crate::utils::{impl_len, impl_load};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Escrow {
    pub maker: [u8; 32],
    pub mint_a: [u8; 32],
    pub mint_b: [u8; 32],
    pub amount_to_receive: u64,
    pub amount_to_give: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl_len!(Escrow);
impl_load!(Escrow);
