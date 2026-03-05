use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub points_per_stake: u32,   // Points earned per staked NFT per day
    pub freeze_period: u8,       // Minimum required time in days for an NFT to be unstaked
    pub rewards_bump: u8,        // bump seed for the rewards mint PDA
    pub config_bump: u8,         // bump seed for the config account PDA
}