use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_token_account_owner,
        verify_token_program, verify_writable,
    },
};

pub struct ClaimDirectAccounts<'a> {
    pub recipient: &'a AccountView,
    pub distribution: &'a AccountView,
    pub recipient_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub distribution_vault: &'a AccountView,
    pub recipient_token_account: &'a AccountView,
    pub token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimDirectAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [recipient, distribution, recipient_account, mint, distribution_vault, recipient_token_account, token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. Validate signers
        verify_signer(recipient, false)?;

        // 2. Validate writable
        verify_writable(distribution, true)?;
        verify_writable(recipient_account, true)?;
        verify_writable(distribution_vault, true)?;
        verify_writable(recipient_token_account, true)?;

        // 2b. Validate read-only accounts
        verify_readonly(mint)?;

        // 3. Validate program IDs
        verify_token_program(token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. Validate accounts owned by current program
        verify_current_program_account(distribution)?;
        verify_current_program_account(recipient_account)?;

        // 5. Validate token account ownership
        verify_owned_by(mint, token_program.address())?;
        verify_owned_by(recipient_token_account, token_program.address())?;
        verify_token_account_owner(recipient_token_account, recipient.address())?;

        // 6. Validate distribution_vault ATA
        validate_associated_token_account(distribution_vault, distribution.address(), mint, token_program)?;

        Ok(Self {
            recipient,
            distribution,
            recipient_account,
            mint,
            distribution_vault,
            recipient_token_account,
            token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for ClaimDirectAccounts<'a> {}
