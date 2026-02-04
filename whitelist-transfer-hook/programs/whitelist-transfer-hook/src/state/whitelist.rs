use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub address: Pubkey,
    pub bump: u8,
}

impl Whitelist {
    pub const LEN: usize = 8 + 32 + 1;
}
