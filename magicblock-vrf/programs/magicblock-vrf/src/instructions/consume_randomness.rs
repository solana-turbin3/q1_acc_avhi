use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::rnd::random_u64;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct ConsumeRandomnessCtx<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> ConsumeRandomnessCtx<'info> {
    pub fn consume_randomness(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rnd_u64 = random_u64(&randomness);
        self.user_account.data = rnd_u64;
        Ok(())
    }
}
