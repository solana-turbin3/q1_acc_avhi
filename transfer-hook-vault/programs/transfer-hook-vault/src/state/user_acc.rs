use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub account: Pubkey,
    pub amount: u64,
}

impl UserAccount {
    pub const LEN: usize = 8 + 32 + 8;
}
