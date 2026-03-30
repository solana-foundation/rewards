use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_readonly, verify_signer,
        verify_token_2022_program, verify_writable,
    },
};

pub struct RevokePointsAccounts<'a> {
    pub authority: &'a AccountView,
    pub points_config: &'a AccountView,
    pub points_mint: &'a AccountView,
    pub user: &'a AccountView,
    pub user_token_account: &'a AccountView,
    pub token_2022_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RevokePointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, points_config, points_mint, user, user_token_account, token_2022_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;

        verify_readonly(points_config)?;
        verify_writable(points_mint, true)?;
        verify_writable(user_token_account, true)?;

        verify_readonly(user)?;

        verify_current_program_account(points_config)?;

        verify_token_2022_program(token_2022_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self {
            authority,
            points_config,
            points_mint,
            user,
            user_token_account,
            token_2022_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for RevokePointsAccounts<'a> {}
