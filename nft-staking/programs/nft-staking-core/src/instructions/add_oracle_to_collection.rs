use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1,
    instructions::AddCollectionExternalPluginAdapterV1CpiBuilder,
    types::{
        ExternalCheckResult, ExternalPluginAdapterInitInfo, HookableLifecycleEvent, OracleInitInfo,
        ValidationResultsOffset,
    },
    ID as MPL_CORE_ID,
};

use crate::errors::StakingError;
use crate::helpers::ORACLE_ACCOUNT;
use crate::state::Oracle;

#[derive(Accounts)]
pub struct AddOracleToCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: validated below
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"oracle"],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: mpl-core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddOracleToCollection<'info> {
    pub fn add_oracle_to_collection(
        &mut self,
        bumps: &AddOracleToCollectionBumps,
    ) -> Result<()> {
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(
            base_collection.update_authority == self.update_authority.key(),
            StakingError::InvalidAuthority
        );

        let collection_key = self.collection.key();
        let signer_seeds: &[&[u8]] = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        let can_reject = ExternalCheckResult { flags: 4 };

        AddCollectionExternalPluginAdapterV1CpiBuilder::new(
            &self.mpl_core_program.to_account_info(),
        )
        .collection(&self.collection.to_account_info())
        .payer(&self.payer.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .init_info(ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: ORACLE_ACCOUNT,
            init_plugin_authority: None,
            lifecycle_checks: vec![(HookableLifecycleEvent::Transfer, can_reject)],
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Anchor),
        }))
        .invoke_signed(&[signer_seeds])?;

        Ok(())
    }
}
