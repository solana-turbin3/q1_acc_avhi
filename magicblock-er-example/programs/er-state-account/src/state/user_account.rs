use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub user: Pubkey,
    pub data: u64,
    pub bump: u8,
}

impl Space for UserAccount {
    const INIT_SPACE: usize = 32 + 8 + 1 + 8; // Pubkey + u64 + u8 + 8 bytes for account discriminator
}