use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_readonly, verify_signer,
        verify_system_program, verify_writable,
    },
};

pub struct IssuePointsAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub points_config: &'a AccountView,
    pub user: &'a AccountView,
    pub user_points_account: &'a AccountView,
    pub system_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for IssuePointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, points_config, user, user_points_account, system_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(authority, false)?;

        verify_writable(points_config, true)?;
        verify_writable(user_points_account, true)?;

        verify_readonly(user)?;

        verify_current_program_account(points_config)?;

        verify_system_program(system_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self {
            payer,
            authority,
            points_config,
            user,
            user_points_account,
            system_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for IssuePointsAccounts<'a> {}
