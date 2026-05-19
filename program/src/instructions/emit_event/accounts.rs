use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_event_authority, verify_signer},
};

pub struct EmitEventAccounts<'a> {
    pub event_authority: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for EmitEventAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [event_authority] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(event_authority, false)?;
        verify_event_authority(event_authority)?;

        Ok(Self { event_authority })
    }
}

impl<'a> InstructionAccounts<'a> for EmitEventAccounts<'a> {}
