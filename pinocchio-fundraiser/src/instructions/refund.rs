use pinocchio::{
    ProgramResult,
    cpi::{Seed, Signer},
    entrypoint::InstructionContext,
};

use crate::{raw_cpi, states::{Contributor, Fundraiser}};

#[inline(always)]
pub fn process_refund(ctx: &mut InstructionContext) -> ProgramResult {
    let contributor = unsafe { ctx.next_account_unchecked() }.assume_account();
    let fundraiser = unsafe { ctx.next_account_unchecked() }.assume_account();
    let vault = unsafe { ctx.next_account_unchecked() }.assume_account();
    let contributor_ata = unsafe { ctx.next_account_unchecked() }.assume_account();
    let contributor_state = unsafe { ctx.next_account_unchecked() }.assume_account();

    let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
    let fs = unsafe { &*(fundraiser_data.as_ptr() as *const Fundraiser) };

    let maker = fs.maker;
    let bump_arr = [fs.bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(&maker),
        Seed::from(&bump_arr),
    ];

    let contributor_data = unsafe { contributor_state.borrow_unchecked() };
    let cs = unsafe { &*(contributor_data.as_ptr() as *const Contributor) };
    let refund_amount = cs.amount;

    raw_cpi::raw_transfer_signed(
        &vault,
        &contributor_ata,
        &fundraiser,
        refund_amount,
        &[Signer::from(&signer_seeds[..])],
    )?;

    contributor.set_lamports(contributor.lamports() + contributor_state.lamports());
    contributor_state.set_lamports(0);

    Ok(())
}
