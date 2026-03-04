use pinocchio::{
    ProgramResult,
    entrypoint::InstructionContext,
};

use crate::{raw_cpi, states::{Contributor, Fundraiser}};

#[inline(always)]
pub fn process_contribute(ctx: &mut InstructionContext) -> ProgramResult {
    let contributor = unsafe { ctx.next_account_unchecked() }.assume_account();
    let fundraiser = unsafe { ctx.next_account_unchecked() }.assume_account();
    let vault = unsafe { ctx.next_account_unchecked() }.assume_account();
    let contributor_ata = unsafe { ctx.next_account_unchecked() }.assume_account();
    let contributor_state = unsafe { ctx.next_account_unchecked() }.assume_account();
    let _ = unsafe { ctx.next_account_unchecked() }; // token_program

    let ix_data = unsafe { ctx.instruction_data_unchecked() };
    let amount = u64::from_le_bytes(unsafe {
        *(ix_data.as_ptr().add(1) as *const [u8; 8])
    });

    raw_cpi::raw_transfer_signed(&contributor_ata, &vault, &contributor, amount, &[])?;

    let cs = unsafe { &mut *(contributor_state.borrow_unchecked_mut().as_mut_ptr() as *mut Contributor) };
    cs.amount = cs.amount.wrapping_add(amount);

    let fs = unsafe { &mut *(fundraiser.borrow_unchecked_mut().as_mut_ptr() as *mut Fundraiser) };
    fs.current_amount = fs.current_amount.wrapping_add(amount);

    Ok(())
}
