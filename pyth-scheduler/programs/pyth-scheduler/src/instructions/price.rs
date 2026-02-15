use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{PriceStore, PRICE, SOL_USD_FEED};

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

    /// CHECK: Pyth price feed account, address constrained
    #[account(address = SOL_USD_FEED)]
    pub price_feed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdatePrice<'info> {
    pub fn update_price(&mut self, bumps: &UpdatePriceBumps) -> Result<()> {
        let price_update = PriceUpdateV2::try_deserialize(
            &mut self.price_feed.data.borrow().as_ref(),
        )?;

        let price = price_update.get_price_no_older_than(
            &Clock::get()?,
            60,
            &get_feed_id_from_hex(
                "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
            )?,
        )?;

        self.price_store.set_inner(PriceStore {
            price: price.price,
            exponent: price.exponent,
            confidence: price.conf,
            published_at: price.publish_time,
            bump: bumps.price_store,
        });

        msg!(
            "SOL/USD price updated: {} * 10^{}",
            price.price,
            price.exponent
        );

        Ok(())
    }
}
