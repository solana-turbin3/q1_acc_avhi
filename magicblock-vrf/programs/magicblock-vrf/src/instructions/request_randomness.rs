use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

use crate::instruction::ConsumeRandomness;
use crate::state::UserAccount;

#[vrf]
#[derive(Accounts)]
pub struct RequestRandomnessCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"user", payer.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info> RequestRandomnessCtx<'info> {
    pub fn request_randomness(&self) -> Result<()> {
        let clock = Clock::get()?;
        let mut seed = [0u8; 32];
        seed[..8].copy_from_slice(&clock.slot.to_le_bytes());
        seed[8..16].copy_from_slice(&clock.unix_timestamp.to_le_bytes());
        seed[16..24].copy_from_slice(&self.payer.key().to_bytes()[..8]);

        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.payer.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator: ConsumeRandomness::DISCRIMINATOR.to_vec(),
            caller_seed: seed,
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });

        self.invoke_signed_vrf(&self.payer.to_account_info(), &ix)?;

        Ok(())
    }
}
