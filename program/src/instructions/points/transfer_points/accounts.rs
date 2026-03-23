use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_readonly, verify_signer,
        verify_system_program, verify_writable,
    },
};

pub struct TransferPointsAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub from_user: &'a AccountView,
    pub points_config: &'a AccountView,
    pub from_user_points: &'a AccountView,
    pub to_user: &'a AccountView,
    pub to_user_points: &'a AccountView,
    pub system_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TransferPointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, from_user, points_config, from_user_points, to_user, to_user_points, system_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(authority, false)?;
        verify_signer(from_user, false)?;

        verify_writable(from_user_points, true)?;
        verify_writable(to_user_points, true)?;

        verify_readonly(points_config)?;
        verify_readonly(to_user)?;

        verify_current_program_account(points_config)?;
        verify_current_program_account(from_user_points)?;

        verify_system_program(system_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self {
            payer,
            authority,
            from_user,
            points_config,
            from_user_points,
            to_user,
            to_user_points,
            system_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for TransferPointsAccounts<'a> {}
