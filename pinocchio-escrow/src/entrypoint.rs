use pinocchio::{
    address::declare_id, default_panic_handler, error::ProgramError, no_allocator,
    program_entrypoint, AccountView, Address, ProgramResult,
};

use crate::instructions::{
    process_cancel_instruction, process_cancel_v2_instruction, process_make_instruction,
    process_make_v2_instruction, process_take_instruction, process_take_v2_instruction,
};

program_entrypoint!(process_instruction);
no_allocator!();
default_panic_handler!();

declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    match instruction_data.split_first() {
        Some((0, rest)) => process_make_instruction(accounts, rest),
        Some((1, rest)) => process_take_instruction(accounts, rest),
        Some((2, rest)) => process_cancel_instruction(accounts, rest),
        Some((3, rest)) => process_make_v2_instruction(accounts, rest),
        Some((4, rest)) => process_take_v2_instruction(accounts, rest),
        Some((5, rest)) => process_cancel_v2_instruction(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
