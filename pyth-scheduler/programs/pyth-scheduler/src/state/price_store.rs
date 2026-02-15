use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PriceStore {
    pub price: i64,        // the price value
    pub exponent: i32,     // price * 10^exponent = actual price
    pub confidence: u64,   // uncertainty range
    pub published_at: i64, // unix timestamp from pyth
    pub bump: u8,          // bump
}
