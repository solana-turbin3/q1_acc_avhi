use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{PriceStore, PRICE, SOL_USD_FEED_ID};

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [PRICE.as_bytes()],
        bump,
        space = 8 + PriceStore::INIT_SPACE
    )]
    pub price_store: Account<'info, PriceStore>,

    pub price_feed: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdatePrice<'info> {
    pub fn update_price(&mut self, bumps: &UpdatePriceBumps) -> Result<()> {
        let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;

        let price = self.price_feed.get_price_no_older_than(&Clock::get()?, 300, &feed_id)?;

        self.price_store.set_inner(PriceStore {
            price: price.price,
            exponent: price.exponent,
            confidence: price.conf,
            published_at: price.publish_time,
            bump: bumps.price_store,
        });

        msg!("SOL/USD price updated: {} * 10^{}", price.price, price.exponent);

        Ok(())
    }
}
