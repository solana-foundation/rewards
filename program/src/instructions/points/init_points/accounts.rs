use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_event_authority, verify_readonly, verify_signer, verify_system_program,
        verify_token_2022_program, verify_writable,
    },
};

pub struct InitPointsAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub seed: &'a AccountView,
    pub points_config: &'a AccountView,
    pub points_mint: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_2022_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for InitPointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, seed, points_config, points_mint, system_program, token_2022_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(authority, false)?;
        verify_signer(seed, false)?;

        verify_writable(points_config, true)?;
        verify_writable(points_mint, true)?;

        verify_readonly(seed)?;

        verify_system_program(system_program)?;
        verify_token_2022_program(token_2022_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self {
            payer,
            authority,
            seed,
            points_config,
            points_mint,
            system_program,
            token_2022_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for InitPointsAccounts<'a> {}
