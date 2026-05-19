use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_signer, verify_writable,
    },
};

pub struct CloseMerkleClaimAccounts<'a> {
    pub claimant: &'a mut AccountView,
    pub distribution: &'a AccountView,
    pub claim_account: &'a mut AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for CloseMerkleClaimAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [claimant, distribution, claim_account, event_authority, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. Validate signers
        verify_signer(claimant, true)?;

        // 2. Validate writable
        verify_writable(claim_account, true)?;

        // 3. Validate program IDs
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. Validate accounts owned by current program
        // Note: distribution owner is validated in processor (must be system program = closed)
        verify_current_program_account(claim_account)?;

        Ok(Self { claimant, distribution, claim_account, event_authority, program })
    }
}

impl<'a> InstructionAccounts<'a> for CloseMerkleClaimAccounts<'a> {}
