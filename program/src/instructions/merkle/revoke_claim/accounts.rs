use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_system_program,
        verify_token_program, verify_writable,
    },
};

pub struct RevokeMerkleClaimAccounts<'a> {
    pub authority: &'a AccountView,
    pub payer: &'a AccountView,
    pub distribution: &'a AccountView,
    pub claim_account: &'a AccountView,
    pub revocation_marker: &'a AccountView,
    pub claimant: &'a AccountView,
    pub mint: &'a AccountView,
    pub distribution_vault: &'a AccountView,
    pub claimant_token_account: &'a AccountView,
    pub authority_token_account: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RevokeMerkleClaimAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, distribution, claim_account, revocation_marker, claimant, mint, distribution_vault, claimant_token_account, authority_token_account, system_program, token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. Validate signers
        verify_signer(authority, false)?;
        verify_signer(payer, true)?;

        // 2. Validate writable
        verify_writable(distribution, true)?;
        verify_writable(revocation_marker, true)?;
        verify_writable(distribution_vault, true)?;
        verify_writable(claimant_token_account, true)?;
        verify_writable(authority_token_account, true)?;

        // 2b. Validate read-only accounts
        verify_readonly(claim_account)?;
        verify_readonly(claimant)?;
        verify_readonly(mint)?;

        // 3. Validate program IDs
        verify_system_program(system_program)?;
        verify_token_program(token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. Validate accounts owned by current program
        verify_current_program_account(distribution)?;
        // revocation_marker will be created

        // 5. Validate token account ownership
        verify_owned_by(mint, token_program.address())?;
        verify_owned_by(claimant_token_account, token_program.address())?;
        verify_owned_by(authority_token_account, token_program.address())?;

        // 6. Validate distribution_vault ATA
        validate_associated_token_account(distribution_vault, distribution.address(), mint, token_program)?;

        Ok(Self {
            authority,
            payer,
            distribution,
            claim_account,
            revocation_marker,
            claimant,
            mint,
            distribution_vault,
            claimant_token_account,
            authority_token_account,
            system_program,
            token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for RevokeMerkleClaimAccounts<'a> {}
