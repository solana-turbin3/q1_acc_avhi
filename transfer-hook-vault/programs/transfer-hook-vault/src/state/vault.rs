use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}
