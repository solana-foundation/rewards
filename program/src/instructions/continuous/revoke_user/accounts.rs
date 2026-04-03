use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_system_program,
        verify_token_program, verify_writable,
    },
};

pub struct RevokeContinuousUserAccounts<'a> {
    pub authority: &'a AccountView,
    pub payer: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub user_reward_account: &'a AccountView,
    pub revocation_marker: &'a AccountView,
    pub user: &'a AccountView,
    pub rent_destination: &'a AccountView,
    pub user_tracked_token_account: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub user_reward_token_account: &'a AccountView,
    pub authority_reward_token_account: &'a AccountView,
    pub tracked_mint: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub system_program: &'a AccountView,
    pub tracked_token_program: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RevokeContinuousUserAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, reward_pool, user_reward_account, revocation_marker, user, rent_destination, user_tracked_token_account, reward_vault, user_reward_token_account, authority_reward_token_account, tracked_mint, reward_mint, system_program, tracked_token_program, reward_token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;
        verify_signer(payer, true)?;

        verify_writable(reward_pool, true)?;
        verify_writable(user_reward_account, true)?;
        verify_writable(revocation_marker, true)?;
        verify_writable(rent_destination, false)?;
        verify_writable(reward_vault, true)?;
        verify_writable(user_reward_token_account, true)?;
        verify_writable(authority_reward_token_account, true)?;

        verify_readonly(user_tracked_token_account)?;
        verify_readonly(tracked_mint)?;
        verify_readonly(reward_mint)?;

        verify_current_program_account(reward_pool)?;
        verify_current_program_account(user_reward_account)?;

        verify_system_program(system_program)?;
        verify_token_program(tracked_token_program)?;
        verify_token_program(reward_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_owned_by(tracked_mint, tracked_token_program.address())?;
        verify_owned_by(user_tracked_token_account, tracked_token_program.address())?;
        verify_owned_by(reward_mint, reward_token_program.address())?;
        verify_owned_by(user_reward_token_account, reward_token_program.address())?;
        verify_owned_by(authority_reward_token_account, reward_token_program.address())?;

        validate_associated_token_account(reward_vault, reward_pool.address(), reward_mint, reward_token_program)?;
        validate_associated_token_account(
            user_tracked_token_account,
            user.address(),
            tracked_mint,
            tracked_token_program,
        )?;
        validate_associated_token_account(
            authority_reward_token_account,
            authority.address(),
            reward_mint,
            reward_token_program,
        )?;

        Ok(Self {
            authority,
            payer,
            reward_pool,
            user_reward_account,
            revocation_marker,
            user,
            rent_destination,
            user_tracked_token_account,
            reward_vault,
            user_reward_token_account,
            authority_reward_token_account,
            tracked_mint,
            reward_mint,
            system_program,
            tracked_token_program,
            reward_token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for RevokeContinuousUserAccounts<'a> {}
