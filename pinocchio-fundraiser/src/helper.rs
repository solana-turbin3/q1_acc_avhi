use pinocchio::AccountView;
use pinocchio::error::ProgramError;

pub fn check_signer(a: &AccountView, err: ProgramError) -> Result<(), ProgramError> {
    if !a.is_signer() {
        return Err(err);
    }
    Ok(())
}
