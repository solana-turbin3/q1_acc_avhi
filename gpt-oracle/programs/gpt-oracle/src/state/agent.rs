use anchor_lang::prelude::*;

#[account]
pub struct Agent {
    pub context: Pubkey,
    pub bump: u8,
}
