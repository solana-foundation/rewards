use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_token_program, verify_writable,
    },
};

pub struct CloseDirectDistributionAccounts<'a> {
    pub authority: &'a AccountView,
    pub distribution: &'a AccountView,
    pub mint: &'a AccountView,
    pub distribution_vault: &'a AccountView,
    pub authority_token_account: &'a AccountView,
    pub token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CloseDirectDistributionAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, distribution, mint, distribution_vault, authority_token_account, token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. Validate signers
        verify_signer(authority, true)?;

        // 2. Validate writable
        verify_writable(distribution, true)?;
        verify_writable(distribution_vault, true)?;
        verify_writable(authority_token_account, true)?;

        // 2b. Validate read-only accounts
        verify_readonly(mint)?;

        // 3. Validate program IDs
        verify_token_program(token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. Validate accounts owned by current program
        verify_current_program_account(distribution)?;

        // 5. Validate token account ownership
        verify_owned_by(mint, token_program.address())?;
        verify_owned_by(authority_token_account, token_program.address())?;

        // 6. Validate distribution_vault ATA
        validate_associated_token_account(distribution_vault, distribution.address(), mint, token_program)?;

        Ok(Self {
            authority,
            distribution,
            mint,
            distribution_vault,
            authority_token_account,
            token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for CloseDirectDistributionAccounts<'a> {}
