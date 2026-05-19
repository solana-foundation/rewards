use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::utils::{verify_event_authority, verify_signer};

pub fn process_emit_event(_program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    let [event_authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_event_authority(event_authority)?;
    verify_signer(event_authority, false)?;

    Ok(())
}
