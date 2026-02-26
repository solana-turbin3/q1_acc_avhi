use pinocchio::{
    address::declare_id, default_panic_handler, error::ProgramError, no_allocator,
    program_entrypoint, AccountView, Address, ProgramResult,
};

use crate::instructions::{process_make_instruction, process_take_instruction};

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
        Some((2, _))    => Err(ProgramError::InvalidInstructionData),
        _               => Err(ProgramError::InvalidInstructionData),
    }
}
