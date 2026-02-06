pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
mod tests;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3n16mCbPsep8awDkznTGPNDFnJAKhgGRDEcsExX7G33S");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
