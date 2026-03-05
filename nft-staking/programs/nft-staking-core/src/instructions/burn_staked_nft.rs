use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to_checked, Mint, MintToChecked, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority},
    ID as MPL_CORE_ID,
};

use crate::errors::StakingError;
use crate::helpers::SECONDS_IN_A_DAY;
use crate::state::Config;

const BURN_BONUS: u64 = 100_000_000;

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: PDA
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: validated by mpl-core
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: validated by mpl-core
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: mpl-core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> BurnStakedNft<'info> {
    pub fn burn_staked_nft(&mut self, bumps: &BurnStakedNftBumps) -> Result<()> {
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(base_asset.owner == self.user.key(), StakingError::InvalidOwner);
        require!(
            base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            StakingError::InvalidAuthority
        );
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

        let current_timestamp = Clock::get()?.unix_timestamp;

        let fetched = match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.nft.to_account_info(),
            PluginType::Attributes,
        ) {
            Err(_) => return Err(StakingError::NotStaked.into()),
            Ok((_, attrs, _)) => attrs,
        };

        let mut staked_value: Option<String> = None;
        let mut staked_at_value: Option<String> = None;
        for attr in &fetched.attribute_list {
            match attr.key.as_str() {
                "staked" => staked_value = Some(attr.value.clone()),
                "staked_at" => staked_at_value = Some(attr.value.clone()),
                _ => {}
            }
        }

        require!(
            staked_value.as_deref() == Some("true"),
            StakingError::NotStaked
        );

        let staked_at = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;

        let staked_time_days = current_timestamp
            .checked_sub(staked_at)
            .ok_or(StakingError::InvalidTimestamp)?
            .checked_div(SECONDS_IN_A_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(&[signer_seeds])?;

        if let Ok((_, col_attrs, _)) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &self.collection.to_account_info(),
            PluginType::Attributes,
        ) {
            let mut col_list: Vec<Attribute> = Vec::new();
            for attr in col_attrs.attribute_list {
                if attr.key == "total_staked" {
                    let value = attr
                        .value
                        .parse::<usize>()
                        .map_err(|_| StakingError::InvalidNumber)?;
                    col_list.push(Attribute {
                        key: "total_staked".to_string(),
                        value: value.saturating_sub(1).to_string(),
                    });
                } else {
                    col_list.push(attr);
                }
            }
            UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(&self.collection.to_account_info())
                .payer(&self.user.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes {
                    attribute_list: col_list,
                }))
                .invoke_signed(&[signer_seeds])?;
        }

        let time_rewards = (staked_time_days as u64)
            .checked_mul(self.config.points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;
        let amount = BURN_BONUS
            .checked_add(time_rewards)
            .ok_or(StakingError::Overflow)?;

        let config_bump = self.config.config_bump;
        let config_seeds: &[&[u8]] = &[b"config", collection_key.as_ref(), &[config_bump]];
        let config_signer = &[config_seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintToChecked {
                mint: self.rewards_mint.to_account_info(),
                to: self.user_rewards_ata.to_account_info(),
                authority: self.config.to_account_info(),
            },
            config_signer,
        );
        mint_to_checked(cpi_ctx, amount, self.rewards_mint.decimals)?;

        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;

        Ok(())
    }
}
