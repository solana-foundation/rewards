use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_current_program_account, verify_event_authority, verify_signer, verify_writable,
    },
};

pub struct SetContinuousBalanceAccounts<'a> {
    pub authority: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub user_reward_account: &'a AccountView,
    pub user: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for SetContinuousBalanceAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, reward_pool, user_reward_account, user, event_authority, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;

        verify_writable(reward_pool, true)?;
        verify_writable(user_reward_account, true)?;

        verify_current_program_account(reward_pool)?;
        verify_current_program_account(user_reward_account)?;

        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        Ok(Self { authority, reward_pool, user_reward_account, user, event_authority, program })
    }
}

impl<'a> InstructionAccounts<'a> for SetContinuousBalanceAccounts<'a> {}
