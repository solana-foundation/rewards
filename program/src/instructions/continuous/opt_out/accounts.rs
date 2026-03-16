use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_token_program, verify_writable,
        ZK_ELGAMAL_PROOF_PROGRAM_ID,
    },
};

pub struct ContinuousOptOutAccounts<'a> {
    pub user: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub user_reward_account: &'a AccountView,
    pub user_tracked_token_account: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub user_reward_token_account: &'a AccountView,
    pub tracked_mint: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub tracked_token_program: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
    /// Required when `pool.confidential_rewards != 0`.
    pub equality_proof_context: Option<&'a AccountView>,
    /// Required when `pool.confidential_rewards != 0`.
    pub ciphertext_validity_proof_context: Option<&'a AccountView>,
    /// Required when `pool.confidential_rewards != 0`.
    pub range_proof_context: Option<&'a AccountView>,
}

impl<'a> TryFrom<&'a [AccountView]> for ContinuousOptOutAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 12 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let user = &accounts[0];
        let reward_pool = &accounts[1];
        let user_reward_account = &accounts[2];
        let user_tracked_token_account = &accounts[3];
        let reward_vault = &accounts[4];
        let user_reward_token_account = &accounts[5];
        let tracked_mint = &accounts[6];
        let reward_mint = &accounts[7];
        let tracked_token_program = &accounts[8];
        let reward_token_program = &accounts[9];
        let event_authority = &accounts[10];
        let program = &accounts[11];

        verify_signer(user, true)?;

        verify_writable(reward_pool, true)?;
        verify_writable(user_reward_account, true)?;
        verify_writable(reward_vault, true)?;
        verify_writable(user_reward_token_account, true)?;

        verify_readonly(user_tracked_token_account)?;
        verify_readonly(tracked_mint)?;
        verify_readonly(reward_mint)?;

        verify_current_program_account(reward_pool)?;
        verify_current_program_account(user_reward_account)?;

        verify_token_program(tracked_token_program)?;
        verify_token_program(reward_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_owned_by(tracked_mint, tracked_token_program.address())?;
        verify_owned_by(reward_mint, reward_token_program.address())?;
        verify_owned_by(user_tracked_token_account, tracked_token_program.address())?;
        verify_owned_by(user_reward_token_account, reward_token_program.address())?;

        validate_associated_token_account(reward_vault, reward_pool.address(), reward_mint, reward_token_program)?;
        validate_associated_token_account(
            user_tracked_token_account,
            user.address(),
            tracked_mint,
            tracked_token_program,
        )?;

        let (equality_proof_context, ciphertext_validity_proof_context, range_proof_context) = if accounts.len() >= 15 {
            let eq_ctx = &accounts[12];
            let cv_ctx = &accounts[13];
            let rp_ctx = &accounts[14];
            verify_owned_by(eq_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            verify_owned_by(cv_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            verify_owned_by(rp_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            (Some(eq_ctx), Some(cv_ctx), Some(rp_ctx))
        } else {
            (None, None, None)
        };

        Ok(Self {
            user,
            reward_pool,
            user_reward_account,
            user_tracked_token_account,
            reward_vault,
            user_reward_token_account,
            tracked_mint,
            reward_mint,
            tracked_token_program,
            reward_token_program,
            event_authority,
            program,
            equality_proof_context,
            ciphertext_validity_proof_context,
            range_proof_context,
        })
    }
}

impl<'a> InstructionAccounts<'a> for ContinuousOptOutAccounts<'a> {}
