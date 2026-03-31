use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account_address, verify_associated_token_program, verify_current_program,
        verify_current_program_account, verify_event_authority, verify_readonly, verify_signer, verify_system_program,
        verify_token_2022_program, verify_writable,
    },
};

pub struct TransferPointsAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub from_user: &'a AccountView,
    pub to_user: &'a AccountView,
    pub points_config: &'a AccountView,
    pub points_mint: &'a AccountView,
    pub from_token_account: &'a AccountView,
    pub to_token_account: &'a AccountView,
    pub token_2022_program: &'a AccountView,
    pub ata_program: &'a AccountView,
    pub system_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TransferPointsAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, from_user, to_user, points_config, points_mint, from_token_account, to_token_account, token_2022_program, ata_program, system_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(authority, false)?;
        verify_signer(from_user, false)?;

        verify_readonly(points_config)?;
        verify_readonly(to_user)?;

        verify_writable(points_mint, true)?;
        verify_writable(from_token_account, true)?;
        verify_writable(to_token_account, true)?;

        verify_current_program_account(points_config)?;

        validate_associated_token_account_address(
            from_token_account,
            from_user.address(),
            points_mint,
            token_2022_program,
        )?;
        validate_associated_token_account_address(
            to_token_account,
            to_user.address(),
            points_mint,
            token_2022_program,
        )?;

        verify_token_2022_program(token_2022_program)?;
        verify_associated_token_program(ata_program)?;
        verify_system_program(system_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self {
            payer,
            authority,
            from_user,
            to_user,
            points_config,
            points_mint,
            from_token_account,
            to_token_account,
            token_2022_program,
            ata_program,
            system_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for TransferPointsAccounts<'a> {}
