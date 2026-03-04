use pinocchio::{
    ProgramResult,
    cpi::{Seed, Signer},
    entrypoint::InstructionContext,
    sysvars::rent::ACCOUNT_STORAGE_OVERHEAD,
};

use crate::{
    entrypoint::PROGRAM_ADDRESS,
    raw_cpi,
    states::Contributor,
};

const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;
const CONTRIBUTOR_RENT: u64 =
    (ACCOUNT_STORAGE_OVERHEAD + Contributor::LEN as u64) * DEFAULT_LAMPORTS_PER_BYTE;

#[inline(always)]
pub fn process_create_contributor(ctx: &mut InstructionContext) -> ProgramResult {
    let contributor = unsafe { ctx.next_account_unchecked() }.assume_account();
    let fundraiser = unsafe { ctx.next_account_unchecked() }.assume_account();
    let contributor_state = unsafe { ctx.next_account_unchecked() }.assume_account();
    let _ = unsafe { ctx.next_account_unchecked() }; // system_program

    let ix_data = unsafe { ctx.instruction_data_unchecked() };
    let bump = ix_data[1];

    let bump_arr = [bump];
    let contributor_seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_array()),
        Seed::from(contributor.address().as_array()),
        Seed::from(&bump_arr),
    ];

    raw_cpi::raw_create_account_signed(
        &contributor,
        &contributor_state,
        CONTRIBUTOR_RENT,
        Contributor::LEN as u64,
        &PROGRAM_ADDRESS,
        &[Signer::from(&contributor_seeds[..])],
    )?;

    let cs_data = unsafe { contributor_state.borrow_unchecked_mut() };
    let cs = unsafe { &mut *(cs_data.as_mut_ptr() as *mut Contributor) };
    cs.contributor = *contributor.address().as_array();
    cs.amount = 0;
    cs.bump = bump;

    Ok(())
}
