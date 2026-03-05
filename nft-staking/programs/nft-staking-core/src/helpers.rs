use anchor_lang::prelude::*;

pub const SECONDS_IN_A_DAY: i64 = 86_400;
pub const NINE_AM_UTC: i64 = 9 * 3_600;
pub const FIVE_PM_UTC: i64 = 17 * 3_600;
pub const MARGIN: i64 = 10 * 60;
pub const REWARD_LAMPORTS: u64 = 1_000_000;

#[constant]
pub const ORACLE_ACCOUNT: Pubkey =
    Pubkey::from_str_const("53DY5i9HL2bYoB8nD2yjVoaNZ8RLp3Lmr99VfFy4U8eF");

pub fn is_allowed(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight < FIVE_PM_UTC
}

pub fn is_correct_time(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    (seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight < NINE_AM_UTC + MARGIN)
        || (seconds_since_midnight >= FIVE_PM_UTC && seconds_since_midnight < FIVE_PM_UTC + MARGIN)
}
