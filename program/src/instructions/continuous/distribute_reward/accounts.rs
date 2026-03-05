use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_token_program, verify_writable,
    },
};

pub struct DistributeContinuousRewardAccounts<'a> {
    pub authority: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub authority_token_account: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DistributeContinuousRewardAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, reward_pool, reward_mint, reward_vault, authority_token_account, reward_token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;

        verify_writable(reward_pool, true)?;
        verify_writable(reward_vault, true)?;
        verify_writable(authority_token_account, true)?;

        verify_readonly(reward_mint)?;

        verify_current_program_account(reward_pool)?;

        verify_token_program(reward_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_owned_by(reward_mint, reward_token_program.address())?;

        validate_associated_token_account(reward_vault, reward_pool.address(), reward_mint, reward_token_program)?;

        Ok(Self {
            authority,
            reward_pool,
            reward_mint,
            reward_vault,
            authority_token_account,
            reward_token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for DistributeContinuousRewardAccounts<'a> {}
