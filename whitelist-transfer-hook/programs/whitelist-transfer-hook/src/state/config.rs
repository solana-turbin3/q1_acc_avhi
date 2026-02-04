use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + 32 + 1;
}
