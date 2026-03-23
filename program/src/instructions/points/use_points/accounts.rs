use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_signer, verify_writable,
    },
};

pub struct UsePointsAccounts<'a> {
    pub authority: &'a AccountView,
    pub user: &'a AccountView,
    pub points_config: &'a AccountView,
    pub user_points_account: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for UsePointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, user, points_config, user_points_account, event_authority, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;
        verify_signer(user, false)?;

        verify_writable(points_config, true)?;
        verify_writable(user_points_account, true)?;

        verify_current_program_account(points_config)?;
        verify_current_program_account(user_points_account)?;

        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self { authority, user, points_config, user_points_account, event_authority, program })
    }
}

impl<'a> InstructionAccounts<'a> for UsePointsAccounts<'a> {}
