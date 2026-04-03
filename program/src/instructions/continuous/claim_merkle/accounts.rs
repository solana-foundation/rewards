use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_system_program,
        verify_token_program, verify_writable,
    },
};

pub struct ClaimContinuousMerkleAccounts<'a> {
    pub payer: &'a AccountView,
    pub user: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub claim_account: &'a AccountView,
    pub revocation_marker: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub user_reward_token_account: &'a AccountView,
    pub system_program: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimContinuousMerkleAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, user, reward_pool, claim_account, revocation_marker, reward_mint, reward_vault, user_reward_token_account, system_program, reward_token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(user, false)?;

        verify_writable(reward_pool, true)?;
        verify_writable(claim_account, true)?;
        verify_writable(reward_vault, true)?;
        verify_writable(user_reward_token_account, true)?;

        verify_readonly(revocation_marker)?;
        verify_readonly(reward_mint)?;

        verify_system_program(system_program)?;
        verify_token_program(reward_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_current_program_account(reward_pool)?;

        verify_owned_by(reward_mint, reward_token_program.address())?;
        verify_owned_by(user_reward_token_account, reward_token_program.address())?;

        validate_associated_token_account(reward_vault, reward_pool.address(), reward_mint, reward_token_program)?;

        Ok(Self {
            payer,
            user,
            reward_pool,
            claim_account,
            revocation_marker,
            reward_mint,
            reward_vault,
            user_reward_token_account,
            system_program,
            reward_token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for ClaimContinuousMerkleAccounts<'a> {}
