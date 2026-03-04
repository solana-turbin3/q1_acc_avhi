use pinocchio::{
    ProgramResult,
    cpi::{Seed, Signer},
    entrypoint::InstructionContext,
};

use crate::{raw_cpi, states::Fundraiser};

#[inline(always)]
pub fn process_checker(ctx: &mut InstructionContext) -> ProgramResult {
    let maker = unsafe { ctx.next_account_unchecked() }.assume_account();
    let fundraiser = unsafe { ctx.next_account_unchecked() }.assume_account();
    let vault = unsafe { ctx.next_account_unchecked() }.assume_account();
    let maker_ata = unsafe { ctx.next_account_unchecked() }.assume_account();

    let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
    let state = unsafe { &*(fundraiser_data.as_ptr() as *const Fundraiser) };

    let bump = state.bump;
    let current_amount = state.current_amount;

    let bump_arr = [bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_arr),
    ];
    let signer = Signer::from(&signer_seeds[..]);

    raw_cpi::raw_transfer_signed(&vault, &maker_ata, &fundraiser, current_amount, &[signer])?;

    maker.set_lamports(maker.lamports() + fundraiser.lamports());
    fundraiser.set_lamports(0);

    Ok(())
}
