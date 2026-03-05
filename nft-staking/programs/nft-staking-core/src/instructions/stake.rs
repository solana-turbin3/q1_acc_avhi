use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::{BaseAssetV1, BaseCollectionV1}, 
    fetch_plugin, 
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder}, 
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority}
};
use crate::state::Config;
use crate::errors::StakingError;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: PDA Update authority
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
    /// CHECK: NFT account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        
        // Verify NFT owner and update authority
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(base_asset.owner == self.user.key(), StakingError::InvalidOwner);
        require!(base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()), StakingError::InvalidAuthority);
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(base_collection.update_authority == self.update_authority.key(), StakingError::InvalidAuthority);

        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Get the current time
        let current_time = Clock::get()?.unix_timestamp;

        // Check if the NFT has the attribute plugin already added
        match fetch_plugin::<BaseAssetV1, Attributes>(&self.nft.to_account_info(), PluginType::Attributes) {
            Err(_) => {
                // Add the attribute plugin to the NFT if it doesn't have it yet ('staked' and 'staked_at' attributes)
                AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.nft.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(
                        Attributes { 
                            attribute_list: vec![
                                Attribute { 
                                    key: "staked".to_string(), 
                                    value: "true".to_string() 
                                },
                                Attribute { 
                                    key: "staked_at".to_string(), 
                                    value: current_time.to_string() 
                                },
                            ] 
                        }
                    ))
                    .init_authority(PluginAuthority::UpdateAuthority)
                    .invoke_signed(&[signer_seeds])?;
            }
            Ok((_, fetched_attribute_list, _)) => {
                // Verify the fetched attribute list has the 'staked' and 'staked_at' attributes
                let mut attribute_list: Vec<Attribute> = Vec::new();
                let mut staked = false;
                let mut staked_at = false;
                for attribute in fetched_attribute_list.attribute_list {
                    if attribute.key == "staked" {
                        require!(attribute.value == "false", StakingError::AlreadyStaked);
                        attribute_list.push(Attribute { 
                            key: "staked".to_string(), 
                            value: "true".to_string() 
                        });
                        staked = true;
                    }else if attribute.key == "staked_at" {
                        attribute_list.push(Attribute { 
                            key: "staked_at".to_string(), 
                            value: current_time.to_string() 
                        });
                        staked_at = true;
                    }else {
                        attribute_list.push(attribute);
                    }
                }
                // Add the 'staked' and 'staked_at' attributes if they don't exist
                if !staked {
                    attribute_list.push(Attribute { 
                        key: "staked".to_string(), 
                        value: "true".to_string() 
                    });
                }
                if !staked_at {
                    attribute_list.push(Attribute { 
                        key: "staked_at".to_string(), 
                        value: current_time.to_string() 
                    });
                }
                UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.nft.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes( Attributes { attribute_list }))
                    .invoke_signed(&[signer_seeds])?;
            }
        }

        // Freeze the NFT (check if FreezeDelegate already exists from a previous stake)
        match fetch_plugin::<BaseAssetV1, FreezeDelegate>(&self.nft.to_account_info(), PluginType::FreezeDelegate) {
            Err(_) => {
                // First time staking — add FreezeDelegate plugin
                AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.nft.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.user.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
                    .init_authority(PluginAuthority::UpdateAuthority)
                    .invoke()?;
            }
            Ok(_) => {
                // Re-staking — FreezeDelegate exists from a previous unstake, just re-freeze
                UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.nft.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
                    .invoke_signed(&[signer_seeds])?;
            }
        }

        Ok(())
    }
}