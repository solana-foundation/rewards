use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_current_program, verify_current_program_account, verify_event_authority, verify_writable},
};

pub struct CloseDirectRecipientAccounts<'a> {
    pub recipient: &'a AccountView,
    pub original_payer: &'a AccountView,
    pub distribution: &'a AccountView,
    pub recipient_account: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CloseDirectRecipientAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [recipient, original_payer, distribution, recipient_account, event_authority, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. No signer required — permissionless when fully claimed (processor enforces the gate)

        // 2. Validate writable
        verify_writable(original_payer, true)?;
        verify_writable(recipient_account, true)?;

        // 3. Validate program IDs
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. Validate accounts owned by current program
        verify_current_program_account(distribution)?;
        verify_current_program_account(recipient_account)?;

        Ok(Self { recipient, original_payer, distribution, recipient_account, event_authority, program })
    }
}

impl<'a> InstructionAccounts<'a> for CloseDirectRecipientAccounts<'a> {}
