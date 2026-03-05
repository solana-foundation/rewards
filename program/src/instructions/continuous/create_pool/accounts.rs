use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account_address, verify_associated_token_program, verify_current_program,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_system_program,
        verify_token_program, verify_token_program_account, verify_writable,
    },
};

pub struct CreateContinuousPoolAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub seed: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub tracked_mint: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub associated_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CreateContinuousPoolAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, seed, reward_pool, tracked_mint, reward_mint, reward_vault, system_program, reward_token_program, associated_token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(payer, true)?;
        verify_signer(authority, false)?;
        verify_signer(seed, false)?;

        verify_writable(reward_pool, true)?;
        verify_writable(reward_vault, true)?;

        verify_readonly(tracked_mint)?;
        verify_readonly(reward_mint)?;
        verify_readonly(seed)?;

        verify_system_program(system_program)?;
        verify_token_program(reward_token_program)?;
        verify_associated_token_program(associated_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_token_program_account(tracked_mint)?;
        verify_owned_by(reward_mint, reward_token_program.address())?;

        validate_associated_token_account_address(
            reward_vault,
            reward_pool.address(),
            reward_mint,
            reward_token_program,
        )?;

        Ok(Self {
            payer,
            authority,
            seed,
            reward_pool,
            tracked_mint,
            reward_mint,
            reward_vault,
            system_program,
            reward_token_program,
            associated_token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for CreateContinuousPoolAccounts<'a> {}
